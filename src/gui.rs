use std::{error::Error, sync::Arc, time::Instant};

use eframe::egui::{
    self, Align2, Color32, Event, FontFamily, FontId, Key as EguiKey, PointerButton,
};
use ratatui::{
    Terminal,
    backend::TestBackend,
    buffer::{Buffer, Cell},
    style::{Color, Modifier},
};

use crate::{
    app::{self, App},
    input::{InputEvent, Key, KeyInput, Modifiers, MouseInput},
};

type GuiTerminal = Terminal<TestBackend>;
const FONT_SIZE: f32 = 13.0;
const DEFAULT_FG: Color32 = Color32::WHITE;
const DEFAULT_BG: Color32 = Color32::from_rgb(20, 23, 31);

pub fn run() -> Result<(), Box<dyn Error>> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Crosshair — Milestone 1")
            .with_inner_size([800.0, 520.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Crosshair — Milestone 1",
        options,
        Box::new(|cc| Ok(Box::new(GuiApp::new(cc)) as Box<dyn eframe::App>)),
    )?;
    Ok(())
}

struct GuiApp {
    terminal: GuiTerminal,
    state: App,
    paint_items: Vec<PaintItem>,
    observed_size: Option<(u16, u16)>,
    last_resize_change: Instant,
    has_rendered_layout: bool,
    needs_render: bool,
}

impl GuiApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let startup = Instant::now();
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "cascadia_mono".to_owned(),
            Arc::new(egui::FontData::from_static(include_bytes!(
                "../assets/CascadiaMono-Regular.ttf"
            ))),
        );
        fonts
            .families
            .entry(FontFamily::Monospace)
            .or_default()
            .insert(0, "cascadia_mono".to_owned());
        cc.egui_ctx.set_fonts(fonts);

        let mut state = App::default();
        state.scale_factor = f64::from(cc.egui_ctx.pixels_per_point());
        state.startup_time = startup.elapsed();
        Self {
            terminal: Terminal::new(TestBackend::new(120, 40)).expect("create test backend"),
            state,
            paint_items: Vec::new(),
            observed_size: None,
            last_resize_change: Instant::now(),
            has_rendered_layout: false,
            needs_render: true,
        }
    }

    fn handle_keys(&mut self, ctx: &egui::Context) {
        let events = ctx.input(|input| input.events.clone());
        for event in events {
            let Event::Key {
                key,
                pressed: true,
                modifiers,
                ..
            } = event
            else {
                continue;
            };
            self.state.handle(InputEvent::Key(KeyInput {
                key: map_key(key),
                modifiers: Modifiers {
                    super_key: modifiers.mac_cmd,
                    ctrl: modifiers.ctrl,
                    alt: modifiers.alt,
                    shift: modifiers.shift,
                },
            }));
            self.needs_render = true;
        }
    }
}

impl eframe::App for GuiApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.state.scale_factor = f64::from(ctx.pixels_per_point());
        self.handle_keys(&ctx);
        if !self.state.running {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        let click = ctx.input(|input| {
            input
                .pointer
                .button_pressed(PointerButton::Primary)
                .then(|| input.pointer.interact_pos())
                .flatten()
        });
        let scroll = ctx.input(|input| input.smooth_scroll_delta.y);
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(DEFAULT_BG))
            .show_inside(ui, |ui| {
                let font_id = FontId::new(FONT_SIZE, FontFamily::Monospace);
                let (cell_width, cell_height) = ctx.fonts_mut(|fonts| {
                    (
                        fonts.glyph_width(&font_id, 'M').max(1.0),
                        fonts.row_height(&font_id).max(1.0),
                    )
                });
                let available = ui.available_size();
                let width = (available.x / cell_width).floor().max(1.0) as u16;
                let height = (available.y / cell_height).floor().max(1.0) as u16;
                let target_size = (width, height);
                if self.observed_size != Some(target_size) {
                    self.observed_size = Some(target_size);
                    self.last_resize_change = Instant::now();
                }

                let old_size = self.terminal.size().ok();
                let size_mismatch =
                    old_size.is_some_and(|size| size.width != width || size.height != height);
                if size_mismatch {
                    const SETTLE_TIME: std::time::Duration = std::time::Duration::from_millis(24);
                    let elapsed = self.last_resize_change.elapsed();
                    if !self.has_rendered_layout || elapsed >= SETTLE_TIME {
                        self.terminal.backend_mut().resize(width, height);
                        self.state.handle(InputEvent::Resize {
                            columns: width,
                            rows: height,
                        });
                        self.has_rendered_layout = true;
                        self.needs_render = true;
                    } else {
                        ctx.request_repaint_after(SETTLE_TIME - elapsed);
                    }
                }

                let displayed_size = self.terminal.size().ok();
                let size_matches =
                    displayed_size.is_some_and(|size| size.width == width && size.height == height);
                if self.needs_render && size_matches {
                    let started = Instant::now();
                    if let Err(error) = self.terminal.draw(|frame| app::render(frame, &self.state))
                    {
                        self.state.last_event = format!("Draw error: {error}");
                    } else {
                        self.paint_items = build_paint_items(self.terminal.backend().buffer());
                    }
                    self.state.frame_time = started.elapsed();
                    self.needs_render = false;
                    self.has_rendered_layout = true;
                }

                let (response, painter) = ui.allocate_painter(available, egui::Sense::click());
                paint_items(
                    &painter,
                    &self.paint_items,
                    response.rect,
                    cell_width,
                    cell_height,
                    &font_id,
                );

                let cell_width = cell_width.max(1.0);
                let cell_height = cell_height.max(1.0);
                if let Some(position) = click {
                    if response.rect.contains(position) {
                        self.state.handle(InputEvent::Mouse(MouseInput::Click {
                            column: ((position.x - response.rect.left()) / cell_width) as u16,
                            row: ((position.y - response.rect.top()) / cell_height) as u16,
                        }));
                        self.needs_render = true;
                        ctx.request_repaint();
                    }
                }
                if scroll.abs() > 0.0 {
                    if let Some(position) = ctx.input(|input| input.pointer.hover_pos()) {
                        if response.rect.contains(position) {
                            let column = ((position.x - response.rect.left()) / cell_width) as u16;
                            let row = ((position.y - response.rect.top()) / cell_height) as u16;
                            self.state.handle(InputEvent::Mouse(if scroll > 0.0 {
                                MouseInput::ScrollUp { column, row }
                            } else {
                                MouseInput::ScrollDown { column, row }
                            }));
                            self.needs_render = true;
                            ctx.request_repaint();
                        }
                    }
                }
            });
    }
}

#[derive(Clone)]
enum PaintItem {
    Background {
        x: u16,
        y: u16,
        width: u16,
        color: Color32,
    },
    Text {
        x: u16,
        y: u16,
        cells: u16,
        text: String,
        color: Color32,
        modifier: Modifier,
    },
}

fn build_paint_items(buffer: &Buffer) -> Vec<PaintItem> {
    let mut items = Vec::new();
    let width = buffer.area.width;
    let height = buffer.area.height;

    for y in 0..height {
        let mut x = 0;
        while x < width {
            let bg = background_color(&buffer[(x, y)]);
            let start = x;
            x += 1;
            while x < width && background_color(&buffer[(x, y)]) == bg {
                x += 1;
            }
            if bg != DEFAULT_BG {
                items.push(PaintItem::Background {
                    x: start,
                    y,
                    width: x - start,
                    color: bg,
                });
            }
        }

        let mut x = 0;
        while x < width {
            let cell = &buffer[(x, y)];
            let color = foreground_color(cell);
            let modifier = cell.modifier;
            let start = x;
            let mut text = String::new();
            while x < width {
                let next = &buffer[(x, y)];
                if foreground_color(next) != color || next.modifier != modifier {
                    break;
                }
                text.push_str(next.symbol());
                x += 1;
            }
            if text.chars().any(|character| !character.is_whitespace()) {
                items.push(PaintItem::Text {
                    x: start,
                    y,
                    cells: x - start,
                    text,
                    color,
                    modifier,
                });
            }
        }
    }
    items
}

fn paint_items(
    painter: &egui::Painter,
    items: &[PaintItem],
    area: egui::Rect,
    cell_width: f32,
    cell_height: f32,
    font_id: &FontId,
) {
    for item in items {
        match item {
            PaintItem::Background { x, y, width, color } => {
                painter.rect_filled(
                    egui::Rect::from_min_size(
                        egui::pos2(
                            area.left() + f32::from(*x) * cell_width,
                            area.top() + f32::from(*y) * cell_height,
                        ),
                        egui::vec2(f32::from(*width) * cell_width, cell_height),
                    ),
                    0.0,
                    *color,
                );
            }
            PaintItem::Text {
                x,
                y,
                cells,
                text,
                color,
                modifier,
            } => {
                let position = egui::pos2(
                    area.left() + f32::from(*x) * cell_width,
                    area.top() + f32::from(*y) * cell_height,
                );
                painter.text(position, Align2::LEFT_TOP, text, font_id.clone(), *color);
                if modifier.contains(Modifier::BOLD) {
                    painter.text(
                        position + egui::vec2(0.35, 0.0),
                        Align2::LEFT_TOP,
                        text,
                        font_id.clone(),
                        *color,
                    );
                }
                let decoration = if modifier.contains(Modifier::UNDERLINED) {
                    Some(position.y + cell_height - 1.0)
                } else if modifier.contains(Modifier::CROSSED_OUT) {
                    Some(position.y + cell_height * 0.55)
                } else {
                    None
                };
                if let Some(y) = decoration {
                    painter.line_segment(
                        [
                            egui::pos2(position.x, y),
                            egui::pos2(position.x + f32::from(*cells) * cell_width, y),
                        ],
                        egui::Stroke::new(1.0, *color),
                    );
                }
            }
        }
    }
}

fn foreground_color(cell: &Cell) -> Color32 {
    let (fg, bg) = if cell.modifier.contains(Modifier::REVERSED) {
        (
            ansi_color(cell.bg, DEFAULT_BG),
            ansi_color(cell.fg, DEFAULT_FG),
        )
    } else {
        (ansi_color(cell.fg, DEFAULT_FG), background_color(cell))
    };
    if cell.modifier.contains(Modifier::HIDDEN) {
        return bg;
    }
    if cell.modifier.contains(Modifier::DIM) {
        return Color32::from_rgb(fg.r() / 2, fg.g() / 2, fg.b() / 2);
    }
    fg
}

fn background_color(cell: &Cell) -> Color32 {
    if cell.modifier.contains(Modifier::REVERSED) {
        ansi_color(cell.fg, DEFAULT_FG)
    } else {
        ansi_color(cell.bg, DEFAULT_BG)
    }
}

fn ansi_color(color: Color, default: Color32) -> Color32 {
    match color {
        Color::Reset => default,
        Color::Black => Color32::BLACK,
        Color::Red => Color32::from_rgb(128, 0, 0),
        Color::Green => Color32::from_rgb(0, 200, 0),
        Color::Yellow => Color32::from_rgb(255, 220, 0),
        Color::Blue => Color32::from_rgb(0, 80, 255),
        Color::Magenta => Color32::from_rgb(180, 0, 180),
        Color::Cyan => Color32::from_rgb(0, 235, 255),
        Color::Gray => Color32::from_rgb(192, 192, 192),
        Color::DarkGray => Color32::from_rgb(128, 128, 128),
        Color::LightRed => Color32::from_rgb(255, 0, 0),
        Color::LightGreen => Color32::from_rgb(0, 255, 0),
        Color::LightYellow => Color32::from_rgb(255, 255, 0),
        Color::LightBlue => Color32::from_rgb(0, 0, 255),
        Color::LightMagenta => Color32::from_rgb(255, 0, 255),
        Color::LightCyan => Color32::from_rgb(0, 255, 255),
        Color::White => Color32::WHITE,
        Color::Rgb(r, g, b) => Color32::from_rgb(r, g, b),
        Color::Indexed(index) => indexed_color(index),
    }
}

fn indexed_color(index: u8) -> Color32 {
    const BASIC: [[u8; 3]; 16] = [
        [0, 0, 0],
        [128, 0, 0],
        [0, 128, 0],
        [128, 128, 0],
        [0, 0, 128],
        [128, 0, 128],
        [0, 128, 128],
        [192, 192, 192],
        [128, 128, 128],
        [255, 0, 0],
        [0, 255, 0],
        [255, 255, 0],
        [0, 0, 255],
        [255, 0, 255],
        [0, 255, 255],
        [255, 255, 255],
    ];
    let rgb = match index {
        0..=15 => BASIC[usize::from(index)],
        16..=231 => {
            let n = index - 16;
            let component = |value: u8| if value == 0 { 0 } else { value * 40 + 55 };
            [component(n / 36), component((n / 6) % 6), component(n % 6)]
        }
        _ => {
            let gray = 8 + (index - 232) * 10;
            [gray, gray, gray]
        }
    };
    Color32::from_rgb(rgb[0], rgb[1], rgb[2])
}

fn map_key(key: EguiKey) -> Key {
    match key {
        EguiKey::Enter => Key::Enter,
        EguiKey::Escape => Key::Escape,
        EguiKey::Backspace => Key::Backspace,
        EguiKey::Tab => Key::Tab,
        EguiKey::A => Key::Character('a'),
        EguiKey::B => Key::Character('b'),
        EguiKey::C => Key::Character('c'),
        EguiKey::D => Key::Character('d'),
        EguiKey::E => Key::Character('e'),
        EguiKey::F => Key::Character('f'),
        EguiKey::G => Key::Character('g'),
        EguiKey::H => Key::Character('h'),
        EguiKey::I => Key::Character('i'),
        EguiKey::J => Key::Character('j'),
        EguiKey::K => Key::Character('k'),
        EguiKey::L => Key::Character('l'),
        EguiKey::M => Key::Character('m'),
        EguiKey::N => Key::Character('n'),
        EguiKey::O => Key::Character('o'),
        EguiKey::P => Key::Character('p'),
        EguiKey::Q => Key::Character('q'),
        EguiKey::R => Key::Character('r'),
        EguiKey::S => Key::Character('s'),
        EguiKey::T => Key::Character('t'),
        EguiKey::U => Key::Character('u'),
        EguiKey::V => Key::Character('v'),
        EguiKey::W => Key::Character('w'),
        EguiKey::X => Key::Character('x'),
        EguiKey::Y => Key::Character('y'),
        EguiKey::Z => Key::Character('z'),
        _ => Key::Other,
    }
}
