use std::time::Duration;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::input::{InputEvent, Key, MouseInput};

#[derive(Debug)]
pub struct App {
    pub running: bool,
    pub last_event: String,
    pub frame_time: Duration,
    pub scale_factor: f64,
    pub startup_time: Duration,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            last_event: "Waiting for input".into(),
            frame_time: Duration::ZERO,
            scale_factor: 1.0,
            startup_time: Duration::ZERO,
        }
    }
}

impl App {
    pub fn handle(&mut self, event: InputEvent) {
        match event {
            InputEvent::Key(key) => {
                self.last_event = format!("Key: {}", key);
                if key.key == Key::Escape
                    || (key.modifiers.ctrl && matches!(key.key, Key::Character('c' | 'C')))
                {
                    self.running = false;
                }
            }
            InputEvent::Mouse(mouse) => {
                self.last_event = match mouse {
                    MouseInput::Click { column, row } => {
                        format!("Mouse click at column {column}, row {row}")
                    }
                    MouseInput::ScrollUp { column, row } => {
                        format!("Mouse scroll up at column {column}, row {row}")
                    }
                    MouseInput::ScrollDown { column, row } => {
                        format!("Mouse scroll down at column {column}, row {row}")
                    }
                };
            }
            InputEvent::Resize { columns, rows } => {
                self.last_event = format!("Resized to {columns} × {rows} cells")
            }
        }
    }
}

pub fn render(frame: &mut Frame, app: &App) {
    const PANEL_BG: Color = Color::Rgb(20, 23, 31);
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(3)])
        .split(frame.area());

    let content = vec![
        Line::from(vec![
            Span::styled(
                "Crosshair",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  ·  GUI + Ratatui proof"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("Try "),
            Span::styled("Cmd+F", Style::default().fg(Color::Yellow)),
            Span::raw(", "),
            Span::styled("Cmd+S", Style::default().fg(Color::Yellow)),
            Span::raw(" and "),
            Span::styled("Cmd+P", Style::default().fg(Color::Yellow)),
            Span::raw(". Their normalized events appear below."),
        ]),
        Line::from(
            "The input layer records Super/Cmd separately from Ctrl for backend-neutral keymaps.",
        ),
        Line::from("Click or scroll in this panel; resize the window to check cell sizing."),
        Line::from(""),
        Line::from(vec![
            Span::styled("Monospace", Style::default().fg(Color::Green)),
            Span::raw(" · Cascadia Mono · 13 pt · scale factor "),
            Span::raw(format!("{:.2}", app.scale_factor)),
        ]),
        Line::from(format!(
            "GUI renderer ready in {:.0} ms",
            app.startup_time.as_secs_f64() * 1000.0
        )),
        Line::from(Span::styled(
            "Dim styles are supported",
            Style::default().fg(Color::Gray).add_modifier(Modifier::DIM),
        )),
    ];
    let panel = Block::default()
        .title(" Milestone 1 ")
        .title_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue))
        .style(Style::default().bg(PANEL_BG).fg(Color::White));
    frame.render_widget(
        Paragraph::new(content)
            .style(Style::default().fg(Color::White).bg(PANEL_BG))
            .wrap(Wrap { trim: true })
            .block(panel),
        layout[0],
    );

    let status = Line::from(vec![
        Span::styled(" ", Style::default()),
        Span::styled(&app.last_event, Style::default().fg(Color::White)),
        Span::styled(
            format!(
                "   ·   frame {:.2} ms",
                app.frame_time.as_secs_f64() * 1000.0
            ),
            Style::default().fg(Color::Gray),
        ),
    ]);
    frame.render_widget(
        Paragraph::new(status)
            .style(Style::default().fg(Color::Black).bg(Color::White))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray)),
            ),
        layout[1],
    );
}
