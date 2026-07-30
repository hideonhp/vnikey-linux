use crate::buffer::CharBuffer;
use crate::validation::is_valid_vietnamese_syllable;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Idle,
    Composing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Preedit(CharBuffer),
    Commit(CharBuffer),
    PassThrough,
}

pub struct Engine {
    pub state: State,
    pub buffer: CharBuffer,
    pub raw_buffer: CharBuffer,
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl Engine {
    pub fn new() -> Self {
        Self {
            state: State::Idle,
            buffer: CharBuffer::new(),
            raw_buffer: CharBuffer::new(),
        }
    }

    pub fn process_key(&mut self, key: char) -> Action {
        match key {
            '\x08' | '\x7f' => self.handle_backspace(),
            ' ' | '\n' | '\r' => self.handle_commit(key),
            c if c.is_ascii_alphabetic() => self.handle_char(c),
            _ => {
                if self.state == State::Composing {
                    let commit_action = Action::Commit(self.buffer);
                    self.reset();
                    commit_action
                } else {
                    Action::PassThrough
                }
            }
        }
    }

    fn handle_backspace(&mut self) -> Action {
        if self.state == State::Idle {
            return Action::PassThrough;
        }

        self.raw_buffer.pop();

        if self.raw_buffer.is_empty() {
            self.reset();
            return Action::Preedit(CharBuffer::new());
        }

        self.rebuild_buffer();

        Action::Preedit(self.buffer)
    }

    fn handle_commit(&mut self, trigger_key: char) -> Action {
        if self.state == State::Idle {
            return Action::PassThrough;
        }

        if !self.buffer.is_full() {
            self.buffer.push(trigger_key);
        }

        let action = Action::Commit(self.buffer);
        self.reset();
        action
    }

    fn handle_char(&mut self, c: char) -> Action {
        let c = c.to_ascii_lowercase();

        if self.state == State::Idle {
            self.state = State::Composing;
        } else if self.buffer.is_full() || self.raw_buffer.is_full() {
            let commit_action = Action::Commit(self.buffer);
            self.reset();

            self.state = State::Composing;
            self.raw_buffer.push(c);
            self.rebuild_buffer();

            return commit_action;
        }

        self.raw_buffer.push(c);
        self.rebuild_buffer();

        Action::Preedit(self.buffer)
    }

    fn reset(&mut self) {
        self.state = State::Idle;
        self.buffer.clear();
        self.raw_buffer.clear();
    }

    fn rebuild_buffer(&mut self) {
        self.buffer.clear();
        // Copy the raw buffer slice to avoid borrow checker issues when calling apply_telex_rule
        let mut raw_chars = ['\0'; CharBuffer::MAX_CAPACITY];
        let len = self.raw_buffer.len();
        raw_chars[..len].copy_from_slice(self.raw_buffer.as_slice());

        for i in 0..len {
            self.apply_telex_rule(raw_chars[i]);
        }
    }

    fn apply_telex_rule(&mut self, next_char: char) {
        let last_char = self.buffer.last();

        let new_char = match (last_char, next_char) {
            (Some('a'), 's') => Some('á'),
            (Some('a'), 'f') => Some('à'),
            (Some('a'), 'w') => Some('ă'),
            (Some('o'), 'o') => Some('ô'),
            (Some('d'), 'd') => Some('đ'),
            _ => {
                self.buffer.push(next_char);
                return;
            }
        };

        if let Some(c) = new_char {
            let old_char = last_char.unwrap();
            self.buffer.replace_last(c);

            if !is_valid_vietnamese_syllable(self.buffer.as_slice()) {
                // Revert and push as normal character
                self.buffer.replace_last(old_char);
                self.buffer.push(next_char);
            }
        }
    }
}
