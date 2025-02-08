mod poker;

use std::collections::HashMap;
use std::ops::Deref;
use chrono::{DateTime, Utc};
use tokio::time::sleep;
use uuid::Uuid;

use crate::r#match::MatchStartPlayers;
use crate::types::deck::CardDeck;
use crate::types::hand::{Hand, RateHands};
use crate::game::GameTable;
use crate::output::GameStateBroadcaster;
use poker::*;

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
        players: MatchStartPlayers,
        ante_amount: u64,
    ) -> Self {
        let game_table = GameTable::new(match_id, players);
        let card_deck = CardDeck::default();
        let poker_phase = PokerPhase::new(game_table, card_deck, ante_amount);
        GamePhase {
            poker_phase,
            state_time: Utc::now(),
            state_broadcaster,
        }
    }

    pub async fn progress(&mut self) {
        // Handle Game Logic
        self.state_time = Utc::now();
        self.poker_phase.act();

        // Build & Publish State
        self.state_broadcaster.publish().await;

        // Contemplate Your Life Choices
        if let Some(duration) = self.poker_phase.get_post_act_delay() {
            sleep(duration).await;
        }

        // Handle State Progression
        if self.poker_phase.is_phase_completed() {
            if let Some(next_phase) = self.poker_phase.clone().next_phase() {
                self.poker_phase = next_phase
            } else {
                // TODO: Game Over - Cleanup
                return;
            }
        }

        // Schedule Next Run
        // TODO
        // Player Actions Also Call Progress! Replace Scheduled Run / Respect Sleep Timer
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
