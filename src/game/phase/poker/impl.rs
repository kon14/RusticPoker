use std::collections::{HashMap, VecDeque};
use std::ops::Deref;
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::common::error::AppError;
use crate::game::GameTable;
use crate::game::phase::BettingRoundAction;
use crate::game::phase::progression::ActionProgression;
use crate::types::card::Card;
use crate::types::deck::CardDeck;
use crate::types::hand::{Hand, RateHands};
use crate::output::{MatchStatePhaseSpecifics, MatchStatePhaseSpecificsShowdown};
use super::{PokerPhase, PokerPhaseBehavior, PokerPhaseAnte, PokerPhaseBetting, PokerPhaseDealing, PokerPhaseDrawingDiscarding, PokerPhaseDrawingDealing, PokerPhaseFirstBetting, PokerPhaseSecondBetting, PokerPhaseShowdown};

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

        let _ = shift_queue(&mut self.phase_player_queue); // TODO
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

    fn get_action_progression(&self) -> Option<ActionProgression> {
        Some(ActionProgression::delay(500))
    }

    fn get_player_bet_amounts(&self) -> Option<HashMap<Uuid, u64>> {
        let main_pot = self.game_table.credit_pots.values().next().unwrap();
        let player_bet_amounts = main_pot
            .get_participants()
            .into_iter()
            .map(|player_id| (player_id, self.ante_amount))
            .collect();
        Some(player_bet_amounts)
    }

    fn get_phase_specifics(&self) -> MatchStatePhaseSpecifics {
        MatchStatePhaseSpecifics::Ante
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

        let _ = shift_queue(&mut self.phase_player_queue); // TODO
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

    fn get_action_progression(&self) -> Option<ActionProgression> {
        Some(ActionProgression::delay(500))
    }

    fn get_player_bet_amounts(&self) -> Option<HashMap<Uuid, u64>> {
        let main_pot = self.game_table.credit_pots.values().next().unwrap();
        let player_bet_amounts = main_pot
            .get_participants()
            .into_iter()
            .map(|player_id| (player_id, self._ante_amount))
            .collect();
        Some(player_bet_amounts)
    }

    fn get_phase_specifics(&self) -> MatchStatePhaseSpecifics {
        MatchStatePhaseSpecifics::Dealing
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
                first_round_action: true,
            }
        )
    }

    fn init_player_bets(dealing_phase: &PokerPhaseDealing) -> HashMap<Uuid, u64> {
        dealing_phase.player_hands // remaining players
            .iter()
            .map(|(player_id, _)| (player_id.clone(), dealing_phase._ante_amount))
            .collect()
    }

    pub(crate) fn handle_betting_action(
        &mut self,
        player_id: Uuid,
        action: BettingRoundAction,
    ) -> Result<(), AppError> {
        self.0.handle_betting_action(player_id, action)
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
        if self.0.last_man_standing() {
            Some(PokerPhase::Showdown(PokerPhaseShowdown::from_betting(self.0)))
        } else {
            Some(PokerPhase::DrawingDiscarding(PokerPhaseDrawingDiscarding::from_first_betting(self)))
        }
    }

    fn get_active_player_id(&self) -> Option<Uuid> {
        self.0.get_active_player_id()
    }

    fn get_action_progression(&self) -> Option<ActionProgression> {
        self.0.get_action_progression()
    }

    fn get_player_bet_amounts(&self) -> Option<HashMap<Uuid, u64>> {
        self.0.get_player_bet_amounts()
    }

    fn get_phase_specifics(&self) -> MatchStatePhaseSpecifics {
        MatchStatePhaseSpecifics::FirstBetting(
            self.0.get_betting_phase_specifics()
        )
    }
}

impl PokerPhaseSecondBetting {
    pub(super) fn from_drawing_dealing(drawing_dealing_phase: PokerPhaseDrawingDealing) -> Self {
        let phase_player_queue = drawing_dealing_phase.game_table.clone_player_queue();
        PokerPhaseSecondBetting(
            PokerPhaseBetting{
                rpc_action_broadcaster: drawing_dealing_phase.rpc_action_broadcaster,
                game_table: drawing_dealing_phase.game_table,
                card_deck: drawing_dealing_phase.card_deck,
                phase_player_queue,
                player_hands: drawing_dealing_phase.player_hands,
                player_bets: drawing_dealing_phase._player_bets,
                first_round_action: true,
            }
        )
    }

    pub(crate) fn handle_betting_action(
        &mut self,
        player_id: Uuid,
        action: BettingRoundAction,
    ) -> Result<(), AppError> {
        self.0.handle_betting_action(player_id, action)
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
        Some(PokerPhase::Showdown(PokerPhaseShowdown::from_betting(self.0)))
    }

    fn get_active_player_id(&self) -> Option<Uuid> {
        self.0.phase_player_queue.front().cloned()
    }

    fn get_player_bet_amounts(&self) -> Option<HashMap<Uuid, u64>> {
        self.0.get_player_bet_amounts()
    }

    fn get_phase_specifics(&self) -> MatchStatePhaseSpecifics {
        MatchStatePhaseSpecifics::SecondBetting(
            self.0.get_betting_phase_specifics()
        )
    }
}

impl PokerPhaseShowdown {
    fn from_betting(betting_phase: PokerPhaseBetting) -> Self {
        let phase_player_queue = betting_phase.game_table.clone_player_queue();
        PokerPhaseShowdown {
            game_table: betting_phase.game_table,
            card_deck: betting_phase.card_deck,
            phase_player_queue,
            player_hands: betting_phase.player_hands,
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
        // todo!()
        true
    }

    fn get_phase_specifics(&self) -> MatchStatePhaseSpecifics {
        MatchStatePhaseSpecifics::Showdown(
            todo!()
            // MatchStatePhaseSpecificsShowdown {
            //     winning_rank: ,
            //     winner_ids: ,
            //     pot_distribution: ,
            // }
        )
    }
}

pub fn shift_queue(queue: &mut VecDeque<Uuid>) -> Result<Uuid, AppError> {
    let active_player = queue
        .pop_front()
        .ok_or(AppError::internal("Can't shift an empty queue!"))?;
    queue.push_back(active_player);
    Ok(active_player)
}
