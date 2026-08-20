use crate::telex::fast_lower;

/// Vietnamese initials (consonant clusters at start of syllable).
/// Ordered longest-first for greedy matching.
const INITIALS_3: [&[char]; 1] = [&['n', 'g', 'h']];
const INITIALS_2: [&[char]; 10] = [
    &['c', 'h'], &['g', 'h'], &['g', 'i'], &['k', 'h'], &['n', 'g'],
    &['n', 'h'], &['p', 'h'], &['q', 'u'], &['t', 'h'], &['t', 'r'],
];
const INITIALS_1: &[char] = &[
    'b', 'c', 'd', 'đ', 'g', 'h', 'k', 'l', 'm', 'n', 'p', 'q', 'r', 's', 't', 'v', 'x',
];

/// Vietnamese coda consonants (consonant clusters at end of syllable).
const CODAS_2: [&[char]; 3] = [&['c', 'h'], &['n', 'g'], &['n', 'h']];
const CODAS_1: &[char] = &['c', 'm', 'n', 'p', 't', 'o', 'u', 'i', 'y'];

/// Try to consume an initial consonant cluster from `lower_slice[cursor..]`.
/// Returns the new cursor position.
#[inline]
fn consume_initial(lower_slice: &[char], mut cursor: usize) -> usize {
    let remaining = lower_slice.len() - cursor;

    if remaining >= 3 {
        let s = &lower_slice[cursor..cursor + 3];
        for &init in &INITIALS_3 {
            if s == init {
                return cursor + 3;
            }
        }
    }
    if remaining >= 2 {
        let s = &lower_slice[cursor..cursor + 2];
        for &init in &INITIALS_2 {
            if s == init {
                return cursor + 2;
            }
        }
    }
    if remaining >= 1 {
        let c = lower_slice[cursor];
        for &init in INITIALS_1 {
            if c == init {
                cursor += 1;
                return cursor;
            }
        }
    }
    cursor
}

/// Try to consume a coda consonant cluster from `lower_slice[cursor..]`.
/// Returns the new cursor position.
#[inline]
fn consume_coda(lower_slice: &[char], mut cursor: usize) -> usize {
    let remaining = lower_slice.len() - cursor;

    if remaining >= 2 {
        let s = &lower_slice[cursor..cursor + 2];
        for &coda in &CODAS_2 {
            if s == coda {
                return cursor + 2;
            }
        }
    }
    if remaining >= 1 {
        let c = lower_slice[cursor];
        for &coda in CODAS_1 {
            if c == coda {
                cursor += 1;
                return cursor;
            }
        }
    }
    cursor
}

pub fn is_valid_vietnamese_syllable(chars: &[char]) -> bool {
    if chars.is_empty() {
        return true;
    }

    let len = std::cmp::min(chars.len(), 16);

    // Build lowercase working slice on the stack
    let mut lower_chars = ['\x00'; 16];
    for (i, &c) in chars.iter().take(16).enumerate() {
        lower_chars[i] = fast_lower(c);
    }
    let lower_slice = &lower_chars[..len];

    // Step 1: Consume ONE initial consonant cluster (longest match first)
    let mut cursor = consume_initial(lower_slice, 0);

    // Step 2: Consume ALL contiguous valid vowels
    let vowel_start = cursor;
    let mut vowel_count = 0;
    while cursor < len && crate::telex::is_vowel(chars[cursor]) {
        cursor += 1;
        vowel_count += 1;
    }

    // Vietnamese syllables have at most 3 vowels
    if vowel_count > 3 {
        return false;
    }

    // Disallow invalid vowel clusters
    if vowel_count >= 3 {
        let v1 = crate::telex::get_base_vowel_and_tone(chars[vowel_start]).0;
        let v2 = crate::telex::get_base_vowel_and_tone(chars[vowel_start + 1]).0;
        let v3 = crate::telex::get_base_vowel_and_tone(chars[vowel_start + 2]).0;
        let v1_l = fast_lower(v1);
        let v2_l = fast_lower(v2);
        let v3_l = fast_lower(v3);

        if v2_l == 'o' && v3_l == 'o' && v1_l != 'o' {
            // like ăoo, aoo
            return false;
        }
    } else if vowel_count == 2 {
        let v1 = crate::telex::get_base_vowel_and_tone(chars[vowel_start]).0;
        let v2 = crate::telex::get_base_vowel_and_tone(chars[vowel_start + 1]).0;
        let v1_l = fast_lower(v1);
        let v2_l = fast_lower(v2);

        if v1_l == 'e' && (v2_l == 'ơ' || v2_l == 'ư') {
            // eơ is invalid (eow fallback)
            return false;
        }

        if v1_l == 'ư' && v2_l == 'ơ' && cursor == len {
            let v1_has_tone = crate::telex::get_base_vowel_and_tone(chars[vowel_start]).1
                != crate::telex::Tone::None;
            let v2_has_tone = crate::telex::get_base_vowel_and_tone(chars[vowel_start + 1]).1
                != crate::telex::Tone::None;
            if v1_has_tone || v2_has_tone {
                return false;
            }
        }
    }

    // Step 3: Consume ONE valid coda consonant cluster (longest match first)
    cursor = consume_coda(lower_slice, cursor);

    // Valid if we consumed all characters (max 16)
    cursor == chars.len()
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

    #[test]
    fn test_long_syllable_rejection() {
        // The validation function truncates processing at 16 characters for performance
        // and to avoid panics. We test strings > 16 characters to ensure they are
        // safely processed and rejected (since max valid Vietnamese syllable is ~7-8 chars).
        assert!(!check(&"a".repeat(17)));
        assert!(!check(&"b".repeat(20)));
        assert!(!check("nghiengnghiengnghieng")); // 21 chars
    }
}
