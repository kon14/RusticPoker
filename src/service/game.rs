use tonic::{Response, Status};
use crate::service::client::Client;

#[derive(Default)]
pub struct GameService {
    pub(super) clients: Vec<Client>,
}

impl GameService {
    fn get_client_player_name_by_address(&self, address: &String) -> Option<String> {
        self.clients
            .iter()
            .find(|client| &client.address == address)
            .map(|c| c.player_name.clone())
    }

    fn get_client_address_by_player_name(&self, player_name: &String) -> Option<String> {
        self.clients
            .iter()
            .find(|client| &client.player_name == player_name)
            .map(|c| c.address.clone())
    }

    pub(super) fn add_client(&mut self, client: Client) -> Result<(), Status> {
        if self.get_client_address_by_player_name(&client.player_name).is_some() {
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
}
