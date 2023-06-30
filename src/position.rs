use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Position {
    pub x: u16,
    pub y: u16,
}
