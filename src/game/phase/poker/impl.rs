use std::collections::HashMap;

use crate::game::GameTable;
use crate::types::card::Card;
use crate::types::deck::CardDeck;
use crate::types::hand::{Hand, RateHands};
use super::{PokerPhase, PokerPhaseBehavior, PokerPhaseAnte, PokerPhaseBetting, PokerPhaseDealing, PokerPhaseDrawing, PokerPhaseFirstBetting, PokerPhaseSecondBetting, PokerPhaseShowdown};

impl PokerPhaseAnte {
    pub(super) fn new(game_table: GameTable, card_deck: CardDeck, ante_amount: u64) -> Self {
        let phase_player_queue = game_table.clone_player_queue();
        PokerPhaseAnte {
            game_table,
            card_deck,
            phase_player_queue,
            ante_amount,
        }
    }
}

impl PokerPhaseBehavior for PokerPhaseAnte {
    fn act(&mut self) {
        let Some(player_id) = self.phase_player_queue.pop_back() else {
            unreachable!()
        };

        let Some(mut credits) = self.game_table.player_credits.get_mut(&player_id) else {
            unreachable!()
        };
        let credit_pot = self.game_table.credit_pots.values_mut().next().unwrap(); // first pot should exist
        credits.use_credits(self.ante_amount, credit_pot).unwrap();
        // TODO: short delay
    }

    fn is_phase_completed(&self) -> bool {
        self.game_table.player_credits.len() == self.game_table.player_ids.len()
    }

    fn next_phase(self) -> Option<PokerPhase> {
        Some(PokerPhase::Dealing(PokerPhaseDealing::from_ante(self)))
    }
}

impl PokerPhaseDealing {
    fn from_ante(ante_phase: PokerPhaseAnte) -> Self {
        let phase_player_queue = ante_phase.game_table.clone_player_queue();
        let player_count = phase_player_queue.len();
        PokerPhaseDealing {
            game_table: ante_phase.game_table,
            card_deck: ante_phase.card_deck,
            phase_player_queue,
            player_hands: HashMap::with_capacity(player_count),
            // player_cards: HashMap::with_capacity(player_count),
        }
    }
}

impl PokerPhaseBehavior for PokerPhaseDealing {
    fn act(&mut self) {
        let Some(player_id) = self.phase_player_queue.pop_back() else {
            unreachable!()
        };

        // TODO: Deal cards one by one (requires refactoring representation layer)
        let cards: [Card; 5] = (0..5)
            .map(|_| self.card_deck.draw().unwrap()) // fresh deck can't underflow for max players
            .collect::<Vec<Card>>()
            .try_into()
            .unwrap(); // fresh deck can't underflow for max players
        let hand: Hand = cards.try_into().unwrap(); // no dupes
        self.player_hands.insert(player_id, hand);
        // TODO: short delay
    }

    fn is_phase_completed(&self) -> bool {
        self.player_hands.len() == self.game_table.player_ids.len()
    }

    fn next_phase(self) -> Option<PokerPhase> {
        Some(PokerPhase::FirstBetting(PokerPhaseFirstBetting::from_dealing(self)))
    }
}

impl PokerPhaseBetting {
    fn act(&mut self) {
        todo!()
    }

    fn is_phase_completed(&self) -> bool {
        todo!()
    }
}

impl PokerPhaseFirstBetting {
    fn from_dealing(dealing_phase: PokerPhaseDealing) -> Self {
        let phase_player_queue = dealing_phase.game_table.clone_player_queue();
        PokerPhaseFirstBetting(
            PokerPhaseBetting {
                game_table: dealing_phase.game_table,
                card_deck: dealing_phase.card_deck,
                phase_player_queue,
                player_hands: dealing_phase.player_hands,
            }
        )
    }
}

impl PokerPhaseBehavior for PokerPhaseFirstBetting {
    fn act(&mut self) {
        self.0.act()
    }

    fn is_phase_completed(&self) -> bool {
        self.0.is_phase_completed()
    }

    fn next_phase(self) -> Option<PokerPhase> {
        Some(PokerPhase::Drawing(PokerPhaseDrawing::from_first_betting(self)))
    }
}

impl PokerPhaseDrawing {
    fn from_first_betting(betting_phase: PokerPhaseFirstBetting) -> Self {
        let phase_player_queue = betting_phase.game_table.clone_player_queue();
        let player_count = phase_player_queue.len();
        PokerPhaseDrawing {
            game_table: betting_phase.0.game_table,
            card_deck: betting_phase.0.card_deck,
            phase_player_queue,
            player_hands: betting_phase.0.player_hands,
            player_discarded_cards: HashMap::with_capacity(player_count)
        }
    }
}

impl PokerPhaseSecondBetting {
    fn from_drawing(drawing_phase: PokerPhaseDrawing) -> Self {
        let phase_player_queue = drawing_phase.game_table.clone_player_queue();
        PokerPhaseSecondBetting(
            PokerPhaseBetting{
                game_table: drawing_phase.game_table,
                card_deck: drawing_phase.card_deck,
                phase_player_queue,
                player_hands: drawing_phase.player_hands,
            }
        )
    }
}

impl PokerPhaseBehavior for PokerPhaseSecondBetting {
    fn act(&mut self) {
        self.0.act()
    }

    fn is_phase_completed(&self) -> bool {
        self.0.is_phase_completed()
    }

    fn next_phase(self) -> Option<PokerPhase> {
        Some(PokerPhase::Showdown(PokerPhaseShowdown::from_second_betting(self)))
    }
}

impl PokerPhaseBehavior for PokerPhaseDrawing {
    fn act(&mut self) {
        let Some(player_id) = self.phase_player_queue.pop_back() else {
            unreachable!()
        };

        todo!()
    }

    fn is_phase_completed(&self) -> bool {
        self.player_discarded_cards.len() == self.game_table.player_ids.len()
    }

    fn next_phase(self) -> Option<PokerPhase> {
        Some(PokerPhase::SecondBetting(PokerPhaseSecondBetting::from_drawing(self)))
    }
}

impl PokerPhaseShowdown {
    fn from_second_betting(betting_phase: PokerPhaseSecondBetting) -> Self {
        let phase_player_queue = betting_phase.game_table.clone_player_queue();
        PokerPhaseShowdown {
            game_table: betting_phase.0.game_table,
            card_deck: betting_phase.0.card_deck,
            phase_player_queue,
            player_hands: betting_phase.0.player_hands,
        }
    }
}

impl PokerPhaseBehavior for PokerPhaseShowdown {
    fn act(&mut self) {
        let hands: Vec<Hand> = self.player_hands.values().cloned().collect();
        let winners = hands.determine_winners();
        // TODO: refactor determine_winners(), returning:
        // - winner ids
        // - best rank
        // TODO: propagate these fields outwards
    }

    fn is_phase_completed(&self) -> bool {
        todo!()
    }

    fn next_phase(self) -> Option<PokerPhase> {
        None
    }
}
