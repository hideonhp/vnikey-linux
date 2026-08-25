
use crate::common::simulate_typing_str;
use vnikey_core::engine::{Engine, InputMethod};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_khuyu_thuy_dynamic_tone_shifting() {
        // Ensures that typing "k h u y r u" shifts tone correctly from y to u when necessary.
        // Mechanical check
        let mut engine = Engine::new(InputMethod::Telex, false);
        assert_eq!(simulate_typing_str(&mut engine, "k h u y r u"), "khuỷu");

        let mut engine_vni = Engine::new(InputMethod::Vni, false);
        assert_eq!(simulate_typing_str(&mut engine_vni, "k h u y 3 u"), "khuỷu");

        // Phonetic check
        let mut engine_spell_check = Engine::new(InputMethod::Telex, true);
        assert_eq!(
            simulate_typing_str(&mut engine_spell_check, "k h u y r u"),
            "khuỷu"
        );

        let mut engine_vni_spell_check = Engine::new(InputMethod::Vni, true);
        assert_eq!(
            simulate_typing_str(&mut engine_vni_spell_check, "k h u y 3 u"),
            "khuỷu"
        );

        // And "t h u y r" -> "thủy"
        let mut engine2 = Engine::new(InputMethod::Telex, false);
        assert_eq!(simulate_typing_str(&mut engine2, "t h u y r"), "thủy");
    }

    #[test]
    fn test_modifier_overriding_bug() {
        // Ensures modifier 'w' overrides '^' without appending 'ư'
        // Mechanical check
        let mut engine = Engine::new(InputMethod::Telex, false);
        assert_eq!(simulate_typing_str(&mut engine, "a a w o o o"), "ăoo");

        // Phonetic check with trailing space
        let mut engine_spell_check = Engine::new(InputMethod::Telex, true);
        assert_eq!(
            simulate_typing_str(&mut engine_spell_check, "a a w o o o Space"),
            "aawooo "
        );
    }

    #[test]
    fn test_vowel_appending_bug() {
        // Ensures "l u w u" -> "lưu" correctly without issues
        // Mechanical check
        let mut engine = Engine::new(InputMethod::Telex, false);
        assert_eq!(simulate_typing_str(&mut engine, "l u w u"), "lưu");

        // Phonetic check
        let mut engine_spell_check = Engine::new(InputMethod::Telex, true);
        assert_eq!(
            simulate_typing_str(&mut engine_spell_check, "l u w u"),
            "lưu"
        );
    }
}
