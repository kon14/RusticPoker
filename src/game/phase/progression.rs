use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast::Receiver;
use tokio::sync::RwLock;
use tokio::time::sleep;

use crate::common::error::AppError;
use crate::game::GamePhase;

type ActionProgressionTimeoutHandler = Arc<dyn Fn(Arc<RwLock<GamePhase>>) -> Pin<Box<dyn Future<Output = Result<(), AppError>> + Send>> + Send + Sync>;

pub(super) enum ActionProgression {
    /// Action waits for a fixed delay.
    Delay(Duration),
    /// Action waits for external events or up to a fixed delay fallback.
    Event(Duration, ActionProgressionTimeoutHandler),
}

impl ActionProgression {
    pub fn delay(ms: u64) -> ActionProgression {
        let duration = Duration::from_millis(ms);
        ActionProgression::Delay(duration)
    }

    pub fn event(max_ms: u64, timeout_handler: ActionProgressionTimeoutHandler) -> ActionProgression {
        let max_duration = Duration::from_millis(max_ms);
        ActionProgression::Event(max_duration, timeout_handler)
    }

    pub(super) async fn await_next_action(
        self,
        event_receiver: &mut Receiver<()>,
        phase_arc: Arc<RwLock<GamePhase>>,
    ) {
        // TODO: GamePhase::progress() should check and dismiss invalid player calls early so as not to overflow the channel...
        match self {
            ActionProgression::Delay(duration) => {
                sleep(duration).await;
            },
            ActionProgression::Event(max_duration, timeout_handler) => {
                let timer = sleep(max_duration);
                tokio::select! {
                    _ = timer => {
                        if let Err(err) = timeout_handler(phase_arc).await {
                            eprintln!("{err}"); // TODO
                        }
                    },
                    _ = event_receiver.recv() => {},
                }
            },
        }
    }
}
