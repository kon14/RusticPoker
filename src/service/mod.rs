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
use tonic::{Request, Response, Status, Streaming};
use proto::{
    rustic_poker_server::RusticPoker,
    RateHandsRequest,
    RateHandsResponse,
    ConnectRequest,
    GetLobbiesResponse,
    CreateLobbyRequest,
    CreateLobbyResponse,
};
use crate::types::hand::{Hand, RateHands};
use client::Client;
use game::GameService;
use crate::service::proto::LobbyInfoPrivate;

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
                    .iter()
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
        let ConnectRequest { player_name } = request.into_inner();
        let client = Client::new(peer_address, player_name);
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
        self.server.lock().unwrap().heartbeat(peer_address)
    }

    async fn get_lobbies(&self, _: Request<()>) -> Result<Response<GetLobbiesResponse>, Status> {
        let lobbies = self.server
            .lock()
            .unwrap()
            .lobbies
            .iter()
            .map(|lobby| (&*lobby.as_ref().read().unwrap()).into())
            .collect();
        Ok(Response::new(GetLobbiesResponse{ lobbies }))
    }

    async fn create_lobby(&self, request: Request<CreateLobbyRequest>) -> Result<Response<CreateLobbyResponse>, Status> {
        let peer_address = extract_client_address!(request)?;
        let CreateLobbyRequest { name } = request.into_inner();
        let lobby = self.server
            .lock()
            .unwrap()
            .create_lobby(&peer_address, name)?;
        let lobby = &*lobby.as_ref().read().unwrap();
        let lobby: LobbyInfoPrivate = lobby.into();
        // https://github.com/protocolbuffers/protobuf/issues/249
        let lobby = Some(lobby);
        Ok(Response::new(CreateLobbyResponse{ lobby }))
    }
}
