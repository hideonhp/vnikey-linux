use vnikey_core::window_state::WindowStateManager;

use notify::{EventKind, RecursiveMode, Watcher};
use std::collections::HashSet;
use std::sync::RwLock;
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU8, Ordering},
};
use vnikey_config::Config;
use vnikey_core::engine::{Action, Engine, InputMethod};
use x11rb::CURRENT_TIME;
use x11rb::connection::Connection;
use x11rb::protocol::Event;
use x11rb::protocol::xproto::{ChangeWindowAttributesAux, ConnectionExt as _, EventMask, GrabMode};
use x11rb::protocol::xtest::ConnectionExt as _;

/// XTest event type for KeyPress
const KEY_PRESS: u8 = 2;
/// XTest event type for KeyRelease
const KEY_RELEASE: u8 = 3;

fn inject_text_via_clipboard<C: Connection>(
    conn: &C,
    root: u32,
    text: &str,
    shift_l_keycode: Option<u8>,
    insert_keycode: Option<u8>,
    backspace_keycode: Option<u8>,
    backspaces_to_send: usize,
) {
    if let (Some(shift_l), Some(insert)) = (shift_l_keycode, insert_keycode)
        && let Ok(mut clipboard) = arboard::Clipboard::new()
    {
        // [NEW] 1. Save current clipboard
        let saved_clipboard = clipboard.get_text().ok();

        struct ClipboardGuard<'a> {
            clipboard: &'a mut arboard::Clipboard,
            saved: Option<String>,
        }

        impl<'a> Drop for ClipboardGuard<'a> {
            fn drop(&mut self) {
                if let Some(saved) = self.saved.take() {
                    let _ = self.clipboard.set_text(saved);
                } else {
                    let _ = self.clipboard.clear();
                }
            }
        }

        let mut guard = ClipboardGuard {
            clipboard: &mut clipboard,
            saved: saved_clipboard,
        };

        let _ = guard.clipboard.set_text(text);
        let _ = conn.ungrab_keyboard(CURRENT_TIME);
        let _ = conn.flush();

        if let Some(backspace) = backspace_keycode {
            for _ in 0..backspaces_to_send {
                let _ = conn.xtest_fake_input(KEY_PRESS, backspace, CURRENT_TIME, root, 0, 0, 0);
                let _ = conn.xtest_fake_input(KEY_RELEASE, backspace, CURRENT_TIME, root, 0, 0, 0);
            }
        }

        let _ = conn.xtest_fake_input(KEY_PRESS, shift_l, CURRENT_TIME, root, 0, 0, 0);
        let _ = conn.xtest_fake_input(KEY_PRESS, insert, CURRENT_TIME, root, 0, 0, 0);
        let _ = conn.xtest_fake_input(KEY_RELEASE, insert, CURRENT_TIME, root, 0, 0, 0);
        let _ = conn.xtest_fake_input(KEY_RELEASE, shift_l, CURRENT_TIME, root, 0, 0, 0);

        let _ = conn.flush();

        // Wait for target app to read clipboard
        std::thread::sleep(std::time::Duration::from_millis(20));

        // ClipboardGuard will automatically restore clipboard when dropped here.

        if let Ok(cookie) =
            conn.grab_keyboard(false, root, CURRENT_TIME, GrabMode::ASYNC, GrabMode::ASYNC)
        {
            let _ = cookie.reply();
        }
        let _ = conn.flush();
    }
}

fn pass_through_key<C: Connection>(
    conn: &C,
    root: u32,
    keycode: u8,
    is_press: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let type_ = if is_press { KEY_PRESS } else { KEY_RELEASE };

    // Ungrab
    conn.ungrab_keyboard(CURRENT_TIME)?;

    // Inject
    conn.xtest_fake_input(type_, keycode, CURRENT_TIME, root, 0, 0, 0)?;
    conn.flush()?;

    // Regrab
    conn.grab_keyboard(false, root, CURRENT_TIME, GrabMode::ASYNC, GrabMode::ASYNC)?
        .reply()?;
    conn.flush()?;

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load();
    let start_enabled = config.start_enabled;
    let initial_input_method = config.get_input_method();

    let input_method_tray_val = if initial_input_method == InputMethod::Vni {
        1
    } else {
        0
    };
    let input_method_tray = Arc::new(AtomicU8::new(input_method_tray_val));

    let config_lock = Arc::new(RwLock::new(config));

    let watcher_config_lock = Arc::clone(&config_lock);
    let watcher_input_method_tray = Arc::clone(&input_method_tray);
    let mut watcher =
        notify::recommended_watcher(move |res: notify::Result<notify::Event>| match res {
            Ok(event) => {
                if let EventKind::Modify(_) | EventKind::Create(_) = event.kind {
                    let new_config = Config::load();
                    let new_im_val = if new_config.get_input_method() == InputMethod::Vni {
                        1
                    } else {
                        0
                    };
                    if let Ok(mut lock) = watcher_config_lock.write() {
                        *lock = new_config;
                        watcher_input_method_tray.store(new_im_val, Ordering::Relaxed);
                        println!("Configuration reloaded!");
                    }
                }
            }
            Err(e) => eprintln!("watch error: {:?}", e),
        })
        .expect("Failed to create config watcher");

    if let Some(proj_dirs) = directories::ProjectDirs::from("", "", "vnikey") {
        let config_dir = proj_dirs.config_dir().to_path_buf();
        if !config_dir.exists() {
            #[allow(clippy::collapsible_if)]
            if let Err(e) = std::fs::create_dir_all(&config_dir) {
                eprintln!(
                    "Warning: Failed to create config directory at {:?}: {}",
                    config_dir, e
                );
            }
        }
        if config_dir.exists() {
            if let Err(e) = watcher.watch(&config_dir, RecursiveMode::NonRecursive) {
                eprintln!(
                    "Warning: Failed to watch config directory at {:?}: {}",
                    config_dir, e
                );
            }
        } else {
            eprintln!(
                "Warning: Config directory does not exist at {:?}, skipping watcher setup.",
                config_dir
            );
        }
    }
    let (conn, screen_num) = x11rb::xcb_ffi::XCBConnection::connect(None)?;
    let setup = conn.setup();
    let screen = &setup.roots[screen_num];
    let root = screen.root;

    println!("Successfully connected to X11 Server!");

    // Verify xtest extension exists
    let _xtest = conn.xtest_get_version(2, 2)?.reply()?;

    // Initialize XKB Context and State
    let xkb_context = xkbcommon::xkb::Context::new(xkbcommon::xkb::CONTEXT_NO_FLAGS);

    // Get raw XCB connection pointer
    let device_id = xkbcommon::xkb::x11::get_core_keyboard_device_id(&conn);
    if device_id == -1 {
        return Err("Failed to get core keyboard device ID".into());
    }

    let keymap = xkbcommon::xkb::x11::keymap_new_from_device(
        &xkb_context,
        &conn,
        device_id,
        xkbcommon::xkb::KEYMAP_COMPILE_NO_FLAGS,
    );

    let mut xkb_state = xkbcommon::xkb::x11::state_new_from_device(&keymap, &conn, device_id);

    let mut shift_l_keycode: Option<u8> = None;
    let mut insert_keycode: Option<u8> = None;
    let mut backspace_keycode: Option<u8> = None;

    let min_kc: u32 = keymap.min_keycode().into();
    let max_kc: u32 = keymap.max_keycode().into();
    for kc in min_kc..=max_kc {
        let keycode = xkbcommon::xkb::Keycode::from(kc);
        let syms = keymap.key_get_syms_by_level(keycode, 0, 0);
        if shift_l_keycode.is_none() && syms.contains(&xkbcommon::xkb::keysyms::KEY_Shift_L.into())
        {
            shift_l_keycode = Some(kc as u8);
        }
        if insert_keycode.is_none() && syms.contains(&xkbcommon::xkb::keysyms::KEY_Insert.into()) {
            insert_keycode = Some(kc as u8);
        }
        if backspace_keycode.is_none()
            && syms.contains(&xkbcommon::xkb::keysyms::KEY_BackSpace.into())
        {
            backspace_keycode = Some(kc as u8);
        }
    }

    println!("Detected Shift_L keycode: {:?}", shift_l_keycode);
    println!("Detected Insert keycode: {:?}", insert_keycode);
    println!("Detected BackSpace keycode: {:?}", backspace_keycode);

    let mut engine = Engine::new(initial_input_method, true);
    let mut intercepted_keys = HashSet::new();
    let is_vietnamese_enabled = Arc::new(AtomicBool::new(start_enabled));
    let tray_handle = vnikey_tray::spawn_tray(
        Arc::clone(&is_vietnamese_enabled),
        Arc::clone(&input_method_tray),
    );
    let mut current_preedit_len: usize = 0;
    let mut text_buffer = String::with_capacity(64);

    let net_active_window = conn
        .intern_atom(false, b"_NET_ACTIVE_WINDOW")?
        .reply()?
        .atom;
    conn.change_window_attributes(
        root,
        &ChangeWindowAttributesAux::new().event_mask(EventMask::PROPERTY_CHANGE),
    )?;

    conn.grab_keyboard(false, root, CURRENT_TIME, GrabMode::ASYNC, GrabMode::ASYNC)?
        .reply()?;
    println!("Keyboard grabbed successfully.");

    let mut window_manager = WindowStateManager::new();

    loop {
        let event = conn.wait_for_event()?;
        match event {
            Event::PropertyNotify(event) => {
                if event.atom == net_active_window {
                    engine.reset_context();
                    current_preedit_len = 0;

                    let current_config = config_lock.read().unwrap_or_else(|e| e.into_inner());
                    if current_config.per_window_state
                        && let Ok(cookie) = conn.get_property(
                            false,
                            root,
                            net_active_window,
                            x11rb::protocol::xproto::AtomEnum::WINDOW,
                            0,
                            1,
                        )
                        && let Ok(reply) = cookie.reply()
                        && let Some(value) = reply.value32().and_then(|mut iter| iter.next())
                    {
                        window_manager.set_active_window(value);
                        if let Some(saved_state) = window_manager.get_state_for_current_window() {
                            is_vietnamese_enabled.store(saved_state, Ordering::SeqCst);
                            tray_handle.update(|_| {});
                        }
                    }
                }
            }
            Event::KeyPress(event) => {
                let keycode = event.detail;
                xkb_state.update_key(keycode.into(), xkbcommon::xkb::KeyDirection::Down);

                let current_config = config_lock.read().unwrap_or_else(|e| e.into_inner());

                if keycode == 9 && current_config.vim_mode {
                    let is_enabled = is_vietnamese_enabled.load(Ordering::SeqCst);
                    if is_enabled {
                        if let Some(Action::Commit(buffer)) = engine.flush() {
                                text_buffer.clear();
                                use std::fmt::Write;
                                let _ = write!(text_buffer, "{}", buffer);
                                inject_text_via_clipboard(
                                    &conn,
                                    root,
                                    &text_buffer,
                                shift_l_keycode,
                                insert_keycode,
                                backspace_keycode,
                                current_preedit_len,
                            );
                        }
                        is_vietnamese_enabled.store(false, Ordering::SeqCst);
                        tray_handle.update(|_| {});
                        current_preedit_len = 0;
                        if current_config.per_window_state {
                            window_manager.save_state_for_current_window(false);
                        }
                    }
                }

                let tray_im_val = input_method_tray.load(Ordering::Relaxed);
                let tray_im = if tray_im_val == 1 {
                    InputMethod::Vni
                } else {
                    InputMethod::Telex
                };

                if tray_im != engine.get_input_method() {
                    engine.set_input_method(tray_im);
                }
                if current_config.spell_check != engine.spell_check {
                    engine.spell_check = current_config.spell_check;
                }
                let mut is_toggle = false;

                let has_ctrl = xkb_state.mod_name_is_active(
                    &xkbcommon::xkb::MOD_NAME_CTRL,
                    xkbcommon::xkb::STATE_MODS_DEPRESSED,
                );
                let has_shift = xkb_state.mod_name_is_active(
                    &xkbcommon::xkb::MOD_NAME_SHIFT,
                    xkbcommon::xkb::STATE_MODS_DEPRESSED,
                );
                let has_alt = xkb_state.mod_name_is_active(
                    &xkbcommon::xkb::MOD_NAME_ALT,
                    xkbcommon::xkb::STATE_MODS_DEPRESSED,
                );
                let has_super = xkb_state.mod_name_is_active(
                    &xkbcommon::xkb::MOD_NAME_LOGO,
                    xkbcommon::xkb::STATE_MODS_DEPRESSED,
                );

                let keysym = xkb_state.key_get_one_sym(keycode.into());
                let key_name = xkbcommon::xkb::keysym_get_name(keysym).to_lowercase();

                let config_mod = current_config.get_toggle_modifier_normalized();
                let config_key = current_config.get_toggle_key_normalized();

                let mod_match = if config_mod.is_empty() {
                    true
                } else {
                    (has_ctrl
                        && (config_mod.contains("control") || "control".contains(&config_mod)))
                        || (has_shift
                            && (config_mod.contains("shift") || "shift".contains(&config_mod)))
                        || (has_alt && (config_mod.contains("alt") || "alt".contains(&config_mod)))
                        || (has_super
                            && (config_mod.contains("super") || "super".contains(&config_mod)))
                };

                let key_match = key_name == config_key || key_name.contains(&config_key);

                if mod_match && key_match {
                    is_toggle = true;
                }

                if is_toggle {
                    let is_enabled = is_vietnamese_enabled.load(Ordering::SeqCst);
                    let new_state = !is_enabled;
                    if is_enabled
                        && let Some(Action::Commit(buffer)) =
                            engine.set_input_method(engine.get_input_method())
                    {
                        text_buffer.clear();
                        use std::fmt::Write;
                        let _ = write!(text_buffer, "{}", buffer);
                        println!("Output: {}", text_buffer);

                        inject_text_via_clipboard(
                            &conn,
                            root,
                            &text_buffer,
                            shift_l_keycode,
                            insert_keycode,
                            None,
                            0,
                        );
                    }
                    is_vietnamese_enabled.store(new_state, Ordering::SeqCst);
                    tray_handle.update(|_| {});
                    if current_config.per_window_state {
                        window_manager.save_state_for_current_window(new_state);
                    }
                    current_preedit_len = 0;
                    intercepted_keys.insert(keycode);
                    continue;
                }

                let is_enabled = is_vietnamese_enabled.load(Ordering::SeqCst);
                if !is_enabled {
                    current_preedit_len = 0;
                    pass_through_key(&conn, root, keycode, true)?;
                    continue;
                }

                let utf8 = xkb_state.key_get_utf8(keycode.into());
                let c = if utf8.is_empty() {
                    None
                } else {
                    utf8.chars().next()
                };

                if let Some(c) = c {
                    let action = engine.process_key(c);
                    match action {
                        Action::Preedit(buffer) => {
                            text_buffer.clear();
                            use std::fmt::Write;
                            let _ = write!(text_buffer, "{}", buffer);

                            let text_len = text_buffer.chars().count();
                            inject_text_via_clipboard(
                                &conn,
                                root,
                                &text_buffer,
                                shift_l_keycode,
                                insert_keycode,
                                backspace_keycode,
                                current_preedit_len,
                            );

                            current_preedit_len = text_len;
                            intercepted_keys.insert(keycode);
                        }
                        Action::Commit(buffer) => {
                            text_buffer.clear();
                            use std::fmt::Write;
                            let _ = write!(text_buffer, "{}", buffer);
                            println!("Output: {}", text_buffer);

                            inject_text_via_clipboard(
                                &conn,
                                root,
                                &text_buffer,
                                shift_l_keycode,
                                insert_keycode,
                                backspace_keycode,
                                current_preedit_len,
                            );

                            current_preedit_len = 0;
                            intercepted_keys.insert(keycode);
                        }
                        Action::CommitAndPassThrough(buffer) => {
                            text_buffer.clear();
                            use std::fmt::Write;
                            let _ = write!(text_buffer, "{}", buffer);
                            println!("Output: {}", text_buffer);

                            inject_text_via_clipboard(
                                &conn,
                                root,
                                &text_buffer,
                                shift_l_keycode,
                                insert_keycode,
                                backspace_keycode,
                                current_preedit_len,
                            );

                            current_preedit_len = 0;
                            // Need to pass through the key
                            pass_through_key(&conn, root, keycode, true)?;
                        }
                        Action::PassThrough => {
                            current_preedit_len = 0;
                            pass_through_key(&conn, root, keycode, true)?;
                        }
                        Action::SurroundingRecompose {
                            preedit,
                            delete_count,
                            ..
                        } => {
                            text_buffer.clear();
                            use std::fmt::Write;
                            let _ = write!(text_buffer, "{}", preedit);
                            let text_len = text_buffer.chars().count();

                            inject_text_via_clipboard(
                                &conn,
                                root,
                                &text_buffer,
                                shift_l_keycode,
                                insert_keycode,
                                backspace_keycode,
                                delete_count,
                            );

                            current_preedit_len = text_len;
                            intercepted_keys.insert(keycode);
                        }
                    }
                } else {
                    let keysym = xkb_state.key_get_one_sym(keycode.into()).raw();
                    #[allow(non_upper_case_globals)]
                    let is_modifier = {
                        use xkbcommon::xkb::keysyms::*;
                        matches!(
                            keysym,
                            KEY_Shift_L
                                | KEY_Shift_R
                                | KEY_Control_L
                                | KEY_Control_R
                                | KEY_Alt_L
                                | KEY_Alt_R
                                | KEY_Super_L
                                | KEY_Super_R
                                | KEY_Meta_L
                                | KEY_Meta_R
                                | KEY_Caps_Lock
                                | KEY_Num_Lock
                        )
                    };

                    if is_modifier {
                        pass_through_key(&conn, root, keycode, true)?;
                    } else {
                        // Navigation/function keys: flush engine (commit preedit), then pass through
                        if let Some(Action::Commit(buffer)) = engine.flush() {
                            text_buffer.clear();
                            use std::fmt::Write;
                            let _ = write!(text_buffer, "{}", buffer);
                            inject_text_via_clipboard(
                                &conn,
                                root,
                                &text_buffer,
                                shift_l_keycode,
                                insert_keycode,
                                backspace_keycode,
                                current_preedit_len,
                            );
                        }
                        current_preedit_len = 0;
                        pass_through_key(&conn, root, keycode, true)?;
                    }
                }
            }
            Event::KeyRelease(event) => {
                let keycode = event.detail;
                xkb_state.update_key(keycode.into(), xkbcommon::xkb::KeyDirection::Up);

                if intercepted_keys.contains(&keycode) {
                    intercepted_keys.remove(&keycode);
                } else {
                    pass_through_key(&conn, root, keycode, false)?;
                }
            }
            _ => {}
        }
    }
}
