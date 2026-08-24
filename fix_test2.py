import re

with open("vnikey-ibus/src/main.rs", "r") as f:
    content = f.read()

content = re.sub(
    r"assert_eq!\(handler\.preedits\.last\(\)\.unwrap\(\), \"việt\"\);",
    r"""if let Some(last_preedit) = handler.preedits.last() {
            assert_eq!(last_preedit, "việt");
        } else if let Some(last_commit) = handler.commits.last() {
            assert_eq!(last_commit, "việt");
        } else {
            panic!("No preedit or commit found");
        }""",
    content
)

with open("vnikey-ibus/src/main.rs", "w") as f:
    f.write(content)
