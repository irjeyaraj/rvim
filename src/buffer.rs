// Copyright (c) 2026 Immmanuel Jeyaraj <irj@sefier.com>. MIT License.

use std::fs;

#[derive(Debug, Clone)]
pub struct Buffer {
    pub lines: Vec<String>,
    pub filename: Option<String>,
    pub modified: bool,
    pub cx: usize,
    pub cy: usize,
}

impl Buffer {
    pub fn new() -> Self {
        Self {
            lines: vec![String::new()],
            filename: None,
            modified: false,
            cx: 0,
            cy: 0,
        }
    }

    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
        let lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
        let lines = if lines.is_empty() {
            vec![String::new()]
        } else {
            lines
        };
        Ok(Self {
            lines,
            filename: Some(path.to_string()),
            modified: false,
            cx: 0,
            cy: 0,
        })
    }

    pub fn save(&mut self) -> Result<(), String> {
        let path = self.filename.as_ref().ok_or("No filename")?;
        let content = self.lines.join("\n");
        fs::write(path, &content).map_err(|e| e.to_string())?;
        self.modified = false;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn save_as(&mut self, path: &str) -> Result<(), String> {
        self.filename = Some(path.to_string());
        self.save()
    }

    pub fn insert_char(&mut self, c: char) {
        let line = &mut self.lines[self.cy];
        line.insert(self.cx, c);
        self.cx += 1;
        self.modified = true;
    }

    pub fn delete_char_before(&mut self) -> Option<char> {
        if self.cx == 0 {
            if self.cy == 0 {
                return None;
            }
            let removed = self.lines.remove(self.cy);
            self.cy -= 1;
            let prev_len = self.lines[self.cy].len();
            self.lines[self.cy].push_str(&removed);
            self.cx = prev_len;
            self.modified = true;
            Some('\n')
        } else {
            let line = &mut self.lines[self.cy];
            let c = line.remove(self.cx - 1);
            self.cx -= 1;
            self.modified = true;
            Some(c)
        }
    }

    pub fn delete_char_at(&mut self) -> Option<char> {
        if self.lines[self.cy].is_empty() {
            if self.cy + 1 < self.lines.len() {
                self.lines.remove(self.cy + 1);
                self.modified = true;
                Some('\n')
            } else {
                None
            }
        } else {
            let line = &mut self.lines[self.cy];
            if self.cx < line.len() {
                Some(line.remove(self.cx))
            } else {
                None
            }
        }
    }

    pub fn insert_newline(&mut self) {
        let line = &mut self.lines[self.cy];
        let rest = line.split_off(self.cx);
        self.lines.insert(self.cy + 1, rest);
        self.cy += 1;
        self.cx = 0;
        self.modified = true;
    }

    pub fn join_lines(&mut self) {
        if self.cy + 1 >= self.lines.len() {
            return;
        }
        let next = self.lines.remove(self.cy + 1);
        let line = &mut self.lines[self.cy];
        if !line.is_empty() && !next.is_empty() {
            line.push(' ');
        }
        line.push_str(&next);
        self.modified = true;
    }

    pub fn insert_line_above(&mut self) {
        self.lines.insert(self.cy, String::new());
        self.cx = 0;
        self.modified = true;
    }

    pub fn insert_line_below(&mut self) {
        self.lines.insert(self.cy + 1, String::new());
        self.cy += 1;
        self.cx = 0;
        self.modified = true;
    }

    pub fn delete_line(&mut self) -> String {
        let line = self.lines.remove(self.cy);
        if self.lines.is_empty() {
            self.lines.push(String::new());
        }
        if self.cy >= self.lines.len() {
            self.cy = self.lines.len() - 1;
        }
        self.clamp_cx();
        self.modified = true;
        line
    }

    pub fn delete_range(&mut self, start: (usize, usize), end: (usize, usize)) -> String {
        if start > end {
            return self.delete_range(end, start);
        }
        let mut result = String::new();
        if start.0 == end.0 {
            let line = &mut self.lines[start.0];
            let tail: String = line.drain(start.1..end.1).collect();
            result.push_str(&tail);
        } else {
            let first = &mut self.lines[start.0];
            let first_tail: String = first.drain(start.1..).collect();
            result.push_str(&first_tail);
            result.push('\n');
            for row in (start.0 + 1..end.0).rev() {
                let line = self.lines.remove(row);
                result.push_str(&line);
                result.push('\n');
            }
            let last = &mut self.lines[end.0];
            let last_head: String = last.drain(..end.1).collect();
            result.push_str(&last_head);
            let first = self.lines[start.0].clone();
            let removed = self.lines.remove(start.0 + 1);
            self.lines[start.0] = format!("{}{}", first, removed);
        }
        self.cy = start.0;
        self.cx = start.1;
        self.modified = true;
        result
    }

    pub fn yank_range(&self, start: (usize, usize), end: (usize, usize)) -> String {
        if start > end {
            return self.yank_range(end, start);
        }
        let mut result = String::new();
        if start.0 == end.0 {
            result.push_str(&self.lines[start.0][start.1..end.1]);
        } else {
            result.push_str(&self.lines[start.0][start.1..]);
            result.push('\n');
            for row in start.0 + 1..end.0 {
                result.push_str(&self.lines[row]);
                result.push('\n');
            }
            result.push_str(&self.lines[end.0][..end.1]);
        }
        result
    }

    pub fn clamp_cx(&mut self) {
        let max = self.lines[self.cy].len();
        if self.cx > max {
            self.cx = max;
        }
    }

    pub fn clamp_cy(&mut self) {
        if self.cy >= self.lines.len() {
            self.cy = self.lines.len().saturating_sub(1);
        }
    }

    pub fn first_non_blank(&self) -> usize {
        self.lines[self.cy].len() - self.lines[self.cy].trim_start().len()
    }

    pub fn line_len(&self, row: usize) -> usize {
        if row < self.lines.len() {
            self.lines[row].len()
        } else {
            0
        }
    }

    pub fn lineno(&self) -> usize {
        self.cy + 1
    }

    pub fn total_lines(&self) -> usize {
        self.lines.len()
    }

    pub fn name_display(&self) -> &str {
        self.filename.as_deref().unwrap_or("[No Name]")
    }
}
