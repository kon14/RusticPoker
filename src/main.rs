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
use crate::types::hand::Hand;

#[derive(Debug, Default)]
pub struct RusticPokerService {}

#[tonic::async_trait]
impl RusticPoker for RusticPokerService {
    async fn rate_hands(&self, request: Request<RateHandsRequest>) -> Result<Response<RateHandsResponse>, Status> {
        let RateHandsRequest { hands } = request.into_inner();
        if hands.len() == 0 {
            return Err(Status::new(tonic::Code::InvalidArgument, "No poker hands provided!"));
        }
        let hands: Result<Vec<Hand>, _> = hands
            .into_iter()
            .map(|h| h.as_str().try_into())
            .collect();
        let Ok(mut hands) = hands else {
            return Err(Status::new(tonic::Code::InvalidArgument, "Invalid poker hands!"));
        };
        hands.sort_by(|a, b| b.cmp(a));
        let top_hand = hands[0].clone();
        let mut top_hands: Vec<Hand> = vec![];
        for hand in hands.into_iter() {
            if hand == top_hand {
                top_hands.push(hand);
            }
        }
        let winners = top_hands.into_iter().map(|h| h.raw_hand_str).collect();
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
