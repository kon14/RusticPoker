mod phase;
mod r#impl;

use std::collections::HashMap;
use std::ops::Deref;
use std::time::Duration;
use uuid::Uuid;
use crate::types::hand::{Hand, RateHands};
use crate::types::deck::CardDeck;
use crate::game::GameTable;
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
}

impl PokerPhase {
    pub(super) fn new(game_table: GameTable, card_deck: CardDeck, ante_amount: u64) -> Self {
        PokerPhase::Ante(PokerPhaseAnte::new(game_table, card_deck, ante_amount))
    }


    pub(super) fn get_post_act_delay(&self) -> Option<Duration> {
        match self {
            PokerPhase::Ante(_) => Some(Duration::from_millis(500)),
            PokerPhase::Dealing(_) => Some(Duration::from_millis(500)),
            PokerPhase::FirstBetting(_) => Some(Duration::from_millis(500)),
            PokerPhase::Drawing(_) => Some(Duration::from_millis(500)),
            PokerPhase::SecondBetting(_) => Some(Duration::from_millis(500)),
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
