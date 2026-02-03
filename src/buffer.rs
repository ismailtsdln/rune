use ropey::Rope;

pub struct Buffer {
    pub content: Rope,
}

impl Buffer {
    pub fn new() -> Self {
        Self {
            content: Rope::new(),
        }
    }

    pub fn from_str(text: &str) -> Self {
        Self {
            content: Rope::from_str(text),
        }
    }

    pub fn insert_char(&mut self, char_idx: usize, c: char) {
        if char_idx <= self.content.len_chars() {
            self.content.insert_char(char_idx, c);
        }
    }

    pub fn delete_char(&mut self, char_idx: usize) {
        if char_idx < self.content.len_chars() {
            self.content.remove(char_idx..char_idx + 1);
        }
    }
}
