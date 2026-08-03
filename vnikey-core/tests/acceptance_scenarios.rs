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
        let mut engine = Engine::new(InputMethod::Telex);
        let keystrokes = [
            "n", "g", "h", "i", "e", "e", "n", "g", "Space", "n", "u", "w", "o", "w", "c", "s",
            "Space", "n", "g", "h", "i", "e", "e", "n", "g", "Space", "t", "h", "a", "n", "h", "f",
        ];
        let result = simulate_typing(&mut engine, &keystrokes);
        assert_eq!(result, "nghiêng nước nghiêng thành");
    }

    #[test]
    fn test_scenario_1_perfect_typing_vni() {
        let mut engine = Engine::new(InputMethod::Vni);
        let keystrokes = [
            "n", "g", "h", "i", "e", "6", "n", "g", "Space", "n", "u", "7", "o", "7", "c", "1",
            "Space", "n", "g", "h", "i", "e", "6", "n", "g", "Space", "t", "h", "a", "n", "h", "2",
        ];
        let result = simulate_typing(&mut engine, &keystrokes);
        assert_eq!(result, "nghiêng nước nghiêng thành");
    }

    #[test]
    fn test_scenario_2_typo_and_correction_telex() {
        let mut engine = Engine::new(InputMethod::Telex);
        let keystrokes = ["n", "g", "i", "BackSpace", "h", "i", "e", "e", "n", "g"];
        let result = simulate_typing(&mut engine, &keystrokes);
        assert_eq!(result, "nghiêng");
    }

    #[test]
    fn test_scenario_2_typo_and_correction_vni() {
        let mut engine = Engine::new(InputMethod::Vni);
        let keystrokes = ["n", "g", "i", "BackSpace", "h", "i", "e", "6", "n", "g"];
        let result = simulate_typing(&mut engine, &keystrokes);
        assert_eq!(result, "nghiêng");
    }

    #[test]
    fn test_scenario_3_number_interruption_telex() {
        let mut engine = Engine::new(InputMethod::Telex);
        let keystrokes = [
            "h", "o", "a", "n", "g", "f", "Space", "1", "2", "3", "Space", "a", "n", "h",
        ];
        let result = simulate_typing(&mut engine, &keystrokes);
        assert_eq!(result, "hoàng 123 anh");
    }

    #[test]
    fn test_scenario_3_number_interruption_vni() {
        let mut engine = Engine::new(InputMethod::Vni);
        let keystrokes = [
            "h", "o", "a", "n", "g", "2", "Space", "1", "2", "3", "Space", "a", "n", "h",
        ];
        let result = simulate_typing(&mut engine, &keystrokes);
        assert_eq!(result, "hoàng 123 anh");
    }

    #[test]
    fn test_scenario_3_special_character_interruption() {
        let mut engine = Engine::new(InputMethod::Telex);
        let keystrokes = ["h", "o", "a", "n", "g", "f", ",", " ", "a", "n", "h"];
        let result = simulate_typing(&mut engine, &keystrokes);
        assert_eq!(result, "hoàng, anh");
    }

    #[test]
    fn test_scenario_4_rapid_mode_switching() {
        let mut engine = Engine::new(InputMethod::Telex);
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
        let mut engine = Engine::new(InputMethod::Telex);
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
        let mut engine = Engine::new(InputMethod::Telex);
        let keystrokes = ["h", "o", "a", "s", "s"];
        // hoas -> hóa
        // hóa + s -> hoas
        let result = simulate_typing(&mut engine, &keystrokes);
        assert_eq!(result, "hoas");
    }
}
