use crate::editor::Editor;
use crate::mode::Mode;
use crossterm::{
    cursor, execute, queue,
    style::{Color, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{self, ClearType},
};
use std::io::{Write, stdout};

pub struct Renderer;

impl Renderer {
    pub fn render(editor: &Editor) -> Result<(), Box<dyn std::error::Error>> {
        let mut stdout = stdout();
        let (width, height) = (editor.terminal_size.0, editor.terminal_size.1);

        queue!(
            stdout,
            terminal::Clear(ClearType::All),
            cursor::MoveTo(0, 0)
        )?;

        // Render buffer
        let start_line = editor.scroll_offset.0;
        let end_line =
            (start_line + height.saturating_sub(1) as usize).min(editor.buffer.content.len_lines());

        for (i, line_idx) in (start_line..end_line).enumerate() {
            let line = editor.buffer.content.line(line_idx);
            queue!(stdout, cursor::MoveTo(0, i as u16))?;

            let start_char = editor.scroll_offset.1;
            let line_len = line.len_chars();
            if start_char < line_len {
                let end_char = (start_char + width as usize).min(line_len);
                let visible_part = line.slice(start_char..end_char).to_string();
                Self::render_line_with_syntax(&visible_part, &mut stdout)?;
            }
        }

        // Render Status Bar
        queue!(
            stdout,
            cursor::MoveTo(0, height - 1),
            terminal::Clear(ClearType::CurrentLine)
        )?;

        if editor.mode == Mode::Command {
            execute!(stdout, SetForegroundColor(Color::Yellow))?;
            write!(stdout, "{}", editor.command_buffer)?;
            execute!(stdout, ResetColor)?;
        } else {
            let (bg, fg, mode_name) = match editor.mode {
                Mode::Normal => (Color::Blue, Color::White, " NORMAL "),
                Mode::Insert => (Color::Green, Color::Black, " INSERT "),
                _ => (Color::Magenta, Color::White, " VISUAL "),
            };

            execute!(stdout, SetBackgroundColor(bg), SetForegroundColor(fg))?;
            write!(stdout, "{}", mode_name)?;
            execute!(
                stdout,
                ResetColor,
                SetBackgroundColor(Color::Rgb {
                    r: 50,
                    g: 50,
                    b: 50
                }),
                SetForegroundColor(Color::White)
            )?;

            let file_name = editor.file_path.as_deref().unwrap_or("[No Name]");
            write!(
                stdout,
                " {} | L:{}, C:{} | {}",
                file_name,
                editor.cursor.0 + 1,
                editor.cursor.1 + 1,
                editor.status_message
            )?;
            execute!(stdout, ResetColor)?;
        }

        // Move cursor
        let cursor_row = (editor.cursor.0 as isize - editor.scroll_offset.0 as isize) as u16;
        let cursor_col = (editor.cursor.1 as isize - editor.scroll_offset.1 as isize) as u16;
        queue!(stdout, cursor::MoveTo(cursor_col, cursor_row))?;
        stdout.flush()?;

        Ok(())
    }

    fn render_line_with_syntax(
        line: &str,
        stdout: &mut std::io::Stdout,
    ) -> Result<(), std::io::Error> {
        let keywords = [
            "fn", "let", "pub", "use", "mod", "match", "if", "else", "impl", "struct", "enum",
            "type", "trait", "return",
        ];
        let mut last_pos = 0;

        for (word_pos, word) in line.match_indices(|c: char| !c.is_alphanumeric() && c != '_') {
            let prev_word = &line[last_pos..word_pos];
            if keywords.contains(&prev_word) {
                execute!(stdout, SetForegroundColor(Color::Cyan))?;
                write!(stdout, "{}", prev_word)?;
                execute!(stdout, ResetColor)?;
            } else {
                write!(stdout, "{}", prev_word)?;
            }
            write!(stdout, "{}", word)?;
            last_pos = word_pos + word.len();
        }
        let remaining = &line[last_pos..];
        if keywords.contains(&remaining) {
            execute!(stdout, SetForegroundColor(Color::Cyan))?;
            write!(stdout, "{}", remaining)?;
            execute!(stdout, ResetColor)?;
        } else {
            write!(stdout, "{}", remaining)?;
        }
        Ok(())
    }
}
