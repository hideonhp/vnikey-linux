use crate::buffer::CharBuffer;
use crate::telex::{self, Tone, fast_lower};
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
    CommitAndPassThrough(CharBuffer),
    PassThrough,
    SurroundingRecompose {
        preedit: CharBuffer,
        delete_count: usize,
        delete_byte_len: usize,
    },
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
    pub spell_check: bool,
    pub last_committed_raw: CharBuffer,
    pub last_committed_text: CharBuffer,
    /// Fallback buffer saved when Smart W (uo→ươ) is applied.
    /// Contains buffer state with ONLY o→ơ applied (not u→ư).
    /// Used to retry if a subsequent tone key makes the Smart W result invalid.
    /// Cleared on commit (reset/reset_context) or when a plain char is pushed.
    /// NOT cleared when modifier path spell check succeeds.
    uo_smart_fallback: Option<CharBuffer>,
    /// Set to true when '@' is typed. While true and engine is Idle,
    /// all alphabetic/digit input is passed through (no composition).
    /// Cleared when Space, newline, or backspace reaches Idle state.
    pass_through_until_space: bool,
}

impl Default for Engine {
    fn default() -> Self {
        Self::new(InputMethod::Telex, true)
    }
}

impl Engine {
    pub fn new(method: InputMethod, spell_check: bool) -> Self {
        Self {
            current_method: method,
            state: State::Idle,
            buffer: CharBuffer::new(),
            raw_buffer: CharBuffer::new(),
            spell_check,
            last_committed_raw: CharBuffer::new(),
            last_committed_text: CharBuffer::new(),
            uo_smart_fallback: None,
            pass_through_until_space: false,
        }
    }

    pub fn get_input_method(&self) -> InputMethod {
        self.current_method
    }

    pub fn set_input_method(&mut self, method: InputMethod) -> Option<Action> {
        self.last_committed_raw.clear();
        self.last_committed_text.clear();
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
            '\x08' | '\x7f' => {
                self.pass_through_until_space = false;
                self.handle_backspace()
            }
            ' ' | '\n' | '\r' => {
                self.pass_through_until_space = false;
                self.handle_commit(key)
            }
            c if c.is_ascii_alphabetic() || c.is_ascii_digit() => {
                // When pass_through_until_space is active (e.g. after '@'), skip
                // composition entirely so email domain/local-part chars are not
                // treated as Telex/VNI modifiers.
                if self.pass_through_until_space && self.state == State::Idle {
                    self.last_committed_raw.clear();
                    self.last_committed_text.clear();
                    return Action::PassThrough;
                }
                self.handle_char(c)
            }
            _ => {
                if self.state == State::Composing {
                    let commit_action = Action::CommitAndPassThrough(self.buffer);
                    self.reset();
                    // '@' triggers pass-through mode for the rest of the token
                    if key == '@' {
                        self.pass_through_until_space = true;
                    }
                    commit_action
                } else {
                    // Idle: '@' or '.' after a committed token → keep/set flag
                    if key == '@' {
                        self.pass_through_until_space = true;
                    }
                    self.last_committed_raw.clear();
                    self.last_committed_text.clear();
                    Action::PassThrough
                }
            }
        }
    }

    fn handle_backspace(&mut self) -> Action {
        if self.state == State::Idle {
            if !self.last_committed_raw.is_empty() {
                let delete_count = self.last_committed_text.len();
                let delete_byte_len: usize = self
                    .last_committed_text
                    .as_slice()
                    .iter()
                    .map(|c| c.len_utf8())
                    .sum();

                self.last_committed_raw.pop();

                if self.last_committed_raw.is_empty() {
                    self.last_committed_text.clear();
                    return Action::SurroundingRecompose {
                        preedit: CharBuffer::new(),
                        delete_count,
                        delete_byte_len,
                    };
                }

                self.state = State::Composing;
                self.raw_buffer = self.last_committed_raw;
                self.last_committed_raw.clear();
                self.last_committed_text.clear();

                self.rebuild_buffer();

                return Action::SurroundingRecompose {
                    preedit: self.buffer,
                    delete_count,
                    delete_byte_len,
                };
            }
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

        self.last_committed_raw = self.raw_buffer;

        if !self.buffer.is_full() {
            self.buffer.push(trigger_key);
        }

        self.last_committed_text = self.buffer;

        let action = Action::Commit(self.buffer);

        self.reset();

        action
    }

    pub fn handle_char(&mut self, c: char) -> Action {
        if self.state == State::Idle {
            self.state = State::Composing;
            self.last_committed_raw.clear();
            self.last_committed_text.clear();
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
        self.uo_smart_fallback = None;
    }

    pub fn reset_context(&mut self) {
        self.state = State::Idle;
        self.buffer.clear();
        self.raw_buffer.clear();
        self.last_committed_raw.clear();
        self.last_committed_text.clear();
        self.uo_smart_fallback = None;
    }

    /// Commit current preedit buffer without appending any trigger key.
    /// Used when a non-character key (arrow, Home, End, etc.) is pressed.
    pub fn flush(&mut self) -> Option<Action> {
        if self.state == State::Composing {
            let action = Action::Commit(self.buffer);
            self.reset();
            // Clear surrounding text because cursor position has changed
            self.last_committed_raw.clear();
            self.last_committed_text.clear();
            Some(action)
        } else {
            self.last_committed_raw.clear();
            self.last_committed_text.clear();
            None
        }
    }

    /// Dynamically shifts the tone to the correct vowel in the word based on Vietnamese rules.
    fn apply_dynamic_tone_shifting(&mut self) {
        // Find the current tone anywhere in the buffer
        let current_tone_in_word = self
            .buffer
            .as_slice()
            .iter()
            .find_map(|&ch| {
                let (_, tone) = telex::get_base_vowel_and_tone(ch);
                if tone != Tone::None { Some(tone) } else { None }
            })
            .unwrap_or(Tone::None);

        if current_tone_in_word == Tone::None {
            return;
        }

        // Strip all tones, find target, re-apply
        let mut temp_buffer = self.buffer;
        for j in 0..temp_buffer.len() {
            let (base, _) = telex::get_base_vowel_and_tone(temp_buffer.as_slice()[j]);
            temp_buffer.replace_at(j, base);
        }
        if let Some(target_idx) = telex::find_tone_target_index(temp_buffer.as_slice()) {
            let target_char = temp_buffer.as_slice()[target_idx];
            let new_char = telex::add_tone(target_char, current_tone_in_word);
            temp_buffer.replace_at(target_idx, new_char);
            self.buffer = temp_buffer;
        }
    }

    fn rebuild_buffer(&mut self) {
        self.buffer.clear();
        let (raw_chars, len) = self.raw_buffer.snapshot();

        let mut fallback_to_raw = false;

        self.raw_buffer.clear();

        for &ch in &raw_chars[..len] {
            self.raw_buffer.push(ch);
            self.apply_keystroke_rule(ch);
            self.apply_dynamic_tone_shifting();

            if self.spell_check && !is_valid_vietnamese_syllable(self.buffer.as_slice()) {
                fallback_to_raw = true;
                break;
            }
        }

        if fallback_to_raw {
            self.buffer.clear();
            self.raw_buffer.clear();
            for &ch in &raw_chars[..len] {
                self.buffer.push(ch);
                self.raw_buffer.push(ch);
            }
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

    // -------------------------------------------------------------------------
    // Shared helpers
    // -------------------------------------------------------------------------

    /// Strip tones from all vowels in `buf`, then apply `tone` to the tone-target vowel.
    /// Returns `true` if the operation produced a valid syllable.
    fn apply_tone_to_buffer(buf: &mut CharBuffer, tone: Tone) -> bool {
        for i in 0..buf.len() {
            let (base, _) = telex::get_base_vowel_and_tone(buf.as_slice()[i]);
            if telex::is_vowel(buf.as_slice()[i]) {
                buf.replace_at(i, base);
            }
        }
        if let Some(target_idx) = telex::find_tone_target_index(buf.as_slice()) {
            let base = telex::get_base_vowel_and_tone(buf.as_slice()[target_idx]).0;
            let new_char = telex::add_tone(base, tone);
            buf.replace_at(target_idx, new_char);
            true
        } else {
            false
        }
    }

    /// Try to apply the Smart W fallback buffer with `tone`.
    /// Returns `Some(buf)` if the result is a valid Vietnamese syllable.
    fn try_smart_w_fallback(&mut self, tone: Tone) -> Option<CharBuffer> {
        let mut fallback_buf = self.uo_smart_fallback.take()?;
        Self::apply_tone_to_buffer(&mut fallback_buf, tone);
        if is_valid_vietnamese_syllable(fallback_buf.as_slice()) {
            Some(fallback_buf)
        } else {
            None
        }
    }

    /// Remove the previous occurrence of a raw char matching `tone` from raw_buffer.
    /// Used to implement "double-press cancellation" for tone keys.
    fn cancel_raw_tone_char(&mut self, tone: Tone) {
        let r_len = self.raw_buffer.len();
        for j in (0..r_len.saturating_sub(1)).rev() {
            let rc = self.raw_buffer.as_slice()[j];
            if telex::Tone::from_char(fast_lower(rc)) == Some(tone) {
                self.raw_buffer.remove(j);
                break;
            }
        }
    }

    /// Remove the previous occurrence of a literal digit char from raw_buffer (VNI tone cancel).
    fn cancel_raw_digit_char(&mut self, digit: char) {
        let r_len = self.raw_buffer.len();
        for j in (0..r_len.saturating_sub(1)).rev() {
            if self.raw_buffer.as_slice()[j] == digit {
                self.raw_buffer.remove(j);
                break;
            }
        }
    }

    // -------------------------------------------------------------------------
    // Telex implementation
    // -------------------------------------------------------------------------

    fn apply_telex_internal(&mut self, next_char: char) {
        let next_char_lower = fast_lower(next_char);
        let (snapshot_data, len) = self.buffer.snapshot();

        // --- Handle 'z': strip all tones, or push literal if no tone found ---
        if next_char_lower == 'z' {
            let mut has_tone = false;
            for (i, &ch) in snapshot_data[..len].iter().enumerate() {
                let (base, tone) = telex::get_base_vowel_and_tone(ch);
                if tone != Tone::None {
                    has_tone = true;
                    self.buffer.replace_at(i, base);
                }
            }
            if !has_tone {
                self.buffer.push(next_char);
            }
            return;
        }

        // --- Handle tone keys: s/f/r/x/j ---
        if let Some(input_tone) = Tone::from_char(next_char_lower)
            && self.try_apply_tone_telex(next_char, input_tone, &snapshot_data, len)
        {
            return;
        }

        // --- Handle vowel modifiers and Smart W ---
        if let Some(last_char) = self.buffer.last()
            && self.try_apply_modifier_telex(
                next_char,
                next_char_lower,
                last_char,
                &snapshot_data,
                len,
            )
        {
            return;
        }

        // Plain character: fallback stale, just push
        self.uo_smart_fallback = None;
        self.buffer.push(next_char);
    }

    /// Try to apply a tone key (s/f/r/x/j) in Telex mode.
    /// Returns `true` if the tone was handled (no further processing needed).
    fn try_apply_tone_telex(
        &mut self,
        next_char: char,
        input_tone: Tone,
        snapshot_data: &[char; CharBuffer::MAX_CAPACITY],
        len: usize,
    ) -> bool {
        // Check if the same tone already exists anywhere in the buffer
        let tone_already_exists = (0..len)
            .any(|i| telex::get_base_vowel_and_tone(self.buffer.as_slice()[i]).1 == input_tone);

        if let Some(target_idx) = telex::find_tone_target_index(self.buffer.as_slice()) {
            let current_char = self.buffer.as_slice()[target_idx];
            let (base, current_tone) = telex::get_base_vowel_and_tone(current_char);

            if current_tone == input_tone || tone_already_exists {
                // Double-press: cancel the tone → strip it and push the literal key
                for i in 0..self.buffer.len() {
                    let (b, t) = telex::get_base_vowel_and_tone(self.buffer.as_slice()[i]);
                    if t == input_tone {
                        self.buffer.replace_at(i, b);
                    }
                }
                self.buffer.push(next_char);
                self.cancel_raw_tone_char(input_tone);
                return true;
            }

            // Apply new tone to target vowel, strip from all others
            let new_char = telex::add_tone(base, input_tone);
            for i in 0..self.buffer.len() {
                if i != target_idx && telex::is_vowel(self.buffer.as_slice()[i]) {
                    let (other_base, _) = telex::get_base_vowel_and_tone(self.buffer.as_slice()[i]);
                    self.buffer.replace_at(i, other_base);
                }
            }
            self.buffer.replace_at(target_idx, new_char);

            if is_valid_vietnamese_syllable(self.buffer.as_slice()) {
                // ⚠️ Do NOT clear uo_smart_fallback here — it survives for the next tone key
                return true;
            }

            // Spell check failed — try Smart W fallback
            if let Some(fallback) = self.try_smart_w_fallback(input_tone) {
                self.buffer = fallback;
                return true;
            }

            // Complete rollback
            self.buffer.restore(snapshot_data, len);
            // Fall through to vowel modifier / plain char handling
        }
        false
    }

    /// Try to apply a vowel modifier (aa/ee/oo/ow/uw/dd/Smart W) in Telex mode.
    /// Returns `true` if the modifier was handled (no further processing needed).
    fn try_apply_modifier_telex(
        &mut self,
        next_char: char,
        next_char_lower: char,
        last_char: char,
        snapshot_data: &[char; CharBuffer::MAX_CAPACITY],
        len: usize,
    ) -> bool {
        let mut applied = false;
        let mut cancelled = false;

        // Smart W look-back: uo → ươ, ua → ưa, uu → ưu
        if next_char_lower == 'w' && self.buffer.len() >= 2 {
            let buf_len = self.buffer.len();
            let second_last = self.buffer.as_slice()[buf_len - 2];
            let (second_last_base, second_last_tone) = telex::get_base_vowel_and_tone(second_last);
            let sbl = fast_lower(second_last_base);
            let (last_base, last_tone) = telex::get_base_vowel_and_tone(last_char);
            let ll = fast_lower(last_base);

            if sbl == 'u' && (ll == 'o' || ll == 'a' || ll == 'u') {
                // Skip if 'q' precedes (e.g. "quo" should not Smart-W)
                let is_q_exception =
                    buf_len >= 3 && fast_lower(self.buffer.as_slice()[buf_len - 3]) == 'q';

                if !is_q_exception {
                    if ll == 'o' {
                        // Save fallback BEFORE transforming: buffer still has 'u' + 'o'
                        let mut fallback = self.buffer;
                        let fallback_o = telex::add_tone(
                            if last_char.is_uppercase() { 'Ơ' } else { 'ơ' },
                            last_tone,
                        );
                        fallback.replace_last(fallback_o); // only o→ơ, 'u' stays
                        self.uo_smart_fallback = Some(fallback);

                        // Transform: uo → ươ
                        let new_u = telex::add_tone(
                            if second_last.is_uppercase() {
                                'Ư'
                            } else {
                                'ư'
                            },
                            second_last_tone,
                        );
                        self.buffer.replace_at(buf_len - 2, new_u);
                        let new_o = telex::add_tone(
                            if last_char.is_uppercase() { 'Ơ' } else { 'ơ' },
                            last_tone,
                        );
                        self.buffer.replace_last(new_o);
                        applied = true;
                    } else if ll == 'a' {
                        // ua → ưa
                        let new_u = telex::add_tone(
                            if second_last.is_uppercase() {
                                'Ư'
                            } else {
                                'ư'
                            },
                            second_last_tone,
                        );
                        self.buffer.replace_at(buf_len - 2, new_u);
                        applied = true;
                    } else {
                        // uu + w → ưu: only transform second_last u → ư
                        let new_u = telex::add_tone(
                            if second_last.is_uppercase() {
                                'Ư'
                            } else {
                                'ư'
                            },
                            second_last_tone,
                        );
                        self.buffer.replace_at(buf_len - 2, new_u);
                        // ⚠️ Do NOT replace last; do NOT save uo_smart_fallback for this case
                        applied = true;
                    }
                }
            }
        }

        if !applied {
            let removed_opt = telex::remove_vowel_modifier(last_char, next_char_lower);
            if let Some(removed) = removed_opt {
                self.buffer.replace_last(removed);
                self.buffer.push(next_char);
                applied = true;
                cancelled = true;

                // Remove the original modifier char from raw_buffer
                let r_len = self.raw_buffer.len();
                for j in (0..r_len.saturating_sub(1)).rev() {
                    let rc = self.raw_buffer.as_slice()[j];
                    if fast_lower(rc) == next_char_lower {
                        self.raw_buffer.remove(j);
                        break;
                    }
                }
            }
        }

        if !applied && fast_lower(last_char) == 'đ' && next_char_lower == 'd' {
            self.buffer
                .replace_last(if last_char.is_uppercase() { 'D' } else { 'd' });
            self.buffer.push('d');
            applied = true;
            cancelled = true;
        }

        if !applied {
            if let Some(modified) = telex::apply_vowel_modifier(last_char, next_char_lower) {
                self.buffer.replace_last(modified);
                applied = true;
            } else if fast_lower(last_char) == 'd' && next_char_lower == 'd' {
                self.buffer
                    .replace_last(if last_char.is_uppercase() { 'Đ' } else { 'đ' });
                applied = true;
            } else if next_char_lower == 'w' {
                self.buffer
                    .push(if next_char.is_uppercase() { 'Ư' } else { 'ư' });
                applied = true;
            }
        }

        if applied {
            if cancelled {
                return true;
            }

            self.apply_dynamic_tone_shifting();

            if is_valid_vietnamese_syllable(self.buffer.as_slice()) {
                return true;
            }

            // Rollback to snapshot
            self.buffer.restore(snapshot_data, len);
            return false; // signal "not handled usefully" — caller will push plain char
        }

        false
    }

    // -------------------------------------------------------------------------
    // VNI implementation
    // -------------------------------------------------------------------------

    fn apply_vni_internal(&mut self, next_char: char) {
        let (snapshot_data, len) = self.buffer.snapshot();

        if !next_char.is_ascii_digit() {
            self.buffer.push(next_char);
            return;
        }

        // '0': reset all modifiers and tones
        if next_char == '0' {
            let mut changed = false;
            for (i, &ch) in snapshot_data[..len].iter().enumerate() {
                let (base, tone) = telex::get_base_vowel_and_tone(ch);
                let new_base = match base {
                    'ă' | 'â' => 'a',
                    'Ă' | 'Â' => 'A',
                    'ê' => 'e',
                    'Ê' => 'E',
                    'ô' | 'ơ' => 'o',
                    'Ô' | 'Ơ' => 'O',
                    'ư' => 'u',
                    'Ư' => 'U',
                    'đ' => 'd',
                    'Đ' => 'D',
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
                applied = self.try_apply_tone_vni(next_char, &snapshot_data, len);
            }
            '6' => {
                applied = self.try_apply_circumflex_vni(&snapshot_data, len);
            }
            '7' => {
                applied = self.try_apply_horn_vni(&snapshot_data, len);
            }
            '8' => {
                applied = self.try_apply_breve_vni(&snapshot_data, len);
            }
            '9' => {
                applied = self.try_apply_stroke_vni(&snapshot_data, len);
            }
            _ => {}
        }

        if applied {
            if is_valid_vietnamese_syllable(self.buffer.as_slice()) {
                return;
            }

            // Try Smart W fallback only for tone keys (1-5)
            if matches!(next_char, '1' | '2' | '3' | '4' | '5') {
                let input_tone = vni_digit_to_tone(next_char);
                if let Some(fallback) = self.try_smart_w_fallback(input_tone) {
                    self.buffer = fallback;
                    return;
                }
            }

            // Rollback
            self.buffer.restore(&snapshot_data, len);
        }

        // Not applied or rollback: push literal digit
        self.uo_smart_fallback = None;
        self.buffer.push(next_char);
    }

    /// Try to apply a tone (digits 1-5) in VNI mode.
    /// Returns `true` if the tone was applied to the buffer (may still fail spell check).
    fn try_apply_tone_vni(
        &mut self,
        digit: char,
        snapshot_data: &[char; CharBuffer::MAX_CAPACITY],
        len: usize,
    ) -> bool {
        let input_tone = vni_digit_to_tone(digit);

        let tone_already_exists = (0..len)
            .any(|i| telex::get_base_vowel_and_tone(self.buffer.as_slice()[i]).1 == input_tone);

        if let Some(target_idx) = telex::find_tone_target_index(self.buffer.as_slice()) {
            let current_char = self.buffer.as_slice()[target_idx];
            let (base, current_tone) = telex::get_base_vowel_and_tone(current_char);

            if current_tone == input_tone || tone_already_exists {
                // Double-press: cancel tone → strip it and push literal digit
                for i in 0..self.buffer.len() {
                    let (b, t) = telex::get_base_vowel_and_tone(self.buffer.as_slice()[i]);
                    if t == input_tone {
                        self.buffer.replace_at(i, b);
                    }
                }
                self.cancel_raw_digit_char(digit);
                return false; // treat as "push literal", not a real tone application
            }

            // Apply new tone
            let new_char = telex::add_tone(base, input_tone);
            for i in 0..self.buffer.len() {
                if i != target_idx && telex::is_vowel(self.buffer.as_slice()[i]) {
                    let (other_base, _) = telex::get_base_vowel_and_tone(self.buffer.as_slice()[i]);
                    self.buffer.replace_at(i, other_base);
                }
            }
            self.buffer.replace_at(target_idx, new_char);
            return true;
        }

        // No target vowel found — rollback and push literal
        self.buffer.restore(snapshot_data, len);
        false
    }

    /// Try to apply circumflex (digit 6) modifier: a→â, e→ê, o→ô.
    fn try_apply_circumflex_vni(
        &mut self,
        _snapshot_data: &[char; CharBuffer::MAX_CAPACITY],
        len: usize,
    ) -> bool {
        let mut applied = false;
        for i in 0..len {
            let (base, tone) = telex::get_base_vowel_and_tone(self.buffer.as_slice()[i]);
            let new_base = match fast_lower(base) {
                'a' => {
                    if base.is_uppercase() {
                        'Â'
                    } else {
                        'â'
                    }
                }
                'e' => {
                    if base.is_uppercase() {
                        'Ê'
                    } else {
                        'ê'
                    }
                }
                'o' => {
                    if base.is_uppercase() {
                        'Ô'
                    } else {
                        'ô'
                    }
                }
                _ => base,
            };
            if new_base != base {
                self.buffer.replace_at(i, telex::add_tone(new_base, tone));
                applied = true;
            }
        }
        applied
    }

    /// Try to apply horn modifier (digit 7): o→ơ, u→ư. Also handles uo→ươ Smart W.
    fn try_apply_horn_vni(
        &mut self,
        _snapshot_data: &[char; CharBuffer::MAX_CAPACITY],
        len: usize,
    ) -> bool {
        // Check for uo → Smart W fallback opportunity
        if len >= 2 {
            let second_last_base =
                fast_lower(telex::get_base_vowel_and_tone(self.buffer.as_slice()[len - 2]).0);
            let last_base =
                fast_lower(telex::get_base_vowel_and_tone(self.buffer.as_slice()[len - 1]).0);
            if second_last_base == 'u' && last_base == 'o' {
                let last_char = self.buffer.as_slice()[len - 1];
                let last_tone = telex::get_base_vowel_and_tone(last_char).1;
                let mut fallback = self.buffer;
                let fallback_o =
                    telex::add_tone(if last_char.is_uppercase() { 'Ơ' } else { 'ơ' }, last_tone);
                fallback.replace_last(fallback_o);
                self.uo_smart_fallback = Some(fallback);
            }
        }

        let mut applied = false;
        for i in 0..len {
            let (base, tone) = telex::get_base_vowel_and_tone(self.buffer.as_slice()[i]);
            let new_base = match fast_lower(base) {
                'o' => {
                    if base.is_uppercase() {
                        'Ơ'
                    } else {
                        'ơ'
                    }
                }
                'u' => {
                    if base.is_uppercase() {
                        'Ư'
                    } else {
                        'ư'
                    }
                }
                _ => base,
            };
            if new_base != base {
                self.buffer.replace_at(i, telex::add_tone(new_base, tone));
                applied = true;
            }
        }
        applied
    }

    /// Try to apply breve modifier (digit 8): a→ă.
    fn try_apply_breve_vni(
        &mut self,
        _snapshot_data: &[char; CharBuffer::MAX_CAPACITY],
        len: usize,
    ) -> bool {
        let mut applied = false;
        for i in 0..len {
            let (base, tone) = telex::get_base_vowel_and_tone(self.buffer.as_slice()[i]);
            if fast_lower(base) == 'a' {
                self.buffer.replace_at(i, telex::add_tone('ă', tone));
                applied = true;
            }
        }
        applied
    }

    /// Try to apply stroke modifier (digit 9): d→đ.
    fn try_apply_stroke_vni(
        &mut self,
        _snapshot_data: &[char; CharBuffer::MAX_CAPACITY],
        len: usize,
    ) -> bool {
        let mut applied = false;
        for i in 0..len {
            let (base, tone) = telex::get_base_vowel_and_tone(self.buffer.as_slice()[i]);
            if fast_lower(base) == 'd' && tone == Tone::None {
                self.buffer.replace_at(i, 'đ');
                applied = true;
            }
        }
        applied
    }
}

/// Map a VNI digit ('1'–'5') to its corresponding `Tone`.
#[inline]
fn vni_digit_to_tone(digit: char) -> Tone {
    match digit {
        '1' => Tone::Acute,
        '2' => Tone::Grave,
        '3' => Tone::Hook,
        '4' => Tone::Tilde,
        '5' => Tone::Underdot,
        _ => Tone::None,
    }
}

#[cfg(test)]
mod boundary_tests {
    use super::*;
    use crate::test_utils::make_buffer;

    #[test]
    fn test_reset_context() {
        let mut engine = Engine::new(InputMethod::Telex, false);

        // Type 'a'
        engine.process_key('a');
        assert_eq!(engine.state, State::Composing);
        assert!(!engine.buffer.is_empty());
        assert!(!engine.raw_buffer.is_empty());

        // Type Space to commit 'a'
        engine.process_key(' ');
        assert_eq!(engine.state, State::Idle);
        assert!(!engine.last_committed_raw.is_empty());
        assert!(!engine.last_committed_text.is_empty());

        // Type 'b' to enter composing again
        engine.process_key('b');
        assert_eq!(engine.state, State::Composing);
        assert!(!engine.buffer.is_empty());
        assert!(!engine.raw_buffer.is_empty());

        // Set fallback just to test it
        engine.uo_smart_fallback = Some(CharBuffer::new());

        engine.reset_context();

        assert_eq!(engine.state, State::Idle);
        assert!(engine.buffer.is_empty());
        assert!(engine.raw_buffer.is_empty());
        assert!(engine.last_committed_raw.is_empty());
        assert!(engine.last_committed_text.is_empty());
        assert!(engine.uo_smart_fallback.is_none());
    }

    #[test]
    fn test_buffer_full_forces_commit() {
        let mut engine = Engine::new(InputMethod::Telex, false);

        for _ in 0..(CharBuffer::MAX_CAPACITY - 1) {
            engine.process_key('b'); // use 'b' to avoid 'a' + 'a' -> 'â' modifier collapse
        }

        // Pushing the last one
        let action = engine.process_key('b');
        assert_eq!(
            action,
            Action::Preedit(make_buffer(&"b".repeat(CharBuffer::MAX_CAPACITY)))
        );

        // Overflow
        let action2 = engine.process_key('c');
        assert_eq!(
            action2,
            Action::Commit(make_buffer(&"b".repeat(CharBuffer::MAX_CAPACITY)))
        );
        assert_eq!(engine.state, State::Composing);
        assert_eq!(engine.buffer.as_slice(), make_buffer("c").as_slice());
    }

    #[test]
    fn test_set_input_method_commits_active_preedit() {
        let mut engine = Engine::new(InputMethod::Telex, false);
        engine.process_key('v');
        engine.process_key('n');
        engine.process_key('i');

        let action = engine.set_input_method(InputMethod::Vni);
        assert_eq!(action, Some(Action::Commit(make_buffer("vni"))));
        assert_eq!(engine.state, State::Idle);
    }

    #[test]
    fn test_set_input_method_idle() {
        let mut engine = Engine::new(InputMethod::Telex, false);
        let action = engine.set_input_method(InputMethod::Vni);
        assert_eq!(action, None);
        assert_eq!(engine.get_input_method(), InputMethod::Vni);
    }

    #[test]
    fn test_backspace_on_idle() {
        let mut engine = Engine::new(InputMethod::Telex, false);
        let action = engine.process_key('\x08');
        assert_eq!(action, Action::PassThrough);
    }

    #[test]
    fn test_backspace_to_idle() {
        let mut engine = Engine::new(InputMethod::Telex, false);
        engine.process_key('a');
        let action = engine.process_key('\x08');
        assert_eq!(action, Action::Preedit(CharBuffer::new()));
        assert_eq!(engine.state, State::Idle);
    }

    #[test]
    fn test_commit_key_on_idle() {
        let mut engine = Engine::new(InputMethod::Telex, false);
        let action = engine.process_key(' ');
        assert_eq!(action, Action::PassThrough);
    }

    #[test]
    fn test_commit_key_with_preedit() {
        let mut engine = Engine::new(InputMethod::Telex, false);
        engine.process_key('a');
        let action = engine.process_key(' ');
        assert_eq!(action, Action::Commit(make_buffer("a ")));
        assert_eq!(engine.state, State::Idle);
    }

    #[test]
    fn test_commit_key_when_buffer_full() {
        let mut engine = Engine::new(InputMethod::Telex, false);
        for _ in 0..CharBuffer::MAX_CAPACITY {
            engine.process_key('b');
        }
        let action = engine.process_key(' ');
        assert_eq!(
            action,
            Action::Commit(make_buffer(&"b".repeat(CharBuffer::MAX_CAPACITY)))
        );
    }

    #[test]
    fn test_unmapped_key_on_idle() {
        let mut engine = Engine::new(InputMethod::Telex, false);
        let action = engine.process_key(',');
        assert_eq!(action, Action::PassThrough);
    }

    #[test]
    fn test_unmapped_key_with_preedit() {
        let mut engine = Engine::new(InputMethod::Telex, false);
        engine.process_key('a');
        let action = engine.process_key(',');
        assert_eq!(action, Action::CommitAndPassThrough(make_buffer("a")));
        assert_eq!(engine.state, State::Idle);
    }

    #[test]
    fn test_telex_digit() {
        let mut engine = Engine::new(InputMethod::Telex, false);
        engine.process_key('a');
        let action = engine.process_key('1');
        assert_eq!(action, Action::Preedit(make_buffer("a1")));
    }

    #[test]
    fn test_vni_modifier() {
        let mut engine = Engine::new(InputMethod::Vni, false);
        engine.process_key('a');
        let action = engine.process_key('s'); // 's' is telex tone
        assert_eq!(action, Action::Preedit(make_buffer("as"))); // in VNI it just adds 's'
    }

    #[test]
    fn test_vni_number_tone_validation_reject() {
        let mut engine = Engine::new(InputMethod::Vni, false);
        engine.process_key('a');
        engine.process_key('1');
        // 'á', now try adding tone again
        let action = engine.process_key('1');
        // should cancel tone and push '1' literal
        assert_eq!(action, Action::Preedit(make_buffer("a1")));
    }

    #[test]
    fn test_vni_zero_reset_tone() {
        let mut engine = Engine::new(InputMethod::Vni, false);
        engine.process_key('a');
        engine.process_key('1');
        assert_eq!(engine.buffer.as_slice(), make_buffer("á").as_slice());
        engine.process_key('0');
        assert_eq!(engine.buffer.as_slice(), make_buffer("a").as_slice());
    }

    #[test]
    fn test_vni_zero_reset_vowel() {
        let mut engine = Engine::new(InputMethod::Vni, false);
        engine.process_key('a');
        engine.process_key('6');
        assert_eq!(engine.buffer.as_slice(), make_buffer("â").as_slice());
        engine.process_key('0');
        assert_eq!(engine.buffer.as_slice(), make_buffer("a").as_slice());
    }

    #[test]
    fn test_telex_vowel_w_alone() {
        let mut engine = Engine::new(InputMethod::Telex, false);
        let action = engine.process_key('w');
        assert_eq!(action, Action::Preedit(make_buffer("w")));
    }

    #[test]
    fn test_telex_dd() {
        let mut engine = Engine::new(InputMethod::Telex, false);
        engine.process_key('d');
        let action = engine.process_key('d');
        assert_eq!(action, Action::Preedit(make_buffer("đ")));
    }

    #[test]
    fn test_telex_ddd() {
        let mut engine = Engine::new(InputMethod::Telex, false);
        engine.process_key('d');
        engine.process_key('d');
        let action = engine.process_key('d');
        assert_eq!(action, Action::Preedit(make_buffer("dd")));
    }

    #[test]
    fn test_telex_dddd() {
        let mut engine = Engine::new(InputMethod::Telex, false);
        engine.process_key('d');
        engine.process_key('d');
        engine.process_key('d');
        let action = engine.process_key('d');
        assert_eq!(action, Action::Preedit(make_buffer("ddd")));
    }

    #[test]
    fn test_vni_six_modifier() {
        let mut engine = Engine::new(InputMethod::Vni, false);
        engine.process_key('o');
        let action = engine.process_key('6');
        assert_eq!(action, Action::Preedit(make_buffer("ô")));
    }

    #[test]
    fn test_vni_seven_modifier() {
        let mut engine = Engine::new(InputMethod::Vni, false);
        engine.process_key('u');
        let action = engine.process_key('7');
        assert_eq!(action, Action::Preedit(make_buffer("ư")));
    }

    #[test]
    fn test_vni_eight_modifier() {
        let mut engine = Engine::new(InputMethod::Vni, false);
        engine.process_key('a');
        let action = engine.process_key('8');
        assert_eq!(action, Action::Preedit(make_buffer("ă")));
    }

    #[test]
    fn test_vni_nine_modifier() {
        let mut engine = Engine::new(InputMethod::Vni, false);
        engine.process_key('d');
        let action = engine.process_key('9');
        assert_eq!(action, Action::Preedit(make_buffer("đ")));
    }

    #[test]
    fn test_handle_char() {
        let mut engine = Engine::new(InputMethod::Telex, false);
        assert_eq!(engine.state, State::Idle);

        // First char
        let action = engine.handle_char('a');
        assert_eq!(action, Action::Preedit(make_buffer("a")));
        assert_eq!(engine.state, State::Composing);

        // Max capacity
        for _ in 0..15 {
            engine.handle_char('b');
        }

        let action = engine.handle_char('c');
        assert_eq!(action, Action::Commit(make_buffer("abbbbbbbbbbbbbbb")));
        assert_eq!(engine.state, State::Composing);
        assert_eq!(engine.buffer.as_slice(), &['c']);
    }

    #[test]
    fn test_flush_when_composing() {
        let mut engine = Engine::new(InputMethod::Telex, false);

        engine.process_key('h');
        engine.process_key('e');
        engine.process_key('l');
        engine.process_key('l');
        engine.process_key('o');

        assert_eq!(engine.state, State::Composing);

        let flush_action = engine.flush();

        assert_eq!(flush_action, Some(Action::Commit(make_buffer("hello"))));
        assert_eq!(engine.state, State::Idle);
        assert_eq!(engine.buffer.len(), 0);
        assert_eq!(engine.last_committed_raw.len(), 0);
        assert_eq!(engine.last_committed_text.len(), 0);
    }

    #[test]
    fn test_flush_when_idle() {
        let mut engine = Engine::new(InputMethod::Telex, false);

        engine.process_key('h');
        engine.process_key('i');
        engine.process_key(' ');

        assert_eq!(engine.state, State::Idle);
        assert!(engine.last_committed_raw.len() > 0);
        assert!(engine.last_committed_text.len() > 0);

        let flush_action = engine.flush();

        assert_eq!(flush_action, None);
        assert_eq!(engine.last_committed_raw.len(), 0);
        assert_eq!(engine.last_committed_text.len(), 0);
    }
}
