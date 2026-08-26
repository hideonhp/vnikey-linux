use crate::common::{TestCase, simulate_typing_str};
use vnikey_core::engine::{Engine, InputMethod};

// === TELEX 'z' TONE CANCEL TESTS ===
//
// 'z' trong Telex: xóa TOÀN BỘ tone trong buffer hiện tại.
// Nếu buffer không có tone nào → push literal 'z'.

// -------------------------------------------------------
// Group A: Xóa từng loại tone (6 dấu)
// -------------------------------------------------------

#[test]
fn test_z_cancels_sac() {
    // s = sắc (́), z clears it → raw chars without tone
    let mut engine = Engine::new(InputMethod::Telex, false);
    let result = simulate_typing_str(&mut engine, "a s z");
    assert_eq!(result, "a");
}

#[test]
fn test_z_cancels_huyen() {
    // f = huyền (̀)
    let mut engine = Engine::new(InputMethod::Telex, false);
    let result = simulate_typing_str(&mut engine, "a f z");
    assert_eq!(result, "a");
}

#[test]
fn test_z_cancels_hoi() {
    // r = hỏi (̉)
    let mut engine = Engine::new(InputMethod::Telex, false);
    let result = simulate_typing_str(&mut engine, "a r z");
    assert_eq!(result, "a");
}

#[test]
fn test_z_cancels_nga() {
    // x = ngã (~)
    let mut engine = Engine::new(InputMethod::Telex, false);
    let result = simulate_typing_str(&mut engine, "a x z");
    assert_eq!(result, "a");
}

#[test]
fn test_z_cancels_nang() {
    // j = nặng (.)
    let mut engine = Engine::new(InputMethod::Telex, false);
    let result = simulate_typing_str(&mut engine, "a j z");
    assert_eq!(result, "a");
}

// -------------------------------------------------------
// Group B: z trên từ nhiều âm tiết (tone nằm giữa buffer)
// -------------------------------------------------------

#[test]
fn test_z_clears_tone_on_vowel_cluster() {
    // "toán" (toans) → z → "toan"
    let mut engine = Engine::new(InputMethod::Telex, false);
    let result = simulate_typing_str(&mut engine, "t o a n s z");
    assert_eq!(result, "toan");
}

#[test]
fn test_z_clears_tone_multichar_word() {
    // "hoàng" (hoafng) → z → "hoang"
    let mut engine = Engine::new(InputMethod::Telex, false);
    let result = simulate_typing_str(&mut engine, "h o a f n g z");
    assert_eq!(result, "hoang");
}

#[test]
fn test_z_clears_tone_with_circumflex() {
    // "hôm" (hoom) → add tone → z strips it
    // "hốm" (hooms) → z → "hôm" (circumflex stays, tone cleared)
    let mut engine = Engine::new(InputMethod::Telex, false);
    let result = simulate_typing_str(&mut engine, "h o o m s z");
    assert_eq!(result, "hôm");
}

#[test]
fn test_z_clears_tone_with_horn() {
    // "ướt" (uwots) → z → "ươt" (horn modifier stays, tone cleared)
    let mut engine = Engine::new(InputMethod::Telex, false);
    let result = simulate_typing_str(&mut engine, "u w o t s z");
    assert_eq!(result, "ươt");
}

// -------------------------------------------------------
// Group C: Nhiều lần z liên tiếp
// -------------------------------------------------------

#[test]
fn test_z_twice_no_tone() {
    // "a" → z (no tone → push 'z') → z lại trên "az" (no tone → push 'z') → "azz"
    let mut engine = Engine::new(InputMethod::Telex, false);
    let result = simulate_typing_str(&mut engine, "a z z");
    assert_eq!(result, "azz");
}

#[test]
fn test_z_after_cancel_then_z_push() {
    // "as" → sắc → "á", "áz" → cancel → "a", "az" → no tone → push 'z' → "az"
    let mut engine = Engine::new(InputMethod::Telex, false);
    let result = simulate_typing_str(&mut engine, "a s z z");
    assert_eq!(result, "az");
}

// -------------------------------------------------------
// Group D: z khi không có tone → push literal 'z'
// -------------------------------------------------------

#[test]
fn test_z_push_literal_no_tone() {
    // "an" có no tone → z push literal
    let mut engine = Engine::new(InputMethod::Telex, false);
    let result = simulate_typing_str(&mut engine, "a n z");
    assert_eq!(result, "anz");
}

#[test]
fn test_z_push_literal_at_idle() {
    // Idle state: z is plain PassThrough (not part of composing)
    let mut engine = Engine::new(InputMethod::Telex, false);
    let result = simulate_typing_str(&mut engine, "a n h Space z");
    assert_eq!(result, "anh z");
}

#[test]
fn test_z_standalone() {
    // Bắt đầu gõ bằng z từ Idle → 'z' là ký tự alphabetic → Composing → push 'z'
    let mut engine = Engine::new(InputMethod::Telex, false);
    let result = simulate_typing_str(&mut engine, "z i p");
    assert_eq!(result, "zip");
}

// -------------------------------------------------------
// Group E: z + commit / z + spell check
// -------------------------------------------------------

#[test]
fn test_z_then_commit() {
    // "toàn" → z → "toan" → Space → commit "toan "
    let mut engine = Engine::new(InputMethod::Telex, false);
    let result = simulate_typing_str(&mut engine, "t o a n f z Space");
    assert_eq!(result, "toan ");
}

#[test]
fn test_z_spell_check_enabled() {
    // spell_check=true: "thắng" → z → "thang" — vẫn valid syllable
    let mut engine = Engine::new(InputMethod::Telex, true);
    let result = simulate_typing_str(&mut engine, "t h a s n g z Space");
    assert_eq!(result, "thang ");
}

#[test]
fn test_z_tone_switch_then_cancel() {
    // Override tone rồi z: "toáns" → f → "toàn" → z → "toan"
    let mut engine = Engine::new(InputMethod::Telex, false);
    let result = simulate_typing_str(&mut engine, "t o a n s f z");
    assert_eq!(result, "toan");
}
