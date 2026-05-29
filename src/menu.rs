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

use crate::input::Key;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MenuAction {
    NewFile,
    OpenFile,
    Save,
    SaveAs,
    CloseFile,
    Quit,
    Undo,
    Redo,
    Copy,
    Paste,
    DeleteLine,
    ToggleLineNumbers,
    ToggleRelativeNumbers,
    ToggleMenuBar,
    Find,
    FindNext,
    FindPrev,
    Help,
    About,
}

pub(crate) struct MenuEntry {
    label: &'static str,
    shortcut: &'static str,
    action: Option<MenuAction>,
    sep: bool,
}

struct Menu {
    title: &'static str,
    alt: char,
    items: Vec<MenuEntry>,
}

pub struct MenuBar {
    pub visible: bool,
    pub active_idx: Option<(usize, usize)>,
    menus: Vec<Menu>,
}

impl MenuBar {
    pub fn new(visible: bool) -> Self {
        Self {
            visible,
            active_idx: None,
            menus: Self::build_menus(),
        }
    }

    fn build_menus() -> Vec<Menu> {
        vec![
            Menu {
                title: "File",
                alt: 'f',
                items: vec![
                    entry("New", "Ctrl+N", MenuAction::NewFile),
                    entry("Open...", "Ctrl+O", MenuAction::OpenFile),
                    entry("Save", "Ctrl+S", MenuAction::Save),
                    entry("Save As...", "", MenuAction::SaveAs),
                    sep(),
                    entry("Close", "", MenuAction::CloseFile),
                    entry("Quit", "Ctrl+Q", MenuAction::Quit),
                ],
            },
            Menu {
                title: "Edit",
                alt: 'e',
                items: vec![
                    entry("Undo", "Ctrl+Z", MenuAction::Undo),
                    entry("Redo", "Ctrl+Y", MenuAction::Redo),
                    sep(),
                    entry("Copy", "", MenuAction::Copy),
                    entry("Paste", "", MenuAction::Paste),
                    sep(),
                    entry("Delete Line", "dd", MenuAction::DeleteLine),
                ],
            },
            Menu {
                title: "View",
                alt: 'v',
                items: vec![
                    entry("Line Numbers", "", MenuAction::ToggleLineNumbers),
                    entry("Relative Numbers", "", MenuAction::ToggleRelativeNumbers),
                    sep(),
                    entry("Menu Bar", "", MenuAction::ToggleMenuBar),
                ],
            },
            Menu {
                title: "Search",
                alt: 's',
                items: vec![
                    entry("Find...", "/", MenuAction::Find),
                    entry("Find Next", "n", MenuAction::FindNext),
                    entry("Find Prev", "N", MenuAction::FindPrev),
                ],
            },
            Menu {
                title: "Tools",
                alt: 't',
                items: vec![
                    entry("Help", "F1", MenuAction::Help),
                    sep(),
                    entry("About rvim", "", MenuAction::About),
                ],
            },
        ]
    }

    pub fn open(&mut self, alt: char) {
        for (i, m) in self.menus.iter().enumerate() {
            if m.alt == alt {
                self.active_idx = Some((i, self.first_selectable(i)));
                return;
            }
        }
    }

    pub fn is_open(&self) -> bool {
        self.active_idx.is_some()
    }

    pub fn close(&mut self) {
        self.active_idx = None;
    }

    fn first_selectable(&self, mi: usize) -> usize {
        if let Some(m) = self.menus.get(mi) {
            for (i, item) in m.items.iter().enumerate() {
                if !item.sep {
                    return i;
                }
            }
        }
        0
    }

    fn prev_selectable(&self, mi: usize, ii: usize) -> usize {
        if let Some(m) = self.menus.get(mi) {
            for i in (0..ii).rev() {
                if let Some(item) = m.items.get(i) {
                    if !item.sep {
                        return i;
                    }
                }
            }
            // wrap to bottom
            for i in (0..m.items.len()).rev() {
                if !m.items[i].sep {
                    return i;
                }
            }
        }
        ii
    }

    fn next_selectable(&self, mi: usize, ii: usize) -> usize {
        if let Some(m) = self.menus.get(mi) {
            for i in ii + 1..m.items.len() {
                if !m.items[i].sep {
                    return i;
                }
            }
            // wrap to top
            for i in 0..m.items.len() {
                if !m.items[i].sep {
                    return i;
                }
            }
        }
        ii
    }

    pub fn handle_key(&mut self, key: &Key) -> Option<MenuAction> {
        let (mi, ii) = self.active_idx?;
        match key {
            Key::Left => {
                let new_mi = if mi > 0 { mi - 1 } else { self.menus.len() - 1 };
                self.active_idx = Some((new_mi, self.first_selectable(new_mi)));
            }
            Key::Right => {
                let new_mi = (mi + 1) % self.menus.len();
                self.active_idx = Some((new_mi, self.first_selectable(new_mi)));
            }
            Key::Up => {
                let new_ii = self.prev_selectable(mi, ii);
                self.active_idx = Some((mi, new_ii));
            }
            Key::Down => {
                let new_ii = self.next_selectable(mi, ii);
                self.active_idx = Some((mi, new_ii));
            }
            Key::Enter => {
                self.active_idx = None;
                return self
                    .menus
                    .get(mi)
                    .and_then(|m| m.items.get(ii).and_then(|item| item.action));
            }
            Key::Escape => {
                self.active_idx = None;
            }
            Key::Alt(c) => {
                self.open(*c);
            }
            _ => {}
        }
        None
    }

    pub fn title_at(&self, idx: usize) -> Option<&str> {
        self.menus.get(idx).map(|m| m.title)
    }

    pub fn alt_at(&self, idx: usize) -> Option<char> {
        self.menus.get(idx).map(|m| m.alt)
    }

    pub fn menu_count(&self) -> usize {
        self.menus.len()
    }

    pub fn active_menu(&self) -> Option<(usize, &[MenuEntry])> {
        let (mi, _) = self.active_idx?;
        self.menus.get(mi).map(|m| (mi, m.items.as_slice()))
    }

    pub fn active_item(&self) -> Option<(usize, usize)> {
        self.active_idx
    }

    pub fn items_for(&self, idx: usize) -> Option<&[MenuEntry]> {
        self.menus.get(idx).map(|m| m.items.as_slice())
    }

    pub fn item_count(&self, idx: usize) -> usize {
        self.menus.get(idx).map(|m| m.items.len()).unwrap_or(0)
    }

    pub fn item_label(&self, idx: usize, item_idx: usize) -> Option<(&str, &str, bool, bool)> {
        self.menus.get(idx).and_then(|m| {
            m.items
                .get(item_idx)
                .map(|e| (e.label, e.shortcut, e.sep, e.action.is_some()))
        })
    }
}

fn entry(label: &'static str, shortcut: &'static str, action: MenuAction) -> MenuEntry {
    MenuEntry {
        label,
        shortcut,
        action: Some(action),
        sep: false,
    }
}

fn sep() -> MenuEntry {
    MenuEntry {
        label: "",
        shortcut: "",
        action: None,
        sep: true,
    }
}
