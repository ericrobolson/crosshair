use std::{error::Error, io, time::Instant};

use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode as CKey, KeyEventKind,
        KeyModifiers as CModifiers, MouseEventKind,
    },
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

use crate::{
    app::{self, App},
    input::{InputEvent, Key, KeyInput, Modifiers, MouseInput},
};

struct TerminalGuard;
impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
    }
}

pub fn run() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
    let _guard = TerminalGuard;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    let mut state = App::default();
    loop {
        let start = Instant::now();
        terminal.draw(|frame| app::render(frame, &state))?;
        state.frame_time = start.elapsed();
        if !state.running {
            break;
        }
        let event = event::read()?;
        match event {
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                let normalized = match key.code {
                    CKey::Char(ch) => Key::Character(ch),
                    CKey::Enter => Key::Enter,
                    CKey::Esc => Key::Escape,
                    CKey::Backspace => Key::Backspace,
                    CKey::Tab => Key::Tab,
                    _ => Key::Other,
                };
                state.handle(InputEvent::Key(KeyInput {
                    key: normalized,
                    modifiers: Modifiers {
                        super_key: key.modifiers.contains(CModifiers::SUPER),
                        ctrl: key.modifiers.contains(CModifiers::CONTROL),
                        alt: key.modifiers.contains(CModifiers::ALT),
                        shift: key.modifiers.contains(CModifiers::SHIFT),
                    },
                }));
            }
            Event::Mouse(mouse) => {
                let kind = match mouse.kind {
                    MouseEventKind::Down(_) => Some(MouseInput::Click {
                        column: mouse.column,
                        row: mouse.row,
                    }),
                    MouseEventKind::ScrollUp => Some(MouseInput::ScrollUp {
                        column: mouse.column,
                        row: mouse.row,
                    }),
                    MouseEventKind::ScrollDown => Some(MouseInput::ScrollDown {
                        column: mouse.column,
                        row: mouse.row,
                    }),
                    _ => None,
                };
                if let Some(kind) = kind {
                    state.handle(InputEvent::Mouse(kind));
                }
            }
            Event::Resize(columns, rows) => state.handle(InputEvent::Resize { columns, rows }),
            _ => {}
        }
    }
    Ok(())
}
