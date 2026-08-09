<<<<<<< SEARCH
    // Step 2: Consume ALL contiguous valid vowels
    // Vowels list: a, ă, â, e, ê, i, o, ô, ơ, u, ư, y and their tones
    let is_vowel = |c: char| -> bool { crate::telex::is_vowel(c) };

    let mut vowel_count = 0;
    while cursor < len && is_vowel(chars[cursor]) {
        cursor += 1;
        vowel_count += 1;
    }

    // Quick heuristic: Vietnamese syllables rarely have more than 3 vowels
    if vowel_count > 3 {
        return false;
    }
=======
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
    if vowel_count == 3 {
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
>>>>>>> REPLACE
