use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub email: String,
}
