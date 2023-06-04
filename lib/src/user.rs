use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct User {
    id: String,
}

impl User {
    pub fn new(id: String) -> Self {
        Self { id }
    }
}
