pub fn is_valid_vietnamese_syllable(chars: &[char]) -> bool {
    if chars.is_empty() {
        return true;
    }

    let mut cursor = 0;
    let len = chars.len();

    // Initials: b, c, ch, d, đ, g, gh, gi, h, k, kh, l, m, n, ng, ngh, nh, p, ph, q, qu, r, s, t, th, tr, v, x.
    let initials_3 = ["ngh"];
    let initials_2 = ["ch", "gh", "gi", "kh", "ng", "nh", "ph", "qu", "th", "tr"];
    let initials_1 = ['b', 'c', 'd', 'đ', 'g', 'h', 'k', 'l', 'm', 'n', 'p', 'q', 'r', 's', 't', 'v', 'x'];

    // Step 1: Consume ONE initial (longest match first)
    if cursor < len {
        if len - cursor >= 3 {
            let mut matched = false;
            let slice = &chars[cursor..cursor + 3];
            for &init in &initials_3 {
                if slice.iter().zip(init.chars()).all(|(&a, b)| a == b) {
                    cursor += 3;
                    matched = true;
                    break;
                }
            }
            if !matched {
                let slice = &chars[cursor..cursor + 2];
                for &init in &initials_2 {
                    if slice.iter().zip(init.chars()).all(|(&a, b)| a == b) {
                        cursor += 2;
                        matched = true;
                        break;
                    }
                }
            }
            if !matched {
                let c = chars[cursor];
                for &init in &initials_1 {
                    if c == init {
                        cursor += 1;
                        break;
                    }
                }
            }
        } else if len - cursor >= 2 {
            let mut matched = false;
            let slice = &chars[cursor..cursor + 2];
            for &init in &initials_2 {
                if slice.iter().zip(init.chars()).all(|(&a, b)| a == b) {
                    cursor += 2;
                    matched = true;
                    break;
                }
            }
            if !matched {
                let c = chars[cursor];
                for &init in &initials_1 {
                    if c == init {
                        cursor += 1;
                        break;
                    }
                }
            }
        } else {
            let c = chars[cursor];
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
    let is_vowel = |c: char| -> bool {
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
    };

    while cursor < len && is_vowel(chars[cursor]) {
        cursor += 1;
    }

    // Step 3: Consume ONE valid coda (longest match first)
    // Codas list: c, ch, m, n, ng, nh, p, t, o, u, i, y.
    let codas_2 = ["ch", "ng", "nh"];
    let codas_1 = ['c', 'm', 'n', 'p', 't', 'o', 'u', 'i', 'y'];

    if cursor < len {
        if len - cursor >= 2 {
            let mut matched = false;
            let slice = &chars[cursor..cursor + 2];
            for &coda in &codas_2 {
                if slice.iter().zip(coda.chars()).all(|(&a, b)| a == b) {
                    cursor += 2;
                    matched = true;
                    break;
                }
            }
            if !matched {
                let c = chars[cursor];
                for &coda in &codas_1 {
                    if c == coda {
                        cursor += 1;
                        break;
                    }
                }
            }
        } else {
            let c = chars[cursor];
            for &coda in &codas_1 {
                if c == coda {
                    cursor += 1;
                    break;
                }
            }
        }
    }

    cursor == len
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
