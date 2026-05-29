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

use crate::buffer::Buffer;

pub fn word_forward(b: &Buffer) -> (usize, usize) {
    let mut cy = b.cy;
    let mut cx = b.cx;
    let line = &b.lines[cy];

    if cx >= line.len() {
        if cy + 1 < b.lines.len() {
            cy += 1;
            cx = 0;
        }
        return (cy, cx);
    }

    let is_word = line.as_bytes()[cx].is_ascii_alphanumeric() || line.as_bytes()[cx] == b'_';

    if is_word {
        while cx < line.len()
            && (line.as_bytes()[cx].is_ascii_alphanumeric() || line.as_bytes()[cx] == b'_')
        {
            cx += 1;
        }
    } else if line.as_bytes()[cx].is_ascii_whitespace() {
        while cx < line.len() && line.as_bytes()[cx].is_ascii_whitespace() {
            cx += 1;
        }
    } else {
        while cx < line.len()
            && !line.as_bytes()[cx].is_ascii_alphanumeric()
            && line.as_bytes()[cx] != b'_'
            && !line.as_bytes()[cx].is_ascii_whitespace()
        {
            cx += 1;
        }
    }

    if cx >= line.len() && cy + 1 < b.lines.len() {
        cy += 1;
        cx = 0;
    }
    if cx > b.lines[cy].len() {
        cx = b.lines[cy].len();
    }
    (cy, cx)
}

pub fn word_back(b: &Buffer) -> (usize, usize) {
    let mut cy = b.cy;
    let mut cx = b.cx;

    if cx == 0 {
        if cy == 0 {
            return (0, 0);
        }
        cy -= 1;
        cx = b.lines[cy].len();
        return (cy, cx);
    }
    cx -= 1;
    let line = &b.lines[cy];

    while cx > 0 && line.as_bytes()[cx].is_ascii_whitespace() {
        cx -= 1;
    }
    if cx == 0 && line.as_bytes()[0].is_ascii_whitespace() {
        return (cy, 0);
    }

    let c = line.as_bytes()[cx];
    let is_word = c.is_ascii_alphanumeric() || c == b'_';

    if is_word {
        while cx > 0
            && (line.as_bytes()[cx - 1].is_ascii_alphanumeric() || line.as_bytes()[cx - 1] == b'_')
        {
            cx -= 1;
        }
    } else {
        while cx > 0
            && !line.as_bytes()[cx - 1].is_ascii_alphanumeric()
            && line.as_bytes()[cx - 1] != b'_'
            && !line.as_bytes()[cx - 1].is_ascii_whitespace()
        {
            cx -= 1;
        }
    }
    (cy, cx)
}

pub fn end_of_word(b: &Buffer) -> (usize, usize) {
    let mut cy = b.cy;
    let mut cx = b.cx;
    let line = &b.lines[cy];

    if cx >= line.len() {
        if cy + 1 < b.lines.len() {
            cy += 1;
            cx = 0;
        } else {
            return (cy, line.len().saturating_sub(1));
        }
    }
    let line = &b.lines[cy];
    if cx >= line.len() {
        return (cy, line.len().saturating_sub(1));
    }

    let c = line.as_bytes()[cx];
    let is_word = c.is_ascii_alphanumeric() || c == b'_';

    if is_word {
        while cx + 1 < line.len()
            && (line.as_bytes()[cx + 1].is_ascii_alphanumeric() || line.as_bytes()[cx + 1] == b'_')
        {
            cx += 1;
        }
    } else if !c.is_ascii_whitespace() {
        while cx + 1 < line.len()
            && !line.as_bytes()[cx + 1].is_ascii_alphanumeric()
            && line.as_bytes()[cx + 1] != b'_'
            && !line.as_bytes()[cx + 1].is_ascii_whitespace()
        {
            cx += 1;
        }
    }
    (cy, cx)
}

pub fn matching_bracket(b: &Buffer) -> Option<(usize, usize)> {
    let line = &b.lines[b.cy];
    if b.cx >= line.len() {
        return None;
    }
    let c = line.as_bytes()[b.cx];
    let (open, close, forward) = match c {
        b'(' => (b'(', b')', true),
        b')' => (b'(', b')', false),
        b'[' => (b'[', b']', true),
        b']' => (b'[', b']', false),
        b'{' => (b'{', b'}', true),
        b'}' => (b'{', b'}', false),
        _ => return None,
    };

    let mut depth = 0;
    if forward {
        for i in b.cx..line.len() {
            let bc = line.as_bytes()[i];
            if bc == open {
                depth += 1;
            } else if bc == close {
                depth -= 1;
                if depth == 0 {
                    return Some((b.cy, i));
                }
            }
        }
        for row in b.cy + 1..b.lines.len() {
            let l = &b.lines[row];
            for i in 0..l.len() {
                let bc = l.as_bytes()[i];
                if bc == open {
                    depth += 1;
                } else if bc == close {
                    depth -= 1;
                    if depth == 0 {
                        return Some((row, i));
                    }
                }
            }
        }
    } else {
        for i in (0..=b.cx).rev() {
            let bc = line.as_bytes()[i];
            if bc == close {
                depth += 1;
            } else if bc == open {
                depth -= 1;
                if depth == 0 {
                    return Some((b.cy, i));
                }
            }
        }
        for row in (0..b.cy).rev() {
            let l = &b.lines[row];
            for i in (0..l.len()).rev() {
                let bc = l.as_bytes()[i];
                if bc == close {
                    depth += 1;
                } else if bc == open {
                    depth -= 1;
                    if depth == 0 {
                        return Some((row, i));
                    }
                }
            }
        }
    }
    None
}
