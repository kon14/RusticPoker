use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tokio::sync::broadcast::Receiver;
use uuid::Uuid;

use crate::common::error::AppError;
use crate::lobby::Lobby;
use crate::output::structs::{MatchState, PlayerState};
use crate::player::PlayerRegistry;
use super::GameState;

#[derive(Clone, Debug)]
pub(crate) struct GameStateBroadcaster {
    lobby_arc: Option<Arc<RwLock<Lobby>>>,
    broadcaster: broadcast::Sender<Option<GameState>>,
    player_registry: Arc<RwLock<PlayerRegistry>>,
}

impl GameStateBroadcaster {
    pub fn new(
        broadcast_channel_capacity: usize,
        player_registry: Arc<RwLock<PlayerRegistry>>,
    ) -> Self {
        let (broadcaster, _) = broadcast::channel(broadcast_channel_capacity);
        GameStateBroadcaster {
            lobby_arc: None,
            broadcaster,
            player_registry,
        }
    }

    pub fn set_lobby(&mut self, lobby_arc: Arc<RwLock<Lobby>>) {
        self.lobby_arc = Some(lobby_arc);
    }

    pub fn subscribe(&self) -> Receiver<Option<GameState>> {
        self.broadcaster.subscribe()
    }

    pub async fn publish(&self) {
        // TODO: drop player repository and figure out inner broadcaster error handling
        if let Err(err) = self._publish(None).await {
            eprintln!("{}", err);
        }
    }

    /// Used to avoid deadlocks over Lobby Arcs.
    pub async fn publish_with_lobby(&self, lobby: &Lobby) {
        // TODO: drop player repository and figure out inner broadcaster error handling
        if let Err(err) = self._publish(Some(lobby)).await {
            eprintln!("{}", err);
        }
    }

    async fn _publish(&self, lobby: Option<&Lobby>) -> Result<(), AppError> {
        let state = self.build_state(lobby).await?;
        let _ = self.broadcaster
            .send(Some(state));
            // Ignore errors caused by no active receivers...
            // .map(|_| ())
            // .map_err(|err| {
            //     eprintln!("{}", err);
            //     AppError::internal("GameStateBroadcaster.publish() call failed!")
            // });
        Ok(())
    }

    // pub fn disconnect(&self) -> Result<(), AppError> {
    //     self.broadcaster
    //         .send(None)
    //         .map(|_| ())
    //         .map_err(|err| {
    //             eprintln!("{}", err);
    //             AppError::internal("GameStateBroadcaster.disconnect() call failed!")
    //         })
    // }

    pub async fn build_state(&self, lobby: Option<&Lobby>) -> Result<GameState, AppError> {
        let Some(lobby_arc) = &self.lobby_arc else {
            unreachable!("Partial GameStatePublisher initialization!");
        };

        let lobby = match lobby {
            Some(lobby) => lobby.clone(),
            None => lobby_arc.read().await.clone(),
        };
        let r#match = lobby.r#match.clone();

        let player_states = self.build_player_states(&lobby).await?;
        let lobby_state = lobby.into();
        let match_state = match r#match {
            Some(r#match) => Some(MatchState::from_match(r#match).await),
            None => None,
        };

        let game_state = GameState::build(
            player_states,
            lobby_state,
            match_state,
        );
        Ok(game_state)
    }
}

impl GameStateBroadcaster {
    async fn build_player_states(&self, lobby: &Lobby) -> Result<HashMap<Uuid, PlayerState>, AppError> {
        let player_ids = lobby.player_ids.clone();
        let player_registry_r = self.player_registry.read().await;
        let players = player_registry_r.get_players(&player_ids).await?;
        let players = players
            .into_iter()
            .map(|(player_id, player)| (player_id, player.into()))
            .collect();
        Ok(players)
    }
}
