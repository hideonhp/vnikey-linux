with open("vnikey-ibus/src/main.rs", "r") as f:
    content = f.read()

import re

# In `process_key_event` for MockIBusHandler, we just push `buf.to_string()` to commits.
# Let's inspect `handler.preedits` and `handler.commits` by printing them!

content = re.sub(
    r"""        assert_eq!\(handler\.engine\.state, vnikey_core::engine::State::Idle\);\n        assert_eq!\(handler\.commits\.last\(\)\.unwrap_or\(\&\"\"\.to_string\(\)\), \"việt \"\);""",
    r"""        assert_eq!(handler.engine.state, vnikey_core::engine::State::Idle);
        let commit_text = handler.commits.join("");
        assert_eq!(commit_text, "việt ");""",
    content
)

content = re.sub(
    r"""        let last_commit = handler\.commits\.last\(\)\.map\(\|s\| s\.as_str\(\)\)\.unwrap_or\(\"\"\);\n        // if the engine flushes, it commits what was typed \(\"vi\"\)\n        assert_eq!\(last_commit, \"vi\"\);""",
    r"""        let commit_text = handler.commits.join("");
        assert_eq!(commit_text, "vi");""",
    content
)

with open("vnikey-ibus/src/main.rs", "w") as f:
    f.write(content)
