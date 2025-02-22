use std::array;
use std::collections::{HashMap, HashSet};
use std::ops::Deref;
use futures::StreamExt;
use uuid::Uuid;

use crate::game::phase::poker::phase::PokerPhaseSecondBetting;
use crate::game::phase::poker::{PokerPhase, PokerPhaseBehavior};
use crate::game::phase::poker::r#impl::shift_queue;
use crate::game::phase::progression::ActionProgression;
use crate::types::card::Card;
use crate::output::{MatchStatePhaseSpecifics, MatchStatePhaseSpecificsDrawing};
use super::{PokerPhaseDrawingDealing, PokerPhaseDrawingDiscarding};

impl PokerPhaseBehavior for PokerPhaseDrawingDealing {
    /// Handles dealing replacement cards back to the players.<br />
    /// Phase actions are automatically scheduled on a loop until replacement cards are received.<br />
    /// Players receive all their replacement cards at once, in player order.
    fn act(&mut self) {
        self.card_deck.handle_discard_end();

        let Some(player_id) = self.get_active_player_id() else {
            unreachable!()
        };
        self.replenish_player_cards(player_id);

        let _ = shift_queue(&mut self.phase_player_queue); // TODO
    }

    fn is_phase_completed(&self) -> bool {
        self.player_discarded_cards.is_empty()
    }

    fn next_phase(self) -> Option<PokerPhase> {
        Some(PokerPhase::SecondBetting(PokerPhaseSecondBetting::from_drawing_dealing(self)))
    }

    fn get_active_player_id(&self) -> Option<Uuid> {
        self.phase_player_queue.front().cloned()
    }

    fn get_action_progression(&self) -> Option<ActionProgression> {
        Some(ActionProgression::delay(500))
    }

    fn get_player_bet_amounts(&self) -> Option<HashMap<Uuid, u64>> {
        Some(self._player_bets.clone())
    }

    fn get_phase_specifics(&self) -> MatchStatePhaseSpecifics {
        let discarded_cards = self.player_discarded_cards
            .iter()
            .map(|(player_id, discarded_cards)| {
                let cards = discarded_cards
                    .as_ref()
                    .map(|discarded| discarded.deref().clone())
                    .unwrap_or(HashSet::with_capacity(0));
                (player_id.clone(), cards)
            })
            .collect();
        MatchStatePhaseSpecifics::Drawing(
            MatchStatePhaseSpecificsDrawing {
                discard_stage: false,
                discarded_cards,
            }
        )
    }

    fn can_player_act(&self) -> HashMap<Uuid, bool> {
        self.game_table
            .player_ids
            .iter()
            .cloned()
            .map(|player_id| (player_id, false))
            .collect()
    }
}

impl PokerPhaseDrawingDealing {
    pub(crate) fn from_drawing_discarding(discard_phase: PokerPhaseDrawingDiscarding) -> Self {
        let phase_player_queue = discard_phase.game_table.clone_player_queue();
        PokerPhaseDrawingDealing {
            rpc_action_broadcaster: discard_phase.rpc_action_broadcaster,
            game_table: discard_phase.game_table,
            card_deck: discard_phase.card_deck,
            phase_player_queue,
            _player_bets: discard_phase._player_bets,
            player_hands: discard_phase.player_hands,
            player_discarded_cards: discard_phase.player_discarded_cards,
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
