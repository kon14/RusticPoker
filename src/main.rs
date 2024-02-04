mod types;
mod proto {
    include!("./proto/gen/rustic_poker.rs");
    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] = include_bytes!("proto/gen/rustic_poker_descriptor.bin");
}

use std::env;
use tonic::{transport::Server, Request, Response, Status};
use proto::{
    rustic_poker_server::{RusticPoker, RusticPokerServer},
    RateHandsRequest,
    RateHandsResponse,
    FILE_DESCRIPTOR_SET,
};
use crate::types::hand::{Hand, RateHands};

#[derive(Debug, Default)]
pub struct RusticPokerService {}

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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port = env::var("GRPC_PORT").unwrap_or(String::from("55100"));
    let address = format!("0.0.0.0:{}", port).parse().unwrap();
    let rustic_poker_service = RusticPokerService::default();
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
        .build()
        .unwrap();
    let server = Server::builder()
        .add_service(RusticPokerServer::new(rustic_poker_service))
        .add_service(reflection_service)
        .serve(address);
    println!("RusticPoker gRPC server running at 0.0.0.0:{}", port);
    server.await?;
    Ok(())
}
