cat << 'INNER_EOF' > src/telex.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    None,
    Acute,   // s (Sắc)
    Grave,   // f (Huyền)
    Hook,    // r (Hỏi)
    Tilde,   // x (Ngã)
    Underdot,// j (Nặng)
}

impl Tone {
    pub fn from_char(c: char) -> Option<Self> {
        match c {
            's' => Some(Tone::Acute),
            'f' => Some(Tone::Grave),
            'r' => Some(Tone::Hook),
            'x' => Some(Tone::Tilde),
            'j' => Some(Tone::Underdot),
            _ => None,
        }
    }
}

pub fn is_vowel(c: char) -> bool {
    matches!(
        c,
        'a' | 'ă' | 'â' | 'e' | 'ê' | 'i' | 'o' | 'ô' | 'ơ' | 'u' | 'ư' | 'y' |
        'á' | 'à' | 'ả' | 'ã' | 'ạ' |
        'ắ' | 'ằ' | 'ẳ' | 'ẵ' | 'ặ' |
        'ấ' | 'ầ' | 'ẩ' | 'ẫ' | 'ậ' |
        'é' | 'è' | 'ẻ' | 'ẽ' | 'ẹ' |
        'ế' | 'ề' | 'ể' | 'ễ' | 'ệ' |
        'í' | 'ì' | 'ỉ' | 'ĩ' | 'ị' |
        'ó' | 'ò' | 'ỏ' | 'õ' | 'ọ' |
        'ố' | 'ồ' | 'ổ' | 'ỗ' | 'ộ' |
        'ớ' | 'ờ' | 'ở' | 'ỡ' | 'ợ' |
        'ú' | 'ù' | 'ủ' | 'ũ' | 'ụ' |
        'ứ' | 'ừ' | 'ử' | 'ữ' | 'ự' |
        'ý' | 'ỳ' | 'ỷ' | 'ỹ' | 'ỵ'
    )
}

pub fn get_base_vowel_and_tone(c: char) -> (char, Tone) {
    match c {
        'á' => ('a', Tone::Acute), 'à' => ('a', Tone::Grave), 'ả' => ('a', Tone::Hook), 'ã' => ('a', Tone::Tilde), 'ạ' => ('a', Tone::Underdot),
        'ắ' => ('ă', Tone::Acute), 'ằ' => ('ă', Tone::Grave), 'ẳ' => ('ă', Tone::Hook), 'ẵ' => ('ă', Tone::Tilde), 'ặ' => ('ă', Tone::Underdot),
        'ấ' => ('â', Tone::Acute), 'ầ' => ('â', Tone::Grave), 'ẩ' => ('â', Tone::Hook), 'ẫ' => ('â', Tone::Tilde), 'ậ' => ('â', Tone::Underdot),
        'é' => ('e', Tone::Acute), 'è' => ('e', Tone::Grave), 'ẻ' => ('e', Tone::Hook), 'ẽ' => ('e', Tone::Tilde), 'ẹ' => ('e', Tone::Underdot),
        'ế' => ('ê', Tone::Acute), 'ề' => ('ê', Tone::Grave), 'ể' => ('ê', Tone::Hook), 'ễ' => ('ê', Tone::Tilde), 'ệ' => ('ê', Tone::Underdot),
        'í' => ('i', Tone::Acute), 'ì' => ('i', Tone::Grave), 'ỉ' => ('i', Tone::Hook), 'ĩ' => ('i', Tone::Tilde), 'ị' => ('i', Tone::Underdot),
        'ó' => ('o', Tone::Acute), 'ò' => ('o', Tone::Grave), 'ỏ' => ('o', Tone::Hook), 'õ' => ('o', Tone::Tilde), 'ọ' => ('o', Tone::Underdot),
        'ố' => ('ô', Tone::Acute), 'ồ' => ('ô', Tone::Grave), 'ổ' => ('ô', Tone::Hook), 'ỗ' => ('ô', Tone::Tilde), 'ộ' => ('ô', Tone::Underdot),
        'ớ' => ('ơ', Tone::Acute), 'ờ' => ('ơ', Tone::Grave), 'ở' => ('ơ', Tone::Hook), 'ỡ' => ('ơ', Tone::Tilde), 'ợ' => ('ơ', Tone::Underdot),
        'ú' => ('u', Tone::Acute), 'ù' => ('u', Tone::Grave), 'ủ' => ('u', Tone::Hook), 'ũ' => ('u', Tone::Tilde), 'ụ' => ('u', Tone::Underdot),
        'ứ' => ('ư', Tone::Acute), 'ừ' => ('ư', Tone::Grave), 'ử' => ('ư', Tone::Hook), 'ữ' => ('ư', Tone::Tilde), 'ự' => ('ư', Tone::Underdot),
        'ý' => ('y', Tone::Acute), 'ỳ' => ('y', Tone::Grave), 'ỷ' => ('y', Tone::Hook), 'ỹ' => ('y', Tone::Tilde), 'ỵ' => ('y', Tone::Underdot),
        _ => (c, Tone::None),
    }
}

pub fn add_tone(base: char, tone: Tone) -> char {
    match (base, tone) {
        ('a', Tone::Acute) => 'á', ('a', Tone::Grave) => 'à', ('a', Tone::Hook) => 'ả', ('a', Tone::Tilde) => 'ã', ('a', Tone::Underdot) => 'ạ',
        ('ă', Tone::Acute) => 'ắ', ('ă', Tone::Grave) => 'ằ', ('ă', Tone::Hook) => 'ẳ', ('ă', Tone::Tilde) => 'ẵ', ('ă', Tone::Underdot) => 'ặ',
        ('â', Tone::Acute) => 'ấ', ('â', Tone::Grave) => 'ầ', ('â', Tone::Hook) => 'ẩ', ('â', Tone::Tilde) => 'ẫ', ('â', Tone::Underdot) => 'ậ',
        ('e', Tone::Acute) => 'é', ('e', Tone::Grave) => 'è', ('e', Tone::Hook) => 'ẻ', ('e', Tone::Tilde) => 'ẽ', ('e', Tone::Underdot) => 'ẹ',
        ('ê', Tone::Acute) => 'ế', ('ê', Tone::Grave) => 'ề', ('ê', Tone::Hook) => 'ể', ('ê', Tone::Tilde) => 'ễ', ('ê', Tone::Underdot) => 'ệ',
        ('i', Tone::Acute) => 'í', ('i', Tone::Grave) => 'ì', ('i', Tone::Hook) => 'ỉ', ('i', Tone::Tilde) => 'ĩ', ('i', Tone::Underdot) => 'ị',
        ('o', Tone::Acute) => 'ó', ('o', Tone::Grave) => 'ò', ('o', Tone::Hook) => 'ỏ', ('o', Tone::Tilde) => 'õ', ('o', Tone::Underdot) => 'ọ',
        ('ô', Tone::Acute) => 'ố', ('ô', Tone::Grave) => 'ồ', ('ô', Tone::Hook) => 'ổ', ('ô', Tone::Tilde) => 'ỗ', ('ô', Tone::Underdot) => 'ộ',
        ('ơ', Tone::Acute) => 'ớ', ('ơ', Tone::Grave) => 'ờ', ('ơ', Tone::Hook) => 'ở', ('ơ', Tone::Tilde) => 'ỡ', ('ơ', Tone::Underdot) => 'ợ',
        ('u', Tone::Acute) => 'ú', ('u', Tone::Grave) => 'ù', ('u', Tone::Hook) => 'ủ', ('u', Tone::Tilde) => 'ũ', ('u', Tone::Underdot) => 'ụ',
        ('ư', Tone::Acute) => 'ứ', ('ư', Tone::Grave) => 'ừ', ('ư', Tone::Hook) => 'ử', ('ư', Tone::Tilde) => 'ữ', ('ư', Tone::Underdot) => 'ự',
        ('y', Tone::Acute) => 'ý', ('y', Tone::Grave) => 'ỳ', ('y', Tone::Hook) => 'ỷ', ('y', Tone::Tilde) => 'ỹ', ('y', Tone::Underdot) => 'ỵ',
        _ => base,
    }
}

pub fn apply_vowel_modifier(c: char, modifier: char) -> Option<char> {
    let (base, tone) = get_base_vowel_and_tone(c);
    let new_base = match (base, modifier) {
        ('a', 'a') => 'â',
        ('a', 'w') => 'ă',
        ('e', 'e') => 'ê',
        ('o', 'o') => 'ô',
        ('o', 'w') | ('o', '[') => 'ơ',
        ('u', 'w') | ('u', '[') => 'ư',
        _ => return None,
    };
    Some(add_tone(new_base, tone))
}

pub fn remove_vowel_modifier(c: char) -> Option<char> {
    let (base, tone) = get_base_vowel_and_tone(c);
    let new_base = match base {
        'ă' | 'â' => 'a',
        'ê' => 'e',
        'ô' | 'ơ' => 'o',
        'ư' => 'u',
        _ => return None,
    };
    Some(add_tone(new_base, tone))
}

pub fn find_tone_target_index(chars: &[char]) -> Option<usize> {
    if chars.is_empty() {
        return None;
    }

    let mut start_idx = None;
    let mut end_idx = None;

    let len = chars.len();

    for i in 0..len {
        if is_vowel(chars[i]) {
            if start_idx.is_none() {
                start_idx = Some(i);
            }
            end_idx = Some(i);
        } else if start_idx.is_some() {
            break;
        }
    }

    if start_idx.is_none() {
        return None;
    }

    let mut start = start_idx.unwrap();
    let end = end_idx.unwrap();

    // Check `qu` exception
    if start > 0 && chars[start] == 'u' && chars[start - 1] == 'q' {
        start += 1;
    }

    if start > end {
        return None; // e.g. just "qu"
    }

    // Check `gi` exception
    let mut actual_start = start;
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
        2 => if has_coda { 1 } else { 0 },
        3 => {
            // Check `uye` exception
            let v1 = get_base_vowel_and_tone(chars[actual_start]).0;
            let v2 = get_base_vowel_and_tone(chars[actual_start + 1]).0;
            let v3 = get_base_vowel_and_tone(chars[actual_start + 2]).0;
            if (v1 == 'u' || v1 == 'ư') && (v2 == 'y' || v2 == 'i') && (v3 == 'e' || v3 == 'ê') {
                2
            } else {
                1
            }
        },
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

        assert_eq!(remove_vowel_modifier('ắ'), Some('á'));
        assert_eq!(remove_vowel_modifier('ệ'), Some('ẹ'));
    }

    #[test]
    fn test_tone_target_index() {
        assert_eq!(find_tone_target_index(&['c', 'a', 'm']), Some(1));
        assert_eq!(find_tone_target_index(&['m', 'a', 'i']), Some(1));
        assert_eq!(find_tone_target_index(&['h', 'o', 'a']), Some(1));
        assert_eq!(find_tone_target_index(&['t', 'h', 'u', 'y']), Some(2));
        assert_eq!(find_tone_target_index(&['h', 'o', 'a', 'n', 'g']), Some(2));
        assert_eq!(find_tone_target_index(&['o', 'a', 'i']), Some(1));
        assert_eq!(find_tone_target_index(&['k', 'h', 'u', 'y', 'a']), Some(3));
        assert_eq!(find_tone_target_index(&['n', 'g', 'u', 'y', 'ê', 'n']), Some(4));
        assert_eq!(find_tone_target_index(&['q', 'u', 'a']), Some(2));
        assert_eq!(find_tone_target_index(&['q', 'u', 'a', 'n']), Some(2));
        assert_eq!(find_tone_target_index(&['q', 'u', 'a', 'i']), Some(2));
        assert_eq!(find_tone_target_index(&['q', 'u', 'y']), Some(2));
        assert_eq!(find_tone_target_index(&['g', 'i', 'a']), Some(2));
        assert_eq!(find_tone_target_index(&['g', 'i', 'e', 'n', 'g']), Some(2));
        assert_eq!(find_tone_target_index(&['g', 'i', 'ê', 'n', 'g']), Some(2)); // <-- testing this
        assert_eq!(find_tone_target_index(&['g', 'i']), Some(1));
    }
}
INNER_EOF
