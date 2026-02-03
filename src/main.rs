mod buffer;
mod config;
mod editor;
mod mode;
mod renderer;
mod scripting;

use crate::config::Config;
use crate::editor::Editor;
use crate::renderer::Renderer;
use crate::scripting::ScriptEngine;
use crossterm::{
    cursor,
    event::{self, Event},
    execute, terminal,
};
use std::io::stdout;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _config = Config::load();
    let script_engine = ScriptEngine::new();
    script_engine.init()?;

    // Setup terminal
    terminal::enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, terminal::EnterAlternateScreen, cursor::Show)?;

    let mut editor = Editor::new();

    // Handle CLI arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        editor.open_file(&args[1]);
    }

    let (width, height) = terminal::size()?;
    editor.terminal_size = (width, height);

    loop {
        Renderer::render(&editor)?;

        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key_event) => {
                    editor.handle_key_event(key_event);
                }
                Event::Resize(w, h) => {
                    editor.terminal_size = (w, h);
                }
                _ => {}
            }
        }

        if editor.should_quit {
            break;
        }
    }

    // Restore terminal
    execute!(stdout, terminal::LeaveAlternateScreen, cursor::Show)?;
    terminal::disable_raw_mode()?;

    Ok(())
}
