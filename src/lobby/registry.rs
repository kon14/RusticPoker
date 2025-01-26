use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

use crate::common::error::AppError;
use super::Lobby;

#[derive(Clone)]
pub struct LobbyRegistry {
    registry: Arc<RwLock<HashMap<Uuid, Arc<RwLock<Lobby>>>>>,
    broadcaster: broadcast::Sender<Option<Lobby>>,
}

impl LobbyRegistry {
    pub fn new(broadcaster_capacity: usize) -> Self {
        let (broadcaster, _) = broadcast::channel(broadcaster_capacity);
        Self {
            registry: Arc::new(RwLock::new(HashMap::new())),
            broadcaster,
        }
    }

    pub async fn add_lobby(&mut self, lobby: Lobby) -> Result<(), AppError> {
        let mut registry_w = self.registry.write().await;
        registry_w.insert(lobby.lobby_id, Arc::new(RwLock::new(lobby.clone())));
        let _ = self.broadcaster.send(Some(lobby)); // TODO: handle publish errors
        Ok(())
    }

    pub async fn rm_lobby(&mut self, lobby_id: &Uuid) -> Result<(), AppError> {
        let Some(_) = self.registry.write().await.remove(lobby_id) else {
            return Err(AppError::not_found(lobby_id.clone()));
        };
        let _ = self.broadcaster.send(None); // TODO: handle publish errors
        return Ok(());
    }

    pub async fn get_lobby_arc(&self, lobby_id: &Uuid) -> Option<Arc<RwLock<Lobby>>> {
        let registry_r = self.registry.read().await;
        let Some(lobby) = registry_r.get(lobby_id) else {
            return None;
        };
        let lobby = lobby.clone();
        Some(lobby)
    }

    pub async fn get_lobbies(&self) -> Vec<Lobby> {
        let registry_r = self.registry.read().await;
        // registry_r
        //     .values()
        //     .map(async move |lobby| {
        //         let lobby = lobby.read().await;
        //         lobby
        //     })
        //     .collect()
        let futures: Vec<_> = registry_r
            .values()
            .map(|lobby| {
                let lobby = lobby.clone();
                tokio::spawn(async move {
                    let lobby_r = lobby.read().await;
                    lobby_r.clone()
                })
            })
            .collect();
        let mut lobbies = Vec::new();
        for future in futures {
            let lobby = future.await.unwrap();
            lobbies.push(lobby);
        }
        lobbies
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Option<Lobby>> {
        self.broadcaster.subscribe()
    }
}

impl Default for LobbyRegistry {
    fn default() -> Self {
        Self::new(10)
    }
}
