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

#[derive(Debug, Clone)]
pub enum Action {
    Insert {
        pos: (usize, usize),
        c: char,
    },
    Delete {
        pos: (usize, usize),
        c: char,
    },
    InsertLine {
        pos: usize,
        line: String,
    },
    DeleteLine {
        pos: usize,
        line: String,
    },
    InsertNewline {
        pos: (usize, usize),
        line: String,
    },
    JoinLines {
        pos: usize,
        second_line: String,
    },
    Replace {
        pos: (usize, usize),
        old: char,
        new: char,
    },
    Batch {
        actions: Vec<Action>,
    },
}

#[derive(Debug)]
pub struct History {
    undo_stack: Vec<Vec<Action>>,
    redo_stack: Vec<Vec<Action>>,
    batch: Option<Vec<Action>>,
    batch_depth: usize,
}

impl History {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            batch: None,
            batch_depth: 0,
        }
    }

    pub fn push(&mut self, action: Action) {
        self.redo_stack.clear();
        match &mut self.batch {
            Some(batch) => batch.push(action),
            None => self.undo_stack.push(vec![action]),
        }
    }

    pub fn start_batch(&mut self) {
        if self.batch_depth == 0 {
            self.batch = Some(Vec::new());
        }
        self.batch_depth += 1;
    }

    pub fn end_batch(&mut self) {
        self.batch_depth = self.batch_depth.saturating_sub(1);
        if self.batch_depth == 0 {
            if let Some(batch) = self.batch.take() {
                if !batch.is_empty() {
                    self.undo_stack.push(batch);
                }
            }
        }
    }

    pub fn last_undo(&self) -> Option<Vec<Action>> {
        self.undo_stack.last().cloned()
    }

    pub fn undo(&mut self) -> Option<Vec<Action>> {
        let actions = self.undo_stack.pop()?;
        self.redo_stack.push(actions.clone());
        Some(actions)
    }

    pub fn redo(&mut self) -> Option<Vec<Action>> {
        let actions = self.redo_stack.pop()?;
        self.undo_stack.push(actions.clone());
        Some(actions)
    }
}
