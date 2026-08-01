use crate::telex::*;

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
