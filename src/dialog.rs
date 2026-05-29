// Copyright (c) 2026 Immmanuel Jeyaraj <irj@sefier.com>. MIT License.

use crossterm::{cursor, queue, style, terminal};
use std::io::{stdout, Write};

use crate::input::{read_key, Key};

const HELP_LINES: &[&str] = &[
    "  GLOBAL SHORTCUTS",
    "    F1               Help",
    "    F2               Toggle menu bar",
    "    F3               Toggle line numbers",
    "    F4               Toggle relative numbers",
    "    Ctrl+F10         Activate menu",
    "    Alt+letter       Open matching menu",
    "",
    "  NORMAL MODE",
    "    h/j/k/l          Cursor movement (left/down/up/right)",
    "    w                Word forward",
    "    b                Word back",
    "    e                End of word",
    "    0                Line start",
    "    $                Line end",
    "    ^                First non-blank",
    "    gg               File start",
    "    G                File end",
    "    %                Matching bracket",
    "    i                Insert before cursor",
    "    I                Insert at line start",
    "    a                Append after cursor",
    "    A                Append at line end",
    "    o                Open line below",
    "    O                Open line above",
    "    d{motion}        Delete",
    "    dd               Delete line",
    "    y{motion}        Yank (copy)",
    "    yy               Yank line",
    "    c{motion}        Change (delete then insert)",
    "    cc               Change line",
    "    x                Delete char at cursor",
    "    X                Delete char before cursor",
    "    s                Substitute char",
    "    S                Substitute line",
    "    D                Delete to end of line",
    "    C                Change to end of line",
    "    J                Join lines",
    "    r{char}          Replace char",
    "    ~                Toggle case",
    "    .                Repeat last change",
    "    u                Undo",
    "    Ctrl+r           Redo",
    "    p                Paste after",
    "    P                Paste before",
    "    /pattern         Search forward",
    "    ?pattern         Search backward",
    "    n                Repeat search forward",
    "    N                Repeat search backward",
    "    :                Command-line mode",
    "    v                Visual mode",
    "",
    "  NORMAL MODE Ctrl",
    "    Ctrl+s           Save",
    "    Ctrl+q           Quit",
    "    Ctrl+n           New file",
    "    Ctrl+o           Open file",
    "    Ctrl+z           Undo",
    "    Ctrl+y           Redo",
    "    Ctrl+f           Page down",
    "    Ctrl+b           Page up",
    "    Ctrl+d           Half page down",
    "    Ctrl+u           Half page up",
    "",
    "  INSERT MODE",
    "    Escape           Return to Normal",
    "    Enter            New line",
    "    Backspace        Delete char before",
    "    Delete           Delete char at",
    "    Tab              Insert spaces",
    "    Arrow keys       Navigate text",
    "",
    "  VISUAL MODE",
    "    d/x              Delete selection",
    "    y                Yank selection",
    "    c/s              Change selection",
    "    >                Indent selection",
    "    <                Outdent selection",
    "    u                Lowercase selection",
    "    U                Uppercase selection",
    "    Escape           Return to Normal",
    "",
    "  COMMAND-LINE (:commands)",
    "    :q               Quit",
    "    :w               Save (write)",
    "    :wq              Save & quit",
    "    :q!              Force quit",
    "    :e <file>        Open file",
    "    :new             New buffer",
    "    :bn              Next buffer",
    "    :bp              Previous buffer",
    "    :ls              List buffers",
    "    :only            Close other buffers",
    "    :set <opt>       Set option",
    "    :noh             Clear search highlight",
];

pub fn help_dialog() {
    paginated_dialog(" Help ", HELP_LINES);
}

pub fn buffer_list_dialog(buffers: &[String]) -> Option<usize> {
    if buffers.is_empty() {
        return None;
    }
    let mut out = stdout();
    let (w, h) = terminal::size().unwrap_or((80, 24));
    let box_w = (w as usize).clamp(50, 80);
    let avail_h = (h as usize).saturating_sub(4).min(buffers.len() + 4);
    let box_h = avail_h.min(h as usize - 4);
    let box_x = (w as usize).saturating_sub(box_w) / 2;
    let box_y = (h as usize).saturating_sub(box_h) / 2;
    let content_h = box_h.saturating_sub(3).max(1);
    let total = buffers.len();
    let max_scroll = total.saturating_sub(content_h);
    let mut scroll = 0usize;
    let mut num_buf = String::new();

    loop {
        draw_buffer_popup(
            &mut out, buffers, box_x, box_y, box_w, box_h, scroll, total, &num_buf,
        );
        match read_key() {
            Key::Escape | Key::Char('q') | Key::Ctrl('c') => {
                let _ = out.flush();
                return None;
            }
            Key::Enter => {
                let _ = out.flush();
                if let Ok(n) = num_buf.parse::<usize>() {
                    if n >= 1 && n <= total {
                        return Some(n - 1);
                    }
                }
                num_buf.clear();
            }
            Key::Backspace => {
                num_buf.pop();
            }
            Key::Char(c) if c.is_ascii_digit() => {
                num_buf.push(c);
            }
            Key::Up if scroll > 0 => scroll = scroll.saturating_sub(1),
            Key::Down if scroll < max_scroll => scroll += 1,
            Key::PageUp => scroll = scroll.saturating_sub(content_h),
            Key::PageDown => scroll = (scroll + content_h).min(max_scroll),
            Key::Home => scroll = 0,
            Key::End => scroll = max_scroll,
            _ => {}
        }
    }
}

fn paginated_dialog(title: &str, lines: &[impl AsRef<str>]) {
    let mut out = stdout();
    let (w, h) = terminal::size().unwrap_or((80, 24));
    let box_w = (w as usize).clamp(50, 80);
    let avail_h = (h as usize).saturating_sub(4).min(lines.len() + 4);
    let box_h = avail_h.min(h as usize - 4);
    let box_x = (w as usize).saturating_sub(box_w) / 2;
    let box_y = (h as usize).saturating_sub(box_h) / 2;
    let content_h = box_h.saturating_sub(3).max(1);
    let total = lines.len();
    let max_scroll = total.saturating_sub(content_h);
    let mut scroll = 0usize;

    loop {
        draw_text_popup(
            &mut out, title, lines, box_x, box_y, box_w, box_h, scroll, total,
        );
        match read_key() {
            Key::Escape | Key::Char('q') | Key::Ctrl('c') => {
                let _ = out.flush();
                return;
            }
            Key::Up if scroll > 0 => scroll = scroll.saturating_sub(1),
            Key::Down if scroll < max_scroll => scroll += 1,
            Key::PageUp => scroll = scroll.saturating_sub(content_h),
            Key::PageDown => scroll = (scroll + content_h).min(max_scroll),
            Key::Home => scroll = 0,
            Key::End => scroll = max_scroll,
            _ => {}
        }
    }
}

fn draw_buffer_popup(
    out: &mut std::io::Stdout,
    lines: &[String],
    box_x: usize,
    box_y: usize,
    box_w: usize,
    box_h: usize,
    scroll: usize,
    total: usize,
    num_buf: &str,
) {
    let inner = box_w.saturating_sub(4);
    let top = format!("┌{}┐", "─".repeat(box_w.saturating_sub(2)));

    let input_info = if num_buf.is_empty() {
        " type # + Enter ".to_string()
    } else {
        format!("  Go to #{}  ", num_buf)
    };
    let bottom = format!(
        "└{}┘",
        "─".repeat(box_w.saturating_sub(2).saturating_sub(input_info.len()))
    );

    for row in 0..box_h {
        let _ = queue!(out, cursor::MoveTo(box_x as u16, box_y as u16 + row as u16));
        let _ = queue!(out, terminal::Clear(terminal::ClearType::CurrentLine));
        if row == 0 {
            let _ = queue!(out, style::SetForegroundColor(style::Color::Cyan));
            let _ = queue!(out, style::Print(&top));
            let _ = queue!(out, style::ResetColor);
            let _ = queue!(out, cursor::MoveTo(box_x as u16 + 1, box_y as u16));
            let _ = queue!(out, style::SetForegroundColor(style::Color::Cyan));
            let _ = queue!(out, style::Print(" Buffers "));
            let _ = queue!(out, style::ResetColor);
        } else if row == box_h - 1 {
            let _ = queue!(out, style::SetForegroundColor(style::Color::Cyan));
            let _ = queue!(out, style::Print(&bottom));
            let _ = queue!(out, style::ResetColor);
            let _ = queue!(
                out,
                cursor::MoveTo(
                    (box_x + box_w - 1 - input_info.len()) as u16,
                    box_y as u16 + row as u16
                )
            );
            let _ = queue!(out, style::SetForegroundColor(style::Color::Cyan));
            let _ = queue!(out, style::Print(&input_info));
            let _ = queue!(out, style::ResetColor);
        } else {
            let content_row = row.saturating_sub(1) + scroll;
            if content_row < total {
                let line = &lines[content_row];
                let truncated = if line.len() > inner {
                    &line[..inner]
                } else {
                    line.as_str()
                };
                let _ = queue!(
                    out,
                    style::Print(format!("│ {:<inner$} │", truncated, inner = inner))
                );
            } else {
                let _ = queue!(
                    out,
                    style::Print(format!("│{:1$}│", "", box_w.saturating_sub(2)))
                );
            }
        }
    }
    let _ = out.flush();
}

fn draw_text_popup(
    out: &mut std::io::Stdout,
    title: &str,
    lines: &[impl AsRef<str>],
    box_x: usize,
    box_y: usize,
    box_w: usize,
    box_h: usize,
    scroll: usize,
    total: usize,
) {
    let inner = box_w.saturating_sub(4);
    let top = format!("┌{}┐", "─".repeat(box_w.saturating_sub(2)));

    let line_info = format!(" line {}/{} ", scroll + 1, total);
    let bottom = format!(
        "└{}┘",
        "─".repeat(box_w.saturating_sub(2).saturating_sub(line_info.len()))
    );

    for row in 0..box_h {
        let _ = queue!(out, cursor::MoveTo(box_x as u16, box_y as u16 + row as u16));
        let _ = queue!(out, terminal::Clear(terminal::ClearType::CurrentLine));
        if row == 0 {
            let _ = queue!(out, style::SetForegroundColor(style::Color::Cyan));
            let _ = queue!(out, style::Print(&top));
            let _ = queue!(out, style::ResetColor);
            let _ = queue!(out, cursor::MoveTo(box_x as u16 + 1, box_y as u16));
            let _ = queue!(out, style::SetForegroundColor(style::Color::Cyan));
            let _ = queue!(out, style::Print(title));
            let _ = queue!(out, style::ResetColor);
        } else if row == box_h - 1 {
            let _ = queue!(out, style::SetForegroundColor(style::Color::Cyan));
            let _ = queue!(out, style::Print(&bottom));
            let _ = queue!(out, style::ResetColor);
            let _ = queue!(
                out,
                cursor::MoveTo(
                    (box_x + box_w - 1 - line_info.len()) as u16,
                    box_y as u16 + row as u16
                )
            );
            let _ = queue!(out, style::SetForegroundColor(style::Color::Cyan));
            let _ = queue!(out, style::Print(&line_info));
            let _ = queue!(out, style::ResetColor);
        } else {
            let content_row = row.saturating_sub(1) + scroll;
            if content_row < total {
                let line = lines[content_row].as_ref();
                let truncated = if line.len() > inner {
                    &line[..inner]
                } else {
                    line
                };
                let _ = queue!(
                    out,
                    style::Print(format!("│ {:<inner$} │", truncated, inner = inner))
                );
            } else {
                let _ = queue!(
                    out,
                    style::Print(format!("│{:1$}│", "", box_w.saturating_sub(2)))
                );
            }
        }
    }
    let _ = out.flush();
}

pub enum DialogResult {
    Confirmed(String),
    Cancelled,
}

pub fn file_dialog(title: &str, initial: &str) -> DialogResult {
    let mut out = stdout();
    let (w, h) = terminal::size().unwrap_or((80, 24));
    let box_w = (w as usize).clamp(40, 60);
    let box_h: usize = 7;
    let box_x = (w as usize).saturating_sub(box_w) / 2;
    let box_y = (h as usize).saturating_sub(box_h) / 2;

    let mut input = initial.to_string();
    loop {
        draw_file(&mut out, title, &input, box_x, box_y, box_w, box_h);
        match read_key() {
            Key::Enter => {
                let result = input.trim().to_string();
                return if result.is_empty() {
                    DialogResult::Cancelled
                } else {
                    DialogResult::Confirmed(result)
                };
            }
            Key::Escape => return DialogResult::Cancelled,
            Key::Backspace => {
                input.pop();
            }
            Key::Char(c) => {
                input.push(c);
            }
            _ => {}
        }
    }
}

fn draw_file(
    out: &mut std::io::Stdout,
    title: &str,
    input: &str,
    box_x: usize,
    box_y: usize,
    box_w: usize,
    box_h: usize,
) {
    let inner = box_w.saturating_sub(4);

    let title_capped: String = title.chars().take(inner).collect();
    let title_fill = inner.saturating_sub(title_capped.len());
    let top = format!("┌─{}{}─┐", title_capped, "─".repeat(title_fill));

    let bottom = format!("└{}┘", "─".repeat(box_w.saturating_sub(2)));
    let pad = format!("│{:1$}│", "", box_w.saturating_sub(2));

    let input_capped: String = input.chars().take(inner).collect();
    let input_fill = inner.saturating_sub(input_capped.len());
    let input_line = format!("│ {}{} │", input_capped, " ".repeat(input_fill));

    let hint = format!("│ {:<inner$} │", "[Enter] ok  [Esc] cancel", inner = inner);

    for row in 0..box_h {
        let _ = queue!(out, cursor::MoveTo(box_x as u16, box_y as u16 + row as u16));
        let _ = queue!(out, terminal::Clear(terminal::ClearType::CurrentLine));
        let text = match row {
            0 => &top,
            1 => &pad,
            2 => &input_line,
            3 => &pad,
            4 => &hint,
            5 => &pad,
            _ => &bottom,
        };
        let _ = queue!(out, style::SetForegroundColor(style::Color::Cyan));
        let _ = queue!(out, style::Print(text));
        let _ = queue!(out, style::ResetColor);
    }

    let cursor_x = box_x + 2 + input_capped.len();
    let _ = queue!(out, cursor::MoveTo(cursor_x as u16, box_y as u16 + 2));
    let _ = out.flush();
}

pub fn about_dialog() {
    let mut out = stdout();
    let (w, h) = terminal::size().unwrap_or((80, 24));
    let box_w = (w as usize).clamp(42, 60);
    let box_h: usize = 9;
    let box_x = (w as usize).saturating_sub(box_w) / 2;
    let box_y = (h as usize).saturating_sub(box_h) / 2;
    let inner = box_w.saturating_sub(4);

    loop {
        let _ = queue!(out, cursor::MoveTo(0, 0));
        let _ = terminal::Clear(terminal::ClearType::All);

        let lines = [
            format!("┌{}┐", "─".repeat(box_w.saturating_sub(2))),
            format!("│{:1$}│", "", box_w.saturating_sub(2)),
            format!(
                "│ {:^inner$} │",
                "rvim — A minimal Vim clone in Rust",
                inner = inner
            ),
            format!("│{:1$}│", "", box_w.saturating_sub(2)),
            format!("│ {:^inner$} │", "Author: Immmanuel Jeyaraj", inner = inner),
            format!("│ {:^inner$} │", "<irj@sefier.com>", inner = inner),
            format!("│{:1$}│", "", box_w.saturating_sub(2)),
            format!("│ {:>inner$} │", "[Esc] close", inner = inner),
            format!("└{}┘", "─".repeat(box_w.saturating_sub(2))),
        ];

        for (row, text) in lines.iter().enumerate() {
            let _ = queue!(out, cursor::MoveTo(box_x as u16, box_y as u16 + row as u16));
            let _ = queue!(out, terminal::Clear(terminal::ClearType::CurrentLine));
            let _ = queue!(out, style::SetForegroundColor(style::Color::Cyan));
            let _ = queue!(out, style::Print(text));
            let _ = queue!(out, style::ResetColor);
        }

        let _ = out.flush();
        match read_key() {
            Key::Escape | Key::Char('q') | Key::Ctrl('c') => {
                let _ = queue!(out, cursor::MoveTo(0, 0));
                let _ = terminal::Clear(terminal::ClearType::All);
                let _ = out.flush();
                return;
            }
            _ => {}
        }
    }
}
