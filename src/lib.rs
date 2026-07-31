pub mod buffer;
pub mod engine;
pub mod telex;
pub mod validation;

#[cfg(test)]
mod tests {
    use crate::buffer::CharBuffer;
    use crate::engine::{Action, Engine, State};

    fn make_buffer(s: &str) -> CharBuffer {
        let mut buf = CharBuffer::new();
        for c in s.chars() {
            buf.push(c);
        }
        buf
    }

    #[test]
    fn test_basic_typing_and_commit() {
        let mut engine = Engine::new(crate::engine::InputMethod::Telex);

        let action1 = engine.process_key('c');
        assert_eq!(action1, Action::Preedit(make_buffer("c")));
        assert_eq!(engine.state, State::Composing);

        let action2 = engine.process_key('h');
        assert_eq!(action2, Action::Preedit(make_buffer("ch")));

        let action3 = engine.process_key('a');
        assert_eq!(action3, Action::Preedit(make_buffer("cha")));

        let action4 = engine.process_key('o');
        assert_eq!(action4, Action::Preedit(make_buffer("chao")));

        // Commit with space
        let action_commit = engine.process_key(' ');
        assert_eq!(action_commit, Action::Commit(make_buffer("chao ")));
        assert_eq!(engine.state, State::Idle);
    }

    #[test]
    fn test_telex_mapping() {
        let mut engine = Engine::new(crate::engine::InputMethod::Telex);

        // 'a' + 's' -> 'á'
        engine.process_key('a');
        let action = engine.process_key('s');
        assert_eq!(action, Action::Preedit(make_buffer("á")));
        engine.process_key(' '); // reset

        // 'a' + 'f' -> 'à'
        engine.process_key('a');
        let action = engine.process_key('f');
        assert_eq!(action, Action::Preedit(make_buffer("à")));
        engine.process_key(' '); // reset

        // 'o' + 'o' -> 'ô'
        engine.process_key('o');
        let action = engine.process_key('o');
        assert_eq!(action, Action::Preedit(make_buffer("ô")));
        engine.process_key(' '); // reset

        // 'd' + 'd' -> 'đ'
        engine.process_key('d');
        let action = engine.process_key('d');
        assert_eq!(action, Action::Preedit(make_buffer("đ")));
    }

    #[test]
    fn test_backspace() {
        let mut engine = Engine::new(crate::engine::InputMethod::Telex);

        // Backspace on Idle
        assert_eq!(engine.process_key('\x08'), Action::PassThrough);

        // Type 'o', 'o' -> 'ô'
        engine.process_key('o');
        engine.process_key('o');
        assert_eq!(engine.buffer.as_slice(), ['ô']);

        // Backspace should revert to 'o'
        let action = engine.process_key('\x08');
        assert_eq!(action, Action::Preedit(make_buffer("o")));

        // Backspace again should empty buffer and return to Idle
        let action2 = engine.process_key('\x08');
        assert_eq!(action2, Action::Preedit(CharBuffer::new()));
        assert_eq!(engine.state, State::Idle);
    }

    #[test]
    fn test_buffer_limit_auto_commit() {
        let mut engine = Engine::new(crate::engine::InputMethod::Telex);

        // Fill buffer to max capacity (16) with unique characters so they don't combine
        // We'll just push 'q'
        for _ in 0..16 {
            engine.process_key('q');
        }

        assert_eq!(engine.buffer.len(), 16);

        // Type 17th character, should auto-commit
        let action = engine.process_key('b');

        let mut expected_commit_buf = CharBuffer::new();
        for _ in 0..16 {
            expected_commit_buf.push('q');
        }

        assert_eq!(action, Action::Commit(expected_commit_buf));

        // Internal state should be Composing with 'b'
        assert_eq!(engine.state, State::Composing);
        assert_eq!(engine.raw_buffer.as_slice(), ['b']);
    }
}

#[cfg(test)]
mod smart_tests {
    use crate::buffer::CharBuffer;
    use crate::engine::{Action, Engine};

    fn make_buffer(s: &str) -> CharBuffer {
        let mut buf = CharBuffer::new();
        for c in s.chars() {
            buf.push(c);
        }
        buf
    }

    #[test]
    fn test_smart_english() {
        let mut engine = Engine::new(crate::engine::InputMethod::Telex);
        let word = "english";
        for (i, c) in word.chars().enumerate() {
            let action = engine.process_key(c);
            let expected = &word[..i + 1];
            assert_eq!(
                action,
                Action::Preedit(make_buffer(expected)),
                "Failed at char: {}",
                c
            );
        }
    }

    #[test]
    fn test_smart_linux() {
        let mut engine = Engine::new(crate::engine::InputMethod::Telex);
        let word = "linux";
        for (i, c) in word.chars().enumerate() {
            let action = engine.process_key(c);
            let expected = &word[..i + 1];
            assert_eq!(
                action,
                Action::Preedit(make_buffer(expected)),
                "Failed at char: {}",
                c
            );
        }
    }

    #[test]
    fn test_valid_telex() {
        let mut engine = Engine::new(crate::engine::InputMethod::Telex);
        engine.process_key('h');
        engine.process_key('o');
        engine.process_key('a');
        let action1 = engine.process_key('s'); // a+s -> á. hoas -> hóa
        assert_eq!(action1, Action::Preedit(make_buffer("hóa")));
    }
}

#[cfg(test)]
mod tone_placer_tests {
    use crate::buffer::CharBuffer;
    use crate::engine::{Action, Engine};

    fn make_buffer(s: &str) -> CharBuffer {
        let mut buf = CharBuffer::new();
        for c in s.chars() {
            buf.push(c);
        }
        buf
    }

    #[test]
    fn test_hoang_tone() {
        let mut engine = Engine::new(crate::engine::InputMethod::Telex);
        let input = ['h', 'o', 'a', 'n', 'g', 'f'];
        let mut last_action = Action::PassThrough;
        for c in input {
            last_action = engine.process_key(c);
        }
        assert_eq!(last_action, Action::Preedit(make_buffer("hoàng")));
    }

    #[test]
    fn test_nguyen_tone() {
        let mut engine = Engine::new(crate::engine::InputMethod::Telex);
        let input = ['n', 'g', 'u', 'y', 'e', 'e', 'n', 'x'];
        let mut last_action = Action::PassThrough;
        for c in input {
            last_action = engine.process_key(c);
        }
        assert_eq!(last_action, Action::Preedit(make_buffer("nguyễn")));
    }

    #[test]
    fn test_thuy_tone() {
        let mut engine = Engine::new(crate::engine::InputMethod::Telex);
        let input = ['t', 'h', 'u', 'y', 'r'];
        let mut last_action = Action::PassThrough;
        for c in input {
            last_action = engine.process_key(c);
        }
        assert_eq!(last_action, Action::Preedit(make_buffer("thủy")));
    }

    #[test]
    fn test_z_cancel() {
        let mut engine = Engine::new(crate::engine::InputMethod::Telex);
        let input = ['h', 'o', 'a', 's', 'z']; // hoá -> hoa
        let mut last_action = Action::PassThrough;
        for c in input {
            last_action = engine.process_key(c);
        }
        assert_eq!(last_action, Action::Preedit(make_buffer("hoa")));
    }

    #[test]
    fn test_override_tone() {
        let mut engine = Engine::new(crate::engine::InputMethod::Telex);
        let input = ['h', 'o', 'a', 's', 'f']; // hoá -> hoà
        let mut last_action = Action::PassThrough;
        for c in input {
            last_action = engine.process_key(c);
        }
        assert_eq!(last_action, Action::Preedit(make_buffer("hòa")));
    }
}

#[cfg(test)]
mod more_telex_tests {
    use crate::buffer::CharBuffer;
    use crate::engine::{Action, Engine};

    fn make_buffer(s: &str) -> CharBuffer {
        let mut buf = CharBuffer::new();
        for c in s.chars() {
            buf.push(c);
        }
        buf
    }

    fn type_keys(keys: &str) -> Action {
        let mut engine = Engine::new(crate::engine::InputMethod::Telex);
        let mut last_action = Action::PassThrough;
        for c in keys.chars() {
            last_action = engine.process_key(c);
        }
        last_action
    }

    #[test]
    fn test_vowel_cancellations() {
        assert_eq!(type_keys("aa"), Action::Preedit(make_buffer("â")));
        assert_eq!(type_keys("aaa"), Action::Preedit(make_buffer("aa")));

        assert_eq!(type_keys("dd"), Action::Preedit(make_buffer("đ")));
        assert_eq!(type_keys("ddd"), Action::Preedit(make_buffer("dd")));

        assert_eq!(type_keys("ow"), Action::Preedit(make_buffer("ơ")));
        assert_eq!(type_keys("oww"), Action::Preedit(make_buffer("ow")));

        assert_eq!(type_keys("uw"), Action::Preedit(make_buffer("ư")));
        assert_eq!(type_keys("uww"), Action::Preedit(make_buffer("uw")));
    }

    #[test]
    fn test_tone_cancellations() {
        assert_eq!(type_keys("as"), Action::Preedit(make_buffer("á")));
        assert_eq!(type_keys("ass"), Action::Preedit(make_buffer("as")));

        assert_eq!(type_keys("af"), Action::Preedit(make_buffer("à")));
        assert_eq!(type_keys("aff"), Action::Preedit(make_buffer("af")));
    }

    #[test]
    fn test_qu_exception() {
        assert_eq!(type_keys("quais"), Action::Preedit(make_buffer("quái")));
        assert_eq!(type_keys("quy"), Action::Preedit(make_buffer("quy")));
        assert_eq!(type_keys("quys"), Action::Preedit(make_buffer("quý")));
    }

    #[test]
    fn test_gi_exception() {
        assert_eq!(type_keys("giaos"), Action::Preedit(make_buffer("giáo")));
        assert_eq!(type_keys("gieengs"), Action::Preedit(make_buffer("giếng")));
    }

    #[test]
    fn test_standalone_w() {
        assert_eq!(type_keys("w"), Action::Preedit(make_buffer("ư")));
    }

    #[test]
    fn test_z_when_no_tones() {
        assert_eq!(type_keys("z"), Action::Preedit(make_buffer("z")));
        assert_eq!(type_keys("chz"), Action::Preedit(make_buffer("chz")));
    }
}

#[cfg(test)]
mod vni_tests {
    use crate::buffer::CharBuffer;
    use crate::engine::{Action, Engine, InputMethod};

    fn make_buffer(s: &str) -> CharBuffer {
        let mut buf = CharBuffer::new();
        for c in s.chars() {
            buf.push(c);
        }
        buf
    }

    fn type_keys(keys: &str) -> Action {
        let mut engine = Engine::new(InputMethod::Vni);
        let mut last_action = Action::PassThrough;
        for c in keys.chars() {
            last_action = engine.process_key(c);
        }
        last_action
    }

    #[test]
    fn test_vni_basics() {
        assert_eq!(type_keys("a1"), Action::Preedit(make_buffer("á")));
        assert_eq!(type_keys("d9"), Action::Preedit(make_buffer("đ")));
        assert_eq!(type_keys("a8"), Action::Preedit(make_buffer("ă")));
    }

    #[test]
    fn test_vni_complex() {
        assert_eq!(type_keys("hoang2"), Action::Preedit(make_buffer("hoàng")));
        assert_eq!(
            type_keys("nguye6n4"),
            Action::Preedit(make_buffer("nguyễn"))
        );
    }

    #[test]
    fn test_vni_fallback_numbers() {
        assert_eq!(type_keys("vni8"), Action::Preedit(make_buffer("vni8")));
        assert_eq!(type_keys("123"), Action::Preedit(make_buffer("123")));
        assert_eq!(type_keys("a11"), Action::Preedit(make_buffer("á1")));
    }

    #[test]
    fn test_vni_cancellation() {
        assert_eq!(type_keys("a10"), Action::Preedit(make_buffer("a")));
        assert_eq!(type_keys("a80"), Action::Preedit(make_buffer("a")));
        assert_eq!(type_keys("d90"), Action::Preedit(make_buffer("d")));
        assert_eq!(type_keys("o60"), Action::Preedit(make_buffer("o")));
        assert_eq!(type_keys("o70"), Action::Preedit(make_buffer("o")));
        assert_eq!(type_keys("hoang20"), Action::Preedit(make_buffer("hoang")));
    }

    #[test]
    fn test_vni_override_tone() {
        // a + 1 = á, then 2 = à
        assert_eq!(type_keys("a12"), Action::Preedit(make_buffer("à")));
    }
}

#[cfg(test)]
mod cross_contamination_tests {
    use crate::buffer::CharBuffer;
    use crate::engine::{Action, Engine, InputMethod};

    fn make_buffer(s: &str) -> CharBuffer {
        let mut buf = CharBuffer::new();
        for c in s.chars() {
            buf.push(c);
        }
        buf
    }

    #[test]
    fn test_scenario_a_telex_ignoring_vni_keys() {
        let mut engine = Engine::new(InputMethod::Telex);

        // Action: Type a, 1. Expected output: a1
        engine.process_key('a');
        let action = engine.process_key('1');
        assert_eq!(action, Action::Preedit(make_buffer("a1")));
        engine.process_key(' ');

        // Action: Type h, o, a, n, g, 2. Expected output: hoang2
        let keys = ['h', 'o', 'a', 'n', 'g', '2'];
        let mut last_action = Action::PassThrough;
        for k in keys {
            last_action = engine.process_key(k);
        }
        assert_eq!(last_action, Action::Preedit(make_buffer("hoang2")));
        engine.process_key(' ');

        // Action: Type a, s, 0 (á, 0). Expected output: á0
        engine.process_key('a');
        engine.process_key('s');
        let action = engine.process_key('0');
        assert_eq!(action, Action::Preedit(make_buffer("á0")));
    }

    #[test]
    fn test_scenario_b_vni_ignoring_telex_keys() {
        let mut engine = Engine::new(InputMethod::Vni);

        // Action: Type a, s. Expected output: as
        engine.process_key('a');
        let action = engine.process_key('s');
        assert_eq!(action, Action::Preedit(make_buffer("as")));
        engine.process_key(' ');

        // Action: Type h, o, a, n, g, f. Expected output: hoangf
        let keys = ['h', 'o', 'a', 'n', 'g', 'f'];
        let mut last_action = Action::PassThrough;
        for k in keys {
            last_action = engine.process_key(k);
        }
        assert_eq!(last_action, Action::Preedit(make_buffer("hoangf")));
        engine.process_key(' ');

        // Action: Type a, 1, z (á, z). Expected output: áz
        engine.process_key('a');
        engine.process_key('1');
        let action = engine.process_key('z');
        assert_eq!(action, Action::Preedit(make_buffer("áz")));
    }

    #[test]
    fn test_scenario_c_toggle_switch_integrity() {
        let mut engine = Engine::new(InputMethod::Telex);

        // Type a, s -> asserts á
        engine.process_key('a');
        let action = engine.process_key('s');
        assert_eq!(action, Action::Preedit(make_buffer("á")));

        // Switch engine to Vni mode -> should commit "á"
        let switch_action = engine.set_input_method(InputMethod::Vni);
        assert_eq!(switch_action, Some(Action::Commit(make_buffer("á"))));

        // Type a, s -> asserts as (proving VNI mode ignores 's')
        engine.process_key('a');
        let action2 = engine.process_key('s');
        assert_eq!(action2, Action::Preedit(make_buffer("as")));
    }
}
