use crate::common::{TestCase, simulate_typing_str};
use vnikey_core::engine::{Engine, InputMethod};

// === CÂU DÀI VÀ ĐOẠN VĂN ===

#[test]
fn test_full_sentence_xin_chao() {
    let mut engine = Engine::new(InputMethod::Telex, false);
    let result = simulate_typing_str(&mut engine, "x i n Space c h a f o Space b a n j");
    assert_eq!(result, "xin chào bạn");
}

#[test]
fn test_sentence_viet_nam() {
    let mut engine = Engine::new(InputMethod::Telex, false);
    let result = simulate_typing_str(&mut engine, "v i e e t j Space n a m");
    assert_eq!(result, "việt nam");
}

#[test]
fn test_multi_word_sentence_telex() {
    let cases = vec![
        TestCase {
            input: "t o o i Space y e e u Space v i e e t j Space n a m",
            expected: "tôi yêu việt nam",
        },
        TestCase {
            input: "h o o m Space n a y Space t r o w i f Space t r o o i j",
            expected: "hôm nay trời trội",
        },
        TestCase {
            input: "x i n Space c h a f o Space t h e e s Space g i o w i s",
            expected: "xin chào thế giới",
        },
    ];
    for case in cases {
        let mut engine = Engine::new(InputMethod::Telex, false);
        assert_eq!(simulate_typing_str(&mut engine, case.input), case.expected);
    }
}

#[test]
fn test_multi_word_sentence_vni() {
    let cases = vec![
        TestCase {
            input: "t o 6 i Space y e 6 u Space v i e 6 t 5 Space n a m",
            expected: "tôi yêu việt nam",
        },
        TestCase {
            input: "x i n Space c h a 2 o Space t h e 6 1 Space g i o 7 i 1",
            expected: "xin chào thế giới",
        },
    ];
    for case in cases {
        let mut engine = Engine::new(InputMethod::Vni, false);
        assert_eq!(simulate_typing_str(&mut engine, case.input), case.expected);
    }
}

#[test]
fn test_long_paragraph_spell_check() {
    // Đoạn văn dài với spell_check = true
    let mut engine = Engine::new(InputMethod::Telex, true);
    // "Hôm nay trời đẹp" — spell check mode
    let result = simulate_typing_str(
        &mut engine,
        "H o o m Space n a y Space t r o w i f Space d d e p j",
    );
    assert_eq!(result, "Hôm nay trời đẹp");
}

#[test]
fn test_sentence_with_backspace_mid_word() {
    // Gõ sai giữa chừng, backspace, gõ lại
    let mut engine = Engine::new(InputMethod::Telex, false);
    let result = simulate_typing_str(&mut engine, "v i BackSpace i e e t j Space n a m");
    assert_eq!(result, "việt nam");
}

#[test]
fn test_sentence_with_multiple_backspaces() {
    let mut engine = Engine::new(InputMethod::Telex, false);
    let result = simulate_typing_str(
        &mut engine,
        "c h a f o Space n h BackSpace BackSpace a n h Space o i",
    );
    assert_eq!(result, "chào anh oi");
}

#[test]
fn test_common_vietnamese_names() {
    let cases = vec![
        TestCase {
            input: "n g u y e e n x",
            expected: "nguyễn",
        },
        TestCase {
            input: "t r a a n f",
            expected: "trần",
        },
        TestCase {
            input: "p h a m j",
            expected: "phạm",
        },
        TestCase {
            input: "h u y n h f",
            expected: "huỳnh",
        },
        TestCase {
            input: "v o x",
            expected: "võ",
        },
        TestCase {
            input: "d d a w n g j",
            expected: "đặng",
        },
        TestCase {
            input: "b u i f",
            expected: "bùi",
        },
        TestCase {
            input: "d d o o x",
            expected: "đỗ",
        },
    ];
    for case in cases {
        let mut engine = Engine::new(InputMethod::Telex, true);
        assert_eq!(simulate_typing_str(&mut engine, case.input), case.expected);
    }
}

#[test]
fn test_common_vietnamese_words_spell_check() {
    let cases = vec![
        TestCase {
            input: "t r u w o w n g f",
            expected: "trường",
        },
        TestCase {
            input: "t h a n h f",
            expected: "thành",
        },
        TestCase {
            input: "k h o o n g",
            expected: "không",
        },
        TestCase {
            input: "d d a w t j",
            expected: "đặt",
        },
        TestCase {
            input: "c o o n g j",
            expected: "cộng",
        },
        TestCase {
            input: "v i e e t j",
            expected: "việt",
        },
    ];
    for case in cases {
        let mut engine = Engine::new(InputMethod::Telex, true);
        assert_eq!(simulate_typing_str(&mut engine, case.input), case.expected);
    }
}

#[test]
fn test_sentence_vni_long() {
    let mut engine = Engine::new(InputMethod::Vni, false);
    // "tôi yêu việt nam"
    let result = simulate_typing_str(
        &mut engine,
        "t o 6 i Space y e 6 u Space v i e 6 t 5 Space n a m",
    );
    assert_eq!(result, "tôi yêu việt nam");
}

#[test]
fn test_three_word_sentence_with_tones() {
    let mut engine = Engine::new(InputMethod::Telex, false);
    let result = simulate_typing_str(&mut engine, "x i n Space c h a f o Space b a n j Space o i");
    assert_eq!(result, "xin chào bạn oi");
}

#[test]
fn test_word_after_commit_is_independent() {
    // Gõ từ 1, commit. Từ 2 hoàn toàn độc lập — tone shift không bị ảnh hưởng
    let mut engine = Engine::new(InputMethod::Telex, false);
    let result = simulate_typing_str(&mut engine, "h o a s Space h o a n g f");
    assert_eq!(result, "hoá hoàng");
}

#[test]
fn test_tone_override_across_word_boundary() {
    // Đảm bảo tone của từ trước không ảnh hưởng từ sau
    let mut engine = Engine::new(InputMethod::Telex, false);
    let result = simulate_typing_str(&mut engine, "t o a n s Space t o a n f");
    assert_eq!(result, "toán toàn");
}
