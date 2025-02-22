use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::ops::Deref;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::service::proto::respond_betting_phase_request as proto;
use crate::common::error::AppError;
use crate::game::GamePhase;
use crate::game::phase::poker::{PokerPhase, PokerPhaseBehavior};
use crate::game::phase::poker::r#impl::shift_queue;
use crate::game::phase::progression::ActionProgression;
use crate::output::{MatchStatePhaseSpecifics, MatchStatePhaseSpecificsBetting};
use super::{PokerPhaseBetting, PokerPhaseFirstBetting, PokerPhaseSecondBetting};

// TODO: second betting phase, lurking .unwrap() panic!

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
        // Player actions handled via RPC calls.
        // Timeout actions handled via a callback.
        self.first_round_action = false;
        let _ = shift_queue(&mut self.phase_player_queue); // TODO
    }

    /// Checks whether everyone has either folded or matched the highest bet.
    fn is_phase_completed(&self) -> bool {
        if self.last_man_standing() {
            return true;
        }
        let Some((_, matched_bettors)) = self.get_highest_bet_with_bettors() else {
            // No bets placed yet
            return false;
        };
        let remaining_player_count = self.player_hands.len();
        if self.first_round_action || matched_bettors.len() != remaining_player_count {
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

    fn get_action_progression(&self) -> Option<ActionProgression> {
        let active_player_id = self.get_active_player_id().unwrap();
        let timeout_handler = Arc::new(move |game_phase_arc: Arc<RwLock<GamePhase>>| Box::pin(async move {
            // TODO: improve hacky instance resolution of self
            let mut game_phase_w = game_phase_arc.write().await;
            let mut poker_phase = &mut game_phase_w.poker_phase;
            if let Some(mut betting_phase) = match poker_phase {
                PokerPhase::FirstBetting(ref mut phase) => Some(&mut phase.0),
                PokerPhase::SecondBetting(ref mut phase) => Some(&mut phase.0),
                _ => None,
            } {
                // NOTE: matched bettors already checked in is_phase_completed()
                if let Some((_, high_bettors)) = betting_phase.get_highest_bet_with_bettors() {
                    if high_bettors.contains(&active_player_id) {
                        betting_phase.player_calls(active_player_id)?
                    }
                }

                betting_phase.player_folds(active_player_id)?;
            }
            Ok(())
        }) as Pin<Box<dyn Future<Output = Result<(), AppError>> + Send>>);
        Some(ActionProgression::event(15000, timeout_handler))
    }

    fn get_player_bet_amounts(&self) -> Option<HashMap<Uuid, u64>> {
        Some(self.player_bets.clone())
    }

    fn get_phase_specifics(&self) -> MatchStatePhaseSpecifics {
        unreachable!()
    }

    fn can_player_act(&self) -> HashMap<Uuid, bool> {
        let active_player_id = self.get_active_player_id().unwrap();
        self.game_table
            .player_ids
            .iter()
            .cloned()
            .map(|player_id| (player_id, player_id == active_player_id))
            .collect()
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

    pub(crate) fn last_man_standing(&self) -> bool {
        self.player_hands.len() == 1
    }

    pub(crate) fn get_betting_phase_specifics(&self) -> MatchStatePhaseSpecificsBetting {
        MatchStatePhaseSpecificsBetting {
            highest_bet_amount: self.get_highest_bet().unwrap(),
            player_bet_amounts: self.player_bets.clone(),
        }
    }
}

impl PokerPhaseBetting {
    fn player_folds(&mut self, player_id: Uuid) -> Result<(), AppError> {
        if !self.can_player_act(player_id) {
            return Err(AppError::invalid_request("Player can't act out of turn!"));
        }

        let Some(player_hand) = self.player_hands.remove(&player_id) else {
            return Ok(());
        };

        self.card_deck.discard_cards(player_hand.cards.into());
        self.player_bets.remove(&player_id);

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

            self.player_bets.insert(player_id, bet_credits);
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
        // TODO: Drop Option, bets are carried over
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
        println!("Highest bet amount: {high_bet_amount}");
        println!("high bettors: {high_bettors:?}");
        Some((high_bet_amount, high_bettors))
    }
}

impl Deref for PokerPhaseFirstBetting {
    type Target = PokerPhaseBetting;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for PokerPhaseSecondBetting {
    type Target = PokerPhaseBetting;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<PokerPhaseBetting> for PokerPhaseFirstBetting {
    fn as_ref(&self) -> &PokerPhaseBetting {
        &self.0
    }
}

impl AsRef<PokerPhaseBetting> for PokerPhaseSecondBetting {
    fn as_ref(&self) -> &PokerPhaseBetting {
        &self.0
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
