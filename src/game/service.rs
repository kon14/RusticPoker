use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;
use async_stream::__private::AsyncStream;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::common::error::AppError;
use crate::game::DiscardedCards;
use crate::game::phase::BettingRoundAction;
use crate::lobby::{Lobby, LobbyRegistry};
use crate::player::{Player, PlayerRegistry};
use crate::output::{GameStateAsPlayer, LobbyInfoPublic};

#[derive(Default)]
pub struct GameService {
    lobby_registry: Arc<RwLock<LobbyRegistry>>,
    player_registry: Arc<RwLock<PlayerRegistry>>,
    player_lobby_map: Arc<RwLock<HashMap<Uuid, Uuid>>>,
}

// RPCs
impl GameService {
    const LOBBY_BROADCAST_CHANNEL_CAPACITY: usize = 10;

    pub async fn connect_rpc(&self) -> Result<Uuid, AppError> {
        let mut player_registry_w = self.player_registry.write().await;
        let player = Player::register();
        let player_id = player.player_id.clone();
        player_registry_w.add_player(player).await?;
        Ok(player_id)
    }

    pub async fn disconnect_rpc(&self, player_id: &Uuid) -> Result<(), AppError> {
        let mut player_registry_w = self.player_registry.write().await;
        // player_registry_w.rm_player(player_id).await?;
        todo!()
    }

    pub async fn create_lobby_rpc(&self, name: String, player_id: Uuid) -> Result<LobbyInfoPublic, AppError> {
        {
            let player_lobby_map_r = self.player_lobby_map.read().await;
            if let Some(lobby_id) = player_lobby_map_r.get(&player_id) {
                return Err(AppError::precondition_failed(
                    format!("Player ({player_id}) already participating in a lobby ({lobby_id})!")
                ));
            }
        }

        let lobby = Lobby::new(
            Self::LOBBY_BROADCAST_CHANNEL_CAPACITY,
            self.player_registry.clone(),
            name,
            player_id,
        );
        let lobby_id = lobby.lobby_id;
        let lobby_public = lobby.clone().into();
        let lobby_arc = Arc::new(RwLock::new(lobby));

        lobby_arc.write().await.state_broadcaster.set_lobby(lobby_arc.clone());
        let mut lobby_registry_w = self.lobby_registry.write().await;
        let mut player_lobby_map_w = self.player_lobby_map.write().await;
        lobby_registry_w.add_lobby(lobby_id, lobby_arc).await?;
        player_lobby_map_w.insert(player_id, lobby_id);

        Ok(lobby_public)
    }

    pub async fn join_lobby_rpc(&self, lobby_id: Uuid, player_id: Uuid) -> Result<(), AppError> {
        {
            let player_lobby_map_r = self.player_lobby_map.read().await;
            if let Some(joined_lobby_id) = player_lobby_map_r.get(&player_id) {
                return if *joined_lobby_id == lobby_id {
                    Ok(())
                } else {
                    Err(AppError::precondition_failed(
                        format!(
                            "Player ({player_id}) already participating in a lobby ({joined_lobby_id})!"
                        ),
                    ))
                }
            }
        }

        let mut lobby_arc = {
            let lobby_registry_r = self.lobby_registry.read().await;
            lobby_registry_r
                .get_lobby_arc(&lobby_id)
                .await
                .ok_or(AppError::internal("Lobby doesn't exist!"))
        }?;
        let mut lobby_w = lobby_arc.write().await;

        if lobby_w.is_player(&player_id) {
            return Ok(());
        }

        lobby_w.add_player(player_id).await?;

        let mut player_lobby_map_w = self.player_lobby_map.write().await;
        player_lobby_map_w.insert(player_id, lobby_w.lobby_id);
        Ok(())
    }

    pub async fn leave_lobby_rpc(&self, player_id: Uuid) -> Result<(), AppError> {
        let lobby_id = {
            let player_lobby_map_r = self.player_lobby_map.read().await;
            player_lobby_map_r
                .get(&player_id)
                .map(|lobby_id| lobby_id.clone())
                .ok_or(
                    AppError::precondition_failed(
                        format!("Player ({player_id}) not participating in any lobbies!")
                    )
                )
        }?;

        let mut lobby_arc = {
            let lobby_registry_r = self.lobby_registry.read().await;
            lobby_registry_r
                .get_lobby_arc(&lobby_id)
                .await
                .ok_or(
                    AppError::internal("Incomplete state [DEBUG]") // TODO
                )
        }?;
        let mut lobby_w = lobby_arc.write().await;

        if lobby_w.is_host_player(&player_id) {
            // TODO: disband or pick next host (if any)...
        }

        // TODO: only check in_game(), otherwise force-disable matchmaking and leave
        lobby_w.lobby_locked_validation()?;

        lobby_w.rm_player(&player_id).await
    }

    // // TODO: restructure as non-rpc, allow users to call leave_lobby instead?
    // pub async fn remove_lobby_rpc(&self, player_id: Uuid) -> Result<(), AppError> {
    //     let lobby_id = {
    //         let player_lobby_map_r = self.player_lobby_map.read().await;
    //         player_lobby_map_r
    //             .get(&player_id)
    //             .map(|lobby_id| lobby_id.clone())
    //             .ok_or(
    //                 AppError::precondition_failed(
    //                 format!("Player ({player_id}) not participating in any lobbies!")
    //                 )
    //             )
    //     }?;
    //
    //     let lobby = {
    //         let lobby_registry_r = self.lobby_registry.read().await;
    //         let lobby_arc = lobby_registry_r
    //             .get_lobby_arc(&lobby_id)
    //             .await
    //             .ok_or(
    //                 AppError::internal("Incomplete state [DEBUG]") // TODO
    //             )?;
    //         let lobby = lobby_arc.read().await.clone();
    //         lobby
    //     };
    //
    //     if !lobby.is_host_player(&player_id) {
    //         return Err(AppError::unauthorized("Only the host player may remove a lobby!"));
    //     }
    //
    //     lobby.lobby_locked_validation()?;
    //
    //     let mut lobby_registry_w = self.lobby_registry.write().await;
    //     lobby_registry_w.rm_lobby(&lobby.lobby_id).await
    // }

    pub async fn get_lobbies_rpc(&self) -> Vec<LobbyInfoPublic> {
        let lobbies;
        {
            let lobby_registry_r = self.lobby_registry.read().await;
            lobbies = lobby_registry_r.get_lobbies().await;
        }
        lobbies
            .into_iter()
            .map(|lobby| lobby.into())
            .collect()
    }

    pub async fn set_lobby_matchmaking_status_rpc(
        &self,
        player_id: Uuid,
        matchmaking: bool,
    ) -> Result<(), AppError> {
        let lobby_id = {
            let player_lobby_map_r = self.player_lobby_map.read().await;
            player_lobby_map_r
                .get(&player_id)
                .map(|lobby_id| lobby_id.clone())
                .ok_or(
                    AppError::precondition_failed(
                        format!("Player ({player_id}) not participating in any lobbies!")
                    )
                )
        }?;

        let mut lobby_arc = {
            let lobby_registry_r = self.lobby_registry.read().await;
            lobby_registry_r
                .get_lobby_arc(&lobby_id)
                .await
                .ok_or(
                    AppError::internal("Incomplete state [DEBUG]") // TODO
                )
        }?;
        let mut lobby_w = lobby_arc.write().await;

        if !lobby_w.is_host_player(&player_id) {
            return Err(AppError::unauthorized("Only the host player may toggle matchmaking!"));
        }

        if lobby_w.is_in_game() {
            return Err(AppError::unauthorized("Cannot modify matchmaking status while in-game!"));
        }

        if matchmaking {
            lobby_w.start_matchmaking().await?;
        } else {
            lobby_w.stop_matchmaking().await;
        }
        Ok(())
    }

    pub async fn respond_lobby_matchmaking_rpc(
        &self,
        player_id: Uuid,
        acceptance: bool,
    ) -> Result<(), AppError> {
        let lobby_id = {
            let player_lobby_map_r = self.player_lobby_map.read().await;
            player_lobby_map_r
                .get(&player_id)
                .map(|lobby_id| lobby_id.clone())
                .ok_or(
                    AppError::precondition_failed(
                        format!("Player ({player_id}) not participating in any lobbies!")
                    )
                )
        }?;

        let mut lobby_arc = {
            let lobby_registry_r = self.lobby_registry.read().await;
            lobby_registry_r
                .get_lobby_arc(&lobby_id)
                .await
                .ok_or(
                    AppError::internal("Incomplete state [DEBUG]") // TODO
                )
        }?;
        let mut lobby_w = lobby_arc.write().await;

        if lobby_w.is_host_player(&player_id) {
            // TODO: disband or pick next host (if any)...
        } else if !lobby_w.is_player(&player_id) {
            return Err(AppError::internal("Incomplete state [DEBUG]")); // TODO
        }

        if !lobby_w.is_matchmaking() {
            return Err(AppError::precondition_failed("Lobby not currently matchmaking!"));
        }

        lobby_w.set_match_acceptance(player_id, acceptance).await
    }

    pub async fn start_lobby_game_rpc(&self, player_id: Uuid) -> Result<(), AppError> {
        let lobby_id = {
            let player_lobby_map_r = self.player_lobby_map.read().await;
            player_lobby_map_r
                .get(&player_id)
                .map(|lobby_id| lobby_id.clone())
                .ok_or(
                    AppError::precondition_failed(
                        format!("Player ({player_id}) not participating in any lobbies!")
                    )
                )
        }?;

        let mut lobby_arc = {
            let lobby_registry_r = self.lobby_registry.read().await;
            lobby_registry_r
                .get_lobby_arc(&lobby_id)
                .await
                .ok_or(
                    AppError::internal("Incomplete state [DEBUG]") // TODO
                )
        }?;

        {
            let mut lobby_w = lobby_arc.write().await;
            lobby_w.check_game_start_possible()?;
            if !lobby_w.is_host_player(&player_id) {
                return Err(AppError::unauthorized("Only the host player may initiate a game!"));
            }

            if lobby_w.check_game_start_possible().is_ok() {
                let player_registry_r = self.player_registry.read().await;
                let players = player_registry_r
                    .get_players(&lobby_w.player_ids)
                    .await?
                    .into_values()
                    .collect();
                lobby_w.start_match(players).await;
            }
        }

        Ok(())
    }

    pub async fn respond_betting_phase_rpc(
        &self,
        player_id: Uuid,
        betting_action: BettingRoundAction,
    ) -> Result<(), AppError> {
        let lobby_id = {
            let player_lobby_map_r = self.player_lobby_map.read().await;
            player_lobby_map_r
                .get(&player_id)
                .map(|lobby_id| lobby_id.clone())
                .ok_or(
                    AppError::precondition_failed(
                        format!("Player ({player_id}) not participating in any lobbies!")
                    )
                )
        }?;

        let mut lobby_arc = {
            let lobby_registry_r = self.lobby_registry.read().await;
            lobby_registry_r
                .get_lobby_arc(&lobby_id)
                .await
                .ok_or(
                    AppError::internal("Incomplete state [DEBUG]") // TODO
                )
        }?;

        {
            let mut lobby_w = lobby_arc.write().await;
            let Some(ref mut r#match) = lobby_w.r#match else {
                return Err(AppError::invalid_request("Lobby not currently in-game!"))
            };

            let mut game_phase_w = r#match.phase.write().await;
            game_phase_w.handle_betting_action(player_id, betting_action).await?;
        }

        Ok(())
    }

    pub async fn respond_drawing_phase_rpc(
        &self,
        player_id: Uuid,
        discarded_cards: Option<DiscardedCards>,
    ) -> Result<(), AppError> {
        let lobby_id = {
            let player_lobby_map_r = self.player_lobby_map.read().await;
            player_lobby_map_r
                .get(&player_id)
                .map(|lobby_id| lobby_id.clone())
                .ok_or(
                    AppError::precondition_failed(
                        format!("Player ({player_id}) not participating in any lobbies!")
                    )
                )
        }?;

        let mut lobby_arc = {
            let lobby_registry_r = self.lobby_registry.read().await;
            lobby_registry_r
                .get_lobby_arc(&lobby_id)
                .await
                .ok_or(
                    AppError::internal("Incomplete state [DEBUG]") // TODO
                )
        }?;

        {
            let mut lobby_w = lobby_arc.write().await;
            let Some(ref mut r#match) = lobby_w.r#match else {
                return Err(AppError::invalid_request("Lobby not currently in-game!"))
            };

            let mut game_phase_w = r#match.phase.write().await;
            game_phase_w.handle_drawing_action(player_id, discarded_cards).await?;
        }

        Ok(())
    }

    pub async fn watch_state_rpc(&self, player_id: Uuid) -> Result<AsyncStream<Result<GameStateAsPlayer, AppError>, impl Future<Output=()> + Sized>, AppError> {
        let Some(lobby) = self.get_player_lobby(&player_id).await else {
            return Err(AppError::precondition_failed("Player not currently participating in a lobby!"));
        };

        let mut receiver = lobby.state_broadcaster.subscribe();

        let stream = async_stream::try_stream! {
            // Stream Current State
            match lobby.state_broadcaster.build_state(None).await {
                Ok(state) => match state.as_player(player_id.clone()) {
                    Ok(state_as_player) => yield state_as_player,
                    Err(err) => eprintln!("{}", err),
                },
                Err(err) => eprintln!("{}", err),
            };

            // Stream Upcoming States
            while let Ok(state) = receiver.recv().await {
                let Some(state) = state else {
                    break;
                };

                if let Ok(game_state_as_player) = state.as_player(player_id) {
                    yield game_state_as_player;
                }
            }
        };
        Ok(stream)
    }
}

impl GameService {
    async fn get_player_lobby(&self, player_id: &Uuid) -> Option<Lobby> {
        let lobby_id;
        {
            let player_lobby_map_r = self.player_lobby_map.read().await;
            lobby_id = match player_lobby_map_r.get(player_id) {
                Some(id) => *id,
                None => return None,
            };
        }

        let lobby_registry_r = self.lobby_registry.read().await;
        let Some(lobby_arc) = lobby_registry_r.get_lobby_arc(&lobby_id).await else {
            let mut player_lobby_map_w = self.player_lobby_map.write().await;
            player_lobby_map_w.remove(player_id);
            return None;
        };
        let lobby = lobby_arc.read().await.clone();
        Some(lobby)
    }
}
