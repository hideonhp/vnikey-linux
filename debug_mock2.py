with open("vnikey-ibus/src/main.rs", "r") as f:
    content = f.read()

import re

content = content.replace(
    """            let ch = if is_backspace {
                Some('\x08')
            } else if keyval == 0xFF0D {
                Some('\n')
            } else {
                keyval_to_char(keyval)
            };""",
    """            let ch = if is_backspace {
                Some('\x08')
            } else if keyval == 0xFF0D {
                Some('\n')
            } else {
                keyval_to_char(keyval)
            };
            println!("ch for {} is {:?}", keyval, ch);"""
)

content = content.replace(
    """                let action = self.engine.process_key(c);""",
    """                let action = self.engine.process_key(c);
                println!("action for {} is {:?}", c, action);"""
)

with open("vnikey-ibus/src/main.rs", "w") as f:
    f.write(content)
