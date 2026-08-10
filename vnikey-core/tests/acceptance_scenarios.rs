use vnikey_core::engine::{Action, Engine, InputMethod};

pub struct TestCase<'a> {
    pub input: &'a str,
    pub expected: &'a str,
}

pub fn simulate_typing_str(engine: &mut Engine, keystrokes_str: &str) -> String {
    let keys: Vec<&str> = keystrokes_str.split_whitespace().collect();
    simulate_typing(engine, &keys)
}

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
    fn test_perfect_typing() {
        let cases_telex = vec![TestCase {
            input: "n g h i e e n g Space n u w o w c s Space n g h i e e n g Space t h a n h f",
            expected: "nghiêng nước nghiêng thành",
        }];
        for case in cases_telex {
            let mut engine = Engine::new(InputMethod::Telex, false);
            assert_eq!(simulate_typing_str(&mut engine, case.input), case.expected);
        }

        let cases_vni = vec![TestCase {
            input: "n g h i e 6 n g Space n u 7 o 7 c 1 Space n g h i e 6 n g Space t h a n h 2",
            expected: "nghiêng nước nghiêng thành",
        }];
        for case in cases_vni {
            let mut engine = Engine::new(InputMethod::Vni, false);
            assert_eq!(simulate_typing_str(&mut engine, case.input), case.expected);
        }
    }

    #[test]
    fn test_typo_and_correction() {
        let cases_telex = vec![TestCase {
            input: "n g i BackSpace h i e e n g",
            expected: "nghiêng",
        }];
        for case in cases_telex {
            let mut engine = Engine::new(InputMethod::Telex, false);
            assert_eq!(simulate_typing_str(&mut engine, case.input), case.expected);
        }

        let cases_vni = vec![TestCase {
            input: "n g i BackSpace h i e 6 n g",
            expected: "nghiêng",
        }];
        for case in cases_vni {
            let mut engine = Engine::new(InputMethod::Vni, false);
            assert_eq!(simulate_typing_str(&mut engine, case.input), case.expected);
        }
    }

    #[test]
    fn test_interruptions() {
        let cases_telex = vec![
            TestCase {
                input: "h o a n g f Space 1 2 3 Space a n h",
                expected: "hoàng 123 anh",
            },
            TestCase {
                input: "h o a n g f , Space a n h",
                expected: "hoàng, anh",
            },
        ];
        for case in cases_telex {
            let mut engine = Engine::new(InputMethod::Telex, false);
            assert_eq!(simulate_typing_str(&mut engine, case.input), case.expected);
        }

        let cases_vni = vec![TestCase {
            input: "h o a n g 2 Space 1 2 3 Space a n h",
            expected: "hoàng 123 anh",
        }];
        for case in cases_vni {
            let mut engine = Engine::new(InputMethod::Vni, false);
            assert_eq!(simulate_typing_str(&mut engine, case.input), case.expected);
        }
    }

    #[test]
    fn test_rapid_mode_switching() {
        let cases = vec![TestCase {
            input: "v i e e t j Space [[ToggleVni]] n a m",
            expected: "việt nam",
        }];
        for case in cases {
            let mut engine = Engine::new(InputMethod::Telex, false);
            assert_eq!(simulate_typing_str(&mut engine, case.input), case.expected);
        }
    }

    #[test]
    fn test_redundant_modifier_keys() {
        let cases_telex = vec![
            TestCase {
                input: "a a w o o o",
                expected: "ăoo", // mechanically raw modifier overrides
            },
            TestCase {
                input: "h o a s s",
                expected: "hoas", // Double s removes the tone and adds s
            },
            TestCase {
                input: "o o o",
                expected: "oo",
            },
            TestCase {
                input: "a w w",
                expected: "aw",
            },
            TestCase {
                input: "a s s",
                expected: "as", // Double s removes the tone and adds s
            },
            TestCase {
                input: "o o f f",
                expected: "ôf",
            },
        ];
        for case in cases_telex {
            let mut engine = Engine::new(InputMethod::Telex, false);
            assert_eq!(simulate_typing_str(&mut engine, case.input), case.expected);
        }
    }

    #[test]
    fn test_w_at_beginning() {
        let cases_telex = vec![TestCase {
            input: "w i f i",
            expected: "wifi",
        }];
        for case in cases_telex {
            let mut engine = Engine::new(InputMethod::Telex, false);
            assert_eq!(simulate_typing_str(&mut engine, case.input), case.expected);
        }
    }

    #[test]
    fn test_extreme_vowels() {
        let cases_telex = vec![
            TestCase {
                input: "g i u w o w n g f",
                expected: "giường",
            },
            TestCase {
                input: "q u y e e n r",
                expected: "quyển",
            },
            TestCase {
                input: "q u y e e n x",
                expected: "quyễn",
            },
            TestCase {
                input: "t h u o w r",
                expected: "thuở",
            },
            TestCase {
                input: "n g u y e e n x",
                expected: "nguyễn",
            },
        ];
        for case in cases_telex {
            let mut engine = Engine::new(InputMethod::Telex, false);
            assert_eq!(simulate_typing_str(&mut engine, case.input), case.expected);
        }

        let cases_vni = vec![
            TestCase {
                input: "g i u 7 o 7 n g 2",
                expected: "giường",
            },
            TestCase {
                input: "q u y e 6 n 3",
                expected: "quyển",
            },
            TestCase {
                input: "t h u o 7 3",
                expected: "thưở",
            },
            TestCase {
                input: "n g u y e 6 n 4",
                expected: "nguyễn",
            },
        ];
        for case in cases_vni {
            let mut engine = Engine::new(InputMethod::Vni, false);
            assert_eq!(simulate_typing_str(&mut engine, case.input), case.expected);
        }
    }

    #[test]
    fn test_mixed_case() {
        let cases_telex = vec![
            TestCase {
                input: "H o a n g f",
                expected: "Hoàng",
            },
            TestCase {
                input: "V I E E T J Space N A M",
                expected: "VIỆT NAM",
            },
        ];
        for case in cases_telex {
            let mut engine = Engine::new(InputMethod::Telex, false);
            assert_eq!(simulate_typing_str(&mut engine, case.input), case.expected);
        }

        let cases_vni = vec![
            TestCase {
                input: "H o a n g 2",
                expected: "Hoàng",
            },
            TestCase {
                input: "V I E 6 T 5 Space N A M",
                expected: "VIỆT NAM",
            },
        ];
        for case in cases_vni {
            let mut engine = Engine::new(InputMethod::Vni, false);
            assert_eq!(simulate_typing_str(&mut engine, case.input), case.expected);
        }
    }

    #[test]
    fn test_backspace_chaos() {
        let cases_telex = vec![
            TestCase {
                input: "n g u u BackSpace y e e n x",
                expected: "nguyễn",
            },
            TestCase {
                input: "q u y e e n x",
                expected: "quyễn",
            },
            TestCase {
                input: "h o a n g f BackSpace BackSpace BackSpace BackSpace BackSpace BackSpace a n h",
                expected: "anh",
            },
        ];
        for case in cases_telex {
            let mut engine = Engine::new(InputMethod::Telex, false);
            assert_eq!(simulate_typing_str(&mut engine, case.input), case.expected);
        }

        let cases_vni = vec![
            TestCase {
                input: "n g u u BackSpace y e 6 n 4",
                expected: "nguyễn",
            },
            TestCase {
                input: "h o a n g 2 BackSpace BackSpace BackSpace BackSpace BackSpace BackSpace a n h",
                expected: "anh",
            },
        ];
        for case in cases_vni {
            let mut engine = Engine::new(InputMethod::Vni, false);
            assert_eq!(simulate_typing_str(&mut engine, case.input), case.expected);
        }
    }

    #[test]
    fn test_non_vietnamese_gibberish() {
        let cases = vec![TestCase {
            input: "j j j j j j j j",
            expected: "jjjjjjjj",
        }];
        for case in cases {
            let mut engine = Engine::new(InputMethod::Telex, false);
            assert_eq!(simulate_typing_str(&mut engine, case.input), case.expected);
        }
    }

    #[test]
    fn test_vni_number_chaos() {
        let cases_vni = vec![
            TestCase {
                input: "a 1 1",
                expected: "a1",
            },
            TestCase {
                input: "c o n g t y 1 2 3",
                expected: "congty123",
            },
        ];
        for case in cases_vni {
            let mut engine = Engine::new(InputMethod::Vni, false);
            assert_eq!(simulate_typing_str(&mut engine, case.input), case.expected);
        }
    }

    #[test]
    fn test_complex_clusters() {
        let cases_telex = vec![
            TestCase {
                input: "n g h i e e n g",
                expected: "nghiêng",
            },
            TestCase {
                input: "q u y n h f",
                expected: "quỳnh",
            },
            TestCase {
                input: "h u w o w u",
                expected: "hươu",
            },
            TestCase {
                input: "k h u y r u", // kh u y r u -> khuỷu
                expected: "khuỷu",
            },
        ];
        for case in cases_telex {
            let mut engine = Engine::new(InputMethod::Telex, true);
            assert_eq!(simulate_typing_str(&mut engine, case.input), case.expected);
        }

        let cases_vni = vec![
            TestCase {
                input: "n g h i e 6 n g",
                expected: "nghiêng",
            },
            TestCase {
                input: "q u y n h 2",
                expected: "quỳnh",
            },
            TestCase {
                input: "h u 7 o 7 u",
                expected: "hươu",
            },
            TestCase {
                input: "k h u y 3 u",
                expected: "khuỷu",
            },
        ];
        for case in cases_vni {
            let mut engine = Engine::new(InputMethod::Vni, true);
            assert_eq!(simulate_typing_str(&mut engine, case.input), case.expected);
        }
    }

    #[test]
    fn test_smart_spell_check_space_restoration() {
        let mut engine = Engine::new(InputMethod::Telex, true);
        let result = simulate_typing_str(&mut engine, "a a w o o o Space");
        assert_eq!(result, "aawooo ");
    }

    #[test]
    fn test_state_machine_edge_cases() {
        let cases_telex = vec![
            TestCase {
                input: "t o a n s f",
                expected: "toàn", // f overrides s
            },
            TestCase {
                input: "t o a n s z",
                expected: "toan", // z clears the tone
            },
            TestCase {
                input: "d d d",
                expected: "dd",
            },
        ];
        for case in cases_telex {
            let mut engine = Engine::new(InputMethod::Telex, false);
            assert_eq!(simulate_typing_str(&mut engine, case.input), case.expected);
        }

        let cases_vni = vec![
            TestCase {
                input: "t o a n 1 2",
                expected: "toàn", // 2 overrides 1
            },
            TestCase {
                input: "t o a n 1 0",
                expected: "toan", // 0 clears tone
            },
            TestCase {
                input: "d 9 9",
                expected: "đ9", // VNI 9 is modifier
            },
        ];
        for case in cases_vni {
            let mut engine = Engine::new(InputMethod::Vni, false);
            assert_eq!(simulate_typing_str(&mut engine, case.input), case.expected);
        }
    }

    #[test]
    fn test_smart_spell_check_foreign_words() {
        let cases_telex = vec![
            TestCase {
                input: "f a c e b o o k Space",
                expected: "facebook ",
            },
            TestCase {
                input: "l i n u x Space",
                expected: "linux ",
            },
            TestCase {
                input: "v a m p i r e Space",
                expected: "vampire ",
            },
        ];
        for case in cases_telex {
            let mut engine = Engine::new(InputMethod::Telex, true); // true = smart spell check
            assert_eq!(simulate_typing_str(&mut engine, case.input), case.expected);
        }

        // Mechanical behavior
        let mut engine = Engine::new(InputMethod::Telex, false);
        assert_eq!(
            simulate_typing_str(&mut engine, "f a c e b o o k"),
            "facebook"
        );

        let mut engine2 = Engine::new(InputMethod::Telex, false);
        assert_eq!(simulate_typing_str(&mut engine2, "l i n u x"), "linux"); // actually 'linux' is an invalid syllable mechanically, so it falls back to raw!
    }
}
