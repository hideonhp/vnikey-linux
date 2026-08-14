mod common;
use common::{TestCase, simulate_typing_str};
use vnikey_core::engine::{Engine, InputMethod};

// === MIXED CONTENT: Vietnamese + Số + Ký tự đặc biệt ===

#[test]
fn test_address_style() {
    // "số 12, đường Nguyễn Văn A"
    let mut engine = Engine::new(InputMethod::Telex, true);
    let result = simulate_typing_str(
        &mut engine,
        "s o o s Space 1 2 , Space d d u w o w n g f Space N g u y e e n x Space V a w n Space A",
    );
    assert_eq!(result, "số 12, đường Nguyễn Văn A");
}

#[test]
fn test_date_format() {
    let mut engine = Engine::new(InputMethod::Telex, true);
    let result = simulate_typing_str(&mut engine, "1 1 / 0 8 / 2 0 2 6");
    assert_eq!(result, "11/08/2026");
}

#[test]
fn test_time_format() {
    let mut engine = Engine::new(InputMethod::Telex, true);
    let result = simulate_typing_str(&mut engine, "1 7 : 3 0");
    assert_eq!(result, "17:30");
}

#[test]
fn test_phone_number() {
    let mut engine = Engine::new(InputMethod::Telex, true);
    let result = simulate_typing_str(&mut engine, "0 9 0 8 - 1 2 3 - 4 5 6");
    assert_eq!(result, "0908-123-456");
}

#[test]
fn test_price_format() {
    let mut engine = Engine::new(InputMethod::Telex, true);
    // "giá 50.000đ"
    let result = simulate_typing_str(&mut engine, "g i a s Space 5 0 . 0 0 0 d");
    // 'd' alone is alphabetic, engine composing → spell check "d" valid? Yes single consonant is valid
    // 'd' in telex → composing, then '.' → CommitAndPassThrough("d") + PassThrough '.'
    // Hmm, let's simplify:
    assert_eq!(result, "giá 50.000d");
}

#[test]
fn test_code_comment_style() {
    // "// Hàm xử lý input"
    let mut engine = Engine::new(InputMethod::Telex, true);
    let result = simulate_typing_str(
        &mut engine,
        "/ / Space H a f m Space x u w r Space l y s Space i n p u t",
    );
    assert_eq!(result, "// Hàm xử lý input");
}

#[test]
fn test_markdown_heading() {
    // "## Tiêu đề chính"
    let mut engine = Engine::new(InputMethod::Telex, true);
    let result = simulate_typing_str(
        &mut engine,
        "# # Space T i e e u Space d d e e f Space c h i n h s",
    );
    assert_eq!(result, "## Tiêu đề chính");
}

#[test]
fn test_mixed_english_and_vietnamese_spell_check() {
    // English words should pass through unchanged with spell check enabled
    let cases = vec![
        TestCase {
            input: "c o d e Space l a n g u a g e",
            expected: "code language",
        },
        TestCase {
            input: "g i t h u b . c o m",
            expected: "github.com",
        },
        TestCase {
            input: "l i n u x Space k e r n e l",
            expected: "linux kernel",
        },
    ];
    for case in cases {
        let mut engine = Engine::new(InputMethod::Telex, true);
        assert_eq!(simulate_typing_str(&mut engine, case.input), case.expected);
    }
}

#[test]
fn test_vni_mixed_number_not_modifier() {
    // VNI: số sau khi commit không phải modifier
    let mut engine = Engine::new(InputMethod::Vni, true);
    let result = simulate_typing_str(&mut engine, "n a m Space 2 0 2 6 Space t o 6 t 1");
    assert_eq!(result, "nam 2026 tốt");
}

#[test]
fn test_telex_number_passthrough_after_commit() {
    // Telex: digits after commit → PassThrough (not modifier)
    let mut engine = Engine::new(InputMethod::Telex, true);
    let result = simulate_typing_str(&mut engine, "h o a s Space 1 9 7 5");
    assert_eq!(result, "hoá 1975");
}

#[test]
fn test_long_mixed_string() {
    // "Vnikey v1.0 - bộ gõ tiếng Việt cho Linux"
    let mut engine = Engine::new(InputMethod::Telex, true);
    let result = simulate_typing_str(
        &mut engine,
        "V n i k e y Space v 1 . 0 Space - Space b o o j Space g o x Space t i e e n g s Space V i e e t j Space c h o Space L i n u x",
    );
    assert_eq!(result, "Vnikey v1.0 - bộ gõ tiếng Việt cho Linux");
}

#[test]
fn test_question_mark_after_vietnamese() {
    let mut engine = Engine::new(InputMethod::Telex, true);
    let result = simulate_typing_str(
        &mut engine,
        "b a n j Space c o o j Space k h o e r Space k h o o n g ?",
    );
    assert_eq!(result, "bạn cộ khoẻ không?");
}

#[test]
fn test_exclamation_after_toned_word() {
    let mut engine = Engine::new(InputMethod::Telex, true);
    let result = simulate_typing_str(&mut engine, "v a n g f !");
    assert_eq!(result, "vàng!");
}

#[test]
fn test_multiline_input() {
    // Enter key mid sentence
    let mut engine = Engine::new(InputMethod::Telex, true);
    let result = simulate_typing_str(&mut engine, "d d o n g j Enter t a y j");
    assert_eq!(result, "đọng\ntạy");
}

#[test]
fn test_tab_key_passthrough() {
    let mut engine = Engine::new(InputMethod::Telex, true);
    let result = simulate_typing_str(&mut engine, "a n h Tab e m");
    // Tab when idle = PassThrough → '\t' appended
    // But actually Tab triggers while composing "anh": CommitAndPassThrough("anh") + '\t'
    assert_eq!(result, "anh\tem");
}
