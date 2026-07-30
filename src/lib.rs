pub mod buffer;
pub mod engine;
#[cfg(test)]
mod tests {
    use crate::engine::{Engine, Action, State};
    use crate::buffer::CharBuffer;

    fn make_buffer(s: &str) -> CharBuffer {
        let mut buf = CharBuffer::new();
        for c in s.chars() {
            buf.push(c);
        }
        buf
    }

    #[test]
    fn test_basic_typing_and_commit() {
        let mut engine = Engine::new();

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
        let mut engine = Engine::new();

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
        let mut engine = Engine::new();

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
        let mut engine = Engine::new();

        // Fill buffer to max capacity (16)
        for _ in 0..16 {
            engine.process_key('a');
        }

        assert_eq!(engine.buffer.len(), 16);

        // Type 17th character, should auto-commit
        let action = engine.process_key('b');

        let mut expected_commit_buf = CharBuffer::new();
        for _ in 0..16 {
            expected_commit_buf.push('a');
        }

        assert_eq!(action, Action::Commit(expected_commit_buf));

        // Internal state should be Composing with 'b'
        assert_eq!(engine.state, State::Composing);
        assert_eq!(engine.raw_buffer.as_slice(), ['b']);
    }
}
