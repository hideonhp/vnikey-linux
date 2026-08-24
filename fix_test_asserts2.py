# Wait, why are preedits and commits empty? Let me print everything!
with open("vnikey-ibus/src/main.rs", "r") as f:
    content = f.read()

import re

content = re.sub(
    r"""    #\[test\]\n    fn test_flow_go_tieng_viet_co_ban\(\) \{.*?(?=    #\[test\]\n    fn test_flow_toggle_qua_phim_tat\(\) \{)""",
    r"""    #[test]
    fn test_flow_go_tieng_viet_co_ban() {
        let mut handler = MockIBusHandler::new();
        handler.is_vietnamese_enabled = true;

        let keyvals = vec![0x0076, 0x0069, 0x0065, 0x0065, 0x0074, 0x006A, 0x0020];
        for k in keyvals {
            let res = handler.process_key_event(k, 0, "", "");
            println!("Processed {k}: {res}");
        }

        println!("Final preedits: {:?}", handler.preedits);
        println!("Final commits: {:?}", handler.commits);

        assert_eq!(handler.engine.state, vnikey_core::engine::State::Idle);
        let commit_text = handler.commits.join("");
        assert_eq!(commit_text, "việt ");
    }

""",
    content, flags=re.DOTALL
)

with open("vnikey-ibus/src/main.rs", "w") as f:
    f.write(content)
