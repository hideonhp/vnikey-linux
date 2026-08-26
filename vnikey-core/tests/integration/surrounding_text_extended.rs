use crate::common::simulate_typing_str;
use vnikey_core::engine::{Engine, InputMethod};

// === SURROUNDING TEXT: VNI MODE ===

#[test]
fn test_surrounding_vni_basic() {
    // Commit "hoang" (raw vni: "hoang2") → backspace → recompose
    let mut engine = Engine::new(InputMethod::Vni, false);
    let result = simulate_typing_str(&mut engine, "h o a n g 2 Space BackSpace");
    assert_eq!(result, "hoang");
}

#[test]
fn test_surrounding_vni_add_tone_after() {
    // VNI: commit "tien" → backspace → type "g" → result "tieng" preedit
    let mut engine = Engine::new(InputMethod::Vni, false);
    let result = simulate_typing_str(&mut engine, "t i e n Space BackSpace g");
    assert_eq!(result, "tieg");
}

#[test]
fn test_surrounding_toned_word_telex() {
    // Commit "hoà" (raw: "hoaf") → backspace → should recompose "hoa" → type "s" → "hoá"
    let mut engine = Engine::new(InputMethod::Telex, false);
    let result = simulate_typing_str(&mut engine, "h o a f Space BackSpace s");
    assert_eq!(result, "hoá");
}

#[test]
fn test_surrounding_toned_word_vni() {
    // Commit "hoà" (raw: "hoang2" wait no — "hoaf" is telex)
    // VNI: "hoa2" → commit "hoà" → backspace → raw "hoa" popped last → "ho" → preedit "ho"
    // Then type "a2" → commit "hoà"
    let mut engine = Engine::new(InputMethod::Vni, false);
    let result = simulate_typing_str(&mut engine, "h o a 2 Space BackSpace a 2 Space");
    assert_eq!(result, "hoàa ");
}

#[test]
fn test_surrounding_multiple_commits_only_last() {
    // Commit word 1, commit word 2, then backspace only recomposes word 2
    let mut engine = Engine::new(InputMethod::Telex, false);
    let result = simulate_typing_str(&mut engine, "a n h Space h o a n g f Space BackSpace");
    assert_eq!(result, "anh hoang");
}

#[test]
fn test_surrounding_then_second_commit_clears() {
    // After surrounding recompose, commit new word, then backspace again = recompose NEW word
    let mut engine = Engine::new(InputMethod::Telex, false);
    let result = simulate_typing_str(&mut engine, "t i e n g Space BackSpace g s Space BackSpace");
    assert_eq!(result, "tieng");
}

#[test]
fn test_surrounding_enter_as_commit() {
    // Enter also commits, but it should clear context, so backspace just deletes newline
    let mut engine = Engine::new(InputMethod::Telex, false);
    let result = simulate_typing_str(&mut engine, "x i n Enter BackSpace");
    assert_eq!(result, "xi");
}

#[test]
fn test_surrounding_commit_then_space_then_backspace() {
    // Gõ từ, space, rồi thêm backspace 2 lần
    let mut engine = Engine::new(InputMethod::Telex, false);
    let result = simulate_typing_str(&mut engine, "a n h Space BackSpace BackSpace");
    assert_eq!(result, "a");
}

#[test]
fn test_surrounding_backspace_all_and_retype() {
    // Xóa hết từ cũ bằng surrounding rồi gõ lại
    let mut engine = Engine::new(InputMethod::Telex, false);
    let result = simulate_typing_str(&mut engine, "a Space BackSpace BackSpace h o a Space");
    // commit "a " → BackSpace1: SurroundingRecompose(delete_count=1, preedit="") empty
    //   committed "a " minus 1 = " ", preedit = ""
    // BackSpace2: now Idle + last_committed_raw cleared → PassThrough
    //   simulate_typing PassThrough for '\x08': pop from committed_text " " → ""
    // type "hoa" → preedit "hoa", Space → commit "hoa "
    // result = "hoa "
    assert_eq!(result, "hoa ");
}

#[test]
fn test_surrounding_vni_toned_complex() {
    // VNI: "nguye6n4" → "nguyễn" → backspace → recompose "nguyễ" (pop 'n' from raw)
    let mut engine = Engine::new(InputMethod::Vni, false);
    let result = simulate_typing_str(&mut engine, "n g u y e 6 n 4 Space BackSpace");
    assert_eq!(result, "nguyên");
}

#[test]
fn test_surrounding_spell_check_true() {
    // With spell_check = true, surrounding still works
    let mut engine = Engine::new(InputMethod::Telex, true);
    let result = simulate_typing_str(&mut engine, "h o a n g f Space BackSpace");
    assert_eq!(result, "hoang");
}

#[test]
fn test_surrounding_space_between_words_not_affected() {
    // The space in between two words (already committed PassThrough) is not part of surrounding
    // "anh " is committed as Commit (with space in buffer), not separate PassThrough
    // But "anh" then comma "," would CommitAndPassThrough then PassThrough for ","
    let mut engine = Engine::new(InputMethod::Telex, false);
    let result = simulate_typing_str(&mut engine, "a n h , Space BackSpace");
    assert_eq!(result, "anh,");
}

// ============================================================
// EXTENDED COVERAGE: Edge cases & interaction patterns
// ============================================================

#[test]
fn test_surrounding_double_backspace_then_composing() {
    // First BackSpace → SurroundingRecompose path (Idle → Composing)
    // Second BackSpace → regular Composing pop path
    // "hoan " → BS1: SurroundingRecompose("hoa", delete=5) → BS2: Composing pop → "ho"
    let mut engine = Engine::new(InputMethod::Telex, false);
    let result = simulate_typing_str(&mut engine, "h o a n Space BackSpace BackSpace");
    assert_eq!(result, "ho");
}

#[test]
fn test_surrounding_single_char_word() {
    // Single char committed word: pop 'a' → raw empty → SurroundingRecompose(empty, 2)
    // committed "a " → delete 2 → "" ; preedit = ""
    let mut engine = Engine::new(InputMethod::Telex, false);
    let result = simulate_typing_str(&mut engine, "a Space BackSpace");
    assert_eq!(result, "");
}

#[test]
fn test_surrounding_exhausts_then_passthrough() {
    // BS1: pop 'h' → raw empty → SurroundingRecompose(empty, 2) → committed ""
    // BS2: Idle + last_committed_raw empty → PassThrough → pop from "" → nothing
    let mut engine = Engine::new(InputMethod::Telex, false);
    let result = simulate_typing_str(&mut engine, "h Space BackSpace BackSpace");
    assert_eq!(result, "");
}

#[test]
fn test_surrounding_continue_typing_no_commit() {
    // After surrounding recompose → still Composing → handle_char continues normally
    // "hoan " → BS: SurroundingRecompose("hoa", 5), raw=['h','o','a'] (Composing)
    // type 'g': raw=['h','o','a','g'] → preedit "hoag"
    let mut engine = Engine::new(InputMethod::Telex, false);
    let result = simulate_typing_str(&mut engine, "h o a n Space BackSpace g");
    assert_eq!(result, "hoag");
}

#[test]
fn test_surrounding_telex_add_coda_after_recompose() {
    // "hoa " → BS: pop 'a' → raw=['h','o'] → SurroundingRecompose("ho", 4) → Composing
    // type 'n': raw=['h','o','n'] → "hon" preedit
    let mut engine = Engine::new(InputMethod::Telex, false);
    let result = simulate_typing_str(&mut engine, "h o a Space BackSpace n");
    assert_eq!(result, "hon");
}

#[test]
fn test_surrounding_telex_tone_stripped_on_recompose() {
    // Commit "hoàn " (raw "hoafn") → BS: pop 'n' → raw=['h','o','a','f']
    // rebuild "hoà" (tone stays on remaining raw) → SurroundingRecompose("hoà", 5)
    // delete_count = len("hoàn ") = 5 chars (h,o,à,n,space)
    let mut engine = Engine::new(InputMethod::Telex, false);
    let result = simulate_typing_str(&mut engine, "h o a f n Space BackSpace");
    assert_eq!(result, "hoà");
}

#[test]
fn test_surrounding_telex_double_modifier_recompose() {
    // Commit "tân " (raw "taan") → BS: pop 'n' → raw=['t','a','a']
    // rebuild "tâ" (aa → â modifier preserved) → SurroundingRecompose("tâ", 4)
    // delete_count = len("tân ") = 4 chars (t,â,n,space)
    let mut engine = Engine::new(InputMethod::Telex, false);
    let result = simulate_typing_str(&mut engine, "t a a n Space BackSpace");
    assert_eq!(result, "tâ");
}

#[test]
fn test_surrounding_vni_modifier6_recompose() {
    // VNI: commit "hô " (raw "ho6") → BS: pop '6' → raw=['h','o']
    // rebuild "ho" → SurroundingRecompose("ho", 3)
    // type '6': raw=['h','o','6'] → "hô" → Space → commit "hô "
    let mut engine = Engine::new(InputMethod::Vni, false);
    let result = simulate_typing_str(&mut engine, "h o 6 Space BackSpace 6 Space");
    assert_eq!(result, "hô ");
}

#[test]
fn test_surrounding_vni_smart_w_recompose() {
    // VNI: "thuong7" → "thương" → commit "thương " (raw=['t','h','u','o','n','g','7'])
    // BS: pop '7' → raw=['t','h','u','o','n','g'] → rebuild "thuong" (no '7' = no ơ modifier)
    // SurroundingRecompose("thuong", 7) — delete_count=7: t,h,ư,ơ,n,g,space
    let mut engine = Engine::new(InputMethod::Vni, false);
    let result = simulate_typing_str(&mut engine, "t h u o n g 7 Space BackSpace");
    assert_eq!(result, "thuong");
}

#[test]
fn test_surrounding_commit_recomposed_word() {
    // Full round-trip: commit → recompose → add tone back → commit again
    // "hoàn " → BS: pop 'f' → "hoan" preedit → type 'f' → "hoàn" → Space → commit "hoàn "
    // Final committed_text = "hoàn "
    let mut engine = Engine::new(InputMethod::Telex, false);
    let result = simulate_typing_str(&mut engine, "h o a n f Space BackSpace f Space");
    assert_eq!(result, "hoàn ");
}

#[test]
fn test_surrounding_spell_check_invalid_recompose() {
    // spell_check=true: "xabc" is invalid → raw fallback → committed "xabc "
    // BS: pop 'c' → raw=['x','a','b'] → rebuild: "xab" also invalid → raw fallback "xab"
    // SurroundingRecompose("xab", 5), delete_count=5: x,a,b,c,space
    let mut engine = Engine::new(InputMethod::Telex, true);
    let result = simulate_typing_str(&mut engine, "x a b c Space BackSpace");
    assert_eq!(result, "xab");
}

