use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    borrow::Cow,
};
use rand::Rng;
use tonic::{Response, Status};
use crate::service::proto::{
    LobbyInfoPrivate,
    LobbyStatus,
    set_lobby_matchmaking_status_request::MatchmakingStatus,
};
use crate::{
    types::{
        lobby::Lobby,
        player::Player,
    },
    service::client::Client,
};

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
            Some(mut client) => {
                client.keep_alive();
                #[cfg(feature = "conn_logging")]
                println!("<3 @ {}", client.address);
                Ok(Response::new(()))
            },
            None => Err(Status::not_found("Client has disconnected!"))
        }
    }

    pub(super) fn create_lobby(&mut self, client_address: &String, lobby_name: String) -> Result<(), Status> {
        let Some(client) = self.clients.get(client_address) else {
            return Err(
                Status::failed_precondition(
                    "Client not registered! Calling connect() is a prerequisite!"
                )
            );
        };
        if self.user_in_lobby(&client.user.name) {
            return Err(Status::failed_precondition("User is already participating in another lobby!"));
        }
        let lobby_id = loop {
            let random_number: u32 = rand::thread_rng().gen_range(0..=99999999);
            let id = format!("{:08}", random_number);
            if !self.lobbies.contains_key(&id) {
                break id;
            }
        };
        let lobby = Lobby::new(lobby_id.clone(), lobby_name, client.user.clone());
        self.lobbies.insert(lobby_id, lobby.clone());
        let mut client = self.clients.get_mut(client_address).unwrap();
        client.lobby = Some(Arc::downgrade(&lobby));
        Ok(())
    }

    pub(super) fn join_lobby(&mut self, client_address: &String, lobby_id: &String) -> Result<(), Status> {
        let Some(mut client) = self.clients.get_mut(client_address) else {
            return Err(
                Status::failed_precondition(
                    "Client not registered! Calling connect() is a prerequisite!"
                )
            );
        };
        let mut lobby = self.lobbies.get_mut(lobby_id);
        let Some(lobby) = lobby else {
            return Err(Status::not_found("Lobby doesn't exist!"));
        };
        lobby.write().unwrap().add_player(
            Arc::new(
                Player {
                    user: Arc::downgrade(&client.user),
                    lobby: Arc::downgrade(lobby),
                }
            )
        )?;
        client.lobby = Some(Arc::downgrade(lobby));
        Ok(())
    }

    pub(super) fn leave_lobby(&mut self, client_address: &String) -> Result<(), Status> {
        let Some(mut client) = self.clients.get_mut(client_address) else {
            return Err(
                Status::failed_precondition(
                    "Client not registered! Calling connect() is a prerequisite!"
                )
            );
        };
        let Some(lobby) = &client.lobby else {
            return Err(
                Status::not_found(
                    "User isn't currently participating in a lobby!"
                )
            );
        };
        let lobby = lobby.upgrade().unwrap();
        lobby.write().unwrap().rm_player(&client.user.id)?;
        client.lobby = None;
        Ok(())
    }

    pub(super) fn kick_lobby_player(&mut self, client_address: &String, user_id: &String) -> Result<(), Status> {
        let Some(mut client) = self.clients.get_mut(client_address) else {
            return Err(
                Status::failed_precondition(
                    "Client not registered! Calling connect() is a prerequisite!"
                )
            );
        };
        let Some(lobby) = &client.lobby else {
            return Err(
                Status::not_found(
                    "User isn't currently participating in a lobby!"
                )
            );
        };
        let lobby = lobby.upgrade().unwrap();
        let lobby_r = lobby.read().unwrap();
        if lobby_r.host_user != client.user {
            return Err(
                Status::permission_denied(
                    "User isn't the lobby host!"
                )
            );
        }
        drop(lobby_r);
        if &client.user.id == user_id {
            return Err(
                Status::permission_denied(
                    "Can't kick yourself!"
                )
            );
        }
        let mut lobby_w = lobby.write().unwrap();
        lobby_w.rm_player(user_id)?;
        drop(lobby_w);
        if let Some(mut client) = self.clients.values_mut().find(|client| &client.user.id == user_id) {
            client.lobby = None;
        };
        Ok(())
    }

    pub(super) fn get_lobby_state(&self, client_address: &String) -> Result<Response<LobbyInfoPrivate>, Status> {
        let Some(client) = self.clients.get(client_address) else {
            return Err(
                Status::failed_precondition(
                    "Client not registered! Calling connect() is a prerequisite!"
                )
            );
        };
        let Some(lobby) = &client.lobby else {
            return Err(
                Status::not_found(
                    "User isn't currently participating in a lobby!"
                )
            );
        };
        let lobby = lobby.upgrade().unwrap();
        let lobby= &*lobby.read().unwrap();
        Ok(Response::new(lobby.into()))
    }

    pub(super) fn set_lobby_matchmaking_status(&mut self, client_address: &String, status: MatchmakingStatus) -> Result<Response<()>, Status> {
        let Some(client) = self.clients.get(client_address) else {
            return Err(
                Status::failed_precondition(
                    "Client not registered! Calling connect() is a prerequisite!"
                )
            );
        };
        let Some(lobby) = &client.lobby else {
            return Err(
                Status::not_found(
                    "User isn't currently participating in a lobby!"
                )
            );
        };
        let lobby = lobby.upgrade().unwrap();
        let mut lobby= lobby.write().unwrap();
        if lobby.host_user != client.user {
            return Err(
                Status::permission_denied(
                    "User isn't the lobby host!"
                )
            );
        }
        let status: LobbyStatus = status.into();
        if lobby.get_status() == &status {
            return Ok(Response::new(()));
        }
        match status {
            LobbyStatus::Idle => lobby.set_status_idle(),
            LobbyStatus::Matchmaking => lobby.set_status_matchmaking(),
            _ => unreachable!(),
        }?;
        Ok(Response::new(()))
    }

    pub(super) fn respond_matchmaking(&self, client_address: &String, accept: bool) -> Result<Response<()>, Status> {
        let Some(client) = self.clients.get(client_address) else {
            return Err(
                Status::failed_precondition(
                    "Client not registered! Calling connect() is a prerequisite!"
                )
            );
        };
        let Some(lobby) = &client.lobby else {
            return Err(
                Status::not_found(
                    "User isn't currently participating in a lobby!"
                )
            );
        };
        let lobby = lobby.upgrade().unwrap();
        let mut lobby= lobby.write().unwrap();
        lobby.set_matchmaking_acceptance(Cow::Borrowed(&client.user.name), accept)?;
        Ok(Response::new(()))
    }
}
