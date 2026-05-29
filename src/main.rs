// Copyright (c) 2026 Immmanuel Jeyaraj <irj@sefier.com>. MIT License.

#![allow(dead_code)]

mod buffer;
mod command;
mod config;
mod dialog;
mod editor;
mod history;
mod input;
mod menu;
mod motion;
mod screen;
mod search;

use crossterm::{cursor, execute, terminal};
use std::io::{self, Result};

fn main() -> Result<()> {
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        cursor::Hide,
        terminal::Clear(terminal::ClearType::All)
    )?;

    let mut ed = editor::Editor::new()?;
    let result = ed.run(&mut stdout);

    execute!(
        stdout,
        cursor::Show,
        cursor::MoveTo(0, 0),
        terminal::Clear(terminal::ClearType::All),
    )?;
    terminal::disable_raw_mode()?;

    if let Err(e) = result {
        eprintln!("rvim error: {}", e);
    }
    Ok(())
}
