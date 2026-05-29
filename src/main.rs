/*************************************************************************
    rvim — A minimal Vim clone in Rust

    Copyright (C) 2026 Immanuel Jeyaraj <irj@sefier.com>

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*************************************************************************/

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
