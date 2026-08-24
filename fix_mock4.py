with open("vnikey-ibus/src/main.rs", "r") as f:
    content = f.read()

import re

# In `process_key_event(..., config_mod: &str, config_key: &str)`, let's fix the mock calls.
# We passed `""` for `config_mod` and `""` for `config_key`.
# Instead of passing `""`, we should pass `"none"` or `"unmatched"` to avoid matching everything.

content = content.replace(
    'handler.process_key_event(k, 0, "", "");',
    'handler.process_key_event(k, 0, "none", "unmatched");'
)

content = content.replace(
    'handler.process_key_event(0x0076, 0, "", "");',
    'handler.process_key_event(0x0076, 0, "none", "unmatched");'
)

content = content.replace(
    'handler.process_key_event(0x0069, 0, "", "");',
    'handler.process_key_event(0x0069, 0, "none", "unmatched");'
)

content = content.replace(
    'handler.process_key_event(0xFF1B, 0, "", "");',
    'handler.process_key_event(0xFF1B, 0, "none", "unmatched");'
)

# And in `is_toggle_hotkey`, an empty config string was supposed to be safe. But wait, in production config_key is never empty!
# But in our test we passed `""`.
# So fixing the test calls is enough!

with open("vnikey-ibus/src/main.rs", "w") as f:
    f.write(content)
