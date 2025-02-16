mod poker;
mod progression;

pub(crate) use poker::BettingRoundAction;

use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use chrono::{DateTime, Utc};
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

use crate::r#match::MatchStartPlayers;
use crate::types::deck::CardDeck;
use crate::types::hand::{Hand, RateHands};
use crate::game::GameTable;
use crate::output::GameStateBroadcaster;
use poker::*;
use crate::common::error::AppError;

#[derive(Clone, Debug)]
pub struct GamePhase {
    poker_phase: PokerPhase,
    state_time: DateTime<Utc>,
    state_broadcaster: GameStateBroadcaster
}

impl GamePhase {
    pub fn new(
        match_id: Uuid,
        state_broadcaster: GameStateBroadcaster,
        rpc_action_broadcaster: broadcast::Sender<()>,
        players: MatchStartPlayers,
        ante_amount: u64,
    ) -> Self {
        let game_table = GameTable::new(match_id, players);
        let card_deck = CardDeck::default();
        let poker_phase = PokerPhase::new(
            rpc_action_broadcaster,
            game_table,
            card_deck,
            ante_amount);

        GamePhase {
            poker_phase,
            state_time: Utc::now(),
            state_broadcaster,
        }
    }

    pub async fn progress(phase_arc: Arc<RwLock<GamePhase>>, mut rpc_action_receiver: broadcast::Receiver<()>) {
        let mut first_run = true;
        loop {
            // Contemplate Life Choices
            if first_run {
                first_run = false;
            } else {
                let progression = {
                    let mut phase_w = phase_arc.write().await;
                    phase_w.poker_phase.get_action_progression()
                };
                if let Some(progression) = progression {
                    progression.await_next_action(&mut rpc_action_receiver).await;
                } else {
                    // No more progressions...
                    break;
                }
            }

            // Handle Game Logic
            let state_broadcaster = {
                let mut phase_w = phase_arc.write().await;
                phase_w.state_time = Utc::now();
                phase_w.poker_phase.act();

                phase_w.state_broadcaster.clone()
            };

            // Build & Publish State
            state_broadcaster.publish().await;

            // Handle State Progression
            {
                let mut phase_w = phase_arc.write().await;
                if phase_w.poker_phase.is_phase_completed() {
                    if let Some(next_phase) = phase_w.poker_phase.clone().next_phase() {
                        phase_w.poker_phase = next_phase
                    } else {
                        // TODO: Game Over - Cleanup
                        // TODO: handle this via ActionProgression or sth.
                        return;
                    }
                }
            }
        }

        // TODO
        // Player Actions Also Call Progress! Replace Scheduled Run / Respect Sleep Timer
    }

    pub async fn handle_betting_action(
        &mut self,
        player_id: Uuid,
        betting_action: BettingRoundAction,
    ) -> Result<(), AppError> {
        self.poker_phase.handle_betting_action(player_id, betting_action).await
    }

    pub async fn handle_drawing_action(
        &mut self,
        player_id: Uuid,
        // drawing_action: ,
    ) -> Result<(), AppError> {
        self.poker_phase.handle_drawing_action(player_id, ).await
    }

    // TODO: build inner state internally, grab atomically - re-downgrade PokerPhase.game_table visibility
    pub fn get_table(&self) -> &GameTable {
        self.poker_phase.get_table()
    }

    // TODO: build inner state internally, grab atomically - re-downgrade PokerPhase.player_hands visibility
    pub fn get_player_hands(&self) -> Option<&HashMap<Uuid, Hand>> {
       self.poker_phase.get_player_hands()
    }
}
