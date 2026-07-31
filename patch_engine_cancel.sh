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

    fn apply_telex_rule(&mut self, next_char: char) {
        // Snapshot the buffer
        let mut snapshot_data = ['\0'; 16];
        let len = self.buffer.len();
        snapshot_data[..len].copy_from_slice(self.buffer.as_slice());

        // Special case for standalone 'w' -> 'ư'
        if next_char == 'w' || next_char == ']' {
            let mut applied = false;
            if let Some(last) = self.buffer.last() {
                if let Some(modified) = telex::apply_vowel_modifier(last, 'w') {
                    self.buffer.replace_last(modified);
                    applied = true;
                }
            }
            if !applied {
                self.buffer.push('ư');
            }

            // Note: In `oww` scenario, `ow` gives `ơ`, and another `w` should revert `ơ` back to `ow`.
            // But we already handle it in vowel modifiers block. Wait, the `w` block executes first and might intercept it!
            // Let's remove the special case here, or only do it if the buffer is empty or last char is not vowel.
        }

        // We should just use one coherent block. Let's rebuild this cleaner.
        self.buffer.clear();
        for i in 0..len {
            self.buffer.push(snapshot_data[i]);
        }

        if next_char == 'z' {
            let mut has_tone = false;
            for i in 0..len {
                let (base, tone) = telex::get_base_vowel_and_tone(snapshot_data[i]);
                if tone != Tone::None {
                    has_tone = true;
                    self.buffer.replace_at(i, base);
                }
            }
            if has_tone { return; } else { self.buffer.push(next_char); return; }
        }

        if let Some(input_tone) = Tone::from_char(next_char) {
            let mut tone_found_anywhere = false;
            for i in 0..len {
                if telex::get_base_vowel_and_tone(self.buffer.as_slice()[i]).1 == input_tone {
                    tone_found_anywhere = true;
                    break;
                }
            }

            if let Some(target_idx) = telex::find_tone_target_index(self.buffer.as_slice()) {
                let current_char = self.buffer.as_slice()[target_idx];
                let (base, current_tone) = telex::get_base_vowel_and_tone(current_char);

                if current_tone == input_tone || tone_found_anywhere {
                    for i in 0..self.buffer.len() {
                        let (b, t) = telex::get_base_vowel_and_tone(self.buffer.as_slice()[i]);
                        if t == input_tone {
                            self.buffer.replace_at(i, b);
                        }
                    }
                    self.buffer.push(next_char);
                } else {
                    let new_char = telex::add_tone(base, input_tone);
                    for i in 0..self.buffer.len() {
                        if i != target_idx && telex::is_vowel(self.buffer.as_slice()[i]) {
                            let (other_base, _) = telex::get_base_vowel_and_tone(self.buffer.as_slice()[i]);
                            self.buffer.replace_at(i, other_base);
                        }
                    }
                    self.buffer.replace_at(target_idx, new_char);
                }

                if current_tone == input_tone || tone_found_anywhere { return; }

                if is_valid_vietnamese_syllable(self.buffer.as_slice()) { return; }
                else {
                    self.buffer.clear();
                    for i in 0..len { self.buffer.push(snapshot_data[i]); }
                }
            }
        }

        if let Some(last_char) = self.buffer.last() {
            let mut applied = false;
            let mut cancelled = false;

            if let Some(removed) = telex::remove_vowel_modifier(last_char) {
                let (base, tone) = telex::get_base_vowel_and_tone(last_char);
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
                    self.buffer.replace_last(telex::add_tone(removed_base, tone));
                    self.buffer.push(next_char);
                    applied = true;
                    cancelled = true;
                }
            }

            if !applied && last_char == 'đ' && next_char == 'd' {
                self.buffer.replace_last('d');
                self.buffer.push('d');
                applied = true;
                cancelled = true;
            }

            if !applied {
                if let Some(modified) = telex::apply_vowel_modifier(last_char, next_char) {
                    self.buffer.replace_last(modified);
                    applied = true;
                } else if last_char == 'd' && next_char == 'd' {
                    self.buffer.replace_last('đ');
                    applied = true;
                } else if next_char == 'w' || next_char == ']' {
                    self.buffer.push('ư');
                    applied = true;
                }
            }

            if applied {
                if cancelled { return; }
                if is_valid_vietnamese_syllable(self.buffer.as_slice()) { return; }
                else {
                    self.buffer.clear();
                    for i in 0..len { self.buffer.push(snapshot_data[i]); }
                }
            }
        } else {
            if next_char == 'w' || next_char == ']' {
                self.buffer.push('ư');
                if is_valid_vietnamese_syllable(self.buffer.as_slice()) { return; }
                else { self.buffer.clear(); }
            }
        }

        self.buffer.push(next_char);
    }
}
INNER_EOF
