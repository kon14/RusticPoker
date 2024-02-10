use crate::{
    service::proto::PlayerInfo,
    types::player::Player,
};

impl Into<PlayerInfo> for &Player {
    fn into(self) -> PlayerInfo {
        let user = self.user.upgrade().unwrap();
        PlayerInfo {
            name: user.name.clone(),
            credits: user.credits,
        }
    }
}
