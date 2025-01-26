use std::collections::HashSet;
use uuid::Uuid;

use crate::common::error::AppError;
use crate::lobby::LobbySettings;
use crate::r#match::Match;

#[derive(Clone, Debug)]
pub struct Lobby {
    pub lobby_id: Uuid,
    pub name: String,
    pub host_player_id: Uuid,
    pub player_ids: HashSet<Uuid>,
    pub game_acceptance: Option<HashSet<Uuid>>, // per player
    pub match_id: Option<Uuid>,
    pub settings: LobbySettings,
}

impl Lobby {
    pub fn new(name: String, host_player_id: Uuid) -> Self {
        let lobby_id = Uuid::new_v4();
        let player_ids = HashSet::from([host_player_id]);
        Lobby {
            lobby_id,
            name,
            host_player_id,
            player_ids,
            game_acceptance: None,
            match_id: None,
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
        self.match_id.is_some()
    }

    pub fn is_in_game_then_id(&self) -> Option<Uuid> {
        self.match_id.clone()
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

    pub fn start_match(&self) -> (Lobby, Match) {
        let r#match = Match::new(self.lobby_id, self.player_ids.clone());
        let mut lobby = self.clone();
        // TODO: Make game_acceptance + match_id type-wise impossible
        lobby.game_acceptance = None;
        lobby.match_id = Some(r#match.match_id);
        (lobby, r#match)
    }

    pub fn start_matchmaking(&mut self) -> Result<(), AppError> {
        if self.is_matchmaking() {
           return Ok(())
        }
        self.lobby_locked_validation()?;
        let game_acceptance = HashSet::from([self.host_player_id]);
        self.game_acceptance = Some(game_acceptance);
        Ok(())
    }

    pub fn clear_matchmaking(&mut self) {
        if let Some(game_acceptance) = self.is_matchmaking_then_acceptance_mut() {
            game_acceptance.clear();
        } else {
            self.game_acceptance = None;
        }
    }

    pub fn add_player(&mut self, player_id: Uuid) -> Result<(), AppError> {
        self.lobby_locked_validation()?;

        self.player_ids.insert(player_id);
        self.clear_matchmaking(); // technically can't join while matchmaking...
        Ok(())
    }

    pub fn rm_player(&mut self, player_id: &Uuid) -> Result<(), AppError> {
        self.lobby_locked_validation()?;

        if !self.player_ids.contains(player_id) {
            return Ok(());
        }
        self.player_ids.remove(player_id);
        self.clear_matchmaking();
        Ok(())
    }

    pub fn set_match_acceptance(&mut self, player_id: Uuid, acceptance: bool) -> Result<(), AppError> {
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

        Ok(())
    }

    pub fn is_player(&self, player_id: &Uuid) -> bool {
        self.player_ids.contains(player_id)
    }

    pub fn is_host_player(&self, player_id: &Uuid) -> bool {
       self.host_player_id == *player_id
    }

    // RM ME -- NO ACTUALLY EXPLICIT START ???
    // pub async fn start_game(&mut self, registry: &mut MatchRegistry) -> Result<(), AppError> {
    //     self.check_game_start_possible()?;
    //
    //     let poker_match = Match::new(self.lobby_id, self.player_ids.clone());
    //     let match_id = poker_match.match_id;
    //     registry.add_match(poker_match).await?;
    //     self.match_id = Some(match_id);
    //     Ok(())
    // }
}
