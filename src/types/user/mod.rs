#[derive(Clone, Debug)]
pub(crate) struct User {
    pub(crate) name: String,
    pub(super) credits: u64,
}

impl User {
    pub(crate) fn new(name: String) -> Self {
        Self {
            name,
            credits: 1000,
        }
    }
}
