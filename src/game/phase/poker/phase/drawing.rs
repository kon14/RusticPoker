use std::array;
use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::ops::Deref;
use std::pin::Pin;
use std::sync::Arc;
use futures::StreamExt;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::common::error::AppError;
use crate::game::GamePhase;
use crate::game::phase::poker::phase::{PokerPhaseDrawing, PokerPhaseFirstBetting, PokerPhaseSecondBetting};
use crate::game::phase::poker::{PokerPhase, PokerPhaseBehavior};
use crate::game::phase::poker::r#impl::shift_queue;
use crate::game::phase::progression::ActionProgression;
use crate::service::proto;
use crate::types::card::Card;

impl PokerPhaseBehavior for PokerPhaseDrawing {
    /// Handles the optional replacement of player cards.<br />
    /// Discarded cards are initially declared by everyone (via player action).<br />
    /// Any players failing to decide within a fixed amount of time get to discard no cards.<br />
    /// Replacement cards are then dealt back to the players.<br />
    /// Phase actions are automatically scheduled on a loop until replacement cards are received.<br />
    /// Player actions do initiate phase actions, but the former aren't really required.
    fn act(&mut self) {
        if !self.discard_stage_ongoing() {
            self.card_deck.handle_discard_end();

            let Some(player_id) = self.phase_player_queue.front().cloned() else {
                unreachable!()
            };
            self.replenish_player_cards(player_id);
        }

        let _ = shift_queue(&mut self.phase_player_queue); // TODO
    }

    fn is_phase_completed(&self) -> bool {
        // TODO: fix, skips dealing stage
        self.player_discarded_cards.len() == self.player_hands.len()
    }

    fn next_phase(self) -> Option<PokerPhase> {
        Some(PokerPhase::SecondBetting(PokerPhaseSecondBetting::from_drawing(self)))
    }

    fn get_active_player_id(&self) -> Option<Uuid> {
        // Players may discard cards in any given order.
        None
    }

    fn get_action_progression(&self) -> Option<ActionProgression> {
        if self.discard_stage_ongoing() {
            let active_player_id = self.phase_player_queue.front().cloned().unwrap();
            let timeout_handler = Arc::new(move |game_phase_arc: Arc<RwLock<GamePhase>>| Box::pin(async move {
                // TODO: improve hacky instance resolution of self
                let mut game_phase_w = game_phase_arc.write().await;
                let mut poker_phase = &mut game_phase_w.poker_phase;
                if let Some(mut drawing_phase) = match poker_phase {
                    PokerPhase::Drawing(ref mut phase) => Some(phase),
                    _ => None,
                } {
                    drawing_phase.player_discards(active_player_id, None)?;
                }
                Ok(())
            }) as Pin<Box<dyn Future<Output=Result<(), AppError>> + Send>>);
            Some(ActionProgression::event(15000, timeout_handler))
        } else {
            Some(ActionProgression::delay(500))
        }
    }

    fn get_player_bet_amounts(&self) -> Option<HashMap<Uuid, u64>> {
        Some(self._player_bets.clone())
    }
}

impl PokerPhaseDrawing {
    pub(crate) fn from_first_betting(betting_phase: PokerPhaseFirstBetting) -> Self {
        let phase_player_queue = betting_phase.game_table.clone_player_queue();
        let player_count = phase_player_queue.len();
        PokerPhaseDrawing {
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

    fn replenish_player_cards(&mut self, player_id: Uuid) {
        let Some(mut discarded_cards) = self.player_discarded_cards.remove(&player_id).flatten() else {
            return;
        };
        let count = discarded_cards.0.len();
        if count == 0 {
            return;
        }
        let old_cards = self.player_hands.get(&player_id).unwrap().cards.clone();
        let mut new_cards: Vec<Card> = (0..count)
            .map(|_| self.card_deck.draw().unwrap())
            .collect();
        let next_cards: [Card; 5] = array::from_fn(|i| {
            if !discarded_cards.0.contains(&old_cards[i]) {
                old_cards[i].clone()
            } else {
                discarded_cards.0.remove(&old_cards[i]);
                new_cards.pop().unwrap()
            }
        });
        let next_hand = next_cards.try_into().unwrap();
        self.player_hands.insert(player_id, next_hand);
    }
}

#[derive(Clone, Debug)]
pub(crate) struct DiscardedCards(HashSet<Card>);

impl DiscardedCards {
    pub(crate) fn try_from_proto(request: proto::RespondDrawingPhaseRequest) -> Result<Option<Self>, AppError> {
        let proto::RespondDrawingPhaseRequest { discarded_cards } = request;
        if discarded_cards.is_empty() {
            Ok(None)
        } else {
            let discarded_cards = discarded_cards
                .into_iter()
                .map(|card| card.try_into())
                .collect::<Result<_, _>>()?;
            Ok(Some(DiscardedCards(discarded_cards)))
        }
    }
}

impl Deref for DiscardedCards {
    type Target = HashSet<Card>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
