use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tokio::sync::broadcast::Receiver;

use crate::common::error::AppError;
use crate::lobby::Lobby;
use crate::output::structs::{LobbyState, MatchState};
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

        let lobby_state = LobbyState::from_lobby(lobby, self.player_registry.clone()).await?;
        let match_state = match r#match {
            Some(r#match) => Some(MatchState::from_match(r#match).await),
            None => None,
        };

        let game_state = GameState::build(
            lobby_state,
            match_state,
        );
        Ok(game_state)
    }
}
