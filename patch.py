import sys

with open('vnikey-wayland/src/main.rs', 'r') as f:
    lines = f.readlines()

new_lines = []
for i, line in enumerate(lines):
    new_lines.append(line)
    if "let xkb_keycode = key + 8;" in line:
        indent = " " * (len(line) - len(line.lstrip()))
        new_lines.append(indent + "let mut is_toggle = false;\n")
        new_lines.append(indent + "if let Some(xkb_state) = state.xkb_state.as_ref() {\n")
        new_lines.append(indent + "    let is_ctrl = xkb_state.mod_name_is_active(&xkbcommon::xkb::MOD_NAME_CTRL, xkbcommon::xkb::STATE_MODS_DEPRESSED);\n")
        new_lines.append(indent + "    let keysym = xkb_state.key_get_one_sym(xkb_keycode.into());\n")
        new_lines.append(indent + "    if is_ctrl && keysym == xkbcommon::xkb::keysyms::KEY_space {\n")
        new_lines.append(indent + "        is_toggle = true;\n")
        new_lines.append(indent + "    }\n")
        new_lines.append(indent + "}\n\n")
        new_lines.append(indent + "if is_toggle {\n")
        new_lines.append(indent + "    if state.is_vietnamese_enabled {\n")
        new_lines.append(indent + "        if let Some(Action::Commit(buffer)) = state.engine.set_input_method(state.engine.get_input_method()) {\n")
        new_lines.append(indent + "            let text = String::from_iter(buffer.as_slice());\n")
        new_lines.append(indent + "            state.im.as_ref().unwrap().commit_string(text);\n")
        new_lines.append(indent + "            state.im.as_ref().unwrap().commit(0);\n")
        new_lines.append(indent + "        }\n")
        new_lines.append(indent + "    }\n")
        new_lines.append(indent + "    state.is_vietnamese_enabled = !state.is_vietnamese_enabled;\n")
        new_lines.append(indent + "    state.intercepted_keys.insert(key);\n")
        new_lines.append(indent + "    return;\n")
        new_lines.append(indent + "}\n\n")

        new_lines.append(indent + "if !state.is_vietnamese_enabled {\n")
        new_lines.append(indent + "    state.vk.as_ref().unwrap().key(time, key, key_state_u32);\n")
        new_lines.append(indent + "    return;\n")
        new_lines.append(indent + "}\n")

with open('vnikey-wayland/src/main.rs', 'w') as f:
    f.writelines(new_lines)
