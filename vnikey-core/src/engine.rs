use crate::buffer::CharBuffer;
use crate::telex::{self, Tone};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMethod {
    Telex,
    Vni,
}

pub struct Engine {
    current_method: InputMethod,
    pub state: State,
    pub buffer: CharBuffer,
    pub raw_buffer: CharBuffer,
}

impl Default for Engine {
    fn default() -> Self {
        Self::new(InputMethod::Telex)
    }
}

impl Engine {
    pub fn new(method: InputMethod) -> Self {
        Self {
            current_method: method,
            state: State::Idle,
            buffer: CharBuffer::new(),
            raw_buffer: CharBuffer::new(),
        }
    }

    pub fn set_input_method(&mut self, method: InputMethod) -> Option<Action> {
        if self.state == State::Composing {
            let commit_action = Action::Commit(self.buffer);
            self.reset();
            self.current_method = method;
            Some(commit_action)
        } else {
            self.current_method = method;
            None
        }
    }

    pub fn process_key(&mut self, key: char) -> Action {
        match key {
            '\x08' | '\x7f' => self.handle_backspace(),
            ' ' | '\n' | '\r' => self.handle_commit(key),
            c if c.is_ascii_alphabetic() || c == '[' || c == ']' || c.is_ascii_digit() => {
                self.handle_char(c)
            }
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
            self.apply_keystroke_rule(raw_chars[i]);
        }
    }

    fn apply_keystroke_rule(&mut self, next_char: char) {
        let is_digit = next_char.is_ascii_digit();
        let is_telex_modifier = matches!(
            next_char.to_ascii_lowercase(),
            's' | 'f' | 'r' | 'x' | 'j' | 'a' | 'e' | 'o' | 'w' | 'd' | 'z'
        );

        if (self.current_method == InputMethod::Telex && is_digit)
            || (self.current_method == InputMethod::Vni && is_telex_modifier)
        {
            self.buffer.push(next_char);
            return;
        }

        match self.current_method {
            InputMethod::Telex => self.apply_telex_internal(next_char),
            InputMethod::Vni => self.apply_vni_internal(next_char),
        }
    }

    fn apply_telex_internal(&mut self, next_char: char) {
        // Snapshot the buffer
        let mut snapshot_data = ['\0'; 16];
        let len = self.buffer.len();
        snapshot_data[..len].copy_from_slice(self.buffer.as_slice());

        // Special case for standalone 'w' -> 'ư'
        if next_char == 'w' || next_char == ']' {
            let mut applied = false;
            if let Some(last) = self.buffer.last()
                && let Some(modified) = telex::apply_vowel_modifier(last, 'w')
            {
                self.buffer.replace_last(modified);
                applied = true;
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
            if has_tone {
                return;
            } else {
                self.buffer.push(next_char);
                return;
            }
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
                            let (other_base, _) =
                                telex::get_base_vowel_and_tone(self.buffer.as_slice()[i]);
                            self.buffer.replace_at(i, other_base);
                        }
                    }
                    self.buffer.replace_at(target_idx, new_char);
                }

                if current_tone == input_tone || tone_found_anywhere {
                    return;
                }

                if is_valid_vietnamese_syllable(self.buffer.as_slice()) {
                    return;
                } else {
                    self.buffer.clear();
                    for i in 0..len {
                        self.buffer.push(snapshot_data[i]);
                    }
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
                if next_char == expected_modifier
                    || (next_char == '[' && (base == 'ơ' || base == 'ư'))
                {
                    self.buffer
                        .replace_last(telex::add_tone(removed_base, tone));
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
                if cancelled {
                    return;
                }
                if is_valid_vietnamese_syllable(self.buffer.as_slice()) {
                    return;
                } else {
                    self.buffer.clear();
                    for i in 0..len {
                        self.buffer.push(snapshot_data[i]);
                    }
                }
            }
        } else {
            if next_char == 'w' || next_char == ']' {
                self.buffer.push('ư');
                if is_valid_vietnamese_syllable(self.buffer.as_slice()) {
                    return;
                } else {
                    self.buffer.clear();
                }
            }
        }

        self.buffer.push(next_char);
    }

    fn apply_vni_internal(&mut self, next_char: char) {
        let mut snapshot_data = ['\0'; 16];
        let len = self.buffer.len();
        snapshot_data[..len].copy_from_slice(self.buffer.as_slice());

        if !next_char.is_ascii_digit() {
            self.buffer.push(next_char);
            return;
        }

        if next_char == '0' {
            let mut changed = false;
            for i in 0..len {
                let (base, tone) = telex::get_base_vowel_and_tone(snapshot_data[i]);
                let new_base = match base {
                    '\u{0103}' | '\u{00e2}' => 'a',
                    '\u{00ea}' => 'e',
                    '\u{00f4}' | '\u{01a1}' => 'o',
                    '\u{01b0}' => 'u',
                    '\u{0111}' => 'd',
                    _ => base,
                };
                if new_base != snapshot_data[i] || tone != Tone::None {
                    self.buffer.replace_at(i, new_base);
                    changed = true;
                }
            }
            if !changed {
                self.buffer.push(next_char);
            }
            return;
        }

        let mut applied = false;
        match next_char {
            '1' | '2' | '3' | '4' | '5' => {
                let input_tone = match next_char {
                    '1' => Tone::Acute,
                    '2' => Tone::Grave,
                    '3' => Tone::Hook,
                    '4' => Tone::Tilde,
                    '5' => Tone::Underdot,
                    _ => Tone::None,
                };

                if let Some(target_idx) = telex::find_tone_target_index(self.buffer.as_slice()) {
                    let current_char = self.buffer.as_slice()[target_idx];
                    let (base, current_tone) = telex::get_base_vowel_and_tone(current_char);
                    let mut tone_found_anywhere = false;
                    for i in 0..len {
                        if telex::get_base_vowel_and_tone(self.buffer.as_slice()[i]).1 == input_tone
                        {
                            tone_found_anywhere = true;
                            break;
                        }
                    }

                    if current_tone == input_tone || tone_found_anywhere {
                        // In VNI, typing tone again does not cancel it, but since it's redundant it could just fallback to number typing or be swallowed.
                        // Based on the ticket, "a + 1 -> á. Then 1 again -> á1. Because á1 is not a valid Vietnamese word, the validator will reject the mutation and just append the literal 1".
                        // Therefore, we shouldn't consider this `applied = true`, so it falls back to pushing.
                        // Or we can apply cancellation and the validator will reject it? No, just do nothing so it falls back.
                    } else {
                        let new_char = telex::add_tone(base, input_tone);
                        for i in 0..self.buffer.len() {
                            if i != target_idx && telex::is_vowel(self.buffer.as_slice()[i]) {
                                let (other_base, _) =
                                    telex::get_base_vowel_and_tone(self.buffer.as_slice()[i]);
                                self.buffer.replace_at(i, other_base);
                            }
                        }
                        self.buffer.replace_at(target_idx, new_char);
                        applied = true;
                    }
                }
            }
            '6' => {
                for i in 0..len {
                    let (base, tone) = telex::get_base_vowel_and_tone(self.buffer.as_slice()[i]);
                    let new_base = match base {
                        'a' => '\u{00e2}',
                        'e' => '\u{00ea}',
                        'o' => '\u{00f4}',
                        _ => base,
                    };
                    if new_base != base {
                        self.buffer.replace_at(i, telex::add_tone(new_base, tone));
                        applied = true;
                    }
                }
            }
            '7' => {
                for i in 0..len {
                    let (base, tone) = telex::get_base_vowel_and_tone(self.buffer.as_slice()[i]);
                    let new_base = match base {
                        'o' => '\u{01a1}',
                        'u' => '\u{01b0}',
                        _ => base,
                    };
                    if new_base != base {
                        self.buffer.replace_at(i, telex::add_tone(new_base, tone));
                        applied = true;
                    }
                }
            }
            '8' => {
                for i in 0..len {
                    let (base, tone) = telex::get_base_vowel_and_tone(self.buffer.as_slice()[i]);
                    if base == 'a' {
                        self.buffer.replace_at(i, telex::add_tone('\u{0103}', tone));
                        applied = true;
                    }
                }
            }
            '9' => {
                for i in 0..len {
                    let (base, tone) = telex::get_base_vowel_and_tone(self.buffer.as_slice()[i]);
                    if base == 'd' && tone == Tone::None {
                        self.buffer.replace_at(i, '\u{0111}');
                        applied = true;
                    }
                }
            }
            _ => {}
        }

        if applied {
            if is_valid_vietnamese_syllable(self.buffer.as_slice()) {
                return;
            } else {
                self.buffer.clear();
                for i in 0..len {
                    self.buffer.push(snapshot_data[i]);
                }
            }
        }

        self.buffer.push(next_char);
    }
}
