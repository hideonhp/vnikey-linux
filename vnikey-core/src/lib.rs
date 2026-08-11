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
        let mut engine = Engine::new(crate::engine::InputMethod::Telex, false);

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
        let mut engine = Engine::new(crate::engine::InputMethod::Telex, false);

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
        let mut engine = Engine::new(crate::engine::InputMethod::Telex, false);

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
        let mut engine = Engine::new(crate::engine::InputMethod::Telex, false);

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
mod surrounding_text_tests {
    use crate::buffer::CharBuffer;
    use crate::engine::{Action, Engine, State, InputMethod};

    fn make_buffer(s: &str) -> CharBuffer {
        let mut buf = CharBuffer::new();
        for c in s.chars() {
            buf.push(c);
        }
        buf
    }

    #[test]
    fn test_surrounding_basic_recompose() {
        let mut engine = Engine::new(InputMethod::Telex, false);
        // Type "tieng" and commit
        for c in "tieng".chars() { engine.process_key(c); }
        engine.process_key(' '); // commit "tieng "
        assert_eq!(engine.state, State::Idle);

        // Backspace should trigger surrounding recompose
        let action = engine.process_key('\x08');
        // Should return SurroundingRecompose with "tien" (popped 'g')
        // and delete_count = 5 (length of "tieng")
        match action {
            Action::SurroundingRecompose { preedit, delete_count, delete_byte_len } => {
                assert_eq!(preedit, make_buffer("tien"));
                assert_eq!(delete_count, 5); // "tieng" = 5 chars
                assert_eq!(delete_byte_len, 5); // 5 bytes for "tieng"
            }
            _ => panic!("Expected SurroundingRecompose, got {:?}", action),
        }
        assert_eq!(engine.state, State::Composing);
    }

    #[test]
    fn test_surrounding_full_flow_tieng() {
        let mut engine = Engine::new(InputMethod::Telex, false);
        // Commit "tieng"
        for c in "tieng".chars() { engine.process_key(c); }
        engine.process_key(' ');

        // Backspace 5 times to fully reenter surrounding
        engine.process_key('\x08'); // SurroundingRecompose("tien", 5)

        // Now in Composing with raw="tien", continue typing
        // Type "eengs" → should become "tiếng" via telex
        // Actually after first backspace, engine is Composing with raw_buffer="tien"
        // Subsequent backspaces are normal composing backspaces
        engine.process_key('\x08'); // normal backspace in composing: raw="tie"
        engine.process_key('\x08'); // raw="ti"
        engine.process_key('\x08'); // raw="t"
        engine.process_key('\x08'); // raw="" → Idle

        // Retype with tone
        for c in "tieengs".chars() { engine.process_key(c); }
        // "tieengs" in telex → "tiếng"
        let action = engine.process_key(' ');
        match action {
            Action::Commit(buf) => {
                assert_eq!(buf, make_buffer("tiếng "));
            }
            _ => panic!("Expected Commit"),
        }
    }

    #[test]
    fn test_surrounding_partial_backspace() {
        let mut engine = Engine::new(InputMethod::Telex, false);
        // Commit "tieng"
        for c in "tieng".chars() { engine.process_key(c); }
        engine.process_key(' ');

        // Backspace once → recompose with "tien"
        engine.process_key('\x08');
        // Now composing with raw="tien"
        // Type "gs" to add tone to remaining
        let action_g = engine.process_key('g');
        assert_eq!(action_g, Action::Preedit(make_buffer("tieng")));
        let action_s = engine.process_key('s');
        // Actually, the original word was typed as "tieng" (no tone, no e mod)
        // Backspacing 'g' gives raw="tien". Adding 'g' gives raw="tieng".
        // Adding 's' gives raw="tiengs" which evaluates to "tiéng", not "tiếng".
        // Because "tiếng" requires "tieengs"
        assert_eq!(action_s, Action::Preedit(make_buffer("tiéng")));
    }

    #[test]
    fn test_surrounding_cleared_on_new_char() {
        let mut engine = Engine::new(InputMethod::Telex, false);
        // Commit "abc"
        for c in "abc".chars() { engine.process_key(c); }
        engine.process_key(' ');

        // Type new char (not backspace) → should clear surrounding
        engine.process_key('x');

        // Now backspace should be normal composing backspace, not surrounding
        let action = engine.process_key('\x08');
        assert_eq!(action, Action::Preedit(CharBuffer::new()));
        assert_eq!(engine.state, State::Idle);
    }

    #[test]
    fn test_surrounding_cleared_on_unmapped_key() {
        let mut engine = Engine::new(InputMethod::Telex, false);
        for c in "abc".chars() { engine.process_key(c); }
        engine.process_key(' ');

        // Unmapped key when idle → clear surrounding
        engine.process_key(',');

        // Backspace should be PassThrough
        let action = engine.process_key('\x08');
        assert_eq!(action, Action::PassThrough);
    }

    #[test]
    fn test_surrounding_backspace_all_returns_empty() {
        let mut engine = Engine::new(InputMethod::Telex, false);
        // Commit single char "a"
        engine.process_key('a');
        engine.process_key(' ');

        // Backspace → should recompose with empty (deleted all)
        let action = engine.process_key('\x08');
        match action {
            Action::SurroundingRecompose { preedit, delete_count, delete_byte_len } => {
                assert!(preedit.is_empty());
                assert_eq!(delete_count, 1);
                assert_eq!(delete_byte_len, 1);
            }
            _ => panic!("Expected SurroundingRecompose"),
        }
        assert_eq!(engine.state, State::Idle);
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
        let mut engine = Engine::new(crate::engine::InputMethod::Telex, false);
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
        let mut engine = Engine::new(crate::engine::InputMethod::Telex, false);
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
        let mut engine = Engine::new(crate::engine::InputMethod::Telex, false);
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
        let mut engine = Engine::new(crate::engine::InputMethod::Telex, false);
        let input = ['h', 'o', 'a', 'n', 'g', 'f'];
        let mut last_action = Action::PassThrough;
        for c in input {
            last_action = engine.process_key(c);
        }
        assert_eq!(last_action, Action::Preedit(make_buffer("hoàng")));
    }

    #[test]
    fn test_nguyen_tone() {
        let mut engine = Engine::new(crate::engine::InputMethod::Telex, false);
        let input = ['n', 'g', 'u', 'y', 'e', 'e', 'n', 'x'];
        let mut last_action = Action::PassThrough;
        for c in input {
            last_action = engine.process_key(c);
        }
        assert_eq!(last_action, Action::Preedit(make_buffer("nguyễn")));
    }

    #[test]
    fn test_thuy_tone() {
        let mut engine = Engine::new(crate::engine::InputMethod::Telex, false);
        let input = ['t', 'h', 'u', 'y', 'r'];
        let mut last_action = Action::PassThrough;
        for c in input {
            last_action = engine.process_key(c);
        }
        assert_eq!(last_action, Action::Preedit(make_buffer("thủy")));
    }

    #[test]
    fn test_z_cancel() {
        let mut engine = Engine::new(crate::engine::InputMethod::Telex, false);
        let input = ['h', 'o', 'a', 's', 'z']; // hoá -> hoa
        let mut last_action = Action::PassThrough;
        for c in input {
            last_action = engine.process_key(c);
        }
        assert_eq!(last_action, Action::Preedit(make_buffer("hoa")));
    }

    #[test]
    fn test_override_tone() {
        let mut engine = Engine::new(crate::engine::InputMethod::Telex, false);
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
        let mut engine = Engine::new(crate::engine::InputMethod::Telex, false);
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
        assert_eq!(type_keys("w"), Action::Preedit(make_buffer("w")));
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
        let mut engine = Engine::new(InputMethod::Vni, false);
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
        assert_eq!(type_keys("a11"), Action::Preedit(make_buffer("a1")));
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
mod method_isolation_tests {
    use crate::buffer::CharBuffer;
    use crate::engine::{Action, Engine, InputMethod, State};

    fn make_buffer(s: &str) -> CharBuffer {
        let mut buf = CharBuffer::new();
        for c in s.chars() {
            buf.push(c);
        }
        buf
    }

    fn type_keys(engine: &mut Engine, keys: &str) -> Action {
        engine.spell_check = false;
        let mut last_action = Action::PassThrough;
        for c in keys.chars() {
            last_action = engine.process_key(c);
        }
        last_action
    }

    #[test]
    fn test_scenario_a_telex_ignores_vni() {
        let mut engine = Engine::new(InputMethod::Telex, false);
        assert_eq!(
            type_keys(&mut engine, "a1"),
            Action::Preedit(make_buffer("a1"))
        );

        let mut engine = Engine::new(InputMethod::Telex, false);
        assert_eq!(
            type_keys(&mut engine, "hoang2"),
            Action::Preedit(make_buffer("hoang2"))
        );
    }

    #[test]
    fn test_scenario_b_vni_ignores_telex() {
        let mut engine = Engine::new(InputMethod::Vni, false);
        assert_eq!(
            type_keys(&mut engine, "as"),
            Action::Preedit(make_buffer("as"))
        );

        let mut engine = Engine::new(InputMethod::Vni, false);
        assert_eq!(
            type_keys(&mut engine, "hoangf"),
            Action::Preedit(make_buffer("hoangf"))
        );
    }

    #[test]
    fn test_scenario_c_toggle_integrity() {
        let mut engine = Engine::new(InputMethod::Telex, false);
        assert_eq!(
            type_keys(&mut engine, "as"),
            Action::Preedit(make_buffer("á"))
        );
        assert_eq!(engine.state, State::Composing);

        let toggle_action = engine.set_input_method(InputMethod::Vni);
        assert_eq!(toggle_action, Some(Action::Commit(make_buffer("á"))));
        assert_eq!(engine.state, State::Idle);

        assert_eq!(
            type_keys(&mut engine, "as"),
            Action::Preedit(make_buffer("as"))
        );
    }
}
