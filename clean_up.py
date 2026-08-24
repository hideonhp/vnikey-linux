with open("vnikey-ibus/src/main.rs", "r") as f:
    content = f.read()

import re

# Remove the printlns we added
content = re.sub(r'\s*println!\("Processed.*?\);\n', '\n', content)
content = re.sub(r'\s*println!\("Final preedits.*?\);\n', '\n', content)
content = re.sub(r'\s*println!\("Final commits.*?\);\n', '\n', content)
content = re.sub(r'\s*println!\("ch for.*?\);\n', '\n', content)
content = re.sub(r'\s*println!\("action for.*?\);\n', '\n', content)
content = re.sub(r'let res = handler\.process_key_event', 'handler.process_key_event', content)

with open("vnikey-ibus/src/main.rs", "w") as f:
    f.write(content)
