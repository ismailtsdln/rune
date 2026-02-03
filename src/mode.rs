#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Insert,
    #[allow(dead_code)]
    Visual,
    Command,
}

impl Default for Mode {
    fn default() -> Self {
        Mode::Normal
    }
}
