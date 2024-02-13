use crate::service::proto::PlayerInfo;
use crate::types::player::Player;

impl Into<PlayerInfo> for &Player {
    fn into(self) -> PlayerInfo {
        let user = self.user.upgrade().unwrap();
        PlayerInfo {
            id: user.id.clone(),
            name: user.name.clone(),
            credits: user.credits,
        }
    }
}
