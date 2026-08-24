with open("vnikey-ibus/src/main.rs", "r") as f:
    content = f.read()

import re
content = re.sub(
    r"        if let Some\(last_preedit\) = handler\.preedits\.last\(\) \{\n            assert_eq!\(last_preedit, \"việt\"\);\n        \} else if let Some\(last_commit\) = handler\.commits\.last\(\) \{\n            assert_eq!\(last_commit, \"việt\"\);\n        \} else \{\n            panic!\(\"No preedit or commit found\"\);\n        \}",
    r"""        println!("Preedits: {:?}", handler.preedits);
        println!("Commits: {:?}", handler.commits);
        println!("Engine state: {:?}", handler.engine.state);
        // Let's assert based on manual inspection or just rely on standard engine behaviour
        assert_eq!(handler.engine.state, vnikey_core::engine::State::Composing);
        assert_eq!(handler.preedits.last().unwrap(), "việt");""",
    content
)
with open("vnikey-ibus/src/main.rs", "w") as f:
    f.write(content)
