use crate::Position;

pub struct Selector {
    pub block_ptr: u8,
    pub task_ptr: u8,
    pub position: Position,
}
impl Selector {
    pub fn default() -> Self {
        Self {
            block_ptr: 0,
            task_ptr: 0,
            position: Position { x: 0, y: 0 },
        }
    }
}
