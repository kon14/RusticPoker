mod phase;
mod r#impl;

pub(crate) use phase::{BettingRoundAction, DiscardedCards};

use std::collections::HashMap;
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::types::deck::CardDeck;
use crate::types::stateful::{StatefulCard, get_stateful_cards};
use crate::game::GameTable;
use super::progression::ActionProgression;
use crate::common::error::AppError;
use crate::output::MatchStatePhaseSpecifics;
use phase::*;

#[derive(Clone, Debug)]
pub(crate) enum PokerPhase {
    Ante(PokerPhaseAnte),
    Dealing(PokerPhaseDealing),
    FirstBetting(PokerPhaseFirstBetting),
    DrawingDiscarding(PokerPhaseDrawingDiscarding),
    DrawingDealing(PokerPhaseDrawingDealing),
    SecondBetting(PokerPhaseSecondBetting),
    Showdown(PokerPhaseShowdown),
}

pub(super) trait PokerPhaseBehavior {
    fn act(&mut self);

    fn is_phase_completed(&self) -> bool;

    fn next_phase(self) -> Option<PokerPhase> where Self: Sized {
        None
    }

    fn get_active_player_id(&self) -> Option<Uuid> where Self: Sized {
        None
    }

    fn get_action_progression(&self) -> Option<ActionProgression> where Self: Sized {
        None
    }

    fn get_player_bet_amounts(&self) -> Option<HashMap<Uuid, u64>> {
        None
    }

    fn get_phase_specifics(&self) -> MatchStatePhaseSpecifics;

    fn can_player_act(&self) -> HashMap<Uuid, bool>;
}

impl PokerPhaseBehavior for PokerPhase {
    fn act(&mut self) {
        match self {
            PokerPhase::Ante(phase) => phase.act(),
            PokerPhase::Dealing(phase) => phase.act(),
            PokerPhase::FirstBetting(phase) => phase.act(),
            PokerPhase::DrawingDiscarding(phase) => phase.act(),
            PokerPhase::DrawingDealing(phase) => phase.act(),
            PokerPhase::SecondBetting(phase) => phase.act(),
            PokerPhase::Showdown(phase) => phase.act(),
        }
    }

    fn is_phase_completed(&self) -> bool {
        match self {
            PokerPhase::Ante(phase) => phase.is_phase_completed(),
            PokerPhase::Dealing(phase) => phase.is_phase_completed(),
            PokerPhase::FirstBetting(phase) => phase.is_phase_completed(),
            PokerPhase::DrawingDiscarding(phase) => phase.is_phase_completed(),
            PokerPhase::DrawingDealing(phase) => phase.is_phase_completed(),
            PokerPhase::SecondBetting(phase) => phase.is_phase_completed(),
            PokerPhase::Showdown(phase) => phase.is_phase_completed(),
        }
    }

    fn next_phase(self) -> Option<PokerPhase> {
        match self {
            PokerPhase::Ante(phase) => phase.next_phase(),
            PokerPhase::Dealing(phase) => phase.next_phase(),
            PokerPhase::FirstBetting(phase) => phase.next_phase(),
            PokerPhase::DrawingDiscarding(phase) => phase.next_phase(),
            PokerPhase::DrawingDealing(phase) => phase.next_phase(),
            PokerPhase::SecondBetting(phase) => phase.next_phase(),
            PokerPhase::Showdown(phase) => phase.next_phase(),
        }
    }

    fn get_active_player_id(&self) -> Option<Uuid> {
        match self {
            PokerPhase::Ante(phase) => phase.get_active_player_id(),
            PokerPhase::Dealing(phase) => phase.get_active_player_id(),
            PokerPhase::FirstBetting(phase) => phase.get_active_player_id(),
            PokerPhase::DrawingDiscarding(phase) => phase.get_active_player_id(),
            PokerPhase::DrawingDealing(phase) => phase.get_active_player_id(),
            PokerPhase::SecondBetting(phase) => phase.get_active_player_id(),
            PokerPhase::Showdown(phase) => phase.get_active_player_id(),
        }
    }

    fn get_player_bet_amounts(&self) -> Option<HashMap<Uuid, u64>> {
        match self {
            PokerPhase::Ante(phase) => phase.get_player_bet_amounts(),
            PokerPhase::Dealing(phase) => phase.get_player_bet_amounts(),
            PokerPhase::FirstBetting(phase) => phase.get_player_bet_amounts(),
            PokerPhase::DrawingDiscarding(phase) => phase.get_player_bet_amounts(),
            PokerPhase::DrawingDealing(phase) => phase.get_player_bet_amounts(),
            PokerPhase::SecondBetting(phase) => phase.get_player_bet_amounts(),
            PokerPhase::Showdown(phase) => phase.get_player_bet_amounts(),
        }
    }

    fn get_phase_specifics(&self) -> MatchStatePhaseSpecifics {
        match self {
            PokerPhase::Ante(phase) => phase.get_phase_specifics(),
            PokerPhase::Dealing(phase) => phase.get_phase_specifics(),
            PokerPhase::FirstBetting(phase) => phase.get_phase_specifics(),
            PokerPhase::DrawingDiscarding(phase) => phase.get_phase_specifics(),
            PokerPhase::DrawingDealing(phase) => phase.get_phase_specifics(),
            PokerPhase::SecondBetting(phase) => phase.get_phase_specifics(),
            PokerPhase::Showdown(phase) => phase.get_phase_specifics(),
        }
    }

    fn can_player_act(&self) -> HashMap<Uuid, bool> {
        match self {
            PokerPhase::Ante(phase) => phase.can_player_act(),
            PokerPhase::Dealing(phase) => phase.can_player_act(),
            PokerPhase::FirstBetting(phase) => phase.can_player_act(),
            PokerPhase::DrawingDiscarding(phase) => phase.can_player_act(),
            PokerPhase::DrawingDealing(phase) => phase.can_player_act(),
            PokerPhase::SecondBetting(phase) => phase.can_player_act(),
            PokerPhase::Showdown(phase) => phase.can_player_act(),
        }
    }
}

impl PokerPhase {
    pub(crate) const RPC_ACTION_EVENT_CHANNEL_CAPACITY: usize = 100;

    pub(super) fn new(
        rpc_action_broadcaster: broadcast::Sender<()>,
        game_table: GameTable,
        card_deck: CardDeck,
        ante_amount: u64,
    ) -> Self {


        PokerPhase::Ante(PokerPhaseAnte::new(
            rpc_action_broadcaster,
            game_table,
            card_deck,
            ante_amount,
        ))
    }

    pub(super) fn get_action_progression(&self) -> Option<ActionProgression> {
        match self {
            PokerPhase::Ante(phase) => phase.get_action_progression(),
            PokerPhase::Dealing(phase) => phase.get_action_progression(),
            PokerPhase::FirstBetting(phase) => phase.get_action_progression(),
            PokerPhase::DrawingDiscarding(phase) => phase.get_action_progression(),
            PokerPhase::DrawingDealing(phase) => phase.get_action_progression(),
            PokerPhase::SecondBetting(phase) => phase.get_action_progression(),
            PokerPhase::Showdown(phase) => phase.get_action_progression(),
        }
    }

    pub(crate) async fn handle_betting_action(
        &mut self,
        player_id: Uuid,
        betting_action: BettingRoundAction,
    ) -> Result<(), AppError> {
        match self {
            PokerPhase::FirstBetting(betting_phase) => {
                betting_phase.handle_betting_action(player_id, betting_action)
            },
            PokerPhase::SecondBetting(betting_phase) => {
                betting_phase.handle_betting_action(player_id, betting_action)
            },
            _ => Err(AppError::invalid_request("Game not currently in Betting phase!")),
        }
    }

    pub async fn handle_drawing_action(
        &mut self,
        player_id: Uuid,
        discarded_cards: Option<DiscardedCards>,
    ) -> Result<(), AppError> {
        match self {
            PokerPhase::DrawingDiscarding(discard_phase) => discard_phase.player_discards(player_id, discarded_cards),
            _ => Err(AppError::invalid_request("Game not currently in DrawingDiscard phase!")),
        }
    }
}

impl PokerPhase {
    // TODO: tmp-only exposure
    pub fn get_table(&self) -> &GameTable {
        match self {
            PokerPhase::Ante(phase) => &phase.game_table,
            PokerPhase::Dealing(phase) => &phase.game_table,
            PokerPhase::FirstBetting(phase) => &phase.game_table,
            PokerPhase::DrawingDiscarding(phase) => &phase.game_table,
            PokerPhase::DrawingDealing(phase) => &phase.game_table,
            PokerPhase::SecondBetting(phase) => &phase.game_table,
            PokerPhase::Showdown(phase) => &phase.game_table,
        }
    }

    pub fn get_player_cards(&self) -> Option<HashMap<Uuid, Option<Vec<StatefulCard>>>> {
        match self {
            PokerPhase::Ante(_) => None,
            PokerPhase::Dealing(phase) => Some(get_stateful_cards(&phase.player_hands, None)), // TODO: partial hand
            PokerPhase::FirstBetting(phase) => Some(get_stateful_cards(&phase.player_hands, None)),
            PokerPhase::DrawingDiscarding(phase) => {
                Some(get_stateful_cards(&phase.player_hands, Some(&phase.player_discarded_cards)))
            },
            PokerPhase::DrawingDealing(phase) => {
                Some(get_stateful_cards(&phase.player_hands, Some(&phase.player_discarded_cards)))
            },
            PokerPhase::SecondBetting(phase) => Some(get_stateful_cards(&phase.player_hands, None)),
            PokerPhase::Showdown(phase) => Some(get_stateful_cards(&phase.player_hands, None)),
        }
    }
}
