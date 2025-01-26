use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

use crate::common::error::AppError;
use super::Player;

// TODO: Track Player/Lobby mapping in PlayerRegistry (outside Player struct)

#[derive(Clone)]
pub struct PlayerRegistry {
    registry: Arc<RwLock<HashMap<Uuid, Arc<RwLock<Player>>>>>,
    // broadcaster: broadcast::Sender<Option<Player>>,
}

impl PlayerRegistry {
    pub fn new(broadcaster_capacity: usize) -> Self {
        // let (broadcaster, _) = broadcast::channel(broadcaster_capacity);
        Self {
            registry: Arc::new(RwLock::new(HashMap::new())),
            // broadcaster,
        }
    }

    pub async fn add_player(&mut self, player: Player) -> Result<(), AppError> {
        let mut registry_w = self.registry.write().await;
        registry_w.insert(player.player_id, Arc::new(RwLock::new(player.clone())));
        // let _ = self.broadcaster.send(Some(player)); // TODO: handle publish errors
        Ok(())
    }

    // pub async fn rm_player(&mut self, player_id: &Uuid) -> Result<(), AppError> {
    //     let Some(_) = self.registry.write().await.remove(player_id) else {
    //         return Err(AppError::not_found(player_id.clone()));
    //     };
    //     let _ = self.broadcaster.send(None); // TODO: handle publish errors
    //     return Ok(());
    // }

    pub async fn get_player(&self, player_id: &Uuid) -> Option<Player> {
        let registry_r = self.registry.read().await;
        let Some(player) = registry_r.get(player_id) else {
            return None;
        };
        let player = player.read().await.clone();
        Some(player)
    }

    pub async fn get_players(&self, player_ids: &HashSet<Uuid>) -> Result<HashMap<Uuid, Player>, AppError> {
        let registry_r = self.registry.read().await;
        let reg_players: HashMap<Uuid, Option<Arc<RwLock<Player>>>> = player_ids
            .into_iter()
            .map(|player_id| {
                let player = registry_r
                    .get(player_id)
                    .map(|player_arc| player_arc.clone());
                (player_id.clone(), player)
            })
            .collect();
        let mut players = HashMap::with_capacity(player_ids.len());
        let mut missing_player_ids = HashSet::with_capacity(player_ids.len());
        for (player_id, player_opt) in reg_players {
            if let Some(player) = player_opt {
                let player = player.read().await.clone();
                players.insert(player_id, player);
            } else {
                missing_player_ids.insert(player_id);
            }
        };
        if missing_player_ids.len() > 0 {
            Err(AppError::internal("Missing player IDs!"))
        } else {
            Ok(players)
        }
    }

    // pub fn subscribe(&self) -> broadcast::Receiver<Option<Player>> {
    //     self.broadcaster.subscribe()
    // }
}

impl Default for PlayerRegistry {
    fn default() -> Self {
        Self::new(10)
    }
}
