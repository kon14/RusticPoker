mod phase;
mod r#impl;

use std::collections::HashMap;
use std::ops::Deref;
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::types::hand::{Hand, RateHands};
use crate::types::deck::CardDeck;
use crate::game::GameTable;
use super::progression::ActionProgression;
use phase::*;

#[derive(Clone, Debug)]
pub(super) enum PokerPhase {
    Ante(PokerPhaseAnte),
    Dealing(PokerPhaseDealing),
    FirstBetting(PokerPhaseFirstBetting),
    Drawing(PokerPhaseDrawing),
    SecondBetting(PokerPhaseSecondBetting),
    Showdown(PokerPhaseShowdown),
}

pub(super) trait PokerPhaseBehavior {
    fn act(&mut self);

    fn is_phase_completed(&self) -> bool;

    fn next_phase(self) -> Option<PokerPhase>;

    fn get_active_player_id(&self) -> Option<Uuid>;
}

impl PokerPhaseBehavior for PokerPhase {
    fn act(&mut self) {
        match self {
            PokerPhase::Ante(phase) => phase.act(),
            PokerPhase::Dealing(phase) => phase.act(),
            PokerPhase::FirstBetting(phase) => phase.act(),
            PokerPhase::Drawing(phase) => phase.act(),
            PokerPhase::SecondBetting(phase) => phase.act(),
            PokerPhase::Showdown(phase) => phase.act(),
        }
    }

    fn is_phase_completed(&self) -> bool {
        match self {
            PokerPhase::Ante(phase) => phase.is_phase_completed(),
            PokerPhase::Dealing(phase) => phase.is_phase_completed(),
            PokerPhase::FirstBetting(phase) => phase.is_phase_completed(),
            PokerPhase::Drawing(phase) => phase.is_phase_completed(),
            PokerPhase::SecondBetting(phase) => phase.is_phase_completed(),
            PokerPhase::Showdown(phase) => phase.is_phase_completed(),
        }
    }

    fn next_phase(self) -> Option<PokerPhase> {
        match self {
            PokerPhase::Ante(phase) => phase.next_phase(),
            PokerPhase::Dealing(phase) => phase.next_phase(),
            PokerPhase::FirstBetting(phase) => phase.next_phase(),
            PokerPhase::Drawing(phase) => phase.next_phase(),
            PokerPhase::SecondBetting(phase) => phase.next_phase(),
            PokerPhase::Showdown(phase) => phase.next_phase(),
        }
    }

    fn get_active_player_id(&self) -> Option<Uuid> {
        match self {
            PokerPhase::Ante(phase) => phase.get_active_player_id(),
            PokerPhase::Dealing(phase) => phase.get_active_player_id(),
            PokerPhase::FirstBetting(phase) => phase.get_active_player_id(),
            PokerPhase::Drawing(phase) => phase.get_active_player_id(),
            PokerPhase::SecondBetting(phase) => phase.get_active_player_id(),
            PokerPhase::Showdown(phase) => phase.get_active_player_id(),
        }
    }
}

impl PokerPhase {
    // const RPC_ACTION_EVENT_CHANNEL_CAPACITY: usize = 100;

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

    // pub(super) fn get_post_act_delay(&self) -> Option<Duration> {
    //     match self {
    //         PokerPhase::Ante(_) => Some(Duration::from_millis(500)),
    //         PokerPhase::Dealing(_) => Some(Duration::from_millis(500)),
    //         PokerPhase::FirstBetting(_) => Some(Duration::from_millis(500)),
    //         PokerPhase::Drawing(_) => Some(Duration::from_millis(500)),
    //         PokerPhase::SecondBetting(_) => Some(Duration::from_millis(500)),
    //         PokerPhase::Showdown(_) => None,
    //     }
    // }

    // pub(super) fn get_action_progression(&self) -> Option<ActionProgression> {
    //     match self {
    //         PokerPhase::Ante(_) => Some(ActionProgression::scheduled(500)),
    //         PokerPhase::Dealing(_) => Some(ActionProgression::scheduled(500)),
    //         PokerPhase::FirstBetting(_) => Some(ActionProgression::mixed(500, )),
    //         PokerPhase::Drawing(_) => Some(ActionProgression::mixed(500, )),
    //         PokerPhase::SecondBetting(_) => Some(ActionProgression::mixed(500, )),
    //         PokerPhase::Showdown(_) => None,
    //     }
    // }

    pub(super) fn get_action_progression(&self) -> Option<ActionProgression> {
        // TODO: mv this inside each phase or grab state values from match arm
        match self {
            PokerPhase::Ante(_) => Some(ActionProgression::delay(500)),
            PokerPhase::Dealing(_) => Some(ActionProgression::delay(500)),
            PokerPhase::FirstBetting(_) => Some(ActionProgression::event(10000)),
            PokerPhase::Drawing(_) => Some(ActionProgression::event(10000)),
            PokerPhase::SecondBetting(_) => Some(ActionProgression::event(10000)),
            PokerPhase::Showdown(_) => None,
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
            PokerPhase::Drawing(phase) => &phase.game_table,
            PokerPhase::SecondBetting(phase) => &phase.game_table,
            PokerPhase::Showdown(phase) => &phase.game_table,
        }
    }

    // TODO: tmp-only exposure
    pub fn get_player_hands(&self) -> Option<&HashMap<Uuid, Hand>> {
        match self {
            PokerPhase::Ante(_) => None,
            PokerPhase::Dealing(phase) => Some(&phase.player_hands),
            PokerPhase::FirstBetting(phase) => Some(&phase.player_hands),
            PokerPhase::Drawing(phase) => Some(&phase.player_hands), // TODO: recheck
            PokerPhase::SecondBetting(phase) => Some(&phase.player_hands),
            PokerPhase::Showdown(phase) => Some(&phase.player_hands),
        }
    }
}
