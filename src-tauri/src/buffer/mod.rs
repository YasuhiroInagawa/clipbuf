//! In-memory FIFO buffer of captured items. Never persisted (10.1, 10.2).
//!
//! Items are kept newest first, bounded by `capacity` (2.1, 2.2). Pushing text equal to the
//! newest item is a duplicate and is dropped (1.6). Items are immutable once stored (6.9).

use std::collections::VecDeque;
use std::time::SystemTime;

use crate::model::{ClipItem, ItemId, Warning};

/// Outcome of `Buffer::push`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PushResult {
    Pushed(ItemId),
    /// Same text as the newest item; nothing was added.
    Duplicate,
}

#[derive(Debug)]
pub struct Buffer {
    /// Newest at the front.
    items: VecDeque<ClipItem>,
    capacity: usize,
    next_id: ItemId,
}

impl Buffer {
    /// Create an empty buffer. A capacity of 0 is treated as 1.
    pub fn new(capacity: usize) -> Self {
        Self {
            items: VecDeque::new(),
            capacity: capacity.max(1),
            next_id: 1,
        }
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Add a captured item at the front, evicting the oldest when over capacity.
    pub fn push(
        &mut self,
        text: String,
        html: Option<String>,
        rtf: Option<String>,
        warnings: Vec<Warning>,
    ) -> PushResult {
        if self.items.front().is_some_and(|newest| newest.text == text) {
            return PushResult::Duplicate;
        }
        let id = self.next_id;
        self.next_id += 1;
        self.items.push_front(ClipItem {
            id,
            captured_at: SystemTime::now(),
            text,
            html,
            rtf,
            warnings,
        });
        self.truncate();
        PushResult::Pushed(id)
    }

    pub fn get(&self, id: ItemId) -> Option<&ClipItem> {
        self.items.iter().find(|item| item.id == id)
    }

    /// Remove one item. Returns false when the id is unknown.
    pub fn remove(&mut self, id: ItemId) -> bool {
        match self.items.iter().position(|item| item.id == id) {
            Some(index) => {
                self.items.remove(index);
                true
            }
            None => false,
        }
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }

    /// Change the capacity, dropping the oldest items if the buffer is now over it (2.5).
    pub fn set_capacity(&mut self, capacity: usize) {
        self.capacity = capacity.max(1);
        self.truncate();
    }

    /// Items newest first.
    pub fn items(&self) -> impl Iterator<Item = &ClipItem> {
        self.items.iter()
    }

    fn truncate(&mut self) {
        self.items.truncate(self.capacity);
    }
}

#[cfg(test)]
mod tests;
