use std::io::{stdout, Stdout, Write};

use crossterm::{
    cursor, execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{enable_raw_mode, size, Clear, ClearType, SetSize, SetTitle},
};

use crate::Size;

pub struct TerminalW {
    size: Size,
    hook: Stdout,
}

impl TerminalW {
    pub fn default() -> Result<Self, std::io::Error> {
        enable_raw_mode();
        let size = size()?;
        Ok(Self {
            size: Size {
                width: size.0,
                height: size.1.saturating_sub(2),
            },
            hook: stdout(),
        })
    }

    pub fn set_title(&mut self, title: &str) {
        execute!(self.hook, SetTitle(title));
    }

    pub fn clear_screen(&mut self) {
        execute!(self.hook, Clear(ClearType::All));
    }

    pub fn clear_current_line(&mut self) {
        execute!(self.hook, Clear(ClearType::CurrentLine));
    }

    pub fn cursor_hide(&mut self) {
        execute!(self.hook, cursor::Hide);
    }

    pub fn cursor_show(&mut self) {
        execute!(self.hook, cursor::Show);
    }

    pub fn set_cursor_shape(&mut self, cursor_shape: cursor::SetCursorStyle) {
        execute!(self.hook, cursor_shape);
    }

    pub fn move_cursor(&mut self, x: u16, y: u16) {
        execute!(self.hook, cursor::MoveTo(x, y));
    }

    pub fn put_glyph(&mut self, chr: char, x: u16, y: u16, fg: Color, bg: Color) {
        self.move_cursor(x, y);

        execute!(
            self.hook,
            SetForegroundColor(fg),
            SetBackgroundColor(bg),
            Print(chr),
            ResetColor
        );
    }

    pub fn put_str(&mut self, line_str: String, x: u16, y: u16, fg: Color, bg: Color) {
        self.move_cursor(x, y);

        execute!(
            self.hook,
            SetForegroundColor(fg),
            SetBackgroundColor(bg),
            Print(line_str.to_string()),
            ResetColor
        );
    }

    pub fn draw_rect(
        &mut self,
        x: u16,
        y: u16,
        w: u16,
        h: u16,
        border_color: Color,
        fill_color: Option<Color>,
    ) {
        let h_line = ('\u{2501}').to_string().repeat(w as usize);

        let v_line = "\u{2503}";

        //Horizontal Lines
        self.put_str(h_line.to_string(), x + 1, y, border_color, Color::Reset);
        self.put_str(h_line.to_string(), x + 1, y + h, border_color, Color::Reset);

        //Vertical Lines
        for i in y + 1..y + h {
            self.put_str(v_line.to_string(), x, i, border_color, Color::Reset);
            self.put_str(v_line.to_string(), x + w, i, border_color, Color::Reset);
        }

        //Down and right
        self.put_str("\u{250F}".to_string(), x, y, border_color, Color::Reset);
        //Down and left
        self.put_str("\u{2513}".to_string(), x + w, y, border_color, Color::Reset);
        //Up Right
        self.put_str("\u{2517}".to_string(), x, y + h, border_color, Color::Reset);
        //Up Left
        self.put_str(
            "\u{251B}".to_string(),
            x + w,
            y + h,
            border_color,
            Color::Reset,
        );

        //Fill
        match fill_color {
            Some(c) => {
                for px in x + 2..x + w - 1 {
                    for py in y + 2..y + h - 1 {
                        self.put_glyph(' ', px, py, c, c)
                    }
                }
            }
            None => {}
        }
    }

    pub fn set_size(&mut self, cols: u16, rows: u16) {
        execute!(self.hook, SetSize(cols, rows));
        self.size = Size {
            width: cols,
            height: rows,
        };
    }

    pub fn get_size(&self) -> &Size {
        &self.size
    }

    pub fn flush(&mut self) -> Result<(), std::io::Error> {
        self.hook.flush()
    }
}
