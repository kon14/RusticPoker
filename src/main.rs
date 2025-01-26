mod common;
mod game;
mod lobby;
mod r#match;
mod player;
mod service;
mod types;

use std::env;
use tonic::transport::Server;
use service::{RusticPokerService, RusticPokerServer, FILE_DESCRIPTOR_SET};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port = env::var("GRPC_PORT").unwrap_or(String::from("55100"));
    let address = format!("0.0.0.0:{}", port).parse().unwrap();
    let rustic_poker_service = RusticPokerService::default();
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
        .build_v1()
        .unwrap();

    let server = Server::builder()
        .add_service(RusticPokerServer::new(rustic_poker_service))
        .add_service(reflection_service)
        .serve(address);
    println!("RusticPoker gRPC service running at 0.0.0.0:{}", port);
    server.await?;

    Ok(())
}
