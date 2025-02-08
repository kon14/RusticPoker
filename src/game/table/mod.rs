mod credits;

pub(crate) use credits::{CreditPot, CalculatedPlayerCredits};

use std::collections::{HashMap, HashSet, VecDeque};
use uuid::Uuid;

use crate::r#match::MatchStartPlayers;

#[derive(Clone, Debug)]
pub struct GameTable {
    match_id: Uuid,
    player_queue_immut: VecDeque<Uuid>,
    pub player_ids: HashSet<Uuid>,
    dealer_id: Uuid,
    pub credit_pots: HashMap<Uuid, CreditPot>,
    pub player_credits: HashMap<Uuid, CalculatedPlayerCredits>,
}

impl GameTable {
    pub fn new(match_id: Uuid, players: MatchStartPlayers) -> Self {
        let player_ids = players.player_credits.keys().cloned().collect();
        let player_credits = players.player_credits
            .into_iter()
            .map(|(player_id, credits)| {
                (player_id, CalculatedPlayerCredits::new(player_id, credits))
            })
            .collect();
        let mut table = GameTable {
            match_id,
            player_queue_immut: players.ordered_player_queue,
            dealer_id: players.dealer_id,
            player_ids,
            credit_pots: HashMap::new(),
            player_credits,
        };
        let main_pot = CreditPot::new(true);
        table.add_pot(main_pot);
        table
    }

    pub fn clone_player_queue(&self) -> VecDeque<Uuid> {
        self.player_queue_immut.clone()
    }

    pub fn add_pot(&mut self, pot: CreditPot) {
        self.credit_pots.insert(pot.pot_id.clone(), pot);
    }
}
