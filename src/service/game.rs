use std::sync::{Arc, RwLock};
use rand::Rng;
use tonic::{Response, Status};
use crate::{
    types::{
        lobby::Lobby,
        user::User,
    },
    service::client::Client,
};

#[derive(Default)]
pub struct GameService {
    pub(super) clients: Vec<Client>,
    pub(super) lobbies: Vec<Arc<RwLock<Lobby>>>,
}

impl GameService {
    fn get_client_player_name_by_address(&self, address: &String) -> Option<String> {
        self.clients
            .iter()
            .find(|client| &client.address == address)
            .map(|c| c.user.name.clone())
    }

    fn get_client_address_by_player_name(&self, player_name: &String) -> Option<String> {
        self.clients
            .iter()
            .find(|client| &client.user.name == player_name)
            .map(|c| c.address.clone())
    }

    fn get_user_by_address(&self, address: &String) -> Option<Arc<User>> {
        self.clients
            .iter()
            .find(|client| &client.address == address)
            .map(|client| client.user.clone())
    }

    fn user_in_lobby(&self, user_name: &String) -> bool {
        self.lobbies
            .iter()
            .any(|lobby| {
                let lobby = &*lobby.read().unwrap();
                lobby.has_player_name(user_name)
            })
    }

    pub(super) fn add_client(&mut self, client: Client) -> Result<(), Status> {
        if self.get_client_address_by_player_name(&client.user.name).is_some() {
            return Err(Status::already_exists("Player name already taken!"));
        }
        if let Some(player_name) = self.get_client_player_name_by_address(&client.address) {
            return Err(Status::already_exists(format!("Client already connected as \"{}\"!", player_name)));
        }
        self.clients.push(client);
        Ok(())
    }

    pub(super) fn rm_client(&mut self, address: &String) -> Result<(), Status> {
        if let Some(index) = self.clients.iter().position(|client| &client.address == address) {
            // TODO: Handle Lobby refs
            self.clients.remove(index);
            Ok(())
        } else {
            Err(Status::not_found("Client not connected!"))
        }
    }

    pub(super) fn heartbeat(&mut self, address: String) -> Result<Response<()>, Status> {
        let client = self.clients
            .iter_mut()
            .find(|client| client.address == address);
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

    pub(super) fn create_lobby(&mut self, client_address: &String, name: String) -> Result<Arc<RwLock<Lobby>>, Status> {
        let Some(user) = self.get_user_by_address(client_address) else {
            return Err(
                Status::failed_precondition(
                    "Client not registered! Calling connect() is a prerequisite!"
                )
            )
        };
        if self.user_in_lobby(&user.name) {
            return Err(Status::failed_precondition("User is already participating in another lobby!"))
        }
        let id = loop {
            let random_number: u32 = rand::thread_rng().gen_range(0..=99999999);
            let id = format!("{:08}", random_number);
            if self.lobbies.iter().all(|lobby| lobby.read().unwrap().id != id) {
                break id;
            }
        };
        let lobby = Lobby::new(id, name, user);
        self.lobbies.push(lobby.clone());
        Ok(lobby)
    }
}
