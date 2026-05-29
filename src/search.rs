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
use crate::config::Config;

#[derive(Debug, Clone, PartialEq)]
pub enum Direction {
    Forward,
    Backward,
}

pub fn find_next(
    b: &Buffer,
    pattern: &str,
    config: &Config,
    dir: &Direction,
) -> Option<(usize, usize)> {
    if pattern.is_empty() {
        return None;
    }
    let case_sensitive = if config.smartcase {
        pattern.chars().any(|c| c.is_uppercase())
    } else {
        !config.ignorecase
    };
    match dir {
        Direction::Forward => find_forward(b, pattern, case_sensitive),
        Direction::Backward => find_backward(b, pattern, case_sensitive),
    }
}

fn find_forward(b: &Buffer, pattern: &str, case_sensitive: bool) -> Option<(usize, usize)> {
    let nlines = b.lines.len();
    let start_row = b.cy;
    let start_col = b.cx + 1;

    if let Some(col) = find_in_line(&b.lines[start_row], pattern, case_sensitive, start_col) {
        return Some((start_row, col));
    }
    for row in start_row + 1..nlines {
        if let Some(col) = find_in_line(&b.lines[row], pattern, case_sensitive, 0) {
            return Some((row, col));
        }
    }
    for row in 0..start_row {
        if let Some(col) = find_in_line(&b.lines[row], pattern, case_sensitive, 0) {
            return Some((row, col));
        }
    }
    None
}

fn find_backward(b: &Buffer, pattern: &str, case_sensitive: bool) -> Option<(usize, usize)> {
    let start_row = b.cy;

    let limit = b.cx.saturating_sub(1);
    if let Some(col) = find_in_line_rev(&b.lines[start_row], pattern, case_sensitive, limit) {
        return Some((start_row, col));
    }
    for row in (0..start_row).rev() {
        if let Some(col) =
            find_in_line_rev(&b.lines[row], pattern, case_sensitive, b.lines[row].len())
        {
            return Some((row, col));
        }
    }
    for row in (start_row + 1..b.lines.len()).rev() {
        if let Some(col) =
            find_in_line_rev(&b.lines[row], pattern, case_sensitive, b.lines[row].len())
        {
            return Some((row, col));
        }
    }
    None
}

fn find_in_line(line: &str, pattern: &str, cs: bool, start: usize) -> Option<usize> {
    let search_line = if cs {
        line.to_string()
    } else {
        line.to_lowercase()
    };
    let search_pat = if cs {
        pattern.to_string()
    } else {
        pattern.to_lowercase()
    };
    if start >= search_line.len() {
        return None;
    }
    search_line[start..].find(&search_pat).map(|i| i + start)
}

fn find_in_line_rev(line: &str, pattern: &str, cs: bool, limit: usize) -> Option<usize> {
    let search_line = if cs {
        line.to_string()
    } else {
        line.to_lowercase()
    };
    let search_pat = if cs {
        pattern.to_string()
    } else {
        pattern.to_lowercase()
    };
    if limit < search_pat.len() {
        return None;
    }
    let end = limit.min(search_line.len());
    search_line[..end].rfind(&search_pat)
}

pub fn find_all(b: &Buffer, pattern: &str, cs: bool) -> Vec<(usize, usize, usize)> {
    if pattern.is_empty() {
        return vec![];
    }
    let mut results = Vec::new();
    for (row, line) in b.lines.iter().enumerate() {
        let search_line = if cs {
            line.to_string()
        } else {
            line.to_lowercase()
        };
        let search_pat = if cs {
            pattern.to_string()
        } else {
            pattern.to_lowercase()
        };
        let mut start = 0;
        while let Some(col) = search_line[start..].find(&search_pat) {
            let abs_col = start + col;
            results.push((row, abs_col, abs_col + search_pat.len()));
            start = abs_col + 1;
            if start >= search_line.len() {
                break;
            }
        }
    }
    results
}
