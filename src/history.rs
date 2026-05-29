// Copyright (c) 2026 Immmanuel Jeyaraj <irj@sefier.com>. MIT License.

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
