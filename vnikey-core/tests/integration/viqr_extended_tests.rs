use crate::common::{simulate_typing_str, TestCase};
use vnikey_core::engine::{Engine, InputMethod};

// === VIQR EXTENDED TESTS ===

#[test]
fn test_viqr_all_tones() {
    let cases = vec![
        TestCase {
            input: "a '",
            expected: "á",
        },
        TestCase {
            input: "a `",
            expected: "à",
        },
        TestCase {
            input: "a ?",
            expected: "ả",
        },
        TestCase {
            input: "a ~",
            expected: "ã",
        },
        TestCase {
            input: "a .",
            expected: "ạ",
        },
    ];
    for case in cases {
        let mut engine = Engine::new(InputMethod::Viqr, false);
        assert_eq!(
            simulate_typing_str(&mut engine, case.input),
            case.expected,
            "input: {}",
            case.input
        );
    }
}

#[test]
fn test_viqr_all_vowel_modifiers() {
    let cases = vec![
        TestCase {
            input: "a ^",
            expected: "â",
        },
        TestCase {
            input: "e ^",
            expected: "ê",
        },
        TestCase {
            input: "o ^",
            expected: "ô",
        },
        TestCase {
            input: "a (",
            expected: "ă",
        },
        TestCase {
            input: "u +",
            expected: "ư",
        },
        TestCase {
            input: "o +",
            expected: "ơ",
        },
        TestCase {
            input: "d -",
            expected: "đ",
        },
        TestCase {
            input: "D -",
            expected: "Đ",
        },
    ];
    for case in cases {
        let mut engine = Engine::new(InputMethod::Viqr, false);
        assert_eq!(
            simulate_typing_str(&mut engine, case.input),
            case.expected,
            "input: {}",
            case.input
        );
    }
}

#[test]
fn test_viqr_complex_words() {
    let cases = vec![
        TestCase {
            input: "v i e ^ t .",
            expected: "việt",
        },
        TestCase {
            input: "n g u y e ^ n ~",
            expected: "nguyễn",
        },
        TestCase {
            input: "t r u o + n g `",
            expected: "trường",
        },
        TestCase {
            input: "d - u o + n g",
            expected: "đương",
        },
    ];
    for case in cases {
        let mut engine = Engine::new(InputMethod::Viqr, false);
        assert_eq!(
            simulate_typing_str(&mut engine, case.input),
            case.expected,
            "input: {}",
            case.input
        );
    }
}

#[test]
fn test_viqr_tone_cancellation() {
    let cases = vec![
        TestCase {
            input: "a ' '",
            expected: "a'",
        },
        TestCase {
            input: "a ` `",
            expected: "a`",
        },
        TestCase {
            input: "a ? ?",
            expected: "a?",
        },
        TestCase {
            input: "a ~ ~",
            expected: "a~",
        },
        TestCase {
            input: "a . .",
            expected: "a.",
        },
        TestCase {
            input: "a ^ ^",
            expected: "a^",
        },
        TestCase {
            input: "a ( (",
            expected: "a(",
        },
        TestCase {
            input: "o + +",
            expected: "o+",
        },
        TestCase {
            input: "d - -",
            expected: "d-",
        },
    ];
    for case in cases {
        let mut engine = Engine::new(InputMethod::Viqr, false);
        assert_eq!(
            simulate_typing_str(&mut engine, case.input),
            case.expected,
            "input: {}",
            case.input
        );
    }
}

#[test]
fn test_viqr_combined_modifier_and_tone() {
    let cases = vec![
        TestCase {
            input: "a ^ '",
            expected: "ấ",
        },
        TestCase {
            input: "a ^ `",
            expected: "ầ",
        },
        TestCase {
            input: "a ^ ?",
            expected: "ẩ",
        },
        TestCase {
            input: "a ^ ~",
            expected: "ẫ",
        },
        TestCase {
            input: "a ^ .",
            expected: "ậ",
        },
        TestCase {
            input: "a ( '",
            expected: "ắ",
        },
        TestCase {
            input: "o + `",
            expected: "ờ",
        },
        TestCase {
            input: "u + '",
            expected: "ứ",
        },
    ];
    for case in cases {
        let mut engine = Engine::new(InputMethod::Viqr, false);
        assert_eq!(
            simulate_typing_str(&mut engine, case.input),
            case.expected,
            "input: {}",
            case.input
        );
    }
}
