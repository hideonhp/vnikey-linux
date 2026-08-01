#!/bin/bash
patch -p0 << 'DIFF'
--- vnikey-wayland/src/main.rs
+++ vnikey-wayland/src/main.rs
@@ -160,11 +160,33 @@
             let key_state_u32: u32 = key_state.into();
             let is_pressed = key_state_u32 == 1;

             if is_pressed {
                 // Pressed
                 let xkb_keycode = key + 8;
+
+                let mut is_toggle = false;
+                if let Some(xkb_state) = state.xkb_state.as_ref() {
+                    let is_ctrl = xkb_state.mod_name_is_active(&xkbcommon::xkb::MOD_NAME_CTRL, xkbcommon::xkb::STATE_MODS_DEPRESSED);
+                    let keysym = xkb_state.key_get_one_sym(xkb_keycode.into());
+                    if is_ctrl && keysym == xkbcommon::xkb::keysyms::KEY_space {
+                        is_toggle = true;
+                    }
+                }
+
+                if is_toggle {
+                    if state.is_vietnamese_enabled {
+                        if let Some(Action::Commit(buffer)) = state.engine.set_input_method(state.engine.get_input_method()) {
+                            let text = String::from_iter(buffer.as_slice());
+                            state.im.as_ref().unwrap().commit_string(text);
+                            state.im.as_ref().unwrap().commit(0);
+                        }
+                    }
+                    state.is_vietnamese_enabled = !state.is_vietnamese_enabled;
+                    state.intercepted_keys.insert(key);
+                    return;
+                }
+
                 let c = state.xkb_state.as_ref().and_then(|xkb_state| {
                     let utf8 = xkb_state.key_get_utf8(xkb_keycode.into());
                     if utf8.is_empty() {
DIFF
