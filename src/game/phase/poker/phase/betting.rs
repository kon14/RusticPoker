use std::collections::HashMap;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct BettingRound {
    player_folds: HashMap<Uuid, bool>,
    player_actions: HashMap<Uuid, BettingRoundAction>,
}

#[derive(Clone, Debug)]
pub enum BettingRoundAction {
    Fold,
    Call,
    Raise(u64),
    Check,
}
