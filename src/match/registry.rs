use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

use crate::common::error::AppError;
use super::Match;

#[derive(Clone)]
pub struct MatchRegistry {
    registry: Arc<RwLock<HashMap<Uuid, Arc<RwLock<Match>>>>>,
    broadcaster: broadcast::Sender<Option<Match>>,
}

impl MatchRegistry {
    pub fn new(broadcaster_capacity: usize) -> Self {
        let (broadcaster, _) = broadcast::channel(broadcaster_capacity);
        Self {
            registry: Arc::new(RwLock::new(HashMap::new())),
            broadcaster,
        }
    }

    pub async fn add_match(&mut self, r#match: Match) -> Result<(), AppError> {
        let mut registry_w = self.registry.write().await;
        registry_w.insert(r#match.match_id, Arc::new(RwLock::new(r#match.clone())));
        let _ = self.broadcaster.send(Some(r#match)); // TODO: handle publish errors
        Ok(())
    }

    pub async fn rm_match(&mut self, match_id: &Uuid) -> Result<(), AppError> {
        let Some(_) = self.registry.write().await.remove(match_id) else {
            return Err(AppError::not_found(match_id.clone()));
        };
        let _ = self.broadcaster.send(None); // TODO: handle publish errors
        return Ok(());
    }

    pub async fn get_match_arc(&self, match_id: &Uuid) -> Option<Arc<RwLock<Match>>> {
        let registry_r = self.registry.read().await;
        let Some(r#match) = registry_r.get(match_id) else {
            return None;
        };
        let r#match = r#match.clone();
        Some(r#match)
    }

    // pub async fn get_match(&self, match_id: &Uuid) -> Option<Match> {
    //     let registry_r = self.registry.read().await;
    //     let Some(r#match) = registry_r.get(match_id) else {
    //         return None;
    //     };
    //     let r#match = r#match.read().await.clone();
    //     Some(r#match)
    // }

    pub fn subscribe(&self) -> broadcast::Receiver<Option<Match>> {
        self.broadcaster.subscribe()
    }
}

impl Default for MatchRegistry {
    fn default() -> Self {
        Self::new(10)
    }
}
