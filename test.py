import re

with open('vnikey-core/src/engine.rs', 'r') as f:
    content = f.read()

start_pattern = r'fn try_apply_modifier_telex\(\n.*?\) -> bool \{\n\s+let mut applied = false;\n\s+let mut cancelled = false;\n'
end_pattern = r'\n\s+if applied \{\n\s+if cancelled \{'

match = re.search(start_pattern + r'(.*?)' + end_pattern, content, re.DOTALL)
if match:
    old_code = match.group(1)

    new_code = """
        for i in (0..len).rev() {
            let current_char = snapshot_data[i];
            let current_char_lower = fast_lower(current_char);

            // 1. Check Smart W Look-back (uo -> ươ, ua -> ưa, uu -> ưu)
            if next_char_lower == 'w' && i > 0 {
                let second_last = snapshot_data[i - 1];
                let (second_last_base, second_last_tone) = telex::get_base_vowel_and_tone(second_last);
                let sbl = fast_lower(second_last_base);
                let (last_base, last_tone) = telex::get_base_vowel_and_tone(current_char);
                let ll = fast_lower(last_base);

                if sbl == 'u' && (ll == 'o' || ll == 'a' || ll == 'u') {
                    let is_q_exception = i >= 2 && fast_lower(snapshot_data[i - 2]) == 'q';
                    if !is_q_exception {
                        if ll == 'o' {
                            let mut fallback = self.buffer;
                            let fallback_o = telex::add_tone(
                                if current_char.is_uppercase() { 'Ơ' } else { 'ơ' },
                                last_tone,
                            );
                            fallback.replace_at(i, fallback_o);
                            self.uo_smart_fallback = Some(fallback);

                            let new_u = telex::add_tone(
                                if second_last.is_uppercase() { 'Ư' } else { 'ư' },
                                second_last_tone,
                            );
                            self.buffer.replace_at(i - 1, new_u);
                            let new_o = telex::add_tone(
                                if current_char.is_uppercase() { 'Ơ' } else { 'ơ' },
                                last_tone,
                            );
                            self.buffer.replace_at(i, new_o);
                            applied = true;
                            break;
                        } else if ll == 'a' || ll == 'u' {
                            let new_u = telex::add_tone(
                                if second_last.is_uppercase() { 'Ư' } else { 'ư' },
                                second_last_tone,
                            );
                            self.buffer.replace_at(i - 1, new_u);
                            applied = true;
                            break;
                        }
                    }
                }
            }

            // 2. Check Cancellation
            if let Some(removed) = telex::remove_vowel_modifier(current_char, next_char_lower) {
                self.buffer.replace_at(i, removed);
                self.buffer.push(next_char);
                applied = true;
                cancelled = true;

                let r_len = self.raw_buffer.len();
                for j in (0..r_len.saturating_sub(1)).rev() {
                    let rc = self.raw_buffer.as_slice()[j];
                    if fast_lower(rc) == next_char_lower {
                        self.raw_buffer.remove(j);
                        break;
                    }
                }
                break;
            }

            if current_char_lower == 'đ' && next_char_lower == 'd' {
                self.buffer.replace_at(i, if current_char.is_uppercase() { 'D' } else { 'd' });
                self.buffer.push('d');
                applied = true;
                cancelled = true;
                break;
            }

            // 3. Check Application
            if let Some(modified) = telex::apply_vowel_modifier(current_char, next_char_lower) {
                self.buffer.replace_at(i, modified);
                applied = true;
                break;
            }

            if current_char_lower == 'd' && next_char_lower == 'd' {
                self.buffer.replace_at(i, if current_char.is_uppercase() { 'Đ' } else { 'đ' });
                applied = true;
                break;
            }
        }

        // 4. Special case: If `w` was typed but didn't modify anything, it should insert `ư`.
        if !applied && next_char_lower == 'w' {
            self.buffer.push(if next_char.is_uppercase() { 'Ư' } else { 'ư' });
            applied = true;
        }"""

    content = content[:match.start(1)] + new_code + content[match.end(1):]
    with open('vnikey-core/src/engine.rs', 'w') as f:
        f.write(content)
    print("Replaced!")
else:
    print("Not found!")
