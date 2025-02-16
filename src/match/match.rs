use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use rand::{rng, seq::SliceRandom};
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

use crate::game::GamePhase;
use crate::player::Player;
use crate::types::hand::Hand;
use crate::output::GameStateBroadcaster;

#[derive(Clone, Debug)]
pub struct PlayerState {
    player_id: Uuid,
    hand: Option<Hand>,
    starting_credits: u64,
}

#[derive(Clone, Debug)]
pub(crate) struct Match {
    pub(crate) match_id: Uuid,
    pub(crate) lobby_id: Uuid,
    pub(crate) player_ids: HashSet<Uuid>,
    pub(crate) phase: Arc<RwLock<GamePhase>>,
}

impl Match {
    pub fn new(
        lobby_id: Uuid,
        state_broadcaster: GameStateBroadcaster,
        rpc_action_broadcaster: broadcast::Sender<()>,
        players: HashSet<Player>,
        ante_amount: u64,
    ) -> Self {
        let match_id = Uuid::new_v4();

        let players = MatchStartPlayers::new(players);
        let player_ids = players.player_credits.keys().cloned().collect();

        let phase = GamePhase::new(
            match_id,
            state_broadcaster,
            rpc_action_broadcaster,
            players,
            ante_amount,
        );

        Match {
            match_id,
            lobby_id,
            player_ids,
            phase: Arc::new(RwLock::new(phase)),
        }
    }

    pub async fn play_poker(&mut self, rpc_action_receiver: broadcast::Receiver<()>,) {
        let phase_arc = self.phase.clone();
        tokio::spawn(async move {
            GamePhase::progress(phase_arc, rpc_action_receiver).await;
        });
    }
}

pub(crate) struct MatchStartPlayers {
    pub(crate) ordered_player_queue: VecDeque<Uuid>, // dealer = 0
    pub(crate) player_credits: HashMap<Uuid, u64>,
    pub(crate) dealer_id: Uuid,
}

impl MatchStartPlayers {
    pub fn new(players: HashSet<Player>) -> Self {
        // This ctor assumes unordered play queue.
        // Implement new_from_game() shifting queue by 1.

        let (mut player_vec, player_credits): (Vec<Uuid>, HashMap<Uuid, u64>) = players
            .iter()
            .map(|player| (player.player_id, (player.player_id, player.total_credits)))
            .unzip();

        let mut rng = rng();
        player_vec.shuffle(&mut rng);
        let ordered_player_queue: VecDeque<Uuid> = player_vec.into();
        let dealer_id = ordered_player_queue.iter().next().unwrap().clone();

        MatchStartPlayers {
            ordered_player_queue,
            player_credits,
            dealer_id,
        }
    }
}
