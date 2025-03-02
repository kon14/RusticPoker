use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::common::error::AppError;
use crate::game::{DiscardedCards, GamePhase};
use crate::game::phase::poker::phase::{PokerPhaseDrawingDealing, PokerPhaseFirstBetting};
use crate::game::phase::poker::{PokerPhase, PokerPhaseBehavior};
use crate::game::phase::poker::r#impl::shift_queue;
use crate::game::phase::progression::ActionProgression;
use crate::types::card::Card;
use crate::output::{DrawingStageDiscarding, MatchStatePhaseSpecifics, MatchStatePhaseSpecificsDrawing};
use super::PokerPhaseDrawingDiscarding;

impl PokerPhaseBehavior for PokerPhaseDrawingDiscarding {
    /// Handles the optional replacement of player cards.<br />
    /// Discarded cards are initially declared by everyone (via player action).<br />
    /// Player actions do initiate phase actions, but the former aren't really required.<br />
    /// Any players failing to decide within a fixed amount of time get to discard no cards.
    fn act(&mut self) {
        let _ = shift_queue(&mut self.phase_player_queue); // TODO
    }

    fn is_phase_completed(&self) -> bool {
        self.player_discarded_cards.len() == self.player_hands.len()
    }

    fn next_phase(self) -> Option<PokerPhase> {
        Some(PokerPhase::DrawingDealing(PokerPhaseDrawingDealing::from_drawing_discarding(self)))
    }

    fn get_active_player_id(&self) -> Option<Uuid> {
        // Players may discard cards in any given order.
        None
    }

    fn get_action_progression(&self) -> Option<ActionProgression> {
        let active_player_id = self.phase_player_queue.front().cloned().unwrap();
        let timeout_handler = Arc::new(move |game_phase_arc: Arc<RwLock<GamePhase>>| Box::pin(async move {
            // TODO: improve hacky instance resolution of self
            let mut game_phase_w = game_phase_arc.write().await;
            let mut poker_phase = &mut game_phase_w.poker_phase;
            if let Some(mut drawing_phase) = match poker_phase {
                PokerPhase::DrawingDiscarding(ref mut phase) => Some(phase),
                _ => None,
            } {
                drawing_phase.player_discards(active_player_id, None)?;
            }
            Ok(())
        }) as Pin<Box<dyn Future<Output=Result<(), AppError>> + Send>>);
        Some(ActionProgression::event(15000, timeout_handler))
    }

    fn get_player_bet_amounts(&self) -> Option<HashMap<Uuid, u64>> {
        Some(self._player_bets.clone())
    }

    fn get_phase_specifics(&self) -> MatchStatePhaseSpecifics {
        let player_discard_count = self.player_discarded_cards
            .iter()
            .map(|(player_id, cards)| {
                let discard_count = cards
                    .as_ref()
                    .map_or(0, |cards| cards.len())
                    .try_into()
                    .unwrap();
                (player_id.clone(), discard_count)
            })
            .collect();
        MatchStatePhaseSpecifics::Drawing(
            MatchStatePhaseSpecificsDrawing::Discarding(
                DrawingStageDiscarding {
                    player_discard_count,
                }
            )
        )
    }

    fn can_player_act(&self) -> HashMap<Uuid, bool> {
        self.game_table
            .player_ids
            .iter()
            .cloned()
            .map(|player_id| (player_id, true))
            .collect()
    }
}

impl PokerPhaseDrawingDiscarding {
    pub(crate) fn from_first_betting(betting_phase: PokerPhaseFirstBetting) -> Self {
        let phase_player_queue = betting_phase.game_table.clone_player_queue();
        let player_count = phase_player_queue.len();
        PokerPhaseDrawingDiscarding {
            rpc_action_broadcaster: betting_phase.0.rpc_action_broadcaster,
            game_table: betting_phase.0.game_table,
            card_deck: betting_phase.0.card_deck,
            phase_player_queue,
            _player_bets: betting_phase.0.player_bets,
            player_hands: betting_phase.0.player_hands,
            player_discarded_cards: HashMap::with_capacity(player_count)
        }
    }

    fn discard_stage_ongoing(&self) -> bool {
        self.player_discarded_cards.len() < self.player_hands.len()
    }

    pub fn player_discards(&mut self, player_id: Uuid, cards: Option<DiscardedCards>) -> Result<(), AppError> {
        if self.player_discarded_cards.contains_key(&player_id) {
            return Err(AppError::invalid_request("Player has already discarded cards!"))
        }

        if let Some(cards) = cards.clone() {
            self.player_owns_cards_validation(player_id, &cards.0)?;
            self.card_deck.discard_cards(cards.0);
        };
        self.player_discarded_cards.insert(player_id, cards);

        self.rpc_action_broadcaster.send(()).unwrap(); // TODO: handle dropped receiver
        Ok(())
    }

    fn player_owns_cards_validation(&self, player_id: Uuid, cards: &HashSet<Card>) -> Result<(), AppError> {
        let Some(player_hand) = self.player_hands.get(&player_id) else {
            unreachable!();
        };
        if cards.iter().all(|card| player_hand.cards.contains(card)) {
            Ok(())
        } else {
            Err(AppError::invalid_request(
                "Discarded cards selection includes cards not present in the player's hand!"
            ))
        }
    }
}
