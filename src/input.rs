// Copyright (c) 2026 Immmanuel Jeyaraj <irj@sefier.com>. MIT License.

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, PartialEq)]
pub enum Key {
    Char(char),
    Ctrl(char),
    Alt(char),
    Enter,
    Tab,
    Backspace,
    Escape,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    Delete,
    Insert,
    F(u8),
    CtrlF(u8),
    Null,
}

pub fn read_key() -> Key {
    loop {
        match event::read() {
            Ok(Event::Key(ke)) => return map_key(ke),
            Ok(Event::Resize(_, _)) => continue,
            _ => continue,
        }
    }
}

fn map_key(ke: KeyEvent) -> Key {
    match ke.code {
        KeyCode::Char(c) => {
            if ke.modifiers == KeyModifiers::CONTROL {
                Key::Ctrl(c)
            } else if ke.modifiers == KeyModifiers::ALT {
                Key::Alt(c)
            } else {
                Key::Char(c)
            }
        }
        KeyCode::Enter => Key::Enter,
        KeyCode::Tab => Key::Tab,
        KeyCode::Backspace => Key::Backspace,
        KeyCode::Esc => Key::Escape,
        KeyCode::Left => Key::Left,
        KeyCode::Right => Key::Right,
        KeyCode::Up => Key::Up,
        KeyCode::Down => Key::Down,
        KeyCode::Home => Key::Home,
        KeyCode::End => Key::End,
        KeyCode::PageUp => Key::PageUp,
        KeyCode::PageDown => Key::PageDown,
        KeyCode::Delete => Key::Delete,
        KeyCode::Insert => Key::Insert,
        KeyCode::F(n) => {
            if ke.modifiers.contains(KeyModifiers::CONTROL) {
                Key::CtrlF(n)
            } else {
                Key::F(n)
            }
        }
        _ => Key::Null,
    }
}
