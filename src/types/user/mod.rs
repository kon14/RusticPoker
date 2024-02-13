#[derive(PartialEq, Clone, Debug)]
pub(crate) struct User {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(super) credits: u64,
}

impl User {
    pub(crate) fn new(id: String, name: String) -> Self {
        Self {
            id,
            name,
            credits: 1000,
        }
    }
}
