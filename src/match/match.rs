use std::collections::{HashMap, HashSet, VecDeque};
use chrono::{DateTime, Duration, Utc};
use uuid::Uuid;

use crate::types::card::Card;
use crate::types::hand::Hand;

#[derive(Clone, Debug)]
pub struct CreditPot {
    id: Uuid,
    match_id: Uuid,
    is_main_pot: bool,
    contributor_ids: HashSet<Uuid>,
    total_credits: u64,
    player_credits: HashMap<Uuid, u64>,
}

impl CreditPot {
    pub fn new(match_id: Uuid, is_main_pot: bool) -> Self {
        CreditPot {
            id: Uuid::new_v4(),
            match_id,
            is_main_pot,
            contributor_ids: HashSet::new(),
            total_credits: 0,
            player_credits: HashMap::new(),
        }
    }

    pub fn add_credits(&mut self, player_id: Uuid, credits: u64) {
        self.contributor_ids.insert(player_id);
        let player_entry = self.player_credits.entry(player_id).or_insert(0);
        *player_entry += credits;
        self.total_credits += credits;
    }
}

// #[derive(Clone)]
// pub struct PlayerHand {
//     player_id: Uuid,
//     hand: Hand,
// }

#[derive(Clone, Debug)]
pub struct PlayerState {
    player_id: Uuid,
    // hand: PlayerHand,
    hand: Hand,
    starting_credits: u64,
    remaining_credits: u64,
    pot_credits: HashMap<Uuid, u64>,
}

#[derive(Clone, Debug)]
pub struct RoundQueue {
    player_queue: VecDeque<Uuid>,
    max_turn_time: Option<Duration>,
    active_player_timer: Option<Duration>,
}

impl RoundQueue {
    pub fn new(player_queue: VecDeque<Uuid>, phase: PokerPhase) -> Self {
        let phase_turn_time = Self::get_phase_turn_time(phase);
        RoundQueue {
            player_queue,
            max_turn_time: phase_turn_time, // TODO: do i need .clone() or is this copy
            active_player_timer: phase_turn_time,
        }
    }

    pub fn next_turn(&mut self) {
        if let Some(dealer_id) = self.player_queue.pop_front() {
            self.player_queue.push_back(dealer_id);
        };
        self.active_player_timer = self.max_turn_time
    }

    fn get_phase_turn_time(phase: PokerPhase) -> Option<Duration> {
        match phase {
            PokerPhase::Ante => None,
            PokerPhase::Dealing => None,
            PokerPhase::FirstBetting => Some(Duration::seconds(30)),
            PokerPhase::Drawing => None,
            PokerPhase::SecondBetting => Some(Duration::seconds(30)),
            PokerPhase::Showdown => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct GameTable {
    match_id: Uuid,
    player_ids: HashSet<Uuid>,
    dealer_id: Uuid,
    credit_pots: HashMap<Uuid, CreditPot>,
    round: PokerRound,
}

impl GameTable {
    pub fn new(match_id: Uuid, player_queue: VecDeque<Uuid>, dealer_id: Uuid) -> Self {
        // let mut rng = thread_rng();
        // let mut player_ids: Vec<Uuid> = player_ids.into_iter().collect(); // this shouldn't be rng, followup from previous order...
        // player_ids.shuffle(&mut rng);
        // let queue: VecDeque<Uuid> = VecDeque::from(player_ids);
        let player_ids = player_queue.iter().cloned().collect();
        let round = PokerRound::new_game(player_queue, dealer_id);
        let mut table = GameTable {
            match_id,
            // dealer_queue: queue.clone(),
            // player_queue: queue,
            dealer_id,
            player_ids,
            credit_pots: HashMap::new(),
            round,
        };
        // table.next_player();
        let main_pot = CreditPot::new(match_id, true);
        table.add_pot(main_pot);
        table
    }

    pub fn player_flops() {
        // implement logic to
        // remove player from queues (move to next player where relevant)
        // handler player being a dealer... dealer lookups should check next still
        // update player state
    }

    // pub fn next_dealer(&mut self) {
    //     if let Some(dealer_id) = self.dealer_queue.pop_front() {
    //         self.dealer_queue.push_back(dealer_id);
    //     };
    // }
    //
    // pub fn next_player(&mut self) {
    //     if let Some(player_id) = self.player_queue.pop_front() {
    //         self.player_queue.push_back(player_id);
    //     };
    // }
    //
    // pub fn get_active_dealer(&self) -> &Uuid {
    //     self.dealer_queue.get(0).unwrap()
    // }
    //
    // pub fn get_active_player(&self) -> &Uuid {
    //     self.player_queue.get(0).unwrap()
    // }

    pub fn add_pot(&mut self, pot: CreditPot) {
        self.credit_pots.insert(pot.id.clone(), pot);
    }
}

fn sort_player_queue(mut player_queue: VecDeque<Uuid>, dealer_id: Uuid) -> VecDeque<Uuid> {
    while let back = player_queue.pop_back().unwrap() {
        if back == dealer_id {
            break;
        }
        player_queue.push_front(back);
    }
    player_queue
}

// TODO: DiscardedCards array or include players internally
#[derive(Clone, Debug)]
struct DiscardedCards(HashSet<Card>);

#[derive(Clone, Debug)]
enum PokerPhase {
    Ante,
    Dealing,
    FirstBetting,
    Drawing,
    SecondBetting,
    Showdown,
}

#[derive(Clone, Debug)]
struct PokerRound {
    phase: PokerPhase,
    round_queue: RoundQueue,
}

impl PokerRound {
    pub fn new_game(player_queue: VecDeque<Uuid>, dealer_id: Uuid) -> PokerRound {
        let player_queue = sort_player_queue(player_queue, dealer_id);
        let round_queue = RoundQueue::new(player_queue, PokerPhase::Ante);
        PokerRound {
            phase: PokerPhase::Ante,
            round_queue,
        }
    }
}

#[derive(Clone, Debug)]
pub enum BettingRoundAction {
    Fold,
    Call,
    Raise(u64),
    Check,
}

#[derive(Clone, Debug)]
pub struct BettingRound {
    player_folds: HashMap<Uuid, bool>,
    player_actions: HashMap<Uuid, BettingRoundAction>,
}

#[derive(Clone, Debug)]
pub struct GameState {
    match_id: Uuid,
    state_time: DateTime<Utc>,
    table: GameTable,
    // player_ids: HashMap<Uuid, PlayerState>,
    // round: PokerRound,
}

impl GameState {
    fn new(match_id: Uuid, player_queue: VecDeque<Uuid>, dealer_id: Uuid) -> Self {
        let table = GameTable::new(match_id, player_queue, dealer_id);
        GameState {
            match_id,
            state_time: Utc::now(),
            table,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Match {
    pub match_id: Uuid,
    pub lobby_id: Uuid,
    pub player_ids: HashSet<Uuid>,
    pub state: GameState,
}

impl Match {
    pub fn new(lobby_id: Uuid, player_ids: HashSet<Uuid>) -> Self {
        let match_id = Uuid::new_v4();
        let player_queue = player_ids.iter().cloned().collect();
        let dealer_id = player_ids.iter().next().unwrap().clone(); // TODO: dealer_id selection
        // let player_queue = sort_player_queue(player_queue, dealer_id); // internal for now
        let state = GameState::new(match_id, player_queue, dealer_id);
        Match {
            match_id,
            lobby_id,
            player_ids,
            state,
        }
    }
}

// impl From<InnerMatch> for proto::MatchState {
//     fn from(value: InnerMatch) -> Self {
//         let player_ids: Vec<String> = value.player_ids.into_iter().map(|id| id.into()).collect();
//         proto::MatchState {
//             match_id: value.match_id.into(),
//             lobby_id: value.lobby_id.into(),
//             player_ids,
//         }
//     }
// }

// pub struct GameSettings {
//
// }
