use crate::buffer::Buffer;
use crate::mode::Mode;
use crossterm::event::KeyCode;

pub struct Editor {
    pub buffer: Buffer,
    pub cursor: (usize, usize),        // (row, col)
    pub scroll_offset: (usize, usize), // (row, col)
    pub terminal_size: (u16, u16),
    pub mode: Mode,
    pub clipboard: String,
    pub pending_operator: Option<char>,
    pub command_buffer: String,
    pub search_query: String,
    pub last_search_dir: bool, // true for forward (/), false for backward (?)
    pub undo_stack: Vec<ropey::Rope>,
    pub redo_stack: Vec<ropey::Rope>,
    pub file_path: Option<String>,
    pub status_message: String,
    pub should_quit: bool,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            buffer: Buffer::new(),
            cursor: (0, 0),
            scroll_offset: (0, 0),
            terminal_size: (0, 0),
            mode: Mode::Normal,
            clipboard: String::new(),
            pending_operator: None,
            command_buffer: String::new(),
            search_query: String::new(),
            last_search_dir: true,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            file_path: None,
            status_message: String::from("Welcome to RUNE! Press ':' for commands."),
            should_quit: false,
        }
    }

    pub fn handle_key_event(&mut self, event: crossterm::event::KeyEvent) {
        match self.mode {
            Mode::Normal => self.handle_normal_mode(event),
            Mode::Insert => self.handle_insert_mode(event),
            Mode::Command => self.handle_command_mode(event),
            _ => {}
        }
    }

    fn handle_normal_mode(&mut self, event: crossterm::event::KeyEvent) {
        use crossterm::event::KeyCode;

        if let Some(op) = self.pending_operator {
            match event.code {
                KeyCode::Char('w')
                | KeyCode::Char('b')
                | KeyCode::Char('h')
                | KeyCode::Char('j')
                | KeyCode::Char('k')
                | KeyCode::Char('l')
                | KeyCode::Char('0')
                | KeyCode::Char('$') => {
                    self.execute_operator(op, event.code);
                    self.pending_operator = None;
                }
                KeyCode::Esc => self.pending_operator = None,
                _ => {}
            }
            self.scroll();
            return;
        }

        match event.code {
            KeyCode::Char('i') => {
                self.save_state();
                self.mode = Mode::Insert;
            }
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('h') => self.move_cursor(0, -1),
            KeyCode::Char('j') => self.move_cursor(1, 0),
            KeyCode::Char('k') => self.move_cursor(-1, 0),
            KeyCode::Char('l') => self.move_cursor(0, 1),
            KeyCode::Char('w') => self.move_to_next_word(),
            KeyCode::Char('b') => self.move_to_prev_word(),
            KeyCode::Char('0') => self.cursor.1 = 0,
            KeyCode::Char('$') => {
                let line_idx = self.cursor.0;
                if line_idx < self.buffer.content.len_lines() {
                    let len = self.buffer.content.line(line_idx).len_chars();
                    self.cursor.1 = if len > 0 { len - 1 } else { 0 };
                }
            }
            KeyCode::Char('g') => {
                // Simplified 'gg'
                self.cursor = (0, 0);
            }
            KeyCode::Char('G') => {
                let last_line = self.buffer.content.len_lines().saturating_sub(1);
                self.cursor = (last_line, 0);
            }
            KeyCode::Char('d') => self.pending_operator = Some('d'),
            KeyCode::Char('y') => self.pending_operator = Some('y'),
            KeyCode::Char('p') => self.paste(),
            KeyCode::Char(':') => {
                self.mode = Mode::Command;
                self.command_buffer = String::from(":");
            }
            KeyCode::Char('/') => {
                self.mode = Mode::Command;
                self.command_buffer = String::from("/");
            }
            KeyCode::Char('n') => self.find_next(),
            KeyCode::Char('N') => self.find_prev(),
            KeyCode::Char('u') => self.undo(),
            KeyCode::Char('r')
                if event
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL) =>
            {
                self.redo()
            }
            _ => {}
        }
        self.scroll();
    }

    fn save_state(&mut self) {
        self.undo_stack.push(self.buffer.content.clone());
        if self.undo_stack.len() > 50 {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
    }

    fn undo(&mut self) {
        if let Some(state) = self.undo_stack.pop() {
            self.redo_stack.push(self.buffer.content.clone());
            self.buffer.content = state;
        }
    }

    fn redo(&mut self) {
        if let Some(state) = self.redo_stack.pop() {
            self.undo_stack.push(self.buffer.content.clone());
            self.buffer.content = state;
        }
    }

    fn handle_command_mode(&mut self, event: crossterm::event::KeyEvent) {
        use crossterm::event::KeyCode;
        match event.code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
                self.command_buffer.clear();
            }
            KeyCode::Enter => {
                let cmd = self.command_buffer.clone();
                self.execute_command(cmd);
                self.mode = Mode::Normal;
                self.command_buffer.clear();
            }
            KeyCode::Char(c) => self.command_buffer.push(c),
            KeyCode::Backspace => {
                if self.command_buffer.len() > 1 {
                    self.command_buffer.pop();
                } else {
                    self.mode = Mode::Normal;
                    self.command_buffer.clear();
                }
            }
            _ => {}
        }
    }

    fn execute_command(&mut self, cmd: String) {
        if cmd.starts_with(':') {
            let parts: Vec<&str> = cmd[1..].split_whitespace().collect();
            if parts.is_empty() {
                return;
            }

            match parts[0] {
                "q" => self.should_quit = true,
                "w" => {
                    let path = cmd[1..].trim_start_matches('w').trim();
                    if !path.is_empty() {
                        self.save_file(Some(path.to_string()));
                    } else {
                        self.save_file(None);
                    }
                }
                "wq" => {
                    self.save_file(None);
                    self.should_quit = true;
                }
                "e" => {
                    let path = cmd[1..].trim_start_matches('e').trim();
                    if !path.is_empty() {
                        self.open_file(path);
                    }
                }
                _ => {}
            }
        } else if cmd.starts_with('/') {
            self.search_query = cmd[1..].to_string();
            self.last_search_dir = true;
            self.find_next();
        }
    }

    pub fn open_file(&mut self, path: &str) {
        if let Ok(content) = std::fs::read_to_string(path) {
            self.buffer = Buffer::from_str(&content);
            self.file_path = Some(path.to_string());
            self.cursor = (0, 0);
            self.scroll_offset = (0, 0);
            self.undo_stack.clear();
            self.redo_stack.clear();
        }
    }

    fn save_file(&mut self, path: Option<String>) {
        let save_path = path.or_else(|| self.file_path.clone());
        if let Some(p) = save_path {
            let content = self.buffer.content.to_string();
            match std::fs::write(&p, content) {
                Ok(_) => {
                    self.file_path = Some(p.clone());
                    self.status_message = format!("Saved to {}", p);
                }
                Err(e) => self.status_message = format!("Error saving: {}", e),
            }
        } else {
            self.status_message = String::from("No file path specified");
        }
    }

    fn find_next(&mut self) {
        if self.search_query.is_empty() {
            return;
        }
        let char_idx = self.cursor_to_char_idx() + 1;
        let content = self.buffer.content.to_string();
        if let Some(pos) = content[char_idx..].find(&self.search_query) {
            self.char_idx_to_cursor(char_idx + pos);
        } else if let Some(pos) = content[..char_idx].find(&self.search_query) {
            // Wrap around
            self.char_idx_to_cursor(pos);
        }
    }

    fn find_prev(&mut self) {
        if self.search_query.is_empty() {
            return;
        }
        let char_idx = self.cursor_to_char_idx();
        let content = self.buffer.content.to_string();
        if let Some(pos) = content[..char_idx].rfind(&self.search_query) {
            self.char_idx_to_cursor(pos);
        } else if let Some(pos) = content[char_idx..].rfind(&self.search_query) {
            // Wrap around
            self.char_idx_to_cursor(char_idx + pos);
        }
    }

    fn execute_operator(&mut self, op: char, motion_code: KeyCode) {
        self.save_state();
        let start_idx = self.cursor_to_char_idx();

        // Save current cursor
        let old_cursor = self.cursor;

        // Apply motion
        match motion_code {
            KeyCode::Char('w') => self.move_to_next_word(),
            KeyCode::Char('b') => self.move_to_prev_word(),
            KeyCode::Char('h') => self.move_cursor(0, -1),
            KeyCode::Char('j') => self.move_cursor(1, 0),
            KeyCode::Char('k') => self.move_cursor(-1, 0),
            KeyCode::Char('l') => self.move_cursor(0, 1),
            _ => {}
        }

        let end_idx = self.cursor_to_char_idx();
        let range = if start_idx < end_idx {
            start_idx..end_idx
        } else {
            end_idx..start_idx
        };

        if op == 'd' {
            let deleted = self.buffer.content.slice(range.clone()).to_string();
            self.clipboard = deleted;
            self.buffer.content.remove(range);
            // After delete, cursor should stay at start of range
            self.char_idx_to_cursor(if start_idx < end_idx {
                start_idx
            } else {
                end_idx
            });
        } else if op == 'y' {
            let yanked = self.buffer.content.slice(range).to_string();
            self.clipboard = yanked;
            self.cursor = old_cursor; // Reset cursor after yank
        }
    }

    fn paste(&mut self) {
        if self.clipboard.is_empty() {
            return;
        }
        let idx = self.cursor_to_char_idx();
        self.buffer.content.insert(idx, &self.clipboard);
    }

    fn move_cursor(&mut self, row_delta: i32, col_delta: i32) {
        let mut row = self.cursor.0 as i32 + row_delta;
        let mut col = self.cursor.1 as i32 + col_delta;

        let line_count = self.buffer.content.len_lines();
        row = row.clamp(0, line_count.saturating_sub(1) as i32);

        let line_len = self.buffer.content.line(row as usize).len_chars();
        col = col.clamp(0, line_len.saturating_sub(1).max(0) as i32);

        self.cursor = (row as usize, col as usize);
    }

    fn move_to_next_word(&mut self) {
        let mut char_idx = self.cursor_to_char_idx();
        let len = self.buffer.content.len_chars();

        // Skip current non-whitespace if any
        while char_idx < len && !self.buffer.content.char(char_idx).is_whitespace() {
            char_idx += 1;
        }
        // Skip whitespace
        while char_idx < len && self.buffer.content.char(char_idx).is_whitespace() {
            char_idx += 1;
        }

        self.char_idx_to_cursor(char_idx);
    }

    fn move_to_prev_word(&mut self) {
        let mut char_idx = self.cursor_to_char_idx();
        if char_idx == 0 {
            return;
        }

        char_idx -= 1;
        // Skip current whitespace
        while char_idx > 0 && self.buffer.content.char(char_idx).is_whitespace() {
            char_idx -= 1;
        }
        // Skip non-whitespace
        while char_idx > 0 && !self.buffer.content.char(char_idx - 1).is_whitespace() {
            char_idx -= 1;
        }

        self.char_idx_to_cursor(char_idx);
    }

    fn cursor_to_char_idx(&self) -> usize {
        self.buffer.content.line_to_char(self.cursor.0) + self.cursor.1
    }

    fn char_idx_to_cursor(&mut self, char_idx: usize) {
        let row = self.buffer.content.char_to_line(char_idx);
        let col = char_idx - self.buffer.content.line_to_char(row);
        self.cursor = (row, col);
    }

    fn scroll(&mut self) {
        let (width, height) = self.terminal_size;
        let height = height.saturating_sub(1) as usize; // Reserve space for status bar

        if self.cursor.0 < self.scroll_offset.0 {
            self.scroll_offset.0 = self.cursor.0;
        } else if self.cursor.0 >= self.scroll_offset.0 + height {
            self.scroll_offset.0 = self.cursor.0 - height + 1;
        }

        if self.cursor.1 < self.scroll_offset.1 {
            self.scroll_offset.1 = self.cursor.1;
        } else if self.cursor.1 >= self.scroll_offset.1 + width as usize {
            self.scroll_offset.1 = self.cursor.1 - width as usize + 1;
        }
    }

    fn handle_insert_mode(&mut self, event: crossterm::event::KeyEvent) {
        use crossterm::event::KeyCode;
        match event.code {
            KeyCode::Esc => self.mode = Mode::Normal,
            KeyCode::Char(c) => {
                let idx = self.cursor_to_char_idx();
                self.buffer.insert_char(idx, c);
                if c == '\n' {
                    self.cursor.0 += 1;
                    self.cursor.1 = 0;
                } else {
                    self.cursor.1 += 1;
                }
            }
            KeyCode::Enter => {
                let idx = self.cursor_to_char_idx();
                self.buffer.insert_char(idx, '\n');
                self.cursor.0 += 1;
                self.cursor.1 = 0;
            }
            KeyCode::Backspace => {
                let idx = self.cursor_to_char_idx();
                if idx > 0 {
                    // Update cursor before delete if at start of line
                    let pos_before = self.char_idx_to_pos(idx - 1);
                    self.buffer.delete_char(idx - 1);
                    self.cursor = pos_before;
                }
            }
            _ => {}
        }
        self.scroll();
    }

    fn char_idx_to_pos(&self, char_idx: usize) -> (usize, usize) {
        let row = self.buffer.content.char_to_line(char_idx);
        let col = char_idx - self.buffer.content.line_to_char(row);
        (row, col)
    }
}
