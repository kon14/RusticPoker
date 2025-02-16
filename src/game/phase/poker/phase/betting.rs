use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use crate::service::proto::respond_betting_phase_request as proto;
use crate::common::error::AppError;
use crate::game::phase::poker::{PokerPhase, PokerPhaseBehavior};
use crate::game::phase::poker::r#impl::shift_queue;
use super::PokerPhaseBetting;

#[derive(Clone, Debug)]
pub struct BettingRound {
    player_folds: HashMap<Uuid, bool>,
    player_actions: HashMap<Uuid, BettingRoundAction>,
}

#[derive(Clone, Debug)]
pub(crate) enum BettingRoundAction {
    Bet(u64),
    Call,
    Raise(u64),
    Fold,
}

impl PokerPhaseBehavior for PokerPhaseBetting {
    /// Handles betting phase gameplay actions.<br />
    /// Phase actions are primarily initiated by players.<br />
    /// Players not responding in time fold their hands.
    fn act(&mut self) {
        todo!();

        let _ = shift_queue(&mut self.phase_player_queue); // TODO
    }

    /// Checks whether everyone has either folded or matched the highest bet.
    fn is_phase_completed(&self) -> bool {
        let Some((_, matched_bettors)) = self.get_highest_bet_with_bettors() else {
            // No bets placed yet
            return false;
        };
        let remaining_player_count = self.player_hands.len();
        if matched_bettors.len() != remaining_player_count {
            return false;
        }
        true
    }

    fn next_phase(self) -> Option<PokerPhase> {
        unreachable!()
    }

    fn get_active_player_id(&self) -> Option<Uuid> {
        self.phase_player_queue.front().cloned()
    }
}

impl PokerPhaseBetting {
    pub(crate) fn handle_betting_action(
        &mut self,
        player_id: Uuid,
        action: BettingRoundAction,
    ) -> Result<(), AppError> {
        match action {
            BettingRoundAction::Bet(credits) => self.player_bets_or_raises(player_id, credits),
            BettingRoundAction::Call => self.player_calls(player_id),
            BettingRoundAction::Raise(credits) => self.player_bets_or_raises(player_id, credits),
            BettingRoundAction::Fold => self.player_folds(player_id),
        }
    }
}

impl PokerPhaseBetting {
    fn player_folds(&mut self, player_id: Uuid) -> Result<(), AppError> {
        if !self.can_player_act(player_id) {
            return Err(AppError::invalid_request("Player can't act out of turn!"));
        }

        todo!();
        // TODO: remove from bet amounts, remove hand
        // what if same person runs twice? ignore anyone not with a hand

        self.rpc_action_broadcaster.send(()).unwrap(); // TODO: handle dropped receiver
        Ok(())
    }

    /// Matches the current
    fn player_calls(&mut self, player_id: Uuid) -> Result<(), AppError> {
        if !self.can_player_act(player_id) {
            return Err(AppError::invalid_request("Player can't act out of turn!"));
        }
        let highest_bet = self.get_highest_bet()
            .ok_or(AppError::invalid_request("No bet to call against!"))?;

        self.set_player_bet(player_id, highest_bet)?;
        self.rpc_action_broadcaster.send(()).unwrap(); // TODO: handle dropped receiver
        Ok(())
    }

    /// Sets or raises a player's bet for the betting round.<br />
    /// The bet_credits arg contains the compound bet credit amount, not the amount to be raised by!
    fn player_bets_or_raises(
        &mut self,
        player_id: Uuid,
        bet_credits: u64,
    ) -> Result<(), AppError> {
        if !self.can_player_act(player_id) {
            return Err(AppError::invalid_request("Player can't act out of turn!"));
        }

        self.set_player_bet(player_id, bet_credits)?;
        self.rpc_action_broadcaster.send(()).unwrap(); // TODO: handle dropped receiver
        Ok(())
    }
}

impl PokerPhaseBetting {
    fn check_min_bet(&self, bet_credits: u64) -> Result<(), AppError> {
        let Some(highest_bet) = self.get_highest_bet() else {
            return Ok(());
        };

        if bet_credits < highest_bet {
            let err_msg = format!("Bet amount can't be less than the current high bet ({highest_bet})!");
            Err(AppError::invalid_request(err_msg))
        } else {
            Ok(())
        }
    }

    fn set_player_bet(&mut self, player_id: Uuid, bet_credits: u64) -> Result<(), AppError> {
        self.check_min_bet(bet_credits)?;

        let mut player_credits = self.game_table
            .player_credits
            .get_mut(&player_id)
            .unwrap(); // TODO
        let total_credits = player_credits.starting_credits; // per match
        if bet_credits > total_credits {
            return Err(AppError::invalid_request("Player can't afford bet!"));
        }
        let prev_bet_credits = player_credits.starting_credits - player_credits.remaining_credits;
        let added_credits = bet_credits - prev_bet_credits;

        if added_credits > 0 {
            // TODO: currently assuming a single pot
            let credit_pot = self.game_table.credit_pots.values_mut().next().unwrap();
            player_credits.use_credits(added_credits, credit_pot)?;
        };

        Ok(())
    }

    fn can_player_act(&self, player_id: Uuid) -> bool {
        match self.get_active_player_id() {
            Some(active_player_id) => player_id == active_player_id,
            None => true,
        }
    }

    fn get_highest_bet(&self) -> Option<u64> {
        if self.player_bets.is_empty() {
            return None;
        }
        Some(self.player_bets.values().max().unwrap().clone())
    }
    fn get_highest_bet_with_bettors(&self) -> Option<(u64, HashSet<Uuid>)> {
        if self.player_bets.is_empty() {
            return None;
        }
        let high_bet_amount = self.player_bets.values().max().unwrap().clone();
        let high_bettors: HashSet<Uuid> = self.player_bets.iter()
            .filter_map(|(key, &value)| {
                if value == high_bet_amount {
                    Some(*key)
                } else {
                    None
                }
            })
            .collect();
        Some((high_bet_amount, high_bettors))
    }
}

impl From<proto::BettingAction> for BettingRoundAction {
    fn from(action: proto::BettingAction) -> Self {
        match action {
            proto::BettingAction::Bet(credits) => BettingRoundAction::Bet(credits),
            proto::BettingAction::Call(_) => BettingRoundAction::Call,
            proto::BettingAction::RaiseBet(credits) => BettingRoundAction::Raise(credits),
            proto::BettingAction::Fold(_) => BettingRoundAction::Fold,
        }
    }
}
