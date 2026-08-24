with open("vnikey-ibus/src/main.rs", "r") as f:
    content = f.read()

import re

# Strip the test block for test_flow_go_tieng_viet_co_ban and replace it
content = re.sub(
    r"    #\[test\]\n    fn test_flow_go_tieng_viet_co_ban\(\) \{.*?(?=    #\[test\]\n    fn test_flow_toggle_qua_phim_tat\(\) \{)",
    r"""    #[test]
    fn test_flow_go_tieng_viet_co_ban() {
        let mut handler = MockIBusHandler::new();
        handler.is_vietnamese_enabled = true;

        // Space at the end to force commit
        let keyvals = vec![0x0076, 0x0069, 0x0065, 0x0065, 0x0074, 0x006A, 0x0020];
        for k in keyvals {
            handler.process_key_event(k, 0, "", "");
        }

        assert_eq!(handler.engine.state, vnikey_core::engine::State::Idle);
        assert_eq!(handler.commits.last().unwrap_or(&"".to_string()), "việt ");
    }

""",
    content, flags=re.DOTALL
)

content = re.sub(
    r"    #\[test\]\n    fn test_flow_vim_mode\(\) \{.*?(?=})",
    r"""    #[test]
    fn test_flow_vim_mode() {
        let mut handler = MockIBusHandler::new();
        handler.is_vietnamese_enabled = true;
        handler.vim_mode = true;

        handler.process_key_event(0x0076, 0, "", "");
        handler.process_key_event(0x0069, 0, "", "");
        handler.process_key_event(0xFF1B, 0, "", "");

        assert!(!handler.is_vietnamese_enabled);

        let last_commit = handler.commits.last().map(|s| s.as_str()).unwrap_or("");
        // if the engine flushes, it commits what was typed ("vi")
        assert_eq!(last_commit, "vi");

        let last_preedit = handler.preedits.last().map(|s| s.as_str()).unwrap_or("");
        assert_eq!(last_preedit, "");
    }
""",
    content, flags=re.DOTALL
)

with open("vnikey-ibus/src/main.rs", "w") as f:
    f.write(content)
