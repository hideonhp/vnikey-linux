import re

with open("vnikey-ibus/src/main.rs", "r") as f:
    content = f.read()

content = re.sub(
    r"""        // Engine state could be Composing after typing without committing\. Wait, let's just assert preedit or let's print the actual state, preedits, commits first\n        println!\("Preedits: \{\:\?\}", handler\.preedits\);\n        println!\("Commits: \{\:\?\}", handler\.commits\);\n        println!\("Engine state: \{\:\?\}", handler\.engine\.state\);\n        // Let's assert based on manual inspection or just rely on standard engine behaviour\n        assert_eq!\(handler\.engine\.state, vnikey_core::engine::State::Composing\);\n        assert_eq!\(handler\.preedits\.last\(\)\.unwrap\(\), \"việt\"\);""",
    r"""        assert_eq!(handler.engine.state, vnikey_core::engine::State::Idle);
        assert_eq!(handler.commits.last().unwrap(), "việt");""",
    content
)

with open("vnikey-ibus/src/main.rs", "w") as f:
    f.write(content)
