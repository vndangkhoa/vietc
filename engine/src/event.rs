// SPDX-License-Identifier: MIT
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Typed input event - the core of Event Sourcing.
/// KHÔNG lưu nội dung nhạy cảm, chỉ lưu event sequence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InputEvent {
    /// A character key was typed
    KeyTyped(char),
    /// Backspace was pressed
    Backspace,
    /// A flush character (space, punctuation, enter, tab)
    Flush(char),
    /// Text was pasted
    Paste(String),
}

/// Append-only event store.
/// Source of truth for all user input.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventStore {
    events: Vec<InputEvent>,
}

impl EventStore {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    pub fn push(&mut self, event: InputEvent) {
        self.events.push(event);
    }

    pub fn pop(&mut self) -> Option<InputEvent> {
        self.events.pop()
    }

    pub fn clear(&mut self) {
        self.events.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &InputEvent> {
        self.events.iter()
    }

    pub fn as_slice(&self) -> &[InputEvent] {
        &self.events
    }

    /// Extract raw keystrokes from event log (for auto-restore comparison).
    /// Only reconstructs the literal characters typed, excluding backspaces.
    pub fn raw_keystrokes(&self) -> String {
        let mut s = String::new();
        for event in &self.events {
            match event {
                InputEvent::KeyTyped(c) => s.push(*c),
                InputEvent::Backspace => { s.pop(); }
                InputEvent::Flush(_) => {}
                InputEvent::Paste(text) => s.push_str(text),
            }
        }
        s
    }

    /// Hash the event type sequence (not content) for privacy-safe pattern detection.
    /// Output: sha256 hex of event type characters (K=KeyTyped, B=Backspace, F=Flush, P=Paste).
    /// Không thể recover text gốc — chỉ biết "có X events với pattern Y".
    pub fn pattern_hash(&self) -> String {
        let types: String = self.events.iter().map(|e| match e {
            InputEvent::KeyTyped(_) => 'K',
            InputEvent::Backspace => 'B',
            InputEvent::Flush(_) => 'F',
            InputEvent::Paste(_) => 'P',
        }).collect();
        let mut hasher = Sha256::new();
        hasher.update(types.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

impl Default for EventStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Formalized output commands (Command Pattern).
/// Chỉ chứa diff instruction, không chứa text nhạy cảm.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Type a string of characters
    Type(String),
    /// Backspace N times
    Backspace(usize),
}
