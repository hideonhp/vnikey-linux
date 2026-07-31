cat << 'INNER_EOF' >> src/lib.rs

#[cfg(test)]
mod tone_placer_tests {
    use crate::engine::{Engine, Action};
    use crate::buffer::CharBuffer;

    fn make_buffer(s: &str) -> CharBuffer {
        let mut buf = CharBuffer::new();
        for c in s.chars() {
            buf.push(c);
        }
        buf
    }

    #[test]
    fn test_hoang_tone() {
        let mut engine = Engine::new();
        let input = ['h', 'o', 'a', 'n', 'g', 'f'];
        let mut last_action = Action::PassThrough;
        for c in input {
            last_action = engine.process_key(c);
        }
        assert_eq!(last_action, Action::Preedit(make_buffer("hoàng")));
    }

    #[test]
    fn test_nguyen_tone() {
        let mut engine = Engine::new();
        let input = ['n', 'g', 'u', 'y', 'e', 'e', 'n', 'x'];
        let mut last_action = Action::PassThrough;
        for c in input {
            last_action = engine.process_key(c);
        }
        assert_eq!(last_action, Action::Preedit(make_buffer("nguyễn")));
    }

    #[test]
    fn test_thuy_tone() {
        let mut engine = Engine::new();
        let input = ['t', 'h', 'u', 'y', 'r'];
        let mut last_action = Action::PassThrough;
        for c in input {
            last_action = engine.process_key(c);
        }
        assert_eq!(last_action, Action::Preedit(make_buffer("thủy")));
    }

    #[test]
    fn test_z_cancel() {
        let mut engine = Engine::new();
        let input = ['h', 'o', 'a', 's', 'z']; // hoá -> hoa
        let mut last_action = Action::PassThrough;
        for c in input {
            last_action = engine.process_key(c);
        }
        assert_eq!(last_action, Action::Preedit(make_buffer("hoa")));
    }

    #[test]
    fn test_override_tone() {
        let mut engine = Engine::new();
        let input = ['h', 'o', 'a', 's', 'f']; // hoá -> hoà
        let mut last_action = Action::PassThrough;
        for c in input {
            last_action = engine.process_key(c);
        }
        assert_eq!(last_action, Action::Preedit(make_buffer("hòa")));
    }
}
INNER_EOF
