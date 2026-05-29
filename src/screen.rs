// Copyright (c) 2026 Immmanuel Jeyaraj <irj@sefier.com>. MIT License.

use crossterm::{cursor, queue, style, terminal};
use std::io::{Result, Write};

use crate::buffer::Buffer;
use crate::config::Config;

pub struct Screen {
    pub width: u16,
    pub height: u16,
    pub scroll: usize,
}

pub struct MenuRenderInfo<'a> {
    pub visible: bool,
    pub titles: &'a [(&'a str, char)],
    pub active_menu: Option<usize>,
    pub active_item: Option<usize>,
    pub dropdown: &'a [MenuDropItem<'a>],
}

pub struct MenuDropItem<'a> {
    pub label: &'a str,
    pub shortcut: &'a str,
    pub sep: bool,
    pub selected: bool,
}

impl Screen {
    pub fn new() -> Result<Self> {
        let (width, height) = terminal::size()?;
        Ok(Self {
            width,
            height,
            scroll: 0,
        })
    }

    pub fn resize(&mut self) -> Result<()> {
        let (width, height) = terminal::size()?;
        self.width = width;
        self.height = height;
        Ok(())
    }

    fn content_top(&self, menu_visible: bool) -> u16 {
        if menu_visible {
            1
        } else {
            0
        }
    }

    fn avail_rows(&self, menu_visible: bool) -> usize {
        let menu_lines = if menu_visible { 1 } else { 0 };
        (self.height.saturating_sub(2 + menu_lines)).max(1) as usize
    }

    pub fn render<W: Write>(
        &mut self,
        out: &mut W,
        b: &Buffer,
        config: &Config,
        menu: Option<&MenuRenderInfo>,
        mode: &str,
        cmdline: Option<&str>,
        status_msg: Option<&str>,
    ) -> Result<()> {
        let menu_visible = menu.map_or(false, |m| m.visible);
        let top = self.content_top(menu_visible);
        let avail = self.avail_rows(menu_visible);
        self.compute_scroll(b, avail);

        // ── Menu bar (row 0) ──
        if let Some(mi) = menu {
            if mi.visible {
                queue!(out, cursor::MoveTo(0, 0))?;
                queue!(out, terminal::Clear(terminal::ClearType::CurrentLine))?;
                queue!(out, style::SetBackgroundColor(style::Color::DarkGrey))?;
                queue!(out, style::SetForegroundColor(style::Color::White))?;
                let mut x: u16 = 0;
                for (i, (title, _alt)) in mi.titles.iter().enumerate() {
                    let label = format!(" {} ", title);
                    let label_w = label.len() as u16;
                    if x + label_w > self.width {
                        break;
                    }
                    if mi.active_menu == Some(i) {
                        queue!(out, style::SetBackgroundColor(style::Color::White))?;
                        queue!(out, style::SetForegroundColor(style::Color::Black))?;
                        queue!(out, style::Print(&label))?;
                        queue!(out, style::SetBackgroundColor(style::Color::DarkGrey))?;
                        queue!(out, style::SetForegroundColor(style::Color::White))?;
                    } else {
                        queue!(out, style::Print(&label))?;
                    }
                    queue!(out, style::Print(" "))?;
                    x += label_w + 1;
                }
                queue!(out, style::ResetColor)?;
            }
        }

        // ── Buffer content ──
        let num_width = if config.number || config.relativenumber {
            let digits = b.lines.len().to_string().len().max(2);
            digits + 1
        } else {
            0
        };

        for row in 0..avail as u16 {
            let display_row = top + row;
            queue!(out, cursor::MoveTo(0, display_row))?;
            queue!(out, terminal::Clear(terminal::ClearType::CurrentLine))?;

            let buf_row = self.scroll + row as usize;
            if buf_row >= b.lines.len() {
                queue!(out, style::Print("~"))?;
                continue;
            }

            let line = &b.lines[buf_row];
            let max_col = self.width.saturating_sub(num_width as u16 + 1) as usize;
            let display: &str = if line.len() > max_col {
                &line[..max_col]
            } else {
                line
            };

            if config.number || config.relativenumber {
                let num = if config.relativenumber && buf_row != b.cy {
                    buf_row.abs_diff(b.cy)
                } else {
                    buf_row + 1
                };
                queue!(out, style::SetForegroundColor(style::Color::DarkGrey))?;
                queue!(
                    out,
                    style::Print(format!("{:>width$} ", num, width = num_width - 1))
                )?;
                queue!(out, style::ResetColor)?;
            }

            if buf_row == b.cy {
                queue!(out, style::SetBackgroundColor(style::Color::DarkGrey))?;
                queue!(out, style::SetForegroundColor(style::Color::White))?;
            }
            queue!(out, style::Print(display))?;
            if buf_row == b.cy {
                queue!(out, style::ResetColor)?;
            }
        }

        // ── Cursor ──
        let cursor_row = top as usize + b.cy.saturating_sub(self.scroll);
        let cursor_col = b.cx + num_width;
        queue!(out, cursor::MoveTo(cursor_col as u16, cursor_row as u16))?;

        // ── Status bar ──
        if self.height >= 2 {
            let status_row = self.height.saturating_sub(2);
            queue!(out, cursor::MoveTo(0, status_row))?;
            queue!(out, terminal::Clear(terminal::ClearType::CurrentLine))?;

            let mod_flag = if b.modified { " [+]" } else { "" };
            let msg = status_msg.unwrap_or("");
            let status = format!(
                "{}  {}{}  line {}/{} col {}  {}",
                mode,
                b.name_display(),
                mod_flag,
                b.lineno(),
                b.total_lines(),
                b.cx + 1,
                msg
            );
            queue!(out, style::SetBackgroundColor(style::Color::DarkGrey))?;
            queue!(out, style::SetForegroundColor(style::Color::White))?;
            let end = status.len().min(self.width as usize);
            queue!(out, style::Print(&status[..end]))?;
            queue!(out, style::ResetColor)?;
        }

        // ── Command line ──
        if let Some(cmd) = cmdline {
            let cmd_row = self.height.saturating_sub(1);
            queue!(out, cursor::MoveTo(0, cmd_row))?;
            queue!(out, terminal::Clear(terminal::ClearType::CurrentLine))?;
            queue!(out, style::Print(cmd))?;
            queue!(out, cursor::MoveTo(cmd.len() as u16, cmd_row))?;
        }

        // ── Dropdown (overlays buffer) ──
        if let Some(mi) = menu {
            if mi.visible && mi.active_menu.is_some() && !mi.dropdown.is_empty() {
                let dd_top = 1u16;
                let count = mi.dropdown.len() as u16;
                if dd_top + count < self.height.saturating_sub(2) {
                    for (ii, item) in mi.dropdown.iter().enumerate() {
                        let row = dd_top + ii as u16;
                        queue!(out, cursor::MoveTo(0, row))?;
                        queue!(out, terminal::Clear(terminal::ClearType::CurrentLine))?;
                        if item.sep {
                            queue!(out, style::SetForegroundColor(style::Color::DarkGrey))?;
                            queue!(out, style::Print(" "))?;
                            let sep = "─".repeat((self.width.saturating_sub(2)) as usize);
                            queue!(out, style::Print(&sep))?;
                            queue!(out, style::ResetColor)?;
                        } else {
                            if item.selected {
                                queue!(out, style::SetBackgroundColor(style::Color::Blue))?;
                                queue!(out, style::SetForegroundColor(style::Color::White))?;
                            }
                            queue!(out, style::Print(" "))?;
                            let col_w = self.width.saturating_sub(2) as usize;
                            let label = &item.label[..item
                                .label
                                .len()
                                .min(col_w.saturating_sub(item.shortcut.len() + 2))];
                            queue!(out, style::Print(label))?;
                            if !item.shortcut.is_empty() {
                                let pad = col_w.saturating_sub(label.len() + item.shortcut.len());
                                for _ in 0..pad {
                                    queue!(out, style::Print(" "))?;
                                }
                                queue!(out, style::SetForegroundColor(style::Color::Grey))?;
                                queue!(out, style::Print(item.shortcut))?;
                            }
                            if item.selected {
                                queue!(out, style::ResetColor)?;
                            }
                        }
                    }
                }
            }
        }

        out.flush()?;
        Ok(())
    }

    fn compute_scroll(&mut self, b: &Buffer, avail: usize) {
        if avail == 0 {
            return;
        }
        if b.cy < self.scroll {
            self.scroll = b.cy;
        } else if b.cy >= self.scroll + avail {
            self.scroll = b.cy.saturating_sub(avail).wrapping_add(1);
        }
        if self.scroll + avail > b.lines.len() && b.lines.len() > avail {
            self.scroll = b.lines.len().saturating_sub(avail);
        }
    }
}
