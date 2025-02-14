use std::time::Duration;
use tokio::sync::broadcast::Receiver;
use tokio::time::sleep;

pub(super) enum ActionProgression {
    /// Action waits for a fixed delay.
    Delay(Duration),
    /// Action waits for external events or up to a fixed delay fallback.
    Event(Duration),
}

// pub(super) enum ActionProgression {
//     Scheduled(Duration),
//     // External(tokio::sync::mpsc::Receiver<()>),
//     Mixed((Duration, tokio::sync::mpsc::Receiver<()>)),
// }

impl ActionProgression {
    // pub(super) fn scheduled(ms: u64) -> ActionProgression {
    //     let duration = Duration::from_millis(ms);
    //     ActionProgression::Scheduled(duration)
    // }
    //
    // // pub(super) fn external(receiver: tokio::sync::mpsc::Receiver<()>) -> ActionProgression {
    // //     ActionProgression::External(receiver)
    // // }
    //
    // pub(super) fn mixed(ms: u64, receiver: tokio::sync::mpsc::Receiver<()>) -> ActionProgression {
    //     let duration = Duration::from_millis(ms);
    //     ActionProgression::Mixed((duration, receiver))
    // }
    //
    // pub(super) async fn await_next_action(self) {
    //     match self {
    //         ActionProgression::Scheduled(duration) => {
    //             sleep(duration).await;
    //         },
    //         ActionProgression::Mixed((duration, mut event_receiver)) => {
    //             let timer = sleep(duration);
    //             tokio::select! {
    //                 _ = timer => {
    //                     println!("timer progression ---- !!!"); // TEST
    //                     // Time-based progression happened, continue to next iteration
    //                 }
    //                 _ = event_receiver.recv() => {
    //                     println!("")
    //                     // External event received, continue to next iteration (likely after player input)
    //                 }
    //             }
    //         },
    //     }
    // }

    pub fn delay(ms: u64) -> ActionProgression {
        let duration = Duration::from_millis(ms);
        ActionProgression::Delay(duration)
    }

    pub fn event(max_ms: u64) -> ActionProgression {
        let max_duration = Duration::from_millis(max_ms);
        ActionProgression::Event(max_duration)
    }

    pub(super) async fn await_next_action(self, event_receiver: &mut Receiver<()>) {
        // TODO: GamePhase::progress() should check and dismiss invalid player calls early so as not to overflow the channel...
        match self {
            ActionProgression::Delay(duration) => {
                sleep(duration).await;
            },
            ActionProgression::Event(max_duration) => {
                let timer = sleep(max_duration);
                tokio::select! {
                    _ = timer => {
                        println!("delay max progression ---- !!!"); // TEST
                    }
                    _ = event_receiver.recv() => {
                        println!("event received --- !!!!") // TEST
                    }
                }
            },
        }
    }
}
