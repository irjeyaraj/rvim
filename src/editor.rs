// Copyright (c) 2026 Immmanuel Jeyaraj <irj@sefier.com>. MIT License.

use std::io::{Result, Write};

use crate::buffer::Buffer;
use crate::command;
use crate::config::Config;
use crate::dialog::{self, DialogResult};
use crate::history::{Action, History};
use crate::input::Key;
use crate::menu::{MenuAction, MenuBar};
use crate::motion;
use crate::screen::{MenuDropItem, MenuRenderInfo, Screen};
use crate::search::{self, Direction};

#[derive(Debug, Clone, Copy, PartialEq)]
enum OpType {
    Delete,
    Yank,
    Change,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum MotionType {
    Left,
    Right,
    Up,
    Down,
    WordForward,
    WordBack,
    EndOfWord,
    LineStart,
    LineEnd,
    FirstNonBlank,
    FileStart,
    FileEnd,
    Percent,
}

#[derive(Debug, Clone, PartialEq)]
enum Mode {
    Normal,
    Insert,
    Visual { start: (usize, usize) },
    Command { buf: String },
    Search { buf: String, dir: Direction },
}

pub struct Editor {
    buffers: Vec<Buffer>,
    current_buf: usize,
    mode: Mode,
    config: Config,
    history: History,
    screen: Screen,
    menu_bar: MenuBar,
    clip: String,
    status: Option<String>,
    quit: bool,
    saved_cx: usize,
    count_str: String,
    pending_op: Option<OpType>,
    last_pattern: String,
}

impl Editor {
    pub fn new() -> Result<Self> {
        let args: Vec<String> = std::env::args().skip(1).collect();
        let mut buffers = Vec::new();
        if args.is_empty() {
            buffers.push(Buffer::new());
        } else {
            for path in &args {
                match Buffer::from_file(path) {
                    Ok(b) => buffers.push(b),
                    Err(_) => {
                        let mut b = Buffer::new();
                        b.filename = Some(path.clone());
                        buffers.push(b);
                    }
                }
            }
        }
        let config = Config::load();
        let menu_bar = MenuBar::new(config.menu);
        Ok(Self {
            buffers,
            current_buf: 0,
            mode: Mode::Normal,
            config,
            history: History::new(),
            screen: Screen::new()?,
            menu_bar,
            clip: String::new(),
            status: None,
            quit: false,
            saved_cx: 0,
            count_str: String::new(),
            pending_op: None,
            last_pattern: String::new(),
        })
    }

    pub fn run<W: Write>(&mut self, out: &mut W) -> Result<()> {
        while !self.quit {
            self.screen.resize()?;

            self.render(out)?;
            let key = crate::input::read_key();
            self.status = None;

            // ── Global shortcuts (work regardless of mode) ──
            let mut consumed = false;
            match &key {
                Key::F(1) => {
                    self.handle_menu_action(MenuAction::Help);
                    consumed = true;
                }
                Key::F(2) => {
                    self.menu_bar.visible = !self.menu_bar.visible;
                    consumed = true;
                }
                Key::F(3) => {
                    self.config.number = !self.config.number;
                    consumed = true;
                }
                Key::F(4) => {
                    self.config.relativenumber = !self.config.relativenumber;
                    consumed = true;
                }
                Key::CtrlF(10) => {
                    self.menu_bar.visible = true;
                    self.menu_bar.open('f');
                    consumed = true;
                }
                Key::Alt(c) if !self.menu_bar.is_open() => {
                    if self.menu_bar.visible {
                        self.menu_bar.open(*c);
                        consumed = true;
                    }
                }
                _ => {}
            }
            if consumed {
                continue;
            }

            // ── Menu navigation ──
            if self.menu_bar.is_open() {
                if let Some(action) = self.menu_bar.handle_key(&key) {
                    self.handle_menu_action(action);
                }
                continue;
            }

            // ── Mode dispatch ──
            match self.mode.clone() {
                Mode::Normal => self.handle_normal(key),
                Mode::Insert => self.handle_insert(key),
                Mode::Visual { start } => self.handle_visual(key, start),
                Mode::Command { buf } => self.handle_command(key, &buf),
                Mode::Search { buf, dir } => self.handle_search(key, &buf, dir),
            }
        }
        Ok(())
    }

    // ── Rendering ──

    fn render<W: Write>(&mut self, out: &mut W) -> Result<()> {
        let mode_label = match &self.mode {
            Mode::Normal => "NORMAL",
            Mode::Insert => "INSERT",
            Mode::Visual { .. } => "VISUAL",
            Mode::Command { .. } => "",
            Mode::Search { .. } => "SEARCH",
        };

        let (cmdline, _cmd_cursor) = match &self.mode {
            Mode::Command { buf } => (Some(buf.clone()), buf.len()),
            Mode::Search { buf, dir } => {
                let prompt = match dir {
                    Direction::Forward => "/",
                    Direction::Backward => "?",
                };
                (Some(format!("{}{}", prompt, buf)), buf.len() + 1)
            }
            _ => (None, 0),
        };

        let titles: Vec<(&str, char)> = (0..self.menu_bar.menu_count())
            .map(|i| {
                (
                    self.menu_bar.title_at(i).unwrap_or(""),
                    self.menu_bar.alt_at(i).unwrap_or(' '),
                )
            })
            .collect();

        let (active_menu, dropdown) = if let Some((mi, items_slice)) = self.menu_bar.active_menu() {
            let active_item = self.menu_bar.active_item();
            let count = items_slice.len();
            let mut items = Vec::with_capacity(count);
            for ii in 0..count {
                if let Some((label, shortcut, sep, _has_action)) = self.menu_bar.item_label(mi, ii)
                {
                    items.push(MenuDropItem {
                        label,
                        shortcut,
                        sep,
                        selected: active_item.map(|(_, aii)| aii) == Some(ii),
                    });
                }
            }
            (Some(mi), items)
        } else {
            (None, vec![])
        };

        let menu_info = if self.menu_bar.visible {
            Some(MenuRenderInfo {
                visible: true,
                titles: &titles,
                active_menu,
                active_item: self.menu_bar.active_item().map(|(_, ii)| ii),
                dropdown: &dropdown,
            })
        } else {
            None
        };

        let mi_ref = menu_info.as_ref();

        let b = &self.buffers[self.current_buf];
        self.screen.render(
            out,
            b,
            &self.config,
            mi_ref,
            mode_label,
            cmdline.as_deref(),
            self.status.as_deref(),
        )
    }

    // ── Borrow helpers ──

    fn buf(&self) -> &Buffer {
        &self.buffers[self.current_buf]
    }

    fn buf_mut(&mut self) -> &mut Buffer {
        &mut self.buffers[self.current_buf]
    }

    fn take_count(&mut self) -> usize {
        if self.count_str.is_empty() {
            1
        } else {
            let n: usize = self.count_str.parse().unwrap_or(1);
            self.count_str.clear();
            n
        }
    }

    fn reset_state(&mut self) {
        self.count_str.clear();
        self.pending_op = None;
    }

    fn adjust_cx(&mut self) {
        let b = self.buf();
        let target = self.saved_cx.min(b.line_len(b.cy));
        self.saved_cx = target;
        let b = self.buf_mut();
        b.cx = target;
    }

    // ── Menu actions ──

    fn handle_menu_action(&mut self, action: MenuAction) {
        match action {
            MenuAction::NewFile => {
                self.buffers.push(Buffer::new());
                self.current_buf = self.buffers.len() - 1;
                self.mode = Mode::Normal;
            }
            MenuAction::OpenFile => {
                match dialog::file_dialog("Open File", "") {
                    DialogResult::Confirmed(path) => match Buffer::from_file(&path) {
                        Ok(b) => {
                            self.buffers.push(b);
                            self.current_buf = self.buffers.len() - 1;
                        }
                        Err(e) => self.status = Some(e),
                    },
                    DialogResult::Cancelled => {}
                }
                self.mode = Mode::Normal;
            }
            MenuAction::Save => {
                if self.buf().filename.is_some() {
                    match self.buf_mut().save() {
                        Ok(()) => self.status = Some("Written".into()),
                        Err(e) => self.status = Some(e),
                    }
                } else {
                    match dialog::file_dialog("Save As", "") {
                        DialogResult::Confirmed(path) => match self.buf_mut().save_as(&path) {
                            Ok(()) => self.status = Some("Written".into()),
                            Err(e) => self.status = Some(e),
                        },
                        DialogResult::Cancelled => {}
                    }
                }
                self.mode = Mode::Normal;
            }
            MenuAction::SaveAs => {
                let initial = self.buf().filename.clone().unwrap_or_default();
                match dialog::file_dialog("Save As", &initial) {
                    DialogResult::Confirmed(path) => match self.buf_mut().save_as(&path) {
                        Ok(()) => self.status = Some("Written".into()),
                        Err(e) => self.status = Some(e),
                    },
                    DialogResult::Cancelled => {}
                }
                self.mode = Mode::Normal;
            }
            MenuAction::CloseFile => {
                if self.buffers.len() > 1 {
                    self.buffers.remove(self.current_buf);
                    if self.current_buf >= self.buffers.len() {
                        self.current_buf = self.buffers.len() - 1;
                    }
                } else {
                    self.status = Some("Can't close last buffer".into());
                }
                self.mode = Mode::Normal;
            }
            MenuAction::Quit => {
                if self.buf().modified {
                    self.status = Some("No write since last change".into());
                    self.mode = Mode::Normal;
                } else {
                    self.quit = true;
                }
            }
            MenuAction::Undo => {
                self.undo();
                self.mode = Mode::Normal;
            }
            MenuAction::Redo => {
                self.redo();
                self.mode = Mode::Normal;
            }
            MenuAction::Copy => {
                let cy = self.buf().cy;
                self.clip = self
                    .buf()
                    .yank_range((cy, 0), (cy, self.buf().line_len(cy)));
                self.mode = Mode::Normal;
            }
            MenuAction::Paste => {
                self.paste(false);
                self.mode = Mode::Normal;
            }
            MenuAction::DeleteLine => {
                self.apply_op_linewise(OpType::Delete, 1);
                self.mode = Mode::Normal;
            }
            MenuAction::ToggleLineNumbers => {
                self.config.number = !self.config.number;
                self.mode = Mode::Normal;
            }
            MenuAction::ToggleRelativeNumbers => {
                self.config.relativenumber = !self.config.relativenumber;
                self.mode = Mode::Normal;
            }
            MenuAction::ToggleMenuBar => {
                self.menu_bar.visible = !self.menu_bar.visible;
            }
            MenuAction::Find => {
                self.mode = Mode::Search {
                    buf: String::new(),
                    dir: Direction::Forward,
                };
            }
            MenuAction::FindNext => {
                if !self.last_pattern.is_empty() {
                    let target = search::find_next(
                        self.buf(),
                        &self.last_pattern,
                        &self.config,
                        &Direction::Forward,
                    );
                    if let Some((ty, tx)) = target {
                        let b = self.buf_mut();
                        b.cy = ty;
                        b.cx = tx;
                        self.saved_cx = tx;
                    } else {
                        self.status = Some("Pattern not found".into());
                    }
                }
                self.mode = Mode::Normal;
            }
            MenuAction::FindPrev => {
                if !self.last_pattern.is_empty() {
                    let target = search::find_next(
                        self.buf(),
                        &self.last_pattern,
                        &self.config,
                        &Direction::Backward,
                    );
                    if let Some((ty, tx)) = target {
                        let b = self.buf_mut();
                        b.cy = ty;
                        b.cx = tx;
                        self.saved_cx = tx;
                    } else {
                        self.status = Some("Pattern not found".into());
                    }
                }
                self.mode = Mode::Normal;
            }
            MenuAction::Help => {
                dialog::help_dialog();
                self.mode = Mode::Normal;
            }
            MenuAction::About => {
                dialog::about_dialog();
                self.mode = Mode::Normal;
            }
        }
    }

    // ── Motion / operator-motion ──

    fn compute_motion_target(&self, motion: MotionType, count: usize) -> (usize, usize) {
        let b = self.buf();
        let (mut cy, mut cx) = (b.cy, b.cx);
        for _ in 0..count.max(1) {
            match motion {
                MotionType::Left => {
                    if cx > 0 {
                        cx -= 1;
                    } else if cy > 0 {
                        cy -= 1;
                        cx = b.line_len(cy);
                    }
                }
                MotionType::Right => {
                    if cx < b.line_len(cy) {
                        cx += 1;
                    } else if cy + 1 < b.lines.len() {
                        cy += 1;
                        cx = 0;
                    }
                }
                MotionType::Up => {
                    cy = cy.saturating_sub(1);
                    cx = cx.min(b.line_len(cy));
                }
                MotionType::Down => {
                    if cy + 1 < b.lines.len() {
                        cy += 1;
                    }
                    cx = cx.min(b.line_len(cy));
                }
                MotionType::WordForward => {
                    let p = motion::word_forward(b);
                    cy = p.0;
                    cx = p.1;
                }
                MotionType::WordBack => {
                    let p = motion::word_back(b);
                    cy = p.0;
                    cx = p.1;
                }
                MotionType::EndOfWord => {
                    let p = motion::end_of_word(b);
                    cy = p.0;
                    cx = p.1;
                }
                MotionType::LineStart => cx = 0,
                MotionType::LineEnd => cx = b.line_len(cy),
                MotionType::FirstNonBlank => cx = b.first_non_blank(),
                MotionType::FileStart => {
                    cy = 0;
                    cx = 0;
                }
                MotionType::FileEnd => {
                    cy = b.lines.len() - 1;
                    cx = b.line_len(cy);
                }
                MotionType::Percent => {
                    let total = b.lines.len();
                    let target = ((count as f64 / 100.0) * total as f64) as usize;
                    cy = target.min(total.saturating_sub(1));
                    cx = b.line_len(cy).min(cx);
                }
            }
        }
        (cy, cx)
    }

    fn apply_op_motion(&mut self, op: OpType, motion: MotionType, count: usize) {
        let start = {
            let b = self.buf();
            (b.cy, b.cx)
        };
        let end = self.compute_motion_target(motion, count);
        self.apply_op_range(op, start, end);
    }

    fn apply_op_linewise(&mut self, op: OpType, count: usize) {
        let cy = self.buf().cy;
        let total = self.buf().lines.len();
        let end_cy = (cy + count.saturating_sub(1)).min(total.saturating_sub(1));
        let start = (cy, 0);
        let end = (end_cy, self.buf().line_len(end_cy));
        self.apply_op_range(op, start, end);
    }

    fn apply_op_range(&mut self, op: OpType, start: (usize, usize), end: (usize, usize)) {
        let (s, e) = if start <= end {
            (start, end)
        } else {
            (end, start)
        };
        match op {
            OpType::Delete => {
                self.clip = self.buf().yank_range(s, e);
                self.buf_mut().delete_range(s, e);
                self.adjust_cx();
            }
            OpType::Yank => {
                self.clip = self.buf().yank_range(s, e);
            }
            OpType::Change => {
                self.clip = self.buf().yank_range(s, e);
                self.buf_mut().delete_range(s, e);
                self.adjust_cx();
                self.mode = Mode::Insert;
            }
        }
    }

    fn motion(&mut self, motion: MotionType, count: usize) {
        let target = self.compute_motion_target(motion, count);
        let b = self.buf_mut();
        b.cy = target.0;
        b.cx = target.1;
        self.saved_cx = b.cx;
    }

    fn try_op_motion(&mut self, motion: MotionType, count: usize) {
        if let Some(op) = self.pending_op.take() {
            self.apply_op_motion(op, motion, count);
        } else {
            self.motion(motion, count);
        }
    }

    // ── Normal mode ──

    fn handle_normal(&mut self, key: Key) {
        match key {
            Key::Char(c) if c.is_ascii_digit() => {
                if c == '0' && self.count_str.is_empty() {
                    self.try_op_motion(MotionType::LineStart, 1);
                } else {
                    self.count_str.push(c);
                }
            }
            Key::Char('h') => {
                let n = self.take_count();
                self.try_op_motion(MotionType::Left, n);
            }
            Key::Char('j') => {
                if let Some(op) = self.pending_op.take() {
                    let n = self.take_count();
                    self.apply_op_linewise(op, n);
                }
                let n = self.take_count();
                self.motion(MotionType::Down, n);
            }
            Key::Char('k') => {
                if let Some(op) = self.pending_op.take() {
                    let n = self.take_count();
                    self.apply_op_linewise(op, n);
                }
                let n = self.take_count();
                self.motion(MotionType::Up, n);
            }
            Key::Char('l') => {
                let n = self.take_count();
                self.try_op_motion(MotionType::Right, n);
            }
            Key::Char('w') => {
                let n = self.take_count();
                self.try_op_motion(MotionType::WordForward, n);
            }
            Key::Char('b') => {
                let n = self.take_count();
                self.try_op_motion(MotionType::WordBack, n);
            }
            Key::Char('e') => {
                let n = self.take_count();
                self.try_op_motion(MotionType::EndOfWord, n);
            }
            Key::Char('0') => self.try_op_motion(MotionType::LineStart, 1),
            Key::Char('$') => self.try_op_motion(MotionType::LineEnd, 1),
            Key::Char('^') => self.try_op_motion(MotionType::FirstNonBlank, 1),
            Key::Char('G') => self.try_op_motion(MotionType::FileEnd, 1),
            Key::Char('%') => {
                self.reset_state();
                let target = motion::matching_bracket(self.buf());
                if let Some((cy, cx)) = target {
                    let b = self.buf_mut();
                    b.cy = cy;
                    b.cx = cx;
                    self.saved_cx = cx;
                }
            }
            Key::Char('d') => {
                if self.pending_op.replace(OpType::Delete).is_some() {
                    let n = self.take_count();
                    self.apply_op_linewise(OpType::Delete, n);
                } else {
                    self.count_str.clear();
                }
            }
            Key::Char('y') => {
                if self.pending_op.replace(OpType::Yank).is_some() {
                    let n = self.take_count();
                    let cy = self.buf().cy;
                    let total = self.buf().lines.len();
                    let end = (cy + n.saturating_sub(1)).min(total.saturating_sub(1));
                    self.clip = self
                        .buf()
                        .yank_range((cy, 0), (end, self.buf().line_len(end)));
                } else {
                    self.count_str.clear();
                }
            }
            Key::Char('c') => {
                if self.pending_op.replace(OpType::Change).is_some() {
                    let n = self.take_count();
                    self.apply_op_linewise(OpType::Change, n);
                } else {
                    self.count_str.clear();
                }
            }
            Key::Char('x') => {
                self.reset_state();
                let (cy, cx, maybe_c) = {
                    let b = self.buf_mut();
                    (b.cy, b.cx, b.delete_char_at())
                };
                if let Some(c) = maybe_c {
                    self.push_undo(Action::Delete { pos: (cy, cx), c });
                }
                self.adjust_cx();
            }
            Key::Char('X') => {
                self.reset_state();
                let (pos, c) = {
                    let b = self.buf_mut();
                    let pos = (b.cy, b.cx);
                    (pos, b.delete_char_before())
                };
                if let Some(c) = c {
                    self.push_undo(Action::Delete { pos, c });
                }
            }
            Key::Char('i') => {
                self.reset_state();
                self.mode = Mode::Insert;
            }
            Key::Char('I') => {
                self.reset_state();
                let n = self.buf().first_non_blank();
                self.buf_mut().cx = n;
                self.mode = Mode::Insert;
            }
            Key::Char('a') => {
                self.reset_state();
                let len = self.buf().line_len(self.buf().cy);
                let cx = self.buf().cx;
                let b = self.buf_mut();
                if cx < len {
                    b.cx = cx + 1;
                }
                self.mode = Mode::Insert;
            }
            Key::Char('A') => {
                self.reset_state();
                let len = self.buf().line_len(self.buf().cy);
                self.buf_mut().cx = len;
                self.mode = Mode::Insert;
            }
            Key::Char('o') => {
                self.reset_state();
                self.buf_mut().insert_line_below();
                self.mode = Mode::Insert;
            }
            Key::Char('O') => {
                self.reset_state();
                self.buf_mut().insert_line_above();
                self.mode = Mode::Insert;
            }
            Key::Char('p') => {
                self.reset_state();
                self.paste(false);
            }
            Key::Char('P') => {
                self.reset_state();
                self.paste(true);
            }
            Key::Char('u') => {
                self.reset_state();
                self.undo();
            }
            Key::Ctrl('r') => {
                self.reset_state();
                self.redo();
            }
            Key::Char('s') => {
                self.reset_state();
                self.buf_mut().delete_char_at();
                self.mode = Mode::Insert;
            }
            Key::Char('S') => {
                self.reset_state();
                self.apply_op_linewise(OpType::Change, 1);
            }
            Key::Char('D') => {
                self.reset_state();
                let (cy, cx) = {
                    let b = self.buf();
                    (b.cy, b.cx)
                };
                let eol = {
                    let b = self.buf();
                    b.line_len(cy)
                };
                self.apply_op_range(OpType::Delete, (cy, cx), (cy, eol));
            }
            Key::Char('C') => {
                self.reset_state();
                let (cy, cx) = {
                    let b = self.buf();
                    (b.cy, b.cx)
                };
                let eol = {
                    let b = self.buf();
                    b.line_len(cy)
                };
                self.apply_op_range(OpType::Change, (cy, cx), (cy, eol));
            }
            Key::Char('J') => {
                self.reset_state();
                let second = {
                    let b = self.buf();
                    if b.cy + 1 < b.lines.len() {
                        Some((b.cy, b.lines[b.cy + 1].clone()))
                    } else {
                        None
                    }
                };
                if let Some((pos, s)) = second {
                    self.buf_mut().join_lines();
                    self.push_undo(Action::JoinLines {
                        pos,
                        second_line: s,
                    });
                    self.adjust_cx();
                }
            }
            Key::Char('r') => {
                self.reset_state();
                let next = crate::input::read_key();
                if let Key::Char(c) = next {
                    let result = {
                        let b = self.buf_mut();
                        if b.cx < b.line_len(b.cy) {
                            let old = b.lines[b.cy].remove(b.cx);
                            b.lines[b.cy].insert(b.cx, c);
                            b.modified = true;
                            Some(((b.cy, b.cx), old))
                        } else {
                            None
                        }
                    };
                    if let Some((pos, old)) = result {
                        self.push_undo(Action::Replace { pos, old, new: c });
                    }
                }
            }
            Key::Char('~') => {
                self.reset_state();
                let result = {
                    let b = self.buf_mut();
                    if b.cx < b.line_len(b.cy) {
                        let c = b.lines[b.cy].remove(b.cx);
                        let t = if c.is_ascii_uppercase() {
                            c.to_ascii_lowercase()
                        } else {
                            c.to_ascii_uppercase()
                        };
                        b.lines[b.cy].insert(b.cx, t);
                        b.modified = true;
                        let pos = (b.cy, b.cx);
                        if b.cx < b.line_len(b.cy) {
                            b.cx += 1;
                        }
                        Some((pos, c, t))
                    } else {
                        None
                    }
                };
                if let Some((pos, old, new)) = result {
                    self.push_undo(Action::Replace { pos, old, new });
                }
            }
            Key::Char('/') => {
                self.reset_state();
                self.mode = Mode::Search {
                    buf: String::new(),
                    dir: Direction::Forward,
                };
            }
            Key::Char('?') => {
                self.reset_state();
                self.mode = Mode::Search {
                    buf: String::new(),
                    dir: Direction::Backward,
                };
            }
            Key::Char('n') => {
                self.reset_state();
                if !self.last_pattern.is_empty() {
                    let target = search::find_next(
                        self.buf(),
                        &self.last_pattern,
                        &self.config,
                        &Direction::Forward,
                    );
                    if let Some((ty, tx)) = target {
                        let b = self.buf_mut();
                        b.cy = ty;
                        b.cx = tx;
                        self.saved_cx = tx;
                    }
                }
            }
            Key::Char('N') => {
                self.reset_state();
                if !self.last_pattern.is_empty() {
                    let target = search::find_next(
                        self.buf(),
                        &self.last_pattern,
                        &self.config,
                        &Direction::Backward,
                    );
                    if let Some((ty, tx)) = target {
                        let b = self.buf_mut();
                        b.cy = ty;
                        b.cx = tx;
                        self.saved_cx = tx;
                    }
                }
            }
            Key::Char(':') => {
                self.reset_state();
                self.mode = Mode::Command { buf: String::new() };
            }
            Key::Char('v') => {
                self.reset_state();
                let pos = {
                    let b = self.buf();
                    (b.cy, b.cx)
                };
                self.mode = Mode::Visual { start: pos };
            }
            Key::Char('.') => {
                self.reset_state();
                if let Some(actions) = self.undo_last_batch() {
                    self.history.undo();
                    self.replay_batch(&actions);
                }
            }
            Key::Char('g') => {
                let next = crate::input::read_key();
                if next == Key::Char('g') {
                    let n = self.take_count();
                    self.try_op_motion(MotionType::FileStart, n);
                } else if next == Key::Char('e') {
                    let n = self.take_count();
                    self.try_op_motion(MotionType::EndOfWord, n);
                } else {
                    self.reset_state();
                }
            }
            // Normal-mode Ctrl hotkeys
            Key::Ctrl('s') => {
                self.reset_state();
                self.handle_menu_action(MenuAction::Save);
            }
            Key::Ctrl('q') => {
                self.reset_state();
                self.handle_menu_action(MenuAction::Quit);
            }
            Key::Ctrl('n') => {
                self.reset_state();
                self.handle_menu_action(MenuAction::NewFile);
            }
            Key::Ctrl('o') => {
                self.reset_state();
                self.handle_menu_action(MenuAction::OpenFile);
            }
            Key::Ctrl('z') => {
                self.reset_state();
                self.undo();
            }
            Key::Ctrl('y') => {
                self.reset_state();
                self.redo();
            }
            Key::Ctrl('f') => {
                self.reset_state();
                let n = self.screen.height.saturating_sub(2) as isize;
                self.page_scroll(n);
            }
            Key::Ctrl('b') => {
                self.reset_state();
                let n = -(self.screen.height.saturating_sub(2) as isize);
                self.page_scroll(n);
            }
            Key::Ctrl('d') => {
                self.reset_state();
                let n = self.screen.height.saturating_sub(2) as isize / 2;
                self.page_scroll(n);
            }
            Key::Ctrl('u') => {
                self.reset_state();
                let n = -(self.screen.height.saturating_sub(2) as isize / 2);
                self.page_scroll(n);
            }
            Key::Escape => self.reset_state(),
            _ => self.reset_state(),
        }
    }

    // ── Insert mode ──

    fn handle_insert(&mut self, key: Key) {
        match key {
            Key::Escape => {
                self.mode = Mode::Normal;
                let cx = self.buf().cx;
                if cx > 0 {
                    self.buf_mut().cx = cx - 1;
                }
            }
            Key::Char(c) => {
                let pos = {
                    let b = self.buf();
                    (b.cy, b.cx)
                };
                self.buf_mut().insert_char(c);
                self.push_undo(Action::Insert { pos, c });
            }
            Key::Enter => {
                self.buf_mut().insert_newline();
            }
            Key::Backspace => {
                let (pos, c) = {
                    let b = self.buf_mut();
                    let pos = (b.cy, b.cx);
                    (pos, b.delete_char_before())
                };
                if let Some(c) = c {
                    self.push_undo(Action::Delete { pos, c });
                }
            }
            Key::Delete => {
                let (pos, c) = {
                    let b = self.buf_mut();
                    let pos = (b.cy, b.cx);
                    (pos, b.delete_char_at())
                };
                if let Some(c) = c {
                    self.push_undo(Action::Delete { pos, c });
                }
            }
            Key::Tab => {
                for _ in 0..self.config.tabstop {
                    let pos = {
                        let b = self.buf();
                        (b.cy, b.cx)
                    };
                    self.buf_mut().insert_char(' ');
                    self.push_undo(Action::Insert { pos, c: ' ' });
                }
            }
            Key::Left => {
                let cx = self.buf().cx;
                if cx > 0 {
                    self.buf_mut().cx = cx - 1;
                }
            }
            Key::Right => {
                let (cx, len) = {
                    let b = self.buf();
                    (b.cx, b.line_len(b.cy))
                };
                if cx < len {
                    self.buf_mut().cx = cx + 1;
                }
            }
            Key::Up => {
                let cy = self.buf().cy;
                if cy > 0 {
                    self.buf_mut().cy = cy - 1;
                }
                self.adjust_cx();
            }
            Key::Down => {
                let cy = self.buf().cy;
                if cy + 1 < self.buf().lines.len() {
                    self.buf_mut().cy = cy + 1;
                }
                self.adjust_cx();
            }
            Key::Home => self.buf_mut().cx = 0,
            Key::End => {
                let len = self.buf().line_len(self.buf().cy);
                self.buf_mut().cx = len;
            }
            _ => {}
        }
    }

    // ── Visual mode ──

    fn handle_visual(&mut self, key: Key, start: (usize, usize)) {
        match key {
            Key::Escape => self.mode = Mode::Normal,
            Key::Char('h') => self.motion(MotionType::Left, 1),
            Key::Char('j') => self.motion(MotionType::Down, 1),
            Key::Char('k') => self.motion(MotionType::Up, 1),
            Key::Char('l') => self.motion(MotionType::Right, 1),
            Key::Char('w') => self.motion(MotionType::WordForward, 1),
            Key::Char('b') => self.motion(MotionType::WordBack, 1),
            Key::Char('e') => self.motion(MotionType::EndOfWord, 1),
            Key::Char('0') => self.motion(MotionType::LineStart, 1),
            Key::Char('$') => self.motion(MotionType::LineEnd, 1),
            Key::Char('^') => self.motion(MotionType::FirstNonBlank, 1),
            Key::Char('G') => self.motion(MotionType::FileEnd, 1),
            Key::Char('d') | Key::Char('x') => {
                let end = {
                    let b = self.buf();
                    (b.cy, b.cx)
                };
                self.apply_op_range(OpType::Delete, start, end);
                self.mode = Mode::Normal;
            }
            Key::Char('y') => {
                let end = {
                    let b = self.buf();
                    (b.cy, b.cx)
                };
                self.apply_op_range(OpType::Yank, start, end);
                self.mode = Mode::Normal;
            }
            Key::Char('c') | Key::Char('s') => {
                let end = {
                    let b = self.buf();
                    (b.cy, b.cx)
                };
                self.apply_op_range(OpType::Change, start, end);
                self.mode = Mode::Insert;
            }
            Key::Char('>') => {
                let (s, e) = if start <= (self.buf().cy, self.buf().cx) {
                    (start.0, self.buf().cy)
                } else {
                    (self.buf().cy, start.0)
                };
                for row in s..=e {
                    self.buf_mut().lines[row].insert(0, ' ');
                    self.buf_mut().modified = true;
                }
                self.mode = Mode::Normal;
            }
            Key::Char('<') => {
                let (s, e) = if start <= (self.buf().cy, self.buf().cx) {
                    (start.0, self.buf().cy)
                } else {
                    (self.buf().cy, start.0)
                };
                for row in s..=e {
                    let b = self.buf_mut();
                    if !b.lines[row].is_empty()
                        && (b.lines[row].starts_with(' ') || b.lines[row].starts_with('\t'))
                    {
                        b.lines[row].remove(0);
                        b.modified = true;
                    }
                }
                self.mode = Mode::Normal;
            }
            Key::Char('u') => {
                let end = {
                    let b = self.buf();
                    (b.cy, b.cx)
                };
                let (s, e) = if start <= end {
                    (start, end)
                } else {
                    (end, start)
                };
                for row in s.0..=e.0 {
                    self.buf_mut().lines[row] = self.buf_mut().lines[row].to_lowercase();
                    self.buf_mut().modified = true;
                }
                self.mode = Mode::Normal;
            }
            Key::Char('U') => {
                let end = {
                    let b = self.buf();
                    (b.cy, b.cx)
                };
                let (s, e) = if start <= end {
                    (start, end)
                } else {
                    (end, start)
                };
                for row in s.0..=e.0 {
                    self.buf_mut().lines[row] = self.buf_mut().lines[row].to_uppercase();
                    self.buf_mut().modified = true;
                }
                self.mode = Mode::Normal;
            }
            // Visual-mode Ctrl hotkeys
            Key::Ctrl('s') => {
                self.handle_menu_action(MenuAction::Save);
            }
            _ => {}
        }
    }

    // ── Command mode ──

    fn handle_command(&mut self, key: Key, buf: &str) {
        match key {
            Key::Escape | Key::Ctrl('c') => self.mode = Mode::Normal,
            Key::Enter => match command::parse(buf) {
                Ok(cmd) => self.exec_command(cmd),
                Err(e) => {
                    if !e.is_empty() {
                        self.status = Some(e);
                    }
                    if !matches!(self.mode, Mode::Command { .. }) {
                        self.mode = Mode::Normal;
                    }
                }
            },
            Key::Backspace => {
                if buf.is_empty() {
                    self.mode = Mode::Normal;
                } else {
                    self.mode = Mode::Command {
                        buf: buf[..buf.len() - 1].to_string(),
                    };
                }
            }
            Key::Char(c) => {
                let mut new = buf.to_string();
                new.push(c);
                self.mode = Mode::Command { buf: new };
            }
            _ => {}
        }
    }

    // ── Search mode ──

    fn handle_search(&mut self, key: Key, buf: &str, dir: Direction) {
        match key {
            Key::Escape | Key::Ctrl('c') => self.mode = Mode::Normal,
            Key::Enter => {
                if !buf.is_empty() {
                    self.last_pattern = buf.to_string();
                    let target = search::find_next(self.buf(), buf, &self.config, &dir);
                    if let Some((ty, tx)) = target {
                        let b = self.buf_mut();
                        b.cy = ty;
                        b.cx = tx;
                        self.saved_cx = tx;
                    } else {
                        self.status = Some("Pattern not found".into());
                    }
                }
                self.mode = Mode::Normal;
            }
            Key::Backspace => {
                if buf.is_empty() {
                    self.mode = Mode::Normal;
                } else {
                    self.mode = Mode::Search {
                        buf: buf[..buf.len() - 1].to_string(),
                        dir,
                    };
                }
            }
            Key::Char(c) => {
                let mut new = buf.to_string();
                new.push(c);
                self.mode = Mode::Search { buf: new, dir };
            }
            _ => {}
        }
    }

    // ── Command exec ──

    fn exec_command(&mut self, cmd: command::Command) {
        match cmd {
            command::Command::Quit => {
                if self.buf().modified {
                    self.status = Some("No write since last change (add ! to override)".into());
                    self.mode = Mode::Normal;
                } else {
                    self.quit = true;
                }
            }
            command::Command::ForceQuit => self.quit = true,
            command::Command::Write => {
                match self.buf_mut().save() {
                    Ok(()) => self.status = Some("Written".into()),
                    Err(e) => self.status = Some(e),
                }
                self.mode = Mode::Normal;
            }
            command::Command::ForceWrite => {
                if self.buf().filename.is_none() {
                    self.status = Some("No filename".into());
                } else {
                    let _ = self.buf_mut().save();
                    self.status = Some("Written".into());
                }
                self.mode = Mode::Normal;
            }
            command::Command::WriteQuit => {
                if self.buf().filename.is_some() {
                    let _ = self.buf_mut().save();
                }
                self.quit = true;
            }
            command::Command::ForceWriteQuit => {
                let _ = self.buf_mut().save();
                self.quit = true;
            }
            command::Command::WriteAll => {
                for i in 0..self.buffers.len() {
                    let _ = self.buffers[i].save();
                }
                self.status = Some("Written".into());
                self.mode = Mode::Normal;
            }
            command::Command::QuitAll => {
                if self.buffers.iter().any(|b| b.modified) {
                    self.status = Some("No write since last change (add ! to override)".into());
                    self.mode = Mode::Normal;
                } else {
                    self.quit = true;
                }
            }
            command::Command::ForceQuitAll => self.quit = true,
            command::Command::Open(path) => match Buffer::from_file(&path) {
                Ok(b) => {
                    self.buffers.push(b);
                    self.current_buf = self.buffers.len() - 1;
                    self.mode = Mode::Normal;
                }
                Err(e) => {
                    self.status = Some(e);
                    self.mode = Mode::Normal;
                }
            },
            command::Command::Set(key, value) => {
                match self.config.set(&key, value.as_deref()) {
                    Ok(()) => self.status = Some(format!("{}={}", key, value.unwrap_or_default())),
                    Err(e) => self.status = Some(e),
                }
                self.mode = Mode::Normal;
            }
            command::Command::NoHLSearch => {
                self.config.hlsearch = false;
                self.mode = Mode::Normal;
            }
            command::Command::Help => {
                self.status = Some(
                    "rvim v0.1  F1=Help F2=Menu F3=Numbers F4=RelNums  :q to quit  :w to save"
                        .into(),
                );
                self.mode = Mode::Normal;
            }
            command::Command::ShowOptions => {
                self.status = Some(self.config.get().join("  "));
                self.mode = Mode::Normal;
            }
            command::Command::New => {
                self.buffers.push(Buffer::new());
                self.current_buf = self.buffers.len() - 1;
                self.mode = Mode::Normal;
            }
            command::Command::Vnew => {
                self.buffers.push(Buffer::new());
                self.current_buf = self.buffers.len() - 1;
                self.mode = Mode::Normal;
            }
            command::Command::Only => {
                let keep = self.current_buf;
                let b = self.buffers[keep].clone();
                self.buffers.clear();
                self.buffers.push(b);
                self.current_buf = 0;
                self.mode = Mode::Normal;
            }
            command::Command::BufferNext => {
                if self.current_buf + 1 < self.buffers.len() {
                    self.current_buf += 1;
                    self.status = Some(format!(
                        "Buffer {}/{}",
                        self.current_buf + 1,
                        self.buffers.len()
                    ));
                } else {
                    self.status = Some("Already at last buffer".into());
                }
                self.mode = Mode::Normal;
            }
            command::Command::BufferPrev => {
                if self.current_buf > 0 {
                    self.current_buf -= 1;
                    self.status = Some(format!(
                        "Buffer {}/{}",
                        self.current_buf + 1,
                        self.buffers.len()
                    ));
                } else {
                    self.status = Some("Already at first buffer".into());
                }
                self.mode = Mode::Normal;
            }
            command::Command::BufferList => {
                let lines: Vec<String> = self
                    .buffers
                    .iter()
                    .enumerate()
                    .map(|(i, b)| {
                        let active = if i == self.current_buf { ">>" } else { "  " };
                        format!(
                            "  {} {:>2}  {}",
                            active,
                            i + 1,
                            b.filename.as_deref().unwrap_or("[No Name]")
                        )
                    })
                    .collect();
                if let Some(idx) = dialog::buffer_list_dialog(&lines) {
                    if idx < self.buffers.len() {
                        self.current_buf = idx;
                    }
                }
                self.mode = Mode::Normal;
            }
        }
    }

    // ── Undo / redo / history ──

    fn push_undo(&mut self, a: Action) {
        self.history.push(a);
    }

    fn undo(&mut self) {
        if let Some(actions) = self.history.undo() {
            for a in actions.iter().rev() {
                match a {
                    Action::Insert { pos, .. } => {
                        let b = self.buf_mut();
                        b.cy = pos.0;
                        b.cx = pos.1;
                        if b.cx <= b.line_len(b.cy) && b.cx > 0 {
                            b.lines[b.cy].remove(b.cx - 1);
                            b.cx = b.cx.wrapping_sub(1);
                            b.modified = true;
                        }
                    }
                    Action::Delete { pos, c } => {
                        let b = self.buf_mut();
                        b.cy = pos.0;
                        b.cx = pos.1;
                        b.lines[b.cy].insert(b.cx, *c);
                        b.modified = true;
                    }
                    Action::InsertLine { pos, .. } => {
                        self.buf_mut().lines.remove(*pos);
                        self.buf_mut().modified = true;
                    }
                    Action::DeleteLine { pos, line } => {
                        let b = self.buf_mut();
                        b.lines.insert(*pos, line.clone());
                        b.cy = *pos;
                        b.cx = 0;
                        b.modified = true;
                    }
                    Action::InsertNewline { pos, line } => {
                        let b = self.buf_mut();
                        b.cy = pos.0;
                        b.cx = pos.1;
                        if b.cy + 1 < b.lines.len() {
                            b.lines.remove(b.cy + 1);
                        }
                        b.lines[b.cy] = line.clone();
                        b.modified = true;
                    }
                    Action::JoinLines { pos, second_line } => {
                        let b = self.buf_mut();
                        b.cy = *pos;
                        if b.cy + 1 < b.lines.len() {
                            let _ = b.lines.remove(b.cy + 1);
                        }
                        b.lines[b.cy] = second_line.clone();
                        b.modified = true;
                    }
                    Action::Replace { pos, old, .. } => {
                        let b = self.buf_mut();
                        b.cy = pos.0;
                        b.cx = pos.1;
                        if b.cx < b.line_len(b.cy) {
                            b.lines[b.cy].remove(b.cx);
                            b.lines[b.cy].insert(b.cx, *old);
                            b.modified = true;
                        }
                    }
                    Action::Batch { .. } => {}
                }
            }
            self.adjust_cx();
        } else {
            self.status = Some("Already at oldest change".into());
        }
    }

    fn redo(&mut self) {
        if let Some(actions) = self.history.redo() {
            self.replay_batch(&actions);
            self.adjust_cx();
        } else {
            self.status = Some("Already at newest change".into());
        }
    }

    fn replay_batch(&mut self, actions: &[Action]) {
        for a in actions {
            match a {
                Action::Insert { pos, c } => {
                    let b = self.buf_mut();
                    b.cy = pos.0;
                    b.cx = pos.1;
                    b.insert_char(*c);
                }
                Action::Delete { pos, c } => {
                    let b = self.buf_mut();
                    b.cy = pos.0;
                    b.cx = pos.1;
                    b.lines[b.cy].insert(b.cx, *c);
                    b.modified = true;
                }
                Action::InsertNewline { pos, line: _ } => {
                    let b = self.buf_mut();
                    b.cy = pos.0;
                    b.cx = pos.1;
                    b.insert_newline();
                }
                _ => {}
            }
        }
    }

    fn undo_last_batch(&self) -> Option<Vec<Action>> {
        self.history.last_undo()
    }

    // ── Paste ──

    fn paste(&mut self, before: bool) {
        let text = self.clip.clone();
        if text.is_empty() {
            return;
        }
        let has_nl = text.contains('\n');
        self.history.start_batch();
        if has_nl {
            if before {
                self.buf_mut().insert_line_above();
                for ch in text.chars() {
                    if ch == '\n' {
                        let pos = {
                            let b = self.buf();
                            (b.cy, b.cx)
                        };
                        self.buf_mut().insert_newline();
                        let line = {
                            let b = self.buf();
                            b.lines[b.cy].clone()
                        };
                        self.push_undo(Action::InsertNewline { pos, line });
                    } else {
                        let pos = {
                            let b = self.buf();
                            (b.cy, b.cx)
                        };
                        self.buf_mut().insert_char(ch);
                        self.push_undo(Action::Insert { pos, c: ch });
                    }
                }
            } else {
                self.buf_mut().insert_line_below();
                self.buf_mut().insert_line_above();
                for ch in text.chars() {
                    if ch == '\n' {
                        let pos = {
                            let b = self.buf();
                            (b.cy, b.cx)
                        };
                        self.buf_mut().insert_newline();
                        let line = {
                            let b = self.buf();
                            b.lines[b.cy].clone()
                        };
                        self.push_undo(Action::InsertNewline { pos, line });
                    } else {
                        let pos = {
                            let b = self.buf();
                            (b.cy, b.cx)
                        };
                        self.buf_mut().insert_char(ch);
                        self.push_undo(Action::Insert { pos, c: ch });
                    }
                }
            }
        } else {
            let text_len = text.len();
            let pos = {
                let b = self.buf();
                (b.cy, if before { b.cx } else { b.cx + 1 })
            };
            let b = self.buf_mut();
            let cy = b.cy;
            let cx = if before { b.cx } else { b.cx + 1 };
            b.lines[cy].insert_str(cx, &text);
            b.cx = cx + text_len;
            b.modified = true;
            self.push_undo(Action::Insert { pos, c: '\0' });
        }
        self.history.end_batch();
    }

    fn page_scroll(&mut self, delta: isize) {
        let b = self.buf_mut();
        if delta > 0 {
            b.cy = (b.cy + delta as usize).min(b.lines.len().saturating_sub(1));
        } else {
            b.cy = b.cy.saturating_sub(delta.unsigned_abs());
        }
        self.adjust_cx();
    }
}
