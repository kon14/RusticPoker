use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;
use async_stream::__private::AsyncStream;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

use crate::common::error::AppError;
use crate::game::state::{GameStateAsPlayer, LobbyStatus, PlayerState};
use crate::lobby::{Lobby, LobbyRegistry};
use crate::player::{Player, PlayerRegistry};
use crate::r#match::{Match, MatchRegistry};
use crate::service::proto;
use super::GameState;

pub struct GameService {
    lobby_registry: Arc<RwLock<LobbyRegistry>>,
    match_registry: Arc<RwLock<MatchRegistry>>,
    player_registry: Arc<RwLock<PlayerRegistry>>,
    player_lobby_map: Arc<RwLock<HashMap<Uuid, Uuid>>>,
    lobby_broadcast_channels: Arc<RwLock<HashMap<Uuid, broadcast::Sender<Option<GameState>>>>>,
}

impl GameService {
    const LOBBY_BROADCAST_CHANNEL_CAPACITY: usize = 10;

    pub fn new(broadcaster_capacity: usize) -> Self {
        let lobby_registry = LobbyRegistry::new(broadcaster_capacity);
        let match_registry = MatchRegistry::new(broadcaster_capacity);
        let player_registry = PlayerRegistry::new(broadcaster_capacity);
        let player_lobby_map: HashMap<Uuid, Uuid> = HashMap::new();

        GameService {
            lobby_registry: Arc::new(RwLock::new(lobby_registry)),
            match_registry: Arc::new(RwLock::new(match_registry)),
            player_registry: Arc::new(RwLock::new(player_registry)),
            player_lobby_map: Arc::new(RwLock::new(player_lobby_map)),
            lobby_broadcast_channels: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

// RPCs
impl GameService {
    pub async fn connect_rpc(&self) -> Result<(Uuid), AppError> {
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

    pub async fn create_lobby_rpc(&self, name: String, player_id: Uuid) -> Result<(LobbyInfoPublic), AppError> {
        {
            let player_lobby_map_r = self.player_lobby_map.read().await;
            if let Some(lobby_id) = player_lobby_map_r.get(&player_id) {
                return Err(AppError::precondition_failed(
                    format!("Player ({player_id}) already participating in a lobby ({lobby_id})!")
                ));
            }
        }

        let lobby = Lobby::new(name, player_id);
        let mut lobby_registry_w = self.lobby_registry.write().await;
        let mut player_lobby_map_w = self.player_lobby_map.write().await;
        player_lobby_map_w.insert(player_id, lobby.lobby_id);
        lobby_registry_w.add_lobby(lobby.clone()).await?;

        let lobby_broadcaster = {
            let (lobby_broadcaster, _) = broadcast::channel(Self::LOBBY_BROADCAST_CHANNEL_CAPACITY);
            let mut lobby_broadcast_channels_w = self.lobby_broadcast_channels.write().await;
            lobby_broadcast_channels_w.insert(lobby.lobby_id, lobby_broadcaster.clone());
            lobby_broadcaster
        };
        self.spawn_game_task(lobby.lobby_id, lobby_broadcaster);

        Ok(lobby.into())
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

        lobby_w.add_player(player_id)?;

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

        lobby_w.rm_player(&player_id)
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
            lobby_w.start_matchmaking()?;
        } else {
            lobby_w.clear_matchmaking();
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

        lobby_w.set_match_acceptance(player_id, acceptance)
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
            let lobby_r = lobby_arc.read().await;
            lobby_r.check_game_start_possible()?;
            if !lobby_r.is_host_player(&player_id) {
                return Err(AppError::unauthorized("Only the host player may initiate a game!"));
            }
        }

        let match_init = {
            let lobby_r = lobby_arc.read().await;
            if lobby_r.check_game_start_possible().is_ok() {
                Some(lobby_r.start_match())
            } else {
                None
            }
        };
        if let Some((lobby, r#match)) = match_init {
            let mut match_registry_w = self.match_registry.write().await;
            match_registry_w.add_match(r#match).await?;
            let mut lobby_w = lobby_arc.write().await;
            *lobby_w = lobby;
        };
        Ok(())
    }

    pub async fn watch_state_rpc(&self, player_id: Uuid) -> Result<AsyncStream<Result<GameStateAsPlayer, AppError>, impl Future<Output=()> + Sized>, AppError> {
        let Some(lobby) = self.get_player_lobby(&player_id).await else {
            return Err(AppError::precondition_failed("Player not currently participating in a lobby!"));
        };

        let lobby_broadcast_channels_r = self.lobby_broadcast_channels.read().await;
        let Some(lobby_broadcaster) = lobby_broadcast_channels_r.get(&lobby.lobby_id) else {
            return Err(AppError::internal("Match information lost! [DEBUG]")); // TODO
        };

        let mut receiver = lobby_broadcaster.subscribe();

        let stream = async_stream::try_stream! {
            while let Ok(state) = receiver.recv().await {
                let Some(state) = state else {
                    break;
                };

                if let Ok(game_state_as_player) = state.as_player(&player_id) {
                    yield game_state_as_player;
                }
            }
        };
        Ok(stream)
    }
}

impl GameService {
    fn spawn_game_task(
        &self,
        lobby_id: Uuid,
        lobby_broadcaster: broadcast::Sender<Option<GameState>>,
    ) {
        const INTERVAL_MS: u64 = 500;

        let lobby_registry_arc = Arc::clone(&self.lobby_registry);
        let match_registry_arc = Arc::clone(&self.match_registry);
        let player_registry_arc = Arc::clone(&self.player_registry);

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(core::time::Duration::from_millis(INTERVAL_MS));

            loop {
                let lobby_registry_r = lobby_registry_arc.read().await;

                let lobby_arc = lobby_registry_r.get_lobby_arc(&lobby_id).await;
                let Some(lobby_arc) = lobby_arc else {
                    break;
                };

                let game_state = GameService::game_loop(
                    lobby_arc,
                    lobby_registry_arc.clone(),
                    match_registry_arc.clone(),
                    player_registry_arc.clone(),
                ).await.unwrap(); // TODO: handle result

                if let Err(err) = lobby_broadcaster.send(Some(game_state)) {
                    eprintln!("Failed to broadcast GameState: {}", err); // TODO
                }

                interval.tick().await;
            }
        });
    }

    async fn game_loop(
        lobby_arc: Arc<RwLock<Lobby>>,
        lobby_registry: Arc<RwLock<LobbyRegistry>>,
        match_registry: Arc<RwLock<MatchRegistry>>,
        player_registry: Arc<RwLock<PlayerRegistry>>,
    ) -> Result<GameState, AppError> {
        // Perform Early Checks
        let match_arc = {
            let mut lobby = {
                let lobby_r = lobby_arc.read().await;
                // TODO: lobby-disbanded struct flag, handle here and kick players
                lobby_r.clone()
            };
            let match_arc = match lobby.is_in_game_then_id() {
                None => Ok(None),
                Some(match_id) => {
                    let match_registry_r = match_registry.read().await;
                    let match_arc = match_registry_r
                        .get_match_arc(&match_id)
                        .await
                        .ok_or(AppError::internal("Match information lost! [DEBUG]"))?;
                    Ok(Some(match_arc))
                }
            }?;
            match_arc
        };

        // Handle Game Logic
        if let Some(match_arc) = match_arc.clone() {
            Self::game_loop_in_game(
                lobby_arc.clone(),
                match_arc.clone(),
                lobby_registry.clone(),
                match_registry.clone(),
            ).await?;
        };

        // Retrieve Fresh Data
        let lobby = lobby_arc.read().await.clone();
        let r#match = if let Some(match_arc) = match_arc {
            let r#match = match_arc.read().await.clone();
            Some(r#match)
        } else {
            None
        };

        // Build Representational State
        let player_states = Self::build_player_states(
            &lobby,
            player_registry.clone(),
        ).await?;
        let lobby_state = lobby.into();
        let match_state = r#match.map(|r#match| r#match.into());

        let game_state = GameState::build(
            player_states,
            lobby_state,
            match_state,
        );
        Ok(game_state)
    }

    async fn build_player_states(
        lobby: &Lobby,
        player_registry: Arc<RwLock<PlayerRegistry>>,
    ) -> Result<HashMap<Uuid, PlayerState>, AppError> {
        let player_ids = lobby.player_ids.clone();
        let player_registry_r = player_registry.read().await;
        let players = player_registry_r.get_players(&player_ids).await?;
        let players = players
            .into_iter()
            .map(|(player_id, player)| (player_id, player.into()))
            .collect();
        Ok(players)
    }

    async fn game_loop_in_game(
        lobby_arc: Arc<RwLock<Lobby>>,
        match_arc: Arc<RwLock<Match>>,
        lobby_registry: Arc<RwLock<LobbyRegistry>>,
        match_registry: Arc<RwLock<MatchRegistry>>,
    ) -> Result<(), AppError> {
        // TODO: In-Game Logic
        Ok(())
    }

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

impl Default for GameService {
    fn default() -> Self {
        Self::new(10)
    }
}

// TODO: mv

pub struct LobbyInfoPublic {
    id: Uuid,
    name: String,
    host_player_id: Uuid,
    player_count: u32,
    status: LobbyStatus,
}

impl From<Lobby> for LobbyInfoPublic {
    fn from(lobby: Lobby) -> Self {
        let status = LobbyStatus::from(&lobby);
        LobbyInfoPublic {
            id: lobby.lobby_id,
            name: lobby.name,
            host_player_id: lobby.host_player_id,
            player_count: lobby.player_ids.len().try_into().unwrap(),
            status,
        }
    }
}

impl From<LobbyInfoPublic> for proto::LobbyInfoPublic {
    fn from(lobby_info: LobbyInfoPublic) -> Self {
        proto::LobbyInfoPublic {
            id: lobby_info.id.to_string(),
            name: lobby_info.name,
            host_player_id: lobby_info.host_player_id.to_string(),
            player_count: lobby_info.player_count,
            status: lobby_info.status as i32,
        }
    }
}
