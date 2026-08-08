use vnikey_core::engine::{Action, Engine, InputMethod};

pub fn simulate_typing(engine: &mut Engine, keystrokes: &[&str]) -> String {
    let mut committed_text = String::new();
    let mut preedit_text = String::new();

    for &key in keystrokes {
        if key == "[[ToggleTelex]]" {
            if let Some(action) = engine.set_input_method(InputMethod::Telex) {
                match action {
                    Action::Commit(buf) | Action::CommitAndPassThrough(buf) => {
                        let buf_str: String = buf.as_slice().iter().collect();
                        committed_text.push_str(&buf_str);
                        preedit_text.clear();
                    }
                    _ => {}
                }
            }
            continue;
        }

        if key == "[[ToggleVni]]" {
            if let Some(action) = engine.set_input_method(InputMethod::Vni) {
                match action {
                    Action::Commit(buf) | Action::CommitAndPassThrough(buf) => {
                        let buf_str: String = buf.as_slice().iter().collect();
                        committed_text.push_str(&buf_str);
                        preedit_text.clear();
                    }
                    _ => {}
                }
            }
            continue;
        }

        let c = match key {
            "BackSpace" => '\x08',
            "Space" => ' ',
            "Enter" => '\n',
            "Tab" => '\t',
            "Escape" => '\x1b',
            k if k.len() == 1 => k.chars().next().unwrap(),
            _ => panic!("Unknown key: {}", key),
        };

        let action = engine.process_key(c);
        match action {
            Action::Preedit(buf) => {
                preedit_text = buf.as_slice().iter().collect();
            }
            Action::Commit(buf) => {
                let buf_str: String = buf.as_slice().iter().collect();
                committed_text.push_str(&buf_str);
                preedit_text.clear();
            }
            Action::CommitAndPassThrough(buf) => {
                let buf_str: String = buf.as_slice().iter().collect();
                committed_text.push_str(&buf_str);
                preedit_text.clear();

                // Pass-through
                if c != '\x08' {
                    committed_text.push(c);
                } else if !committed_text.is_empty() {
                    committed_text.pop();
                }
            }
            Action::PassThrough => {
                if c != '\x08' {
                    committed_text.push(c);
                } else if !committed_text.is_empty() {
                    committed_text.pop();
                }
            }
        }
    }

    format!("{}{}", committed_text, preedit_text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use vnikey_core::engine::{Engine, InputMethod};

    #[test]
    fn test_scenario_1_perfect_typing_telex() {
        let mut engine = Engine::new(InputMethod::Telex, false);
        let keystrokes = [
            "n", "g", "h", "i", "e", "e", "n", "g", "Space", "n", "u", "w", "o", "w", "c", "s",
            "Space", "n", "g", "h", "i", "e", "e", "n", "g", "Space", "t", "h", "a", "n", "h", "f",
        ];
        let result = simulate_typing(&mut engine, &keystrokes);
        assert_eq!(result, "nghiêng nước nghiêng thành");
    }

    #[test]
    fn test_scenario_1_perfect_typing_vni() {
        let mut engine = Engine::new(InputMethod::Vni, false);
        let keystrokes = [
            "n", "g", "h", "i", "e", "6", "n", "g", "Space", "n", "u", "7", "o", "7", "c", "1",
            "Space", "n", "g", "h", "i", "e", "6", "n", "g", "Space", "t", "h", "a", "n", "h", "2",
        ];
        let result = simulate_typing(&mut engine, &keystrokes);
        assert_eq!(result, "nghiêng nước nghiêng thành");
    }

    #[test]
    fn test_scenario_2_typo_and_correction_telex() {
        let mut engine = Engine::new(InputMethod::Telex, false);
        let keystrokes = ["n", "g", "i", "BackSpace", "h", "i", "e", "e", "n", "g"];
        let result = simulate_typing(&mut engine, &keystrokes);
        assert_eq!(result, "nghiêng");
    }

    #[test]
    fn test_scenario_2_typo_and_correction_vni() {
        let mut engine = Engine::new(InputMethod::Vni, false);
        let keystrokes = ["n", "g", "i", "BackSpace", "h", "i", "e", "6", "n", "g"];
        let result = simulate_typing(&mut engine, &keystrokes);
        assert_eq!(result, "nghiêng");
    }

    #[test]
    fn test_scenario_3_number_interruption_telex() {
        let mut engine = Engine::new(InputMethod::Telex, false);
        let keystrokes = [
            "h", "o", "a", "n", "g", "f", "Space", "1", "2", "3", "Space", "a", "n", "h",
        ];
        let result = simulate_typing(&mut engine, &keystrokes);
        assert_eq!(result, "hoàng 123 anh");
    }

    #[test]
    fn test_scenario_3_number_interruption_vni() {
        let mut engine = Engine::new(InputMethod::Vni, false);
        let keystrokes = [
            "h", "o", "a", "n", "g", "2", "Space", "1", "2", "3", "Space", "a", "n", "h",
        ];
        let result = simulate_typing(&mut engine, &keystrokes);
        assert_eq!(result, "hoàng 123 anh");
    }

    #[test]
    fn test_scenario_3_special_character_interruption() {
        let mut engine = Engine::new(InputMethod::Telex, false);
        let keystrokes = ["h", "o", "a", "n", "g", "f", ",", " ", "a", "n", "h"];
        let result = simulate_typing(&mut engine, &keystrokes);
        assert_eq!(result, "hoàng, anh");
    }

    #[test]
    fn test_scenario_4_rapid_mode_switching() {
        let mut engine = Engine::new(InputMethod::Telex, false);
        let keystrokes = [
            "v",
            "i",
            "e",
            "e",
            "t",
            "j",
            "Space",
            "[[ToggleVni]]",
            "n",
            "a",
            "m",
        ];
        let result = simulate_typing(&mut engine, &keystrokes);
        assert_eq!(result, "việt nam");
    }

    #[test]
    fn test_scenario_5_redundant_modifier_keys_telex() {
        let mut engine = Engine::new(InputMethod::Telex, false);
        let keystrokes = ["a", "a", "w", "o", "o", "o"];
        // a + a -> â
        // â + w -> âư
        // âư + o -> âưo
        // âưo + o -> âưô
        // âưô + o -> âưoo
        let result = simulate_typing(&mut engine, &keystrokes);
        assert_eq!(result, "âưoo");
    }

    #[test]
    fn test_scenario_5_redundant_modifier_keys_telex_2() {
        let mut engine = Engine::new(InputMethod::Telex, false);
        let keystrokes = ["h", "o", "a", "s", "s"];
        // hoas -> hóa
        // hóa + s -> hoas
        let result = simulate_typing(&mut engine, &keystrokes);
        assert_eq!(result, "hoas");
    }

    #[test]
    fn test_scenario_6_w_at_beginning_telex() {
        let mut engine = Engine::new(InputMethod::Telex, false);
        let keystrokes = ["w", "i", "f", "i"];
        let result = simulate_typing(&mut engine, &keystrokes);
        assert_eq!(result, "wifi");
    }

    #[test]
    fn test_scenario_7_repeated_modifiers_telex() {
        let mut engine = Engine::new(InputMethod::Telex, false);
        let keystrokes = ["o", "o", "o"];
        let result = simulate_typing(&mut engine, &keystrokes);
        assert_eq!(result, "oo");

        let mut engine = Engine::new(InputMethod::Telex, false);
        let keystrokes = ["a", "w", "w"];
        let result = simulate_typing(&mut engine, &keystrokes);
        assert_eq!(result, "aw");
    }

    #[test]
    fn test_scenario_8_double_tones_telex() {
        let mut engine = Engine::new(InputMethod::Telex, false);
        let keystrokes = ["a", "s", "s"];
        let result = simulate_typing(&mut engine, &keystrokes);
        assert_eq!(result, "as");

        let mut engine = Engine::new(InputMethod::Telex, false);
        let keystrokes = ["o", "o", "f", "f"];
        let result = simulate_typing(&mut engine, &keystrokes);
        assert_eq!(result, "ôf");
    }

    #[test]
    fn test_scenario_9_extreme_vowels_telex() {
        let mut engine = Engine::new(InputMethod::Telex, false);
        let keystrokes = ["g", "i", "u", "w", "o", "w", "n", "g", "f"];
        let result = simulate_typing(&mut engine, &keystrokes);
        assert_eq!(result, "giường");

        let mut engine = Engine::new(InputMethod::Telex, false);
        let keystrokes = ["q", "u", "y", "e", "e", "n", "r"];
        let result = simulate_typing(&mut engine, &keystrokes);
        assert_eq!(result, "quyển");

        let mut engine = Engine::new(InputMethod::Telex, false);
        let keystrokes = ["q", "u", "y", "e", "e", "n", "x"];
        let result = simulate_typing(&mut engine, &keystrokes);
        assert_eq!(result, "quyễn");

        let mut engine = Engine::new(InputMethod::Telex, false);
        let keystrokes = ["t", "h", "u", "o", "w", "r"];
        let result = simulate_typing(&mut engine, &keystrokes);
        assert_eq!(result, "thuở");

        let mut engine = Engine::new(InputMethod::Telex, false);
        let keystrokes = ["n", "g", "u", "y", "e", "e", "n", "x"];
        let result = simulate_typing(&mut engine, &keystrokes);
        assert_eq!(result, "nguyễn");
    }

    #[test]
    fn test_scenario_9_extreme_vowels_vni() {
        let mut engine = Engine::new(InputMethod::Vni, false);
        let keystrokes = ["g", "i", "u", "7", "o", "7", "n", "g", "2"];
        let result = simulate_typing(&mut engine, &keystrokes);
        assert_eq!(result, "giường");

        let mut engine = Engine::new(InputMethod::Vni, false);
        let keystrokes = ["q", "u", "y", "e", "6", "n", "3"];
        let result = simulate_typing(&mut engine, &keystrokes);
        assert_eq!(result, "quyển");

        let mut engine = Engine::new(InputMethod::Vni, false);
        let keystrokes = ["t", "h", "u", "o", "7", "3"];
        let result = simulate_typing(&mut engine, &keystrokes);
        // VNI 7 modifies both u and o, producing thưở.
        assert_eq!(result, "thưở");

        let mut engine = Engine::new(InputMethod::Vni, false);
        let keystrokes = ["n", "g", "u", "y", "e", "6", "n", "4"];
        let result = simulate_typing(&mut engine, &keystrokes);
        assert_eq!(result, "nguyễn");
    }

    #[test]
    fn test_scenario_10_mixed_case() {
        let mut engine = Engine::new(InputMethod::Telex, false);
        let keystrokes = ["H", "o", "a", "n", "g", "f"];
        let result = simulate_typing(&mut engine, &keystrokes);
        assert_eq!(result, "Hoàng");

        let mut engine = Engine::new(InputMethod::Telex, false);
        let keystrokes = ["V", "I", "E", "E", "T", "J", "Space", "N", "A", "M"];
        let result = simulate_typing(&mut engine, &keystrokes);
        assert_eq!(result, "VIỆT NAM");

        let mut engine = Engine::new(InputMethod::Vni, false);
        let keystrokes = ["H", "o", "a", "n", "g", "2"];
        let result = simulate_typing(&mut engine, &keystrokes);
        assert_eq!(result, "Hoàng");

        let mut engine = Engine::new(InputMethod::Vni, false);
        let keystrokes = ["V", "I", "E", "6", "T", "5", "Space", "N", "A", "M"];
        let result = simulate_typing(&mut engine, &keystrokes);
        assert_eq!(result, "VIỆT NAM");
    }

    #[test]
    fn test_scenario_11_backspace_chaos() {
        let mut engine = Engine::new(InputMethod::Telex, false);
        let keystrokes = ["n", "g", "u", "u", "BackSpace", "y", "e", "e", "n", "x"];
        let result = simulate_typing(&mut engine, &keystrokes);
        assert_eq!(result, "nguyễn");

        let mut engine = Engine::new(InputMethod::Telex, false);
        let keystrokes = ["q", "u", "y", "e", "e", "n", "x"];
        let result = simulate_typing(&mut engine, &keystrokes);
        assert_eq!(result, "quyễn");

        let mut engine = Engine::new(InputMethod::Telex, false);
        // 6 backspaces to delete 6 characters
        let keystrokes = [
            "h",
            "o",
            "a",
            "n",
            "g",
            "f",
            "BackSpace",
            "BackSpace",
            "BackSpace",
            "BackSpace",
            "BackSpace",
            "BackSpace",
            "a",
            "n",
            "h",
        ];
        let result = simulate_typing(&mut engine, &keystrokes);
        assert_eq!(result, "anh");

        let mut engine = Engine::new(InputMethod::Vni, false);
        let keystrokes = ["n", "g", "u", "u", "BackSpace", "y", "e", "6", "n", "4"];
        let result = simulate_typing(&mut engine, &keystrokes);
        assert_eq!(result, "nguyễn");

        let mut engine = Engine::new(InputMethod::Vni, false);
        // 6 backspaces to delete 6 characters
        let keystrokes = [
            "h",
            "o",
            "a",
            "n",
            "g",
            "2",
            "BackSpace",
            "BackSpace",
            "BackSpace",
            "BackSpace",
            "BackSpace",
            "BackSpace",
            "a",
            "n",
            "h",
        ];
        let result = simulate_typing(&mut engine, &keystrokes);
        assert_eq!(result, "anh");
    }

    #[test]
    fn test_scenario_12_non_vietnamese_gibberish() {
        let mut engine = Engine::new(InputMethod::Telex, false);
        let keystrokes = ["j", "j", "j", "j", "j", "j", "j", "j"];
        let result = simulate_typing(&mut engine, &keystrokes);
        assert_eq!(result, "jjjjjjjj");
    }

    #[test]
    fn test_scenario_13_vni_number_chaos() {
        let mut engine = Engine::new(InputMethod::Vni, false);
        let keystrokes = ["a", "1", "1"];
        let result = simulate_typing(&mut engine, &keystrokes);
        assert_eq!(result, "a1");

        let mut engine = Engine::new(InputMethod::Vni, false);
        let keystrokes = ["c", "o", "n", "g", "t", "y", "1", "2", "3"];
        let result = simulate_typing(&mut engine, &keystrokes);
        assert_eq!(result, "congty123");
    }
}

#[test]
fn test_scenario_14_smart_spell_check_foreign_words() {
    let mut engine = Engine::new(InputMethod::Telex, true);
    let keys = vec![
        "v", "a", "m", "p", "i", "r", "e", " ", // vampire
        "d", "i", "r", "e", "c", "t", "o", "r", " ", // director
        "l", "i", "n", "u", "x", " ", // linux
        "o", "o", "f", "f", " ", // ooff
    ];

    let result = simulate_typing(&mut engine, &keys);
    assert_eq!(result, "vampire director linux ooff ");
}
