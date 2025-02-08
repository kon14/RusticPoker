use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::common::error::AppError;
use super::Lobby;

#[derive(Clone, Debug, Default)]
pub struct LobbyRegistry {
    registry: Arc<RwLock<HashMap<Uuid, Arc<RwLock<Lobby>>>>>,
}

impl LobbyRegistry {
    pub async fn add_lobby(&mut self, lobby_id: Uuid, lobby: Arc<RwLock<Lobby>>) -> Result<(), AppError> {
        let mut registry_w = self.registry.write().await;
        registry_w.insert(lobby_id, lobby);
        Ok(())
    }

    pub async fn rm_lobby(&mut self, lobby_id: &Uuid) -> Result<(), AppError> {
        let Some(_) = self.registry.write().await.remove(lobby_id) else {
            return Err(AppError::not_found(lobby_id.clone()));
        };
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
}
