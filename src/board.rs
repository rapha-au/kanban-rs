use std::env;
use std::fs::File;
use std::path::Path;

use crossterm::{
    cursor,
    event::{read, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    style::{Color, Stylize},
};
use std::io::Write;

use serde_json::json;
use serde_json::Value;

use crate::Block;
use crate::Position;
use crate::Selector;
use crate::Size;
use crate::Task;
use crate::TaskStatus;
use crate::TerminalW;

#[derive(PartialEq)]
pub enum InputMode {
    Command,
    WritingTask,
    WritingBoard,
}
#[derive(PartialEq)]
pub enum WritingTaskType {
    WritingTitle,
    WritingDescription,
}

pub struct Board {
    quit: bool,
    term: TerminalW, // Terminal reference
    is_modified: bool,

    board_name: String,
    block_list: [Block; 3],
    selector: Selector,

    input_state: InputMode, // Input Mode the user is in
    showing_task: bool,
    writing_string: String,
    writing_count: WritingTaskType,

    tmp_task: Task,
}

impl Board {
    pub fn default() -> Self {
        let args: Vec<String> = env::args().collect();

        let mut tmp_filepath = String::from("");
        let mut tmp_filename = String::from("NONE");

        let spacing = 2;

        let starting_x = 15;
        let starting_y = 5;

        let default_width = 30;
        let default_height = 25;

        if args.len() > 1 {
            // LOADING!

            let file_arg: String = args[1].clone();

            let p = Path::new(&file_arg);

            tmp_filepath = p.parent().unwrap().to_str().unwrap().to_string();

            tmp_filename = p.file_name().unwrap().to_str().unwrap().to_string();
        }

        Self {
            quit: false,
            term: TerminalW::default().expect("Failure to initialize."),
            is_modified: true,
            board_name: tmp_filename.to_string(),
            block_list: [
                Block {
                    title: "To Do".to_string(),
                    position: Position {
                        x: starting_x,
                        y: starting_y,
                    },
                    size: Size {
                        width: default_width,
                        height: default_height,
                    },
                    task_list: vec![],
                },
                Block {
                    title: "Active".to_string(),
                    position: Position {
                        x: starting_x + spacing + default_width,
                        y: starting_y,
                    },
                    size: Size {
                        width: default_width,
                        height: default_height,
                    },
                    task_list: vec![],
                },
                Block {
                    title: "Completed".to_string(),
                    position: Position {
                        x: starting_x + (spacing * 2) + (default_width * 2),
                        y: starting_y,
                    },
                    size: Size {
                        width: default_width,
                        height: default_height,
                    },
                    task_list: vec![],
                },
            ],
            selector: Selector::default(),
            input_state: InputMode::Command,
            showing_task: false,
            writing_string: String::from(""),
            writing_count: WritingTaskType::WritingTitle,
            tmp_task: Task {
                status: TaskStatus::Todo,
                title: "".to_string(),
                description: "".to_string(),
            },
        }
    }

    pub fn run(&mut self) {
        //Set Size
        self.term.set_size(128, 36);

        //Not a good solution
        if self.board_name.ne("NONE") {
            self.load();
        }

        //Set Title
        self.term.set_title(self.board_name.as_str());
        //Set Cursor Shape
        self.term
            .set_cursor_shape(cursor::SetCursorStyle::BlinkingBlock);

        self.term.clear_screen();
        self.term.flush().unwrap();

        loop {
            if self.quit {
                self.term.clear_screen();
                break;
            } else {
                self.draw_board();
                self.update();

                self.process_input();

                self.term.flush().unwrap();
            }
        }
    }

    pub fn update(&mut self) {
        if self.is_modified == true {
            let bname: String = self.board_name.clone();
            self.term.set_title(bname.as_str());
        } else {
            self.term.set_title(self.board_name.as_str());
        }

        match self.input_state {
            InputMode::Command => {
                self.put_bottom_bar("q - quit | s - save | c - create task | d - delete task | < or > - quick task block shift | Enter - Show Task/Hide Task".to_string());
                self.calc_selector_pos();
                if self.showing_task == true {
                    self.show_task();
                }
            }
            InputMode::WritingTask => {
                match self.writing_count {
                    WritingTaskType::WritingTitle => {
                        self.put_bottom_bar("Title:".to_string());
                    }
                    WritingTaskType::WritingDescription => {
                        self.put_bottom_bar("Description:".to_string());
                    }
                }

                self.clear_writing_line();

                self.term.put_str(
                    self.writing_string.clone(),
                    0,
                    self.term.get_size().height - 2,
                    Color::Black,
                    Color::White,
                );
            }
            InputMode::WritingBoard => {
                self.clear_writing_line();

                self.term.put_str(
                    self.writing_string.clone(),
                    0,
                    self.term.get_size().height - 2,
                    Color::Black,
                    Color::White,
                );

                if self.writing_count == WritingTaskType::WritingTitle {
                    self.put_bottom_bar("Board Name:".to_string());
                }
            }
        }
    }

    fn calc_selector_pos(&mut self) {
        self.term.move_cursor(
            self.block_list[self.selector.block_ptr as usize].position.x + 4,
            self.block_list[self.selector.block_ptr as usize].position.y + 2,
        );
    }

    fn clear_writing_line(&mut self) {
        self.term.move_cursor(0, self.term.get_size().height - 2);
        self.term.clear_current_line();
    }

    fn create_task(&mut self) {
        match self.selector.block_ptr {
            0 => {
                self.tmp_task.status = TaskStatus::Todo;
            }
            1 => {
                self.tmp_task.status = TaskStatus::Active;
            }
            2 => self.tmp_task.status = TaskStatus::Completed,
            _ => {}
        }

        self.block_list[self.selector.block_ptr as usize]
            .task_list
            .push(self.tmp_task.clone());
        self.is_modified = true;
        self.term.clear_screen();
    }

    fn remove_task(&mut self) -> Option<Task> {
        let mut r = None;
        if self.block_list[self.selector.block_ptr as usize]
            .task_list
            .len()
            != 0
        {
            r = Some(
                self.block_list[self.selector.block_ptr as usize]
                    .task_list
                    .remove(self.selector.task_ptr.into()),
            );
        }
        r
    }

    fn show_task(&mut self) {
        if self.block_list[self.selector.block_ptr as usize]
            .task_list
            .len()
            > 0
        {
            let boxx: u16 = 30;
            let boxy: u16 = 6;
            let boxw: u16 = 50;
            let boxh: u16 = 20;

            self.term
                .draw_rect(boxx, boxy, boxw, boxh, Color::White, Some(Color::White));

            //Title of the entry being displayed
            self.term.put_str(
                self.block_list[self.selector.block_ptr as usize].task_list
                    [self.selector.task_ptr as usize]
                    .title
                    .clone(),
                boxx + 1,
                boxy,
                Color::Black,
                Color::White,
            );

            let mut broken_description: Vec<String> = vec![];
            let mut tmp_str = String::from("");
            let tmp_description = self.block_list[self.selector.block_ptr as usize].task_list
                [self.selector.task_ptr as usize]
                .description
                .clone();

            for c in 0..tmp_description.len() {
                tmp_str.push(tmp_description.chars().nth(c).unwrap());
                if tmp_str.len() as u16 == boxw - 3 || c + 1 >= tmp_description.len() {
                    broken_description.push(tmp_str.clone());
                    tmp_str = "".to_string();
                }
            }

            for line in 0..broken_description.len() {
                self.term.put_str(
                    broken_description[line].to_string(),
                    boxx + 2,
                    boxy + 2 + (line as u16),
                    Color::Black,
                    Color::White,
                )
            }
        }
    }

    pub fn process_input(&mut self) {
        if self.input_state == InputMode::Command {
            if let Event::Key(key) = read().unwrap() {
                match key {
                    // QUIT
                    KeyEvent {
                        code: KeyCode::Char('q'),
                        modifiers: KeyModifiers::NONE,
                        kind: KeyEventKind::Press,
                        ..
                    } => {
                        if self.input_state == InputMode::Command {
                            self.quit = true;
                        }
                    }

                    // SAVE
                    KeyEvent {
                        code: KeyCode::Char('s'),
                        modifiers: KeyModifiers::NONE,
                        kind: KeyEventKind::Release,
                        ..
                    } => {
                        if self.check_save() == true {
                            self.save();
                            self.is_modified = false;
                        } else {
                            self.input_state = InputMode::WritingBoard;
                        }
                        self.term.clear_screen();
                    }

                    // CREATE TASK
                    KeyEvent {
                        code: KeyCode::Char('c'),
                        modifiers: KeyModifiers::NONE,
                        kind: KeyEventKind::Release,
                        ..
                    } => {
                        if self.input_state == InputMode::Command {
                            self.input_state = InputMode::WritingTask;
                        }
                    }

                    // DELETE TASK
                    KeyEvent {
                        code: KeyCode::Char('d'),
                        modifiers: KeyModifiers::NONE,
                        kind: KeyEventKind::Release,
                        ..
                    } => {
                        if self.input_state == InputMode::Command {
                            self.remove_task();
                            self.is_modified = true;
                            self.term.clear_screen();
                        }
                    }

                    // SEE TASK
                    KeyEvent {
                        code: KeyCode::Enter,
                        modifiers: KeyModifiers::NONE,
                        kind: KeyEventKind::Press,
                        ..
                    } => {
                        if self.input_state == InputMode::Command {
                            if self.showing_task == true {
                                self.showing_task = false;
                                self.term.clear_screen();
                            } else {
                                self.showing_task = true;
                                self.term.clear_screen();
                            }
                        }
                    }

                    // MOVE SELECTOR UP
                    KeyEvent {
                        code: KeyCode::Up,
                        kind: KeyEventKind::Release,
                        ..
                    } => {
                        if self.input_state == InputMode::Command {
                            if self.selector.task_ptr > 0 {
                                // Can go up Minus
                                self.selector.task_ptr -= 1;
                                self.term.clear_screen();
                            }
                        }
                    }

                    // MOVE SELECTOR DOWN
                    KeyEvent {
                        code: KeyCode::Down,
                        kind: KeyEventKind::Release,
                        ..
                    } => {
                        if self.input_state == InputMode::Command {
                            if self.block_list[self.selector.block_ptr as usize]
                                .task_list
                                .len()
                                != 0
                            {
                                if (self.selector.task_ptr as usize)
                                    < self.block_list[self.selector.block_ptr as usize]
                                        .task_list
                                        .len()
                                        - 1
                                {
                                    //Can go down - Plus
                                    self.selector.task_ptr += 1;
                                    self.term.clear_screen();
                                }
                            }
                        }
                    }

                    // MOVE SELECTOR RIGHT
                    KeyEvent {
                        code: KeyCode::Right,
                        kind: KeyEventKind::Release,
                        ..
                    } => {
                        if self.input_state == InputMode::Command {
                            if self.selector.block_ptr < (self.block_list.len() - 1) as u8 {
                                self.selector.block_ptr += 1;
                                self.selector.task_ptr = 0;
                                self.term.clear_screen();
                            }
                        }
                    }

                    // MOVE SELECTOR LEFT
                    KeyEvent {
                        code: KeyCode::Left,
                        kind: KeyEventKind::Release,
                        ..
                    } => {
                        if self.input_state == InputMode::Command {
                            if self.selector.block_ptr > 0 {
                                self.selector.block_ptr -= 1;
                                self.selector.task_ptr = 0;
                                self.term.clear_screen();
                            }
                        }
                    }

                    // QUICK PUSH RIGHT
                    KeyEvent {
                        code: KeyCode::Char('>'),
                        kind: KeyEventKind::Release,
                        ..
                    } => {
                        if self.input_state == InputMode::Command {
                            if usize::from(self.selector.block_ptr + 1) < self.block_list.len() {
                                match self.remove_task() {
                                    Some(t) => {
                                        self.block_list[usize::from(self.selector.block_ptr + 1)]
                                            .task_list
                                            .push(t);
                                        self.is_modified = true;
                                    }
                                    None => {}
                                }
                            }
                        }
                        self.term.clear_screen();
                    }

                    // QUICK PUSH LEFT
                    KeyEvent {
                        code: KeyCode::Char('<'),
                        kind: KeyEventKind::Release,
                        ..
                    } => {
                        if self.input_state == InputMode::Command {
                            match self.selector.block_ptr.checked_sub(1) {
                                Some(_) => match self.remove_task() {
                                    Some(t) => {
                                        self.block_list[usize::from(self.selector.block_ptr - 1)]
                                            .task_list
                                            .push(t);
                                        self.is_modified = true;
                                    }
                                    None => {}
                                },
                                None => {}
                            }
                        }
                        self.term.clear_screen();
                    }

                    _ => {}
                }
            }
        } else if self.input_state == InputMode::WritingTask
            || self.input_state == InputMode::WritingBoard
        {
            if let Event::Key(key) = read().unwrap() {
                match key {
                    // CONFIRMATION
                    KeyEvent {
                        code: KeyCode::Enter,
                        modifiers: KeyModifiers::NONE,
                        kind: KeyEventKind::Press,
                        ..
                    } => {
                        if self.input_state == InputMode::WritingTask {
                            match self.writing_count {
                                WritingTaskType::WritingTitle => {
                                    self.tmp_task.title = self.writing_string.clone();
                                    self.writing_string = "".to_string();
                                    self.writing_count = WritingTaskType::WritingDescription;
                                }
                                WritingTaskType::WritingDescription => {
                                    self.tmp_task.description = self.writing_string.clone();
                                    self.writing_string = "".to_string();
                                    self.writing_count = WritingTaskType::WritingTitle;
                                    self.input_state = InputMode::Command;

                                    self.create_task();
                                    self.is_modified = true;
                                }
                            }
                        } else if self.input_state == InputMode::WritingBoard {
                            match self.writing_count {
                                WritingTaskType::WritingTitle => {
                                    self.board_name = self.writing_string.clone();
                                    self.writing_string = "".to_string();
                                    self.writing_count = WritingTaskType::WritingTitle;
                                    self.input_state = InputMode::Command;

                                    self.save();
                                    self.is_modified = false;
                                    self.term.clear_screen();
                                }
                                _ => {}
                            }
                        }
                    }

                    // WRITE CHARS
                    KeyEvent {
                        code: KeyCode::Char(key),
                        kind: KeyEventKind::Release,
                        ..
                    } => {
                        if self.input_state == InputMode::WritingTask
                            || self.input_state == InputMode::WritingBoard
                        {
                            match self.writing_count {
                                WritingTaskType::WritingTitle => {
                                    if self.writing_string.len() < 15 {
                                        //15 is the limit
                                        self.writing_string.push(key);
                                    }
                                }
                                WritingTaskType::WritingDescription => {
                                    if self.writing_string.len() < 250 {
                                        //250 is the limit of characters
                                        self.writing_string.push(key);
                                    }
                                }
                            }
                        }
                    }

                    // ERASE CHARS
                    KeyEvent {
                        code: KeyCode::Backspace,
                        kind: KeyEventKind::Release,
                        ..
                    } => {
                        if self.input_state == InputMode::WritingTask
                            || self.input_state == InputMode::WritingBoard
                        {
                            if self.writing_string.len() > 0 {
                                self.writing_string.pop();
                            }
                        }
                    }

                    _ => {}
                }
            }
        }
    }

    fn put_board_name(&mut self) {
        let mut board_str: String = self.board_name.clone();

        if self.is_modified == true {
            board_str.push('*');
        } else {
            board_str = self.board_name.clone();
        }

        self.term.put_str(
            board_str,
            (self.term.get_size().width / 2) as u16 + (self.board_name.len() / 2) as u16,
            0,
            Color::Black,
            Color::White,
        );
    }

    fn draw_board(&mut self) {
        self.put_board_name();

        self.put_block(0);
    }

    fn trim_str(&self, trim_str: String, trim_index: usize) -> String {
        //starting pos x
        //block x +1
        if trim_str.len() > trim_index {
            let ts = trim_str.clone();
            let mut s: String = ts[..trim_index].into();
            s.push('-');
            return s;
        }
        trim_str
    }

    fn hit_block_bottom(&self, line: usize) -> bool {
        //Bottom is line set 7
        if line >= 7 {
            return true;
        }
        false
    }

    fn put_tasks(&mut self) {
        for block in self.block_list.iter() {
            let mut it = 0;
            let starting_task: usize = self.selector.task_ptr.into();
            if block.task_list.len() > 0 {
                if self.block_list[self.selector.block_ptr as usize].title == block.title {
                    //Rendering the selector block
                    for i in starting_task..block.task_list.len() {
                        //if task position didn't hit the bottom of block
                        // Draws Title
                        if it < 6 {
                            self.term.put_str(
                                self.trim_str(
                                    block.task_list[i].title.clone(),
                                    (block.size.width - 6).into(),
                                ),
                                block.position.x + 5,
                                block.position.y + 2 + (4 * it),
                                Color::White,
                                Color::Reset,
                            );
                            //Draws Description
                            self.term.put_str(
                                self.trim_str(
                                    block.task_list[i].description.clone(),
                                    (block.size.width - 6).into(),
                                ),
                                block.position.x + 5,
                                block.position.y + 3 + (4 * it),
                                Color::Grey,
                                Color::Reset,
                            );

                            it += 1;
                        }
                    }
                    it = 0;
                } else {
                    //Rendering NOT the selector block
                    for i in 0..block.task_list.len() {
                        if it < 6 {
                            //if task position didn't hit the bottom of block
                            // Draws Title
                            self.term.put_str(
                                self.trim_str(
                                    block.task_list[i].title.clone(),
                                    (block.size.width - 6).into(),
                                ),
                                block.position.x + 5,
                                block.position.y + 2 + (4 * it),
                                Color::White,
                                Color::Reset,
                            );
                            //Draws Description
                            self.term.put_str(
                                self.trim_str(
                                    block.task_list[i].description.clone(),
                                    (block.size.width - 6).into(),
                                ),
                                block.position.x + 5,
                                block.position.y + 3 + (4 * it),
                                Color::Grey,
                                Color::Reset,
                            );

                            it += 1;
                        }
                    }
                }
            }
        }
    }

    fn put_block(&mut self, block_index: usize) {
        for (i, block) in self.block_list.iter().enumerate() {
            //Put Name
            self.term.put_str(
                block.title.clone(),
                block.position.x - 1 + block.size.width / 2,
                block.position.y - 2,
                Color::Black,
                Color::White,
            );

            //Put Rect
            let mut color: Color = Color::White;
            match i {
                0 => {
                    color = Color::Red;
                }
                1 => {
                    color = Color::Yellow;
                }
                2 => {
                    color = Color::Green;
                }
                _ => {}
            }

            self.term.draw_rect(
                block.position.x,
                block.position.y,
                block.size.width,
                block.size.height,
                color,
                None,
            );

            //Task Index
            if self.selector.block_ptr as usize == i {
                //Pointer on block
                self.term.put_str(
                    format!("{}/{}", self.selector.task_ptr + 1, block.task_list.len()),
                    block.position.x + block.size.width - 4,
                    block.position.y,
                    Color::Red,
                    Color::White,
                );
            } else {
                //Pointer not on block
                let mut initial = 0;
                let mut end = 0;
                if block.task_list.len() > 0 {
                    initial = 1;
                    end = block.task_list.len();
                }
                self.term.put_str(
                    format!("{}/{}", initial, end),
                    block.position.x + block.size.width - 4,
                    block.position.y,
                    Color::Red,
                    Color::White,
                );
            }
        }

        //Draw Tasks
        self.put_tasks();
    }

    fn put_bottom_bar(&mut self, bar_str: String) {
        let spaces: String = " ".repeat(self.term.get_size().width as usize);
        self.term.move_cursor(0, self.term.get_size().height - 3);
        self.term.clear_current_line();
        print!("{}", spaces.black().on_white());
        self.term.put_str(
            bar_str,
            0,
            self.term.get_size().height - 3,
            Color::Black,
            Color::White,
        );
    }

    fn check_save(&mut self) -> bool {
        Path::new(&format!("{}.json", self.board_name.clone())).exists()
    }

    fn save(&mut self) {
        let args: Vec<String> = env::args().collect();
        let board_json: serde_json::Value;

        board_json = json!({
        "board-name":self.board_name,

            "blocks":{
                "todo":self.block_list[0].task_list,
                "active":self.block_list[1].task_list,
                "completed":self.block_list[2].task_list
            }
        });

        let board_json_str = serde_json::to_string_pretty(&board_json).unwrap();

        let mut file = File::create(self.board_name.clone() + &".json".to_string()).unwrap();
        file.write_all(&board_json_str.into_bytes()).unwrap();
    }

    fn load(&mut self) {
        //Throw error if the file structure doesn't satisfy the requirements
        self.is_modified = false;

        let contents = std::fs::read_to_string(self.board_name.as_str()).unwrap();

        let data: Value = serde_json::from_str(&contents).unwrap();

        self.board_name = data["board-name"].to_string().replace("\"", "");

        self.block_list[0].task_list =
            serde_json::from_value(data["blocks"]["todo"].clone()).unwrap();
        self.block_list[1].task_list =
            serde_json::from_value::<Vec<Task>>(data["blocks"]["active"].clone()).unwrap();
        self.block_list[2].task_list =
            serde_json::from_value::<Vec<Task>>(data["blocks"]["completed"].clone()).unwrap();
    }
}
