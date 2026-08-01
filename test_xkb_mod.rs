fn main() {
    let ctx = xkbcommon::xkb::Context::new(xkbcommon::xkb::CONTEXT_NO_FLAGS);
    let keymap_str = r#"xkb_keymap { xkb_keycodes { include "evdev+aliases(qwerty)" }; xkb_types { include "complete" }; xkb_compat { include "complete" }; xkb_symbols { include "pc+us+inet(evdev)" }; };"#;
    let keymap = xkbcommon::xkb::Keymap::new_from_string(&ctx, keymap_str.to_string(), xkbcommon::xkb::KEYMAP_FORMAT_TEXT_V1, xkbcommon::xkb::KEYMAP_COMPILE_NO_FLAGS).unwrap();
    let state = xkbcommon::xkb::State::new(&keymap);
    println!("{:?}", state.mod_name_is_active(&xkbcommon::xkb::MOD_NAME_CTRL, xkbcommon::xkb::STATE_MODS_DEPRESSED));
}
