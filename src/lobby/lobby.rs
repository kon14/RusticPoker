use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::common::error::AppError;
use crate::lobby::LobbySettings;
use crate::player::{Player, PlayerRegistry};
use crate::r#match::Match;
use crate::output::GameStateBroadcaster;

#[derive(Clone, Debug)]
pub struct Lobby {
    pub lobby_id: Uuid,
    pub state_broadcaster: GameStateBroadcaster,
    pub name: String,
    pub host_player_id: Uuid,
    pub player_ids: HashSet<Uuid>,
    pub game_acceptance: Option<HashSet<Uuid>>, // per player
    pub r#match: Option<Match>,
    pub settings: LobbySettings,
}

impl Lobby {
    pub fn new(
        broadcast_channel_capacity: usize,
        player_registry: Arc<RwLock<PlayerRegistry>>,
        lobby_name: String,
        host_player_id: Uuid,
    ) -> Self {
        let lobby_id = Uuid::new_v4();
        let state_broadcaster = GameStateBroadcaster::new(
            broadcast_channel_capacity,
            player_registry);
        let player_ids = HashSet::from([host_player_id]);
        Lobby {
            lobby_id,
            state_broadcaster,
            name: lobby_name,
            host_player_id,
            player_ids,
            game_acceptance: None,
            r#match: None,
            settings: LobbySettings::default(),
        }
    }

    pub fn lobby_locked_validation(&self) -> Result<(), AppError> {
        if self.is_in_game() {
            return Err(AppError::unauthorized("Cannot operate on lobby while in-game!"));
        }
        if self.is_matchmaking() {
            return Err(AppError::unauthorized("Cannot operate on lobby while matchmaking!"));
        }
        Ok(())
    }

    pub fn is_in_game(&self) -> bool {
        self.r#match.is_some()
    }

    pub fn is_in_game_then_id(&self) -> Option<Uuid> {
        self.r#match.as_ref().map(|r#match| r#match.match_id)
    }

    pub fn is_matchmaking(&self) -> bool {
        self.game_acceptance.is_some()
    }

    pub fn is_matchmaking_then_acceptance(&self) -> Option<&HashSet<Uuid>> {
        self.game_acceptance.as_ref()
    }

    fn is_matchmaking_then_acceptance_mut(&mut self) -> Option<&mut HashSet<Uuid>> {
        self.game_acceptance.as_mut()
    }

    pub fn check_game_start_possible(&self) -> Result<(), AppError> {
        if self.is_in_game() {
            return Err(AppError::precondition_failed("Lobby already in-game!"));
        }
        let Some(game_acceptance) = self.is_matchmaking_then_acceptance() else {
            return Err(AppError::precondition_failed("Lobby not currently matchmaking!"));
        };
        if (self.player_ids.len() as u8) < self.settings.min_players  {
            return Err(AppError::precondition_failed(
                format!("Minimum number of players ({}) unmet!", self.settings.min_players)
            ));
        }
        if *game_acceptance != self.player_ids {
            return Err(AppError::precondition_failed("All players need to accept matchmaking!"));
        }
        Ok(())
    }

    pub async fn start_match(&mut self, players: HashSet<Player>) {
        let r#match = Match::new(
            self.lobby_id,
            self.state_broadcaster.clone(),
            players,
            self.settings.ante_amount,
        );

        // TODO: Make game_acceptance + match_id type-wise impossible
        self.game_acceptance = None;
        self.r#match = Some(r#match);

        self.state_broadcaster.publish_with_lobby(&self).await;
    }

    pub async fn start_matchmaking(&mut self) -> Result<(), AppError> {
        if self.is_matchmaking() {
           return Ok(())
        }
        self.lobby_locked_validation()?;

        if (self.player_ids.len() as u8) < self.settings.min_players  {
            return Err(AppError::precondition_failed(
                format!("Minimum number of players ({}) unmet!", self.settings.min_players)
            ));
        }

        let game_acceptance = HashSet::from([self.host_player_id]);
        self.game_acceptance = Some(game_acceptance);

        self.state_broadcaster.publish_with_lobby(&self).await;
        Ok(())
    }

    pub async fn clear_matchmaking(&mut self) {
        if let Some(game_acceptance) = self.is_matchmaking_then_acceptance_mut() {
            game_acceptance.clear();
        }

        self.state_broadcaster.publish_with_lobby(&self).await;
    }

    pub async fn stop_matchmaking(&mut self) {
        self.game_acceptance = None;

        self.state_broadcaster.publish_with_lobby(&self).await;
    }

    pub async fn add_player(&mut self, player_id: Uuid) -> Result<(), AppError> {
        self.lobby_locked_validation()?;

        self.player_ids.insert(player_id);
        self.clear_matchmaking().await; // technically can't join while matchmaking...

        self.state_broadcaster.publish_with_lobby(&self).await;
        Ok(())
    }

    pub async fn rm_player(&mut self, player_id: &Uuid) -> Result<(), AppError> {
        self.lobby_locked_validation()?;

        if !self.player_ids.contains(player_id) {
            return Ok(());
        }
        self.player_ids.remove(player_id);
        self.clear_matchmaking().await;

        self.state_broadcaster.publish_with_lobby(&self).await;
        Ok(())
    }

    pub async fn set_match_acceptance(&mut self, player_id: Uuid, acceptance: bool) -> Result<(), AppError> {
        if self.is_in_game() {
            return Err(AppError::unauthorized("Cannot modify matchmaking acceptance while in-game!"));
        }

        if !self.player_ids.contains(&player_id) {
            return Err(AppError::not_found(player_id));
        }

        if let Some(game_acceptance) = &mut self.is_matchmaking_then_acceptance_mut() {
            if acceptance {
                game_acceptance.insert(player_id);
            } else {
                game_acceptance.remove(&player_id);
            }
        }

        self.state_broadcaster.publish_with_lobby(&self).await;
        Ok(())
    }

    pub fn is_player(&self, player_id: &Uuid) -> bool {
        self.player_ids.contains(player_id)
    }

    pub fn is_host_player(&self, player_id: &Uuid) -> bool {
       self.host_player_id == *player_id
    }
}
