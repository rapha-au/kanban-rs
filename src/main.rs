mod position;
use position::*;

mod size;
use size::*;

mod selector;
use selector::*;

mod task;
use task::*;

mod block;
use block::*;

mod terminalw;
use terminalw::*;

mod board;
use board::*;

fn main() {
    Board::default().run();
}
