use std::collections::HashMap;
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::game::GameTable;
use crate::types::card::Card;
use crate::types::deck::CardDeck;
use crate::types::hand::{Hand, RateHands};
use super::{PokerPhase, PokerPhaseBehavior, PokerPhaseAnte, PokerPhaseBetting, PokerPhaseDealing, PokerPhaseDrawing, PokerPhaseFirstBetting, PokerPhaseSecondBetting, PokerPhaseShowdown};

impl PokerPhaseAnte {
    pub(super) fn new(
        rpc_action_broadcaster: broadcast::Sender<()>,
        game_table: GameTable,
        card_deck: CardDeck,
        ante_amount: u64,
    ) -> Self {
        let phase_player_queue = game_table.clone_player_queue();
        PokerPhaseAnte {
            _rpc_action_broadcaster: rpc_action_broadcaster,
            game_table,
            card_deck,
            phase_player_queue,
            ante_amount,
        }
    }
}

impl PokerPhaseBehavior for PokerPhaseAnte {
    /// Handles the placement of forced initial bets.<br />
    /// Phase actions are automatically scheduled without any player interaction.
    fn act(&mut self) {
        let Some(player_id) = self.phase_player_queue.front().cloned() else {
            unreachable!()
        };
        let Some(mut credits) = self.game_table.player_credits.get_mut(&player_id) else {
            unreachable!()
        };

        let credit_pot = self.game_table.credit_pots.values_mut().next().unwrap(); // first pot should exist
        credits.use_credits(self.ante_amount, credit_pot).unwrap();
    }

    fn is_phase_completed(&self) -> bool {
        let main_pot = self.game_table.credit_pots.values().next().unwrap();
        let main_pot_participants = main_pot.get_participants();
        main_pot_participants.len() == self.game_table.player_ids.len()
    }

    fn next_phase(self) -> Option<PokerPhase> {
        Some(PokerPhase::Dealing(PokerPhaseDealing::from_ante(self)))
    }

    fn get_active_player_id(&self) -> Option<Uuid> {
        self.phase_player_queue.front().cloned()
    }
}

impl PokerPhaseDealing {
    fn from_ante(ante_phase: PokerPhaseAnte) -> Self {
        let phase_player_queue = ante_phase.game_table.clone_player_queue();
        let player_count = phase_player_queue.len();
        PokerPhaseDealing {
            _rpc_action_broadcaster: ante_phase._rpc_action_broadcaster,
            game_table: ante_phase.game_table,
            card_deck: ante_phase.card_deck,
            phase_player_queue,
            _ante_amount: ante_phase.ante_amount,
            player_hands: HashMap::with_capacity(player_count),
            // player_cards: HashMap::with_capacity(player_count),
        }
    }
}

impl PokerPhaseBehavior for PokerPhaseDealing {
    /// Handles card dealing.<br />
    /// Phase actions are automatically scheduled without any player interaction.
    fn act(&mut self) {
        let Some(player_id) = self.phase_player_queue.front().cloned() else {
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
    }

    fn is_phase_completed(&self) -> bool {
        self.player_hands.len() == self.game_table.player_ids.len()
    }

    fn next_phase(self) -> Option<PokerPhase> {
        Some(PokerPhase::FirstBetting(PokerPhaseFirstBetting::from_dealing(self)))
    }

    fn get_active_player_id(&self) -> Option<Uuid> {
        self.phase_player_queue.front().cloned()
    }
}

impl PokerPhaseFirstBetting {
    fn from_dealing(dealing_phase: PokerPhaseDealing) -> Self {
        let phase_player_queue = dealing_phase.game_table.clone_player_queue();
        let player_bets = Self::init_player_bets(&dealing_phase);
        PokerPhaseFirstBetting(
            PokerPhaseBetting {
                rpc_action_broadcaster: dealing_phase._rpc_action_broadcaster,
                game_table: dealing_phase.game_table,
                card_deck: dealing_phase.card_deck,
                phase_player_queue,
                player_hands: dealing_phase.player_hands,
                player_bets,
            }
        )
    }

    fn init_player_bets(dealing_phase: &PokerPhaseDealing) -> HashMap<Uuid, u64> {
        dealing_phase.player_hands // remaining players
            .iter()
            .map(|(player_id, _)| (player_id.clone(), dealing_phase._ante_amount))
            .collect()
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

    fn get_active_player_id(&self) -> Option<Uuid> {
        self.0.get_active_player_id()
    }
}

impl PokerPhaseDrawing {
    fn from_first_betting(betting_phase: PokerPhaseFirstBetting) -> Self {
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
}

impl PokerPhaseSecondBetting {
    fn from_drawing(drawing_phase: PokerPhaseDrawing) -> Self {
        let phase_player_queue = drawing_phase.game_table.clone_player_queue();
        PokerPhaseSecondBetting(
            PokerPhaseBetting{
                rpc_action_broadcaster: drawing_phase.rpc_action_broadcaster,
                game_table: drawing_phase.game_table,
                card_deck: drawing_phase.card_deck,
                phase_player_queue,
                player_hands: drawing_phase.player_hands,
                player_bets: drawing_phase._player_bets
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

    fn get_active_player_id(&self) -> Option<Uuid> {
        self.0.phase_player_queue.front().cloned()
    }
}

impl PokerPhaseBehavior for PokerPhaseDrawing {
    /// Handles the optional replacement of player cards.<br />
    /// Discarded cards are initially declared by everyone (via player action).<br />
    /// Any players failing to decide within a fixed amount of time get to discard no cards.<br />
    /// Replacement cards are then dealt back to the players.<br />
    /// Phase actions are automatically scheduled on a loop until replacement cards are received.<br />
    /// Player actions do initiate phase actions, but the former aren't really required
    fn act(&mut self) {
        let Some(player_id) = self.phase_player_queue.front().cloned() else {
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

    fn get_active_player_id(&self) -> Option<Uuid> {
        None
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
    /// Handles the calculation of player hand rankings.<br />
    /// A single phase action is automatically scheduled without any player interaction.
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

    fn get_active_player_id(&self) -> Option<Uuid> {
        None
    }
}

// fn shift_queue(queue: &mut VecDeque<Uuid>) -> Result<Uuid, AppError> {
//     let active_player = queue
//         .pop_front()
//         .ok_or(Err(AppError::internal("Can't shift an empty queue!")))?;
//     queue.push_back(active_player);
//     Ok(active_player)
// }
