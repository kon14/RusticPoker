use std::collections::HashMap;
use uuid::Uuid;

use crate::common::error::AppError;
use crate::service::proto;

#[derive(Clone, Debug)]
pub(crate) struct CreditPot {
    pub(super) pot_id: Uuid,
    // pub(super) match_id: Uuid,
    pub(super) is_main_pot: bool,
    // pub(super) contributor_ids: HashSet<Uuid>,
    pub(super) total_credits: u64,
    pub(super) player_credits: HashMap<Uuid, u64>,
}

impl CreditPot {
    pub fn new(is_main_pot: bool) -> Self {
        CreditPot {
            pot_id: Uuid::new_v4(),
            // match_id,
            is_main_pot,
            // contributor_ids: HashSet::new(),
            total_credits: 0,
            player_credits: HashMap::new(),
        }
    }

    pub fn add_credits(
        &mut self,
        player_credits: &mut CalculatedPlayerCredits,
        amount: u64,
    ) {
        let player_id = player_credits.player_id;
        let player_entry = self.player_credits.entry(player_id).or_insert(0);
        *player_entry += amount;
        self.total_credits += amount;
        player_credits.pot_credits.insert(self.pot_id, amount);
    }
}

#[derive(Clone, Debug)]
pub(crate) struct CalculatedPlayerCredits {
    // TODO: re-evaluate field visibility
    pub(crate) player_id: Uuid,
    pub(crate) starting_credits: u64,
    pub(crate) remaining_credits: u64,
    pub(crate) pot_credits: HashMap<Uuid, u64>,
}

impl CalculatedPlayerCredits {
    pub fn new(player_id: Uuid, starting_credits: u64) -> Self {
        CalculatedPlayerCredits {
            player_id,
            starting_credits,
            remaining_credits: starting_credits,
            pot_credits: HashMap::new(),
        }
    }

    pub fn use_credits(
        &mut self,
        amount: u64,
        credit_pot: &mut CreditPot,
    ) -> Result<(), AppError> {
        let Some(remaining_credits) = self.remaining_credits.checked_sub(amount) else {
            return Err(AppError::internal("Not enough credits!"));
        };
        self.remaining_credits = remaining_credits;
        credit_pot.add_credits(self, amount);
        Ok(())
    }
}

impl CalculatedPlayerCredits {
    pub fn get_starting_credits(&self) -> u64 {
        self.starting_credits
    }
}

impl From<CreditPot> for proto::MatchStateCreditPot {
    fn from(pot: CreditPot) -> Self {
        let player_credits = pot.player_credits
            .into_iter()
            .map(|(player_id, credits)| (player_id.into(), credits))
            .collect();
        proto::MatchStateCreditPot {
            pot_id: pot.pot_id.to_string(),
            is_main_pot: pot.is_main_pot,
            total_credits: pot.total_credits,
            player_credits,
        }
    }
}
