mod types;
mod proto {
    include!("./proto/gen/rustic_poker.rs");
}

use std::env;
use tonic::{transport::Server, Request, Response, Status};
use proto::{
    rustic_poker_server::{RusticPoker, RusticPokerServer},
    RateHandsRequest,
    RateHandsResponse,
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
    let server = Server::builder()
        .add_service(RusticPokerServer::new(rustic_poker_service))
        .serve(address);
    println!("RusticPoker gRPC server running at 0.0.0.0:{}", port);
    server.await?;
    Ok(())
}
