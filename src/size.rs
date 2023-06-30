use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Size {
    pub width: u16,
    pub height: u16,
}
