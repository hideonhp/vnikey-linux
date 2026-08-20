#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    None,
    Acute,    // s (Sắc)
    Grave,    // f (Huyền)
    Hook,     // r (Hỏi)
    Tilde,    // x (Ngã)
    Underdot, // j (Nặng)
}

impl Tone {
    pub fn from_char(c: char) -> Option<Self> {
        match c {
            's' | 'S' => Some(Tone::Acute),
            'f' | 'F' => Some(Tone::Grave),
            'r' | 'R' => Some(Tone::Hook),
            'x' | 'X' => Some(Tone::Tilde),
            'j' | 'J' => Some(Tone::Underdot),
            _ => None,
        }
    }
}

/// Fast lowercase conversion — avoids iterator overhead for ASCII chars.
/// All Telex/VNI modifier keys are ASCII, so this is always the fast path in practice.
/// Shared across `engine`, `telex`, and `validation` modules.
#[inline]
pub fn fast_lower(c: char) -> char {
    if c.is_ascii() {
        c.to_ascii_lowercase()
    } else {
        c.to_lowercase().next().unwrap_or(c)
    }
}

/// Restores the original case of `original` onto `result`.
#[inline]
fn restore_case(original: char, result: char) -> char {
    if original.is_uppercase() {
        result.to_uppercase().next().unwrap_or(result)
    } else {
        result
    }
}

/// Returns `true` if `c` is a Vietnamese vowel (with or without tone/modifier).
/// Works for both uppercase and lowercase inputs.
#[inline]
pub fn is_vowel(c: char) -> bool {
    matches!(
        fast_lower(c),
        'a' | 'ă'
            | 'â'
            | 'e'
            | 'ê'
            | 'i'
            | 'o'
            | 'ô'
            | 'ơ'
            | 'u'
            | 'ư'
            | 'y'
            | 'á'
            | 'à'
            | 'ả'
            | 'ã'
            | 'ạ'
            | 'ắ'
            | 'ằ'
            | 'ẳ'
            | 'ẵ'
            | 'ặ'
            | 'ấ'
            | 'ầ'
            | 'ẩ'
            | 'ẫ'
            | 'ậ'
            | 'é'
            | 'è'
            | 'ẻ'
            | 'ẽ'
            | 'ẹ'
            | 'ế'
            | 'ề'
            | 'ể'
            | 'ễ'
            | 'ệ'
            | 'í'
            | 'ì'
            | 'ỉ'
            | 'ĩ'
            | 'ị'
            | 'ó'
            | 'ò'
            | 'ỏ'
            | 'õ'
            | 'ọ'
            | 'ố'
            | 'ồ'
            | 'ổ'
            | 'ỗ'
            | 'ộ'
            | 'ớ'
            | 'ờ'
            | 'ở'
            | 'ỡ'
            | 'ợ'
            | 'ú'
            | 'ù'
            | 'ủ'
            | 'ũ'
            | 'ụ'
            | 'ứ'
            | 'ừ'
            | 'ử'
            | 'ữ'
            | 'ự'
            | 'ý'
            | 'ỳ'
            | 'ỷ'
            | 'ỹ'
            | 'ỵ'
    )
}

pub fn get_base_vowel_and_tone(c: char) -> (char, Tone) {
    let lower = fast_lower(c);
    let (base, tone) = match lower {
        'á' => ('a', Tone::Acute),
        'à' => ('a', Tone::Grave),
        'ả' => ('a', Tone::Hook),
        'ã' => ('a', Tone::Tilde),
        'ạ' => ('a', Tone::Underdot),
        'ắ' => ('ă', Tone::Acute),
        'ằ' => ('ă', Tone::Grave),
        'ẳ' => ('ă', Tone::Hook),
        'ẵ' => ('ă', Tone::Tilde),
        'ặ' => ('ă', Tone::Underdot),
        'ấ' => ('â', Tone::Acute),
        'ầ' => ('â', Tone::Grave),
        'ẩ' => ('â', Tone::Hook),
        'ẫ' => ('â', Tone::Tilde),
        'ậ' => ('â', Tone::Underdot),
        'é' => ('e', Tone::Acute),
        'è' => ('e', Tone::Grave),
        'ẻ' => ('e', Tone::Hook),
        'ẽ' => ('e', Tone::Tilde),
        'ẹ' => ('e', Tone::Underdot),
        'ế' => ('ê', Tone::Acute),
        'ề' => ('ê', Tone::Grave),
        'ể' => ('ê', Tone::Hook),
        'ễ' => ('ê', Tone::Tilde),
        'ệ' => ('ê', Tone::Underdot),
        'í' => ('i', Tone::Acute),
        'ì' => ('i', Tone::Grave),
        'ỉ' => ('i', Tone::Hook),
        'ĩ' => ('i', Tone::Tilde),
        'ị' => ('i', Tone::Underdot),
        'ó' => ('o', Tone::Acute),
        'ò' => ('o', Tone::Grave),
        'ỏ' => ('o', Tone::Hook),
        'õ' => ('o', Tone::Tilde),
        'ọ' => ('o', Tone::Underdot),
        'ố' => ('ô', Tone::Acute),
        'ồ' => ('ô', Tone::Grave),
        'ổ' => ('ô', Tone::Hook),
        'ỗ' => ('ô', Tone::Tilde),
        'ộ' => ('ô', Tone::Underdot),
        'ớ' => ('ơ', Tone::Acute),
        'ờ' => ('ơ', Tone::Grave),
        'ở' => ('ơ', Tone::Hook),
        'ỡ' => ('ơ', Tone::Tilde),
        'ợ' => ('ơ', Tone::Underdot),
        'ú' => ('u', Tone::Acute),
        'ù' => ('u', Tone::Grave),
        'ủ' => ('u', Tone::Hook),
        'ũ' => ('u', Tone::Tilde),
        'ụ' => ('u', Tone::Underdot),
        'ứ' => ('ư', Tone::Acute),
        'ừ' => ('ư', Tone::Grave),
        'ử' => ('ư', Tone::Hook),
        'ữ' => ('ư', Tone::Tilde),
        'ự' => ('ư', Tone::Underdot),
        'ý' => ('y', Tone::Acute),
        'ỳ' => ('y', Tone::Grave),
        'ỷ' => ('y', Tone::Hook),
        'ỹ' => ('y', Tone::Tilde),
        'ỵ' => ('y', Tone::Underdot),
        _ => (lower, Tone::None),
    };
    (restore_case(c, base), tone)
}

pub fn add_tone(base: char, tone: Tone) -> char {
    let lower = fast_lower(base);
    let res = match (lower, tone) {
        ('a', Tone::Acute) => 'á',
        ('a', Tone::Grave) => 'à',
        ('a', Tone::Hook) => 'ả',
        ('a', Tone::Tilde) => 'ã',
        ('a', Tone::Underdot) => 'ạ',
        ('ă', Tone::Acute) => 'ắ',
        ('ă', Tone::Grave) => 'ằ',
        ('ă', Tone::Hook) => 'ẳ',
        ('ă', Tone::Tilde) => 'ẵ',
        ('ă', Tone::Underdot) => 'ặ',
        ('â', Tone::Acute) => 'ấ',
        ('â', Tone::Grave) => 'ầ',
        ('â', Tone::Hook) => 'ẩ',
        ('â', Tone::Tilde) => 'ẫ',
        ('â', Tone::Underdot) => 'ậ',
        ('e', Tone::Acute) => 'é',
        ('e', Tone::Grave) => 'è',
        ('e', Tone::Hook) => 'ẻ',
        ('e', Tone::Tilde) => 'ẽ',
        ('e', Tone::Underdot) => 'ẹ',
        ('ê', Tone::Acute) => 'ế',
        ('ê', Tone::Grave) => 'ề',
        ('ê', Tone::Hook) => 'ể',
        ('ê', Tone::Tilde) => 'ễ',
        ('ê', Tone::Underdot) => 'ệ',
        ('i', Tone::Acute) => 'í',
        ('i', Tone::Grave) => 'ì',
        ('i', Tone::Hook) => 'ỉ',
        ('i', Tone::Tilde) => 'ĩ',
        ('i', Tone::Underdot) => 'ị',
        ('o', Tone::Acute) => 'ó',
        ('o', Tone::Grave) => 'ò',
        ('o', Tone::Hook) => 'ỏ',
        ('o', Tone::Tilde) => 'õ',
        ('o', Tone::Underdot) => 'ọ',
        ('ô', Tone::Acute) => 'ố',
        ('ô', Tone::Grave) => 'ồ',
        ('ô', Tone::Hook) => 'ổ',
        ('ô', Tone::Tilde) => 'ỗ',
        ('ô', Tone::Underdot) => 'ộ',
        ('ơ', Tone::Acute) => 'ớ',
        ('ơ', Tone::Grave) => 'ờ',
        ('ơ', Tone::Hook) => 'ở',
        ('ơ', Tone::Tilde) => 'ỡ',
        ('ơ', Tone::Underdot) => 'ợ',
        ('u', Tone::Acute) => 'ú',
        ('u', Tone::Grave) => 'ù',
        ('u', Tone::Hook) => 'ủ',
        ('u', Tone::Tilde) => 'ũ',
        ('u', Tone::Underdot) => 'ụ',
        ('ư', Tone::Acute) => 'ứ',
        ('ư', Tone::Grave) => 'ừ',
        ('ư', Tone::Hook) => 'ử',
        ('ư', Tone::Tilde) => 'ữ',
        ('ư', Tone::Underdot) => 'ự',
        ('y', Tone::Acute) => 'ý',
        ('y', Tone::Grave) => 'ỳ',
        ('y', Tone::Hook) => 'ỷ',
        ('y', Tone::Tilde) => 'ỹ',
        ('y', Tone::Underdot) => 'ỵ',
        _ => lower,
    };
    restore_case(base, res)
}

pub fn apply_vowel_modifier(c: char, modifier: char) -> Option<char> {
    let (base, tone) = get_base_vowel_and_tone(c);
    let lower_base = fast_lower(base);
    let lower_mod = fast_lower(modifier);

    // Only apply valid modifier overrides/upgrades.
    // If the modifier is NOT a valid upgrade for this specific base, return None.
    let new_base = match (lower_base, lower_mod) {
        ('a', 'a') => 'â',
        ('a', 'w') => 'ă',
        ('â', 'w') => 'ă', // â + w -> ă (override)
        ('ă', 'a') => 'â', // ă + a -> â (override)
        ('e', 'e') => 'ê',
        ('o', 'o') => 'ô',
        ('o', 'w') | ('o', '[') => 'ơ',
        ('ô', 'w') | ('ô', '[') => 'ơ', // ô + w -> ơ (override)
        ('ơ', 'o') => 'ô',              // ơ + o -> ô (override)
        ('u', 'w') | ('u', '[') => 'ư',
        // In Telex, `ư + u` -> NO modification, it's an append. So return None.
        _ => return None,
    };
    Some(add_tone(restore_case(c, new_base), tone))
}

pub fn remove_vowel_modifier(c: char, modifier: char) -> Option<char> {
    let (base, tone) = get_base_vowel_and_tone(c);
    let lower_base = fast_lower(base);
    let lower_mod = fast_lower(modifier);

    // Only cancel if the key pressed exactly matches the modifier that would CREATE this base
    let new_base = match (lower_base, lower_mod) {
        ('ă', 'w') => 'a',
        ('â', 'a') => 'a',
        ('ê', 'e') => 'e',
        ('ô', 'o') => 'o',
        ('ơ', 'w') | ('ơ', '[') => 'o',
        ('ư', 'w') | ('ư', '[') => 'u',
        _ => return None,
    };

    Some(add_tone(restore_case(c, new_base), tone))
}

pub fn find_tone_target_index(chars: &[char]) -> Option<usize> {
    if chars.is_empty() {
        return None;
    }

    let mut start_idx = None;
    let mut end_idx = None;

    let len = chars.len();

    for (i, &c) in chars.iter().enumerate() {
        if is_vowel(c) {
            if start_idx.is_none() {
                start_idx = Some(i);
            }
            end_idx = Some(i);
        } else if start_idx.is_some() {
            break;
        }
    }

    let start = start_idx?;
    let end = end_idx?;

    // Check `qu` exception
    let mut actual_start = start;
    let mut is_qu = false;
    if start > 0 && chars[start] == 'u' && chars[start - 1] == 'q' {
        actual_start += 1;
        is_qu = true;
    }

    if actual_start > end {
        return None; // e.g. just "qu"
    }

    // Check `gi` exception
    if actual_start > 0 && chars[actual_start] == 'i' && chars[actual_start - 1] == 'g' {
        // If the vowel block length is > 1 (excluding `gi`), don't count `i`
        if end > actual_start {
            actual_start += 1;
        }
    }

    if actual_start > end {
        return None;
    }

    let vowel_count = end - actual_start + 1;
    let has_coda = end < len - 1; // If there are characters after the vowel cluster

    let target_offset = match vowel_count {
        1 => 0,
        2 => {
            let v1 = get_base_vowel_and_tone(chars[actual_start]).0;
            let v2 = get_base_vowel_and_tone(chars[actual_start + 1]).0;
            let v1_l = fast_lower(v1);
            let v2_l = fast_lower(v2);

            if (v1_l == 'u' && (v2_l == 'ơ' || v2_l == 'a' || v2_l == 'ê'))
                || (v1_l == 'ư' && v2_l == 'ơ')
                || (v1_l == 'o' && (v2_l == 'a' || v2_l == 'e'))
            {
                1
            } else if (v1_l == 'u' || v1_l == 'ư') && (v2_l == 'y' || v2_l == 'i') {
                if is_qu {
                    0 // 'qu' acts as consonant 'qw', so 'y' or 'i' is the only vowel
                } else if has_coda {
                    1
                } else {
                    0
                }
            } else if has_coda {
                1
            } else {
                0
            }
        }
        3 => {
            // Check `uye` exception
            let v1 = get_base_vowel_and_tone(chars[actual_start]).0;
            let v2 = get_base_vowel_and_tone(chars[actual_start + 1]).0;
            let v3 = get_base_vowel_and_tone(chars[actual_start + 2]).0;
            let v1_l = fast_lower(v1);
            let v2_l = fast_lower(v2);
            let v3_l = fast_lower(v3);

            if (v1_l == 'u' || v1_l == 'ư')
                && (v2_l == 'y' || v2_l == 'i')
                && (v3_l == 'e' || v3_l == 'ê')
            {
                2
            } else if (v1_l == 'u' || v1_l == 'ư')
                && (v2_l == 'y' || v2_l == 'i')
                && (v3_l == 'a' || v3_l == 'u' || v3_l == 'ư')
            {
                // E.g. khuya, khuỷu
                1
            } else {
                1
            }
        }
        _ => 1, // Fallback for 4+ vowels (rare, but just in case)
    };

    Some(actual_start + target_offset)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telex_utils() {
        assert_eq!(get_base_vowel_and_tone('á'), ('a', Tone::Acute));
        assert_eq!(get_base_vowel_and_tone('o'), ('o', Tone::None));

        assert_eq!(add_tone('a', Tone::Acute), 'á');
        assert_eq!(add_tone('ê', Tone::Underdot), 'ệ');

        assert_eq!(apply_vowel_modifier('a', 'a'), Some('â'));
        assert_eq!(apply_vowel_modifier('á', 'w'), Some('ắ'));
        assert_eq!(apply_vowel_modifier('i', 'e'), None);

        assert_eq!(remove_vowel_modifier('ắ', 'w'), Some('á'));
        assert_eq!(remove_vowel_modifier('ệ', 'e'), Some('ẹ'));
    }

    #[test]
    fn test_is_vowel_case_insensitive() {
        // Lowercase
        assert!(is_vowel('a'));
        assert!(is_vowel('ă'));
        assert!(is_vowel('â'));
        assert!(is_vowel('ư'));
        assert!(is_vowel('ơ'));
        assert!(is_vowel('á'));
        assert!(is_vowel('ớ'));
        // Uppercase — must also return true
        assert!(is_vowel('A'));
        assert!(is_vowel('Ă'));
        assert!(is_vowel('Â'));
        assert!(is_vowel('Ư'));
        assert!(is_vowel('Ơ'));
        assert!(is_vowel('Á'));
        assert!(is_vowel('Ớ'));
        // Non-vowels
        assert!(!is_vowel('b'));
        assert!(!is_vowel('c'));
        assert!(!is_vowel('đ'));
        assert!(!is_vowel('1'));
    }

    #[test]
    fn test_tone_target_index() {
        assert_eq!(find_tone_target_index(&['c', 'a', 'm']), Some(1));
        assert_eq!(find_tone_target_index(&['m', 'a', 'i']), Some(1));
        assert_eq!(find_tone_target_index(&['h', 'o', 'a']), Some(2));
        assert_eq!(find_tone_target_index(&['t', 'h', 'u', 'y']), Some(2)); // u is index 2.
        assert_eq!(find_tone_target_index(&['h', 'o', 'a', 'n', 'g']), Some(2));
        assert_eq!(find_tone_target_index(&['o', 'a', 'i']), Some(1));
        assert_eq!(find_tone_target_index(&['k', 'h', 'u', 'y', 'a']), Some(3));
        assert_eq!(
            find_tone_target_index(&['n', 'g', 'u', 'y', 'ê', 'n']),
            Some(4)
        );
        assert_eq!(find_tone_target_index(&['q', 'u', 'a']), Some(2));
        assert_eq!(find_tone_target_index(&['q', 'u', 'a', 'n']), Some(2));
        assert_eq!(find_tone_target_index(&['q', 'u', 'a', 'i']), Some(2));
        assert_eq!(find_tone_target_index(&['q', 'u', 'y']), Some(2));
        assert_eq!(find_tone_target_index(&['g', 'i', 'a']), Some(2));
        assert_eq!(find_tone_target_index(&['g', 'i', 'e', 'n', 'g']), Some(2));
        assert_eq!(find_tone_target_index(&['g', 'i', 'ê', 'n', 'g']), Some(2));
        assert_eq!(find_tone_target_index(&['g', 'i']), Some(1));
    }
}
