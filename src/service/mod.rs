mod client;
mod game;

pub(crate) mod proto {
    include!("../proto/gen/rustic_poker.rs");
    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] = include_bytes!("../proto/gen/rustic_poker_descriptor.bin");
}
pub(crate) use proto::{
    rustic_poker_server::RusticPokerServer,
    FILE_DESCRIPTOR_SET,
};

use std::{
    thread,
    sync::{Arc, Mutex},
    time::{SystemTime, Duration},
};
use rand::Rng;
use tonic::{Request, Response, Status, Streaming};
use proto::{
    rustic_poker_server::RusticPoker,
    RateHandsRequest,
    RateHandsResponse,
    ConnectRequest,
    GetLobbiesResponse,
    CreateLobbyRequest,
    JoinLobbyRequest,
    KickLobbyPlayerRequest,
    LobbyInfoPrivate,
    SetLobbyMatchmakingStatusRequest,
    set_lobby_matchmaking_status_request::MatchmakingStatus,
    RespondStartGameRequest,
};
use crate::types::hand::{Hand, RateHands};
use client::Client;
use game::GameService;

#[derive(Default)]
pub struct RusticPokerService {
    server: Arc<Mutex<GameService>>,
}

impl RusticPokerService {
    pub fn watch_clients_thread(&self) -> thread::JoinHandle<()> {
        let server = Arc::clone(&self.server);
        thread::spawn(move || {
            loop {
                let five_secs_ago = SystemTime::now() - Duration::from_secs(5);
                let server_r = server.lock().unwrap();
                let dropped_ips: Vec<String> = server_r
                    .clients
                    .values()
                    .filter(|client| client.last_heartbeat < five_secs_ago)
                    .map(|client| client.address.clone())
                    .collect();
                drop(server_r);
                let mut server_w = server.lock().unwrap();
                dropped_ips.iter().for_each(|ip| {
                    #[cfg(feature = "conn_logging")]
                    println!("Dropping inactive client @ {}", ip);
                    server_w.rm_client(ip).unwrap();
                });
                drop(server_w);
                thread::sleep(Duration::from_secs(3));
            }
        })
    }

    pub fn watch_lobbies_thread(&self) -> thread::JoinHandle<()> {
        let server = Arc::clone(&self.server);
        thread::spawn(move || {
            loop {
                let mut dropped_lobby_ids: Vec<String> = vec![];
                let server_r = server.lock().unwrap();
                let lobbies = server_r
                    .lobbies
                    .values()
                    .map(|lobby| lobby.read().unwrap());
                for lobby in lobbies {
                    if lobby.players.is_empty() {
                        dropped_lobby_ids.push(lobby.id.clone());
                        continue;
                    }
                }
                drop(server_r);
                let mut server_w = server.lock().unwrap();
                server_w.lobbies.retain(|id, _| !dropped_lobby_ids.contains(id));
                drop(server_w);
                thread::sleep(Duration::from_millis(500));
            }
        })
    }
}

macro_rules! extract_client_address {
    ($request:expr) => {
        match $request.remote_addr() {
            #[cfg(not(feature = "dbg_ignore_client_addr"))]
            Some(addr) => Ok(addr.to_string()),
            #[cfg(feature = "dbg_ignore_client_addr")]
            _ => Ok(String::from("0.0.0.0:55101")),
            None => Err(Status::invalid_argument("Unable to retrieve client address")),
        }
    };
}

#[tonic::async_trait]
impl RusticPoker for RusticPokerService {
    async fn rate_hands(&self, request: Request<RateHandsRequest>) -> Result<Response<RateHandsResponse>, Status> {
        let RateHandsRequest { hands } = request.into_inner();
        if hands.is_empty() {
            return Err(Status::new(tonic::Code::InvalidArgument, "No poker hands provided!"));
        }
        let hands: Result<Vec<Hand>, _> = hands
            .into_iter()
            .map(|h| h.as_str().try_into())
            .collect();
        let Ok(hands) = hands else {
            return Err(Status::new(tonic::Code::InvalidArgument, "Invalid poker hands!"));
        };
        let winners = hands.determine_winners()
            .into_iter()
            .map(|h| h.raw_hand_str)
            .collect();
        Ok(Response::new(RateHandsResponse { winners }))
    }

    async fn connect(&self, request: Request<ConnectRequest>) -> Result<Response<()>, Status> {
        let peer_address = extract_client_address!(request)?;
        let ConnectRequest { user_name } = request.into_inner();
        let user_id = loop {
            let random_number: u32 = rand::thread_rng().gen_range(0..=99999999);
            let id = format!("{:08}", random_number);
            if !self.server.lock().unwrap().clients.values().any(|client| client.user.id == id) {
                break id;
            }
        };
        let client = Client::new(peer_address, user_id, user_name);
        self.server.lock().unwrap().add_client(client)?;
        Ok(Response::new(()))
    }

    async fn disconnect(&self, request: Request<()>) -> Result<Response<()>, Status> {
        let peer_address = extract_client_address!(request)?;
        self.server.lock().unwrap().rm_client(&peer_address)?;
        Ok(Response::new(()))
    }

    async fn heartbeat(&self, request: Request<Streaming<()>>) -> Result<Response<()>, Status> {
        let peer_address = extract_client_address!(request)?;
        self.server.lock().unwrap().heartbeat(&peer_address)
    }

    async fn get_lobbies(&self, _: Request<()>) -> Result<Response<GetLobbiesResponse>, Status> {
        let lobbies = self.server
            .lock()
            .unwrap()
            .lobbies
            .values()
            .map(|lobby| (&*lobby.as_ref().read().unwrap()).into())
            .collect();
        Ok(Response::new(GetLobbiesResponse{ lobbies }))
    }

    async fn create_lobby(&self, request: Request<CreateLobbyRequest>) -> Result<Response<()>, Status> {
        let peer_address = extract_client_address!(request)?;
        let CreateLobbyRequest { lobby_name } = request.into_inner();
        self.server.lock().unwrap().create_lobby(&peer_address, lobby_name)?;
        Ok(Response::new(()))
    }

    async fn join_lobby(&self, request: Request<JoinLobbyRequest>) -> Result<Response<()>, Status> {
        let peer_address = extract_client_address!(request)?;
        let JoinLobbyRequest { lobby_id } = request.into_inner();
        self.server.lock().unwrap().join_lobby(&peer_address, &lobby_id)?;
        Ok(Response::new(()))
    }

    async fn leave_lobby(&self, request: Request<()>) -> Result<Response<()>, Status> {
        let peer_address = extract_client_address!(request)?;
        self.server.lock().unwrap().leave_lobby(&peer_address)?;
        Ok(Response::new(()))
    }

    async fn kick_lobby_player(&self, request: Request<KickLobbyPlayerRequest>) -> Result<Response<()>, Status> {
        let peer_address = extract_client_address!(request)?;
        let KickLobbyPlayerRequest { user_id } = request.into_inner();
        self.server.lock().unwrap().kick_lobby_player(&peer_address, &user_id)?;
        Ok(Response::new(()))
    }

    async fn get_lobby_state(&self, request: Request<()>) -> Result<Response<LobbyInfoPrivate>, Status> {
        let peer_address = extract_client_address!(request)?;
        self.server.lock().unwrap().get_lobby_state(&peer_address)
    }

    async fn set_lobby_matchmaking_status(&self, request: Request<SetLobbyMatchmakingStatusRequest>) -> Result<Response<()>, Status> {
        let peer_address = extract_client_address!(request)?;
        let SetLobbyMatchmakingStatusRequest { status } = request.into_inner();
        let Ok(status) = MatchmakingStatus::try_from(status) else {
            return Err(Status::invalid_argument("Invalid status value"));
        };
        self.server.lock().unwrap().set_lobby_matchmaking_status(&peer_address, status)
    }

    async fn respond_matchmaking(&self, request: Request<RespondStartGameRequest>) -> Result<Response<()>, Status> {
        let peer_address = extract_client_address!(request)?;
        let RespondStartGameRequest { accept } = request.into_inner();
        self.server.lock().unwrap().respond_matchmaking(&peer_address, accept)
    }
}
