cat << 'INNER_EOF' > src/engine.rs
use crate::buffer::CharBuffer;
use crate::validation::is_valid_vietnamese_syllable;
use crate::telex::{self, Tone};

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
            c if c.is_ascii_alphabetic() || c == '[' || c == ']' => self.handle_char(c),
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
        let mut raw_chars = ['\0'; CharBuffer::MAX_CAPACITY];
        let len = self.raw_buffer.len();
        raw_chars[..len].copy_from_slice(self.raw_buffer.as_slice());

        for i in 0..len {
            self.apply_telex_rule(raw_chars[i]);
        }
    }

    fn try_apply_change<F>(&mut self, change_fn: F) -> bool
    where
        F: FnOnce(&mut Engine),
    {
        // Snapshot the buffer
        let mut snapshot_data = ['\0'; 16];
        let len = self.buffer.len();
        snapshot_data[..len].copy_from_slice(self.buffer.as_slice());

        // Apply change
        change_fn(self);

        // Validate
        if is_valid_vietnamese_syllable(self.buffer.as_slice()) {
            return true; // Success
        }

        // Revert on failure
        self.buffer.clear();
        for i in 0..len {
            self.buffer.push(snapshot_data[i]);
        }
        false
    }

    fn apply_telex_rule(&mut self, next_char: char) {
        // Special case for standalone 'w' -> 'ư'
        if next_char == 'w' || next_char == ']' {
            let success = self.try_apply_change(|engine| {
                if let Some(last) = engine.buffer.last() {
                    if let Some(modified) = telex::apply_vowel_modifier(last, 'w') {
                        engine.buffer.replace_last(modified);
                        return;
                    }
                }
                engine.buffer.push('ư');
            });
            if success { return; }
        }

        // Handle z key (free typing/cancel all tones)
        if next_char == 'z' {
            let mut has_tone = false;
            let mut snapshot_data = ['\0'; 16];
            let len = self.buffer.len();
            snapshot_data[..len].copy_from_slice(self.buffer.as_slice());

            for i in 0..len {
                let (base, tone) = telex::get_base_vowel_and_tone(snapshot_data[i]);
                if tone != Tone::None {
                    has_tone = true;
                    self.buffer.replace_at(i, base); // Remove tone
                }
            }

            if has_tone {
                // Return early without adding 'z'
                return;
            } else {
                // No tones to clear, treat as normal char
                self.buffer.push(next_char);
                return;
            }
        }

        // Tones
        if let Some(input_tone) = Tone::from_char(next_char) {
            // Find target vowel index
            if let Some(target_idx) = telex::find_tone_target_index(self.buffer.as_slice()) {
                let current_char = self.buffer.as_slice()[target_idx];
                let (base, current_tone) = telex::get_base_vowel_and_tone(current_char);

                let success = self.try_apply_change(|engine| {
                    if current_tone == input_tone {
                        // Cancellation: 'á' + 's' -> 'a' + 's'
                        engine.buffer.replace_at(target_idx, base);
                        engine.buffer.push(next_char);
                    } else {
                        // Override/Set
                        let new_char = telex::add_tone(base, input_tone);

                        // Clear tones from all other vowels in the buffer
                        for i in 0..engine.buffer.len() {
                            if i != target_idx && telex::is_vowel(engine.buffer.as_slice()[i]) {
                                let (other_base, _) = telex::get_base_vowel_and_tone(engine.buffer.as_slice()[i]);
                                engine.buffer.replace_at(i, other_base);
                            }
                        }

                        engine.buffer.replace_at(target_idx, new_char);
                    }
                });
                if success { return; }
            }
        }

        // Consonant / Vowel Modifiers
        if let Some(last_char) = self.buffer.last() {
            let success = self.try_apply_change(|engine| {
                if let Some(modified) = telex::apply_vowel_modifier(last_char, next_char) {
                    engine.buffer.replace_last(modified);
                } else if last_char == 'd' && next_char == 'd' {
                    engine.buffer.replace_last('đ');
                } else if last_char == 'đ' && next_char == 'd' {
                    // Cancellation: đ + d -> dd
                    engine.buffer.replace_last('d');
                    engine.buffer.push('d');
                } else if let Some(removed) = telex::remove_vowel_modifier(last_char) {
                    // Check cancellation of vowels: â + a -> aa
                    let (base, _) = telex::get_base_vowel_and_tone(last_char);
                    let (removed_base, _) = telex::get_base_vowel_and_tone(removed);
                    let expected_modifier = match removed_base {
                        'a' if base == 'â' => 'a',
                        'a' if base == 'ă' => 'w',
                        'e' if base == 'ê' => 'e',
                        'o' if base == 'ô' => 'o',
                        'o' if base == 'ơ' => 'w',
                        'u' if base == 'ư' => 'w',
                        _ => '\0',
                    };
                    if next_char == expected_modifier || (next_char == '[' && (base == 'ơ' || base == 'ư')) {
                        engine.buffer.replace_last(removed);
                        engine.buffer.push(next_char);
                    } else {
                        engine.buffer.push(next_char);
                    }
                } else {
                    engine.buffer.push(next_char);
                }
            });
            if success { return; }
        }

        // If all else fails or buffer was empty, just push it
        self.buffer.push(next_char);
    }
}
INNER_EOF
