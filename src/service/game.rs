use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use rand::Rng;
use tonic::{Response, Status};
use crate::{
    types::{
        lobby::Lobby,
        player::Player,
    },
    service::client::Client,
};
use crate::service::proto::LobbyInfoPrivate;

#[derive(Default)]
pub struct GameService {
    pub(super) clients: HashMap<String, Client>, // address -> Client
    pub(super) lobbies: HashMap<String, Arc<RwLock<Lobby>>>, // id -> Lobby
}

impl GameService {
    fn user_in_lobby(&self, user_name: &String) -> bool {
        self.lobbies
            .values()
            .any(|lobby| {
                let lobby = &*lobby.read().unwrap();
                lobby.has_player_name(user_name)
            })
    }

    pub(super) fn add_client(&mut self, client: Client) -> Result<(), Status> {
        if self.clients.values().any(|known_client| known_client.user.name == client.user.name) {
            return Err(Status::already_exists("Player name already taken!"));
        }
        if let Some(existing_client) = self.clients.get(&client.address) {
            return Err(Status::already_exists(format!("Client already connected as \"{}\"!", existing_client.user.name)));

        };
        self.clients.insert(client.address.clone(), client);
        Ok(())
    }

    pub(super) fn rm_client(&mut self, address: &String) -> Result<(), Status> {
        if self.clients.remove(address).is_some() {
            // TODO: Handle Lobby refs
            Ok(())
        } else {
            Err(Status::not_found("Client not connected!"))
        }
    }

    pub(super) fn heartbeat(&mut self, address: &String) -> Result<Response<()>, Status> {
        let mut client = self.clients.get_mut(address);
        match client {
            Some(client) => {
                client.keep_alive();
                #[cfg(feature = "conn_logging")]
                println!("<3 @ {}", client.address);
                Ok(Response::new(()))
            },
            None => Err(Status::not_found("Client has disconnected!"))
        }
    }

    pub(super) fn create_lobby(&mut self, client_address: &String, name: String) -> Result<(), Status> {
        let Some(client) = self.clients.get(client_address) else {
            return Err(
                Status::failed_precondition(
                    "Client not registered! Calling connect() is a prerequisite!"
                )
            )
        };
        if self.user_in_lobby(&client.user.name) {
            return Err(Status::failed_precondition("User is already participating in another lobby!"))
        }
        let id = loop {
            let random_number: u32 = rand::thread_rng().gen_range(0..=99999999);
            let id = format!("{:08}", random_number);
            if !self.lobbies.contains_key(&id) {
                break id;
            }
        };
        let lobby = Lobby::new(id.clone(), name, client.user.clone());
        self.lobbies.insert(id, lobby.clone());
        let mut client = self.clients.get_mut(client_address).unwrap();
        client.lobby = Some(Arc::downgrade(&lobby));
        Ok(())
    }

    pub(super) fn join_lobby(&mut self, client_address: &String, id: &String) -> Result<(), Status> {
        let Some(mut client) = self.clients.get_mut(client_address) else {
            return Err(
                Status::failed_precondition(
                    "Client not registered! Calling connect() is a prerequisite!"
                )
            )
        };
        let mut lobby = self.lobbies.get_mut(id);
        let Some(lobby) = lobby else {
            return Err(Status::not_found("Lobby doesn't exist!"));
        };
        lobby.write().unwrap().add_player(
            Arc::new(
                Player {
                    user: Arc::downgrade(&client.user),
                    lobby: Arc::downgrade(&lobby),
                }
            )
        )?;
        client.lobby = Some(Arc::downgrade(lobby));
        Ok(())
    }

    pub(super) fn get_lobby_state(&self, client_address: &String) -> Result<Response<LobbyInfoPrivate>, Status> {
        let Some(client) = self.clients.get(client_address) else {
            return Err(
                Status::failed_precondition(
                    "Client not registered! Calling connect() is a prerequisite!"
                )
            )
        };
        let Some(lobby) = &client.lobby else {
            return Err(
                Status::not_found(
                    "User isn't currently participating in a lobby!"
                )
            )
        };
        let lobby = lobby.upgrade().unwrap();
        let lobby= &*lobby.read().unwrap();
        Ok(Response::new(lobby.into()))
    }
}
