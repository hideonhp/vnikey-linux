pub fn is_valid_vietnamese_syllable(chars: &[char]) -> bool {
    if chars.is_empty() {
        return true;
    }

    let mut cursor = 0;
    let len = std::cmp::min(chars.len(), 16);

    let mut lower_chars = ['\x00'; 16];
    for (i, &c) in chars.iter().take(16).enumerate() {
        lower_chars[i] = c.to_lowercase().next().unwrap_or(c);
    }
    let lower_slice = &lower_chars[..len];

    // Initials: b, c, ch, d, đ, g, gh, gi, h, k, kh, l, m, n, ng, ngh, nh, p, ph, q, qu, r, s, t, th, tr, v, x.
    let initials_3 = ["ngh"];
    let initials_2 = ["ch", "gh", "gi", "kh", "ng", "nh", "ph", "qu", "th", "tr"];
    let initials_1 = [
        'b', 'c', 'd', 'đ', 'g', 'h', 'k', 'l', 'm', 'n', 'p', 'q', 'r', 's', 't', 'v', 'x',
    ];

    // Step 1: Consume ONE initial (longest match first)
    if cursor < len {
        if len - cursor >= 3 {
            let mut matched = false;
            let slice = &lower_slice[cursor..cursor + 3];
            for &init in &initials_3 {
                if slice.iter().zip(init.chars()).all(|(&a, b)| a == b) {
                    cursor += 3;
                    matched = true;
                    break;
                }
            }
            if !matched {
                let slice = &lower_slice[cursor..cursor + 2];
                for &init in &initials_2 {
                    if slice.iter().zip(init.chars()).all(|(&a, b)| a == b) {
                        cursor += 2;
                        matched = true;
                        break;
                    }
                }
            }
            if !matched {
                let c = lower_slice[cursor];
                for &init in &initials_1 {
                    if c == init {
                        cursor += 1;
                        break;
                    }
                }
            }
        } else if len - cursor >= 2 {
            let mut matched = false;
            let slice = &lower_slice[cursor..cursor + 2];
            for &init in &initials_2 {
                if slice.iter().zip(init.chars()).all(|(&a, b)| a == b) {
                    cursor += 2;
                    matched = true;
                    break;
                }
            }
            if !matched {
                let c = lower_slice[cursor];
                for &init in &initials_1 {
                    if c == init {
                        cursor += 1;
                        break;
                    }
                }
            }
        } else {
            let c = lower_slice[cursor];
            for &init in &initials_1 {
                if c == init {
                    cursor += 1;
                    break;
                }
            }
        }
    }

    // Step 2: Consume ALL contiguous valid vowels
    // Vowels list: a, ă, â, e, ê, i, o, ô, ơ, u, ư, y and their tones
    let is_vowel = |c: char| -> bool { crate::telex::is_vowel(c) };

    let vowel_start = cursor;
    let mut vowel_count = 0;
    while cursor < len && is_vowel(chars[cursor]) {
        cursor += 1;
        vowel_count += 1;
    }

    // Quick heuristic: Vietnamese syllables rarely have more than 3 vowels
    if vowel_count > 3 {
        return false;
    }

    // Disallow invalid vowel clusters like "oo", "ooo", "ăoo", etc.
    // We can do a rudimentary check or we can rely on spelling check rules.
    // Actually, "oo" is valid in Vietnamese (e.g. "xoong", "boong"), but "ăoo" is not.
    // For now, let's just make sure "ăoo" fails by checking if `vowel_count >= 2` and the characters contain invalid sequences.
    if vowel_count >= 3 {
        let v1 = crate::telex::get_base_vowel_and_tone(chars[vowel_start]).0;
        let v2 = crate::telex::get_base_vowel_and_tone(chars[vowel_start + 1]).0;
        let v3 = crate::telex::get_base_vowel_and_tone(chars[vowel_start + 2]).0;
        let v1_l = v1.to_lowercase().next().unwrap_or(v1);
        let v2_l = v2.to_lowercase().next().unwrap_or(v2);
        let v3_l = v3.to_lowercase().next().unwrap_or(v3);

        if v2_l == 'o' && v3_l == 'o' && v1_l != 'o' {
            // like ăoo, aoo
            return false;
        }
    }

    // Step 3: Consume ONE valid coda (longest match first)
    // Codas list: c, ch, m, n, ng, nh, p, t, o, u, i, y.
    let codas_2 = ["ch", "ng", "nh"];
    let codas_1 = ['c', 'm', 'n', 'p', 't', 'o', 'u', 'i', 'y'];

    if cursor < len {
        if len - cursor >= 2 {
            let mut matched = false;
            let slice = &lower_slice[cursor..cursor + 2];
            for &coda in &codas_2 {
                if slice.iter().zip(coda.chars()).all(|(&a, b)| a == b) {
                    cursor += 2;
                    matched = true;
                    break;
                }
            }
            if !matched {
                let c = lower_slice[cursor];
                for &coda in &codas_1 {
                    if c == coda {
                        cursor += 1;
                        break;
                    }
                }
            }
        } else {
            let c = lower_slice[cursor];
            for &coda in &codas_1 {
                if c == coda {
                    cursor += 1;
                    break;
                }
            }
        }
    }

    cursor == chars.len() // we checked min with 16 but if chars length > 16 then it can't be valid
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(s: &str) -> bool {
        let chars: Vec<char> = s.chars().collect();
        is_valid_vietnamese_syllable(&chars)
    }

    #[test]
    fn test_valid_vietnamese() {
        assert!(check("hoàng"));
        assert!(check("nghĩ"));
        assert!(check("việt"));
        assert!(check("nam"));
        assert!(check("giường"));
        assert!(check("chuyện"));
        assert!(check("quốc"));
    }

    #[test]
    fn test_invalid_vietnamese() {
        assert!(!check("englí"));
        assert!(!check("linúx"));
        assert!(!check("str"));
    }
}
