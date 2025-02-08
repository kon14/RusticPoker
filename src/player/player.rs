use uuid::Uuid;

#[derive(Clone, Debug)]
#[derive(Eq, PartialEq, Hash)] // object-level equality
pub struct Player {
    pub player_id: Uuid,
    // pub name: String,
    pub total_credits: u64,
    // events => calculate after table actions
    // pub available_credits: u64,
    // pub reserved_credits: u64
}

impl Player {
    const REGISTRATION_CREDITS: u64 = 500;

    pub fn register() -> Self {
        Player {
            player_id: Uuid::new_v4(),
            total_credits: Self::REGISTRATION_CREDITS,
        }
    }
}
