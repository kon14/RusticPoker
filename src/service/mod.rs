pub(crate) mod proto {
    include!("../proto/rustic_poker.rs");
    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] = include_bytes!("../proto/rustic_poker_descriptor.bin");
}
pub(crate) use proto::{
    rustic_poker_server::RusticPokerServer,
    FILE_DESCRIPTOR_SET,
};

use std::sync::Arc;
use std::collections::HashMap;
use std::ops::Deref;
use std::pin::Pin;
use futures::Stream;
use tokio::sync::RwLock;
use rand::Rng;
use tonic::{Request, Response, Status};
use uuid::Uuid;
use futures::stream::{StreamExt, TryStreamExt};

use crate::common::error::AppError;
use crate::game::{DiscardedCards, GameService};

#[derive(Default)]
pub struct RusticPokerService {
    game_service: GameService,
    player_connections: Arc<RwLock<HashMap<PeerAddress, Uuid>>>,
}

macro_rules! extract_client_address {
    ($request:expr) => {
        {
            #[cfg(feature = "dbg_peer_addr_spoofing")]
            let peer_address = $request.metadata().get("peer-address")
                .map(|addr| addr.to_str().ok().map(|addr| addr.to_string()))
                .flatten()
                .or($request.remote_addr().map(|addr| addr.to_string()))
                .map(|addr| PeerAddress(addr.to_string()));
            #[cfg(not(feature = "dbg_peer_addr_spoofing"))]
            let peer_address = $request.
                remote_addr()
                .map(|addr| PeerAddress(addr.to_string()));
            match peer_address {
                Some(addr) => Ok(addr),
                None => Err(Status::invalid_argument("Unable to retrieve client address!")),
            }
        }
    };
}

macro_rules! get_player_id {
    ($self:ident, $peer_address:expr) => {
        {
            let player_conns_r = $self.player_connections.read().await;
            match player_conns_r.get($peer_address) {
                Some(player_id) => Ok(player_id.clone()),
                None => Err(Status::failed_precondition("Client not registered. Use Connect() RPC.")),
            }
        }
    };
}

#[tonic::async_trait]
impl proto::rustic_poker_server::RusticPoker for RusticPokerService {
    type WatchStateStream = Pin<Box<dyn Stream<Item=Result<proto::GameState, Status>> + Send>>;

    async fn connect(&self, request: Request<proto::ConnectRequest>) -> Result<Response<()>, Status> {
        let peer_address = extract_client_address!(request)?;

        let mut player_connections_w = self.player_connections.write().await;
        if player_connections_w.contains_key(&peer_address) {
            return Ok(Response::new(()));
        }

        let player_id = self.game_service.connect_rpc().await?;
        player_connections_w.insert(peer_address, player_id);
        Ok(Response::new(()))
    }

    async fn disconnect(&self, request: Request<()>) -> Result<Response<()>, Status> {
        let peer_address = extract_client_address!(request)?;

        let mut player_connections_r = self.player_connections.read().await;
        let Some(player_id) = player_connections_r.get(&peer_address) else {
            return Err(Status::aborted("No active client connections!"));
        };

        self.game_service.disconnect_rpc(player_id).await?;
        Ok(Response::new(()))
    }

    async fn get_lobbies(&self, _: Request<()>) -> Result<Response<proto::GetLobbiesResponse>, Status> {
        let lobbies = self.game_service
            .get_lobbies_rpc()
            .await
            .into_iter()
            .map(|lobby| lobby.into())
            .collect();
        Ok(Response::new(proto::GetLobbiesResponse{ lobbies }))
    }

    // TODO: return LobbyInfoPrivate instead
    async fn create_lobby(&self, request: Request<proto::CreateLobbyRequest>) -> Result<Response<(proto::LobbyInfoPublic)>, Status> {
        let peer_address = extract_client_address!(request)?;
        let player_id = get_player_id!(self, &peer_address)?;
        let proto::CreateLobbyRequest { lobby_name } = request.into_inner();

        let lobby = self.game_service.create_lobby_rpc(lobby_name, player_id).await?;
        Ok(Response::new(lobby.into()))
    }

    async fn join_lobby(&self, request: Request<proto::JoinLobbyRequest>) -> Result<Response<()>, Status> {
        let peer_address = extract_client_address!(request)?;
        let player_id = get_player_id!(self, &peer_address)?;
        let proto::JoinLobbyRequest { lobby_id } = request.into_inner();

        let lobby_id = Uuid::parse_str(&lobby_id)
            .map_err(|_|
                Status::invalid_argument("JoinLobbyRequest.lobby_id should be a UUID (v4)!")
            )?;
        self.game_service.join_lobby_rpc(lobby_id, player_id).await?;
        Ok(Response::new(()))
    }

    async fn leave_lobby(&self, request: Request<()>) -> Result<Response<()>, Status> {
        let peer_address = extract_client_address!(request)?;
        let player_id = get_player_id!(self, &peer_address)?;

        self.game_service.leave_lobby_rpc(player_id).await?;
        Ok(Response::new(()))
    }

    // async fn kick_lobby_player(&self, request: Request<KickLobbyPlayerRequest>) -> Result<Response<()>, Status> {
    //     let peer_address = extract_client_address!(request)?;
    //     let KickLobbyPlayerRequest { user_id } = request.into_inner();
    //     self.server.lock().unwrap().kick_lobby_player(&peer_address, &user_id)?;
    //     Ok(Response::new(()))
    // }

    async fn set_lobby_matchmaking_status(&self, request: Request<proto::SetLobbyMatchmakingStatusRequest>) -> Result<Response<()>, Status> {
        let peer_address = extract_client_address!(request)?;
        let player_id = get_player_id!(self, &peer_address)?;
        let proto::SetLobbyMatchmakingStatusRequest { status } = request.into_inner();

        let status = proto::set_lobby_matchmaking_status_request::MatchmakingStatus::try_from(status)
            .map_err(|_| Status::invalid_argument("Invalid MatchmakingStatus value"))?;
        let matchmaking = status == proto::set_lobby_matchmaking_status_request::MatchmakingStatus::Matchmaking;
        self.game_service.set_lobby_matchmaking_status_rpc(player_id, matchmaking).await?;
        Ok(Response::new(()))
    }

    async fn respond_lobby_matchmaking(&self, request: Request<proto::RespondLobbyMatchmakingRequest>) -> Result<Response<()>, Status> {
        let peer_address = extract_client_address!(request)?;
        let player_id = get_player_id!(self, &peer_address)?;
        let proto::RespondLobbyMatchmakingRequest { decision } = request.into_inner();

        let decision = proto::respond_lobby_matchmaking_request::MatchmakingDecision::try_from(decision)
            .map_err(|_| Status::invalid_argument("Invalid MatchmakingDecision value provided!"))?;
        let acceptance = decision == proto::respond_lobby_matchmaking_request::MatchmakingDecision::Accept;
        self.game_service.respond_lobby_matchmaking_rpc(player_id, acceptance).await?;
        Ok(Response::new(()))
    }

    async fn start_lobby_game(&self, request: Request<()>) -> Result<Response<()>, Status> {
        let peer_address = extract_client_address!(request)?;
        let player_id = get_player_id!(self, &peer_address)?;

        self.game_service.start_lobby_game_rpc(player_id).await?;
        Ok(Response::new(()))
    }

    async fn respond_betting_phase(&self, request: Request<proto::RespondBettingPhaseRequest>) -> Result<Response<()>, Status> {
        let peer_address = extract_client_address!(request)?;
        let player_id = get_player_id!(self, &peer_address)?;
        let proto::RespondBettingPhaseRequest { betting_action } = request.into_inner();
        let betting_action = betting_action
            .ok_or(AppError::invalid_request("No BettingAction specified!"))?
            .into();

        self.game_service.respond_betting_phase_rpc(player_id, betting_action).await?;
        Ok(Response::new(()))
    }

    async fn respond_drawing_phase(&self, request: Request<proto::RespondDrawingPhaseRequest>) -> Result<Response<()>, Status> {
        let peer_address = extract_client_address!(request)?;
        let player_id = get_player_id!(self, &peer_address)?;
        let discarded_cards = DiscardedCards::try_from_proto(request.into_inner())?;

        self.game_service.respond_drawing_phase_rpc(player_id, discarded_cards).await?;
        Ok(Response::new(()))
    }

    async fn watch_state(&self, request: Request<()>) -> Result<Response<Self::WatchStateStream>, Status> {
        let peer_address = extract_client_address!(request)?;
        let player_id = get_player_id!(self, &peer_address)?;

        let stream = self.game_service
            .watch_state_rpc(player_id)
            .await?
            .map_ok(|game_state| proto::GameState::from(game_state))
            .map_err(|err| err.into());
        Ok(Response::new(Box::pin(stream) as Self::WatchStateStream))
    }
}

#[derive(Eq, PartialEq, Hash)]
pub struct PeerAddress(String);

impl Deref for PeerAddress {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for PeerAddress {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<String> for PeerAddress {
    fn from(s: String) -> Self {
        PeerAddress(s)
    }
}
