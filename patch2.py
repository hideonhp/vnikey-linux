import sys

with open('vnikey-wayland/src/main.rs', 'r') as f:
    lines = f.readlines()

new_lines = []
for i, line in enumerate(lines):
    if "let is_pressed = key_state_u32 == 1;" in line:
        new_lines.append(line)
    else:
        new_lines.append(line)

with open('vnikey-wayland/src/main.rs', 'w') as f:
    f.writelines(new_lines)
