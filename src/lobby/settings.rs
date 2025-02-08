use crate::common::error::AppError;
use crate::service::proto;

#[derive(Clone, Debug)]
pub struct LobbySettings {
    pub min_players: u8,
    pub max_players: u8,
    pub ante_amount: u64,
}

impl LobbySettings {
    const MIN_PLAYERS: u8 = 2;
    const MAX_PLAYERS: u8 = 6; // 8, // TODO: card discard reshuffling
    const DEFAULT_ANTE_AMOUNT: u64 = 10;

    fn new(min_players: u8, max_players: u8, ante_amount: u64) -> Result<Self, AppError> {
        if min_players < Self::MIN_PLAYERS {
          return Err(
              AppError::invalid_request(
                  format!("Minimum number of players ({}) not met!", Self::MIN_PLAYERS)
              )
          )
        }
        if max_players > Self::MAX_PLAYERS {
            return Err(
                AppError::invalid_request(
                    format!("Maximum number of players ({}) exceeded!", Self::MAX_PLAYERS)
                )
            )
        }
        Ok(LobbySettings {
            min_players,
            max_players,
            ante_amount,
        })
    }
}

impl Default for LobbySettings {
    fn default() -> Self {
        LobbySettings {
            min_players: Self::MIN_PLAYERS,
            max_players: Self::MAX_PLAYERS,
            ante_amount: Self::DEFAULT_ANTE_AMOUNT,
        }
    }
}

impl From<LobbySettings> for proto::LobbySettings {
    fn from(settings: LobbySettings) -> Self {
        proto::LobbySettings {
            game_mode: proto::lobby_settings::GameMode::Single as i32,
            min_players: settings.min_players.into(),
            max_players: settings.max_players.into(),
            ante_amount: settings.ante_amount,
        }
    }
}
