use crate::buffer::CharBuffer;
use crate::telex::{self, Tone};
use crate::validation::is_valid_vietnamese_syllable;

/// Fast lowercase conversion — avoids iterator overhead for ASCII chars.
/// All Telex/VNI modifier keys are ASCII, so this is always the fast path in practice.
#[inline]
fn fast_lower(c: char) -> char {
    if c.is_ascii() {
        c.to_ascii_lowercase()
    } else {
        c.to_lowercase().next().unwrap_or(c)
    }
}

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
            '\x08' | '\x7f' => self.handle_backspace(),
            ' ' | '\n' | '\r' => self.handle_commit(key),
            c if c.is_ascii_alphabetic() || c.is_ascii_digit() => self.handle_char(c),
            _ => {
                if self.state == State::Composing {
                    let commit_action = Action::CommitAndPassThrough(self.buffer);
                    self.reset();
                    commit_action
                } else {
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

    fn handle_char(&mut self, c: char) -> Action {
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

    fn rebuild_buffer(&mut self) {
        self.buffer.clear();
        let (raw_chars, len) = self.raw_buffer.snapshot();

        let mut fallback_to_raw = false;

        self.raw_buffer.clear();

        for i in 0..len {
            self.raw_buffer.push(raw_chars[i]);
            self.apply_keystroke_rule(raw_chars[i]);

            // --- DYNAMIC TONE SHIFTING ---
            let mut current_tone_in_word = Tone::None;
            for &ch in self.buffer.as_slice() {
                let (_, tone) = telex::get_base_vowel_and_tone(ch);
                if tone != Tone::None {
                    current_tone_in_word = tone;
                    break;
                }
            }

            if current_tone_in_word != Tone::None {
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

            if self.spell_check && !is_valid_vietnamese_syllable(self.buffer.as_slice()) {
                fallback_to_raw = true;
                break;
            }
        }

        if fallback_to_raw {
            self.buffer.clear();
            self.raw_buffer.clear();
            for i in 0..len {
                self.buffer.push(raw_chars[i]);
                self.raw_buffer.push(raw_chars[i]);
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

    fn apply_telex_internal(&mut self, next_char: char) {
        let next_char_lower = fast_lower(next_char);
        // Snapshot the buffer
        let (snapshot_data, len) = self.buffer.snapshot();

        if next_char_lower == 'z' {
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

        let mut stripped_buffer = self.buffer;
        let mut current_tone_in_word = Tone::None;
        for i in 0..len {
            let (base, tone) = telex::get_base_vowel_and_tone(stripped_buffer.as_slice()[i]);
            if tone != Tone::None {
                current_tone_in_word = tone;
                stripped_buffer.replace_at(i, base);
            }
        }

        if let Some(input_tone) = Tone::from_char(next_char_lower) {
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
                    // Annihilate from raw_buffer
                    // The raw_buffer currently contains the full sequence including this `next_char`.
                    // We want to remove the PREVIOUS occurrence of the character that caused `input_tone`.
                    // We'll search backwards from len - 2 (since len - 1 is `next_char`).

                    let r_len = self.raw_buffer.len();
                    for j in (0..r_len.saturating_sub(1)).rev() {
                        let rc = self.raw_buffer.as_slice()[j];
                        if telex::Tone::from_char(fast_lower(rc))
                            == Some(input_tone)
                        {
                            self.raw_buffer.remove(j);

                            break;
                        }
                    }
                    return;
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

                if is_valid_vietnamese_syllable(self.buffer.as_slice()) {
                    // ⚠️ KHÔNG clear uo_smart_fallback ở đây!
                    // Fallback phải sống sót để dùng khi tone key tiếp theo fail.
                    return;
                } else {
                    // Thử Smart W fallback trước khi rollback hoàn toàn
                    if let Some(mut fallback_buf) = self.uo_smart_fallback.take() {
                        // Áp dụng cùng input_tone lên fallback buffer
                        // (fallback_buf = "thuơ", tone Hook → "thuở")
                        if let Some(target_idx) =
                            telex::find_tone_target_index(fallback_buf.as_slice())
                        {
                            let target_char = fallback_buf.as_slice()[target_idx];
                            let (base, _) = telex::get_base_vowel_and_tone(target_char);
                            // Strip tones từ tất cả vowels trước
                            for i in 0..fallback_buf.len() {
                                if telex::is_vowel(fallback_buf.as_slice()[i]) {
                                    let (b, _) =
                                        telex::get_base_vowel_and_tone(fallback_buf.as_slice()[i]);
                                    fallback_buf.replace_at(i, b);
                                }
                            }
                            let new_char = telex::add_tone(base, input_tone);
                            fallback_buf.replace_at(target_idx, new_char);
                        }

                        if is_valid_vietnamese_syllable(fallback_buf.as_slice()) {
                            // Fallback hợp lệ! (e.g., "thuở") — dùng nó
                            self.buffer = fallback_buf;
                            return;
                        }
                        // Fallback cũng không hợp lệ → rollback bình thường
                    }

                    // Rollback hoàn toàn (code cũ)
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

            if self.current_method == InputMethod::Telex {
                // [MỚI] Smart w look-back: uo → ươ, ua → ưa
                if next_char_lower == 'w' && self.buffer.len() >= 2 {
                    let buf_len = self.buffer.len();
                    let second_last = self.buffer.as_slice()[buf_len - 2];
                    let (second_last_base, second_last_tone) =
                        telex::get_base_vowel_and_tone(second_last);
                    let sbl = fast_lower(second_last_base);
                    let (last_base, last_tone) = telex::get_base_vowel_and_tone(last_char);
                    let ll = fast_lower(last_base);

                    if sbl == 'u' && (ll == 'o' || ll == 'a' || ll == 'u') {
                        let mut is_q_exception = false;
                        if buf_len >= 3 {
                            let third_last = fast_lower(
                                self.buffer.as_slice()[buf_len - 3]
                            );
                            if third_last == 'q' {
                                is_q_exception = true;
                            }
                        }

                        if !is_q_exception {
                            if ll == 'o' {
                                // BƯỚC 1: Lưu fallback TRƯỚC KHI transform (buffer vẫn có 'u' + 'o')
                                {
                                    let mut fallback = self.buffer; // copy NOW, 'u' vẫn là 'u'
                                    let fallback_o = telex::add_tone(
                                        if last_char.is_uppercase() { 'Ơ' } else { 'ơ' },
                                        last_tone,
                                    );
                                    fallback.replace_last(fallback_o); // chỉ o→ơ, 'u' giữ nguyên
                                    self.uo_smart_fallback = Some(fallback); // fallback = "thuơ"
                                }

                                // uo → ươ
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
                            } else if ll == 'u' {
                                // uu + w → ưu: CHỈ transform second_last u → ư
                                // last 'u' KHÔNG được thay đổi
                                let new_u = telex::add_tone(
                                    if second_last.is_uppercase() {
                                        'Ư'
                                    } else {
                                        'ư'
                                    },
                                    second_last_tone,
                                );
                                self.buffer.replace_at(buf_len - 2, new_u);
                                // ⚠️ KHÔNG gọi self.buffer.replace_last() ở đây
                                // ⚠️ KHÔNG lưu uo_smart_fallback cho case này
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

                        // Clean up raw buffer correctly by removing the original modifier char
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
            }

            if !applied
                && fast_lower(last_char) == 'đ'
                && next_char_lower == 'd'
            {
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
                } else if fast_lower(last_char) == 'd'
                    && next_char_lower == 'd'
                {
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
                    return;
                }

                // --- DYNAMIC TONE SHIFTING ---
                if current_tone_in_word != Tone::None {
                    // Strip the current tone
                    for i in 0..self.buffer.len() {
                        let (base, _) = telex::get_base_vowel_and_tone(self.buffer.as_slice()[i]);
                        self.buffer.replace_at(i, base);
                    }
                    // Recalculate target
                    if let Some(target_idx) = telex::find_tone_target_index(self.buffer.as_slice())
                    {
                        let target_char = self.buffer.as_slice()[target_idx];
                        let new_char = telex::add_tone(target_char, current_tone_in_word);
                        self.buffer.replace_at(target_idx, new_char);
                    }
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
            // `w` at the start of a buffer is just `w` in standard smart telex
        }

        self.uo_smart_fallback = None; // plain char → word moved on, fallback stale
        self.buffer.push(next_char);
    }

    fn apply_vni_internal(&mut self, next_char: char) {
        let (snapshot_data, len) = self.buffer.snapshot();

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
                        for i in 0..self.buffer.len() {
                            let (b, t) = telex::get_base_vowel_and_tone(self.buffer.as_slice()[i]);
                            if t == input_tone {
                                self.buffer.replace_at(i, b);
                            }
                        }
                        self.buffer.push(next_char);
                        let r_len = self.raw_buffer.len();
                        for j in (0..r_len.saturating_sub(1)).rev() {
                            let rc = self.raw_buffer.as_slice()[j];
                            if rc == next_char {
                                // in VNI the tone digit is literal
                                self.raw_buffer.remove(j);
                                break;
                            }
                        }
                        return;
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

                        if is_valid_vietnamese_syllable(self.buffer.as_slice()) {
                            return;
                        } else {
                            if let Some(mut fallback_buf) = self.uo_smart_fallback.take() {
                                if let Some(fallback_target_idx) =
                                    telex::find_tone_target_index(fallback_buf.as_slice())
                                {
                                    let fallback_target_char =
                                        fallback_buf.as_slice()[fallback_target_idx];
                                    let (fallback_base, _) =
                                        telex::get_base_vowel_and_tone(fallback_target_char);
                                    for i in 0..fallback_buf.len() {
                                        if telex::is_vowel(fallback_buf.as_slice()[i]) {
                                            let (b, _) = telex::get_base_vowel_and_tone(
                                                fallback_buf.as_slice()[i],
                                            );
                                            fallback_buf.replace_at(i, b);
                                        }
                                    }
                                    let fallback_new_char =
                                        telex::add_tone(fallback_base, input_tone);
                                    fallback_buf.replace_at(fallback_target_idx, fallback_new_char);
                                }

                                if is_valid_vietnamese_syllable(fallback_buf.as_slice()) {
                                    self.buffer = fallback_buf;
                                    return;
                                }
                            }

                            self.buffer.clear();
                            for i in 0..len {
                                self.buffer.push(snapshot_data[i]);
                            }
                        }
                        // DO NOT set applied = true so literal digit can be pushed below
                    }
                }
            }
            '6' => {
                for i in 0..len {
                    let (base, tone) = telex::get_base_vowel_and_tone(self.buffer.as_slice()[i]);
                    let new_base = match fast_lower(base) {
                        'a' => {
                            if base.is_uppercase() {
                                '\u{00c2}'
                            } else {
                                '\u{00e2}'
                            }
                        }
                        'e' => {
                            if base.is_uppercase() {
                                '\u{00ca}'
                            } else {
                                '\u{00ea}'
                            }
                        }
                        'o' => {
                            if base.is_uppercase() {
                                '\u{00d4}'
                            } else {
                                '\u{00f4}'
                            }
                        }
                        _ => base,
                    };
                    if new_base != base {
                        self.buffer.replace_at(i, telex::add_tone(new_base, tone));
                        applied = true;
                    }
                }
            }
            '7' => {
                // VNI: khi second_last='u', last='o', digit='7'
                if len >= 2 {
                    let second_last_base =
                        fast_lower(telex::get_base_vowel_and_tone(self.buffer.as_slice()[len - 2])
                            .0);
                    let last_base = fast_lower(telex::get_base_vowel_and_tone(self.buffer.as_slice()[len - 1])
                        .0);
                    if second_last_base == 'u' && last_base == 'o' {
                        // BƯỚC 1: Lưu fallback TRƯỚC KHI transform
                        let mut fallback = self.buffer; // copy NOW, 'u' vẫn là 'u'
                        let last_char = self.buffer.as_slice()[len - 1];
                        let last_tone = telex::get_base_vowel_and_tone(last_char).1;
                        let fallback_o = telex::add_tone(
                            if last_char.is_uppercase() { 'Ơ' } else { 'ơ' },
                            last_tone,
                        );
                        fallback.replace_last(fallback_o);
                        self.uo_smart_fallback = Some(fallback); // fallback = "thuơ"
                    }
                }

                for i in 0..len {
                    let (base, tone) = telex::get_base_vowel_and_tone(self.buffer.as_slice()[i]);
                    let new_base = match fast_lower(base) {
                        'o' => {
                            if base.is_uppercase() {
                                '\u{01a0}'
                            } else {
                                '\u{01a1}'
                            }
                        }
                        'u' => {
                            if base.is_uppercase() {
                                '\u{01af}'
                            } else {
                                '\u{01b0}'
                            }
                        }
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
                    if fast_lower(base) == 'a' {
                        self.buffer.replace_at(i, telex::add_tone('\u{0103}', tone));
                        applied = true;
                    }
                }
            }
            '9' => {
                for i in 0..len {
                    let (base, tone) = telex::get_base_vowel_and_tone(self.buffer.as_slice()[i]);
                    if fast_lower(base) == 'd' && tone == Tone::None {
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

        self.uo_smart_fallback = None; // plain char → word moved on, fallback stale
        self.buffer.push(next_char);
    }
}

#[cfg(test)]
mod boundary_tests {
    use super::*;

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

    fn make_buffer(s: &str) -> CharBuffer {
        let mut buf = CharBuffer::new();
        for c in s.chars() {
            buf.push(c);
        }
        buf
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
        // A digit is swallowed in Telex if it's pushed? Wait, let's see.
        // Actually, digits are part of the word in Telex, they are just appended? Let's check logic.
        // Or if it returns CommitAndPassThrough? Wait, the test output showed it returns Preedit("a1").
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
        // should append '1' because 'á1' is invalid syllable
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

        // Start composing
        engine.process_key('h');
        engine.process_key('e');
        engine.process_key('l');
        engine.process_key('l');
        engine.process_key('o');

        assert_eq!(engine.state, State::Composing);

        let flush_action = engine.flush();

        // Should return a Commit action with current buffer
        assert_eq!(flush_action, Some(Action::Commit(make_buffer("hello"))));

        // Engine state should be reset to Idle
        assert_eq!(engine.state, State::Idle);
        assert_eq!(engine.buffer.len(), 0);

        // Context tracking buffers should be cleared
        assert_eq!(engine.last_committed_raw.len(), 0);
        assert_eq!(engine.last_committed_text.len(), 0);
    }

    #[test]
    fn test_flush_when_idle() {
        let mut engine = Engine::new(InputMethod::Telex, false);

        // Populate context buffers by completing a composition
        engine.process_key('h');
        engine.process_key('i');
        engine.process_key(' '); // This commits the word 'hi'

        assert_eq!(engine.state, State::Idle);
        assert!(engine.last_committed_raw.len() > 0);
        assert!(engine.last_committed_text.len() > 0);

        let flush_action = engine.flush();

        // Should return None when already Idle
        assert_eq!(flush_action, None);

        // Context tracking buffers should be cleared
        assert_eq!(engine.last_committed_raw.len(), 0);
        assert_eq!(engine.last_committed_text.len(), 0);
    }
}
