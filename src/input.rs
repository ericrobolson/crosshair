use std::fmt;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Modifiers {
    pub super_key: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Key {
    Character(char),
    Enter,
    Escape,
    Backspace,
    Tab,
    Other,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KeyInput {
    pub key: Key,
    pub modifiers: Modifiers,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MouseInput {
    Click { column: u16, row: u16 },
    ScrollUp { column: u16, row: u16 },
    ScrollDown { column: u16, row: u16 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InputEvent {
    Key(KeyInput),
    Mouse(MouseInput),
    Resize { columns: u16, rows: u16 },
}

impl fmt::Display for KeyInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let modifiers = self.modifiers;
        if modifiers.super_key {
            f.write_str("Cmd+")?;
        }
        if modifiers.ctrl {
            f.write_str("Ctrl+")?;
        }
        if modifiers.alt {
            f.write_str("Alt+")?;
        }
        if modifiers.shift {
            f.write_str("Shift+")?;
        }
        match self.key {
            Key::Character(ch) => write!(f, "{}", ch.to_ascii_uppercase()),
            Key::Enter => f.write_str("Enter"),
            Key::Escape => f.write_str("Esc"),
            Key::Backspace => f.write_str("Backspace"),
            Key::Tab => f.write_str("Tab"),
            Key::Other => f.write_str("Other"),
        }
    }
}
