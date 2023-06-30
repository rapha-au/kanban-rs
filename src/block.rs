use serde::{Deserialize, Serialize};

use crate::Position;
use crate::Size;
use crate::Task;

#[derive(Clone, Serialize, Deserialize)]
pub struct Block {
    pub title: String,
    pub position: Position,
    pub size: Size,
    pub task_list: Vec<Task>,
}
