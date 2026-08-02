use std::collections::HashSet;
use vnikey_core::engine::{Action, Engine, InputMethod};
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{ConnectionExt as _, GrabMode};
use x11rb::protocol::xtest::ConnectionExt as _;
use x11rb::protocol::Event;
use x11rb::CURRENT_TIME;

fn pass_through_key<C: Connection>(
    conn: &C,
    root: u32,
    keycode: u8,
    is_press: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let type_ = if is_press { 2 } else { 3 }; // KeyPress = 2, KeyRelease = 3 in X11

    // Ungrab
    conn.ungrab_keyboard(CURRENT_TIME)?;

    // Inject
    conn.xtest_fake_input(type_, keycode, CURRENT_TIME, root, 0, 0, 0)?;
    conn.flush()?;

    // Regrab
    conn.grab_keyboard(false, root, CURRENT_TIME, GrabMode::ASYNC, GrabMode::ASYNC)?.reply()?;
    conn.flush()?;

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (conn, screen_num) =
        x11rb::xcb_ffi::XCBConnection::connect(None).expect("Panic: Cannot connect to X11 server.");
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

    let mut xkb_state = xkbcommon::xkb::x11::state_new_from_device(
        &keymap,
        &conn,
        device_id,
    );

    let mut shift_l_keycode: Option<u8> = None;
    let mut insert_keycode: Option<u8> = None;
    let mut backspace_keycode: Option<u8> = None;

    let min_kc: u32 = keymap.min_keycode().into();
    let max_kc: u32 = keymap.max_keycode().into();
    for kc in min_kc..=max_kc {
        let keycode = xkbcommon::xkb::Keycode::from(kc);
        let syms = keymap.key_get_syms_by_level(keycode, 0, 0);
        if shift_l_keycode.is_none() && syms.contains(&xkbcommon::xkb::keysyms::KEY_Shift_L.into()) {
            shift_l_keycode = Some(kc as u8);
        }
        if insert_keycode.is_none() && syms.contains(&xkbcommon::xkb::keysyms::KEY_Insert.into()) {
            insert_keycode = Some(kc as u8);
        }
        if backspace_keycode.is_none() && syms.contains(&xkbcommon::xkb::keysyms::KEY_BackSpace.into()) {
            backspace_keycode = Some(kc as u8);
        }
    }

    println!("Detected Shift_L keycode: {:?}", shift_l_keycode);
    println!("Detected Insert keycode: {:?}", insert_keycode);
    println!("Detected BackSpace keycode: {:?}", backspace_keycode);

    let mut engine = Engine::new(InputMethod::Telex);
    let mut intercepted_keys = HashSet::new();
    let mut is_vietnamese_enabled = true;
    let mut current_preedit_len: usize = 0;

    conn.grab_keyboard(false, root, CURRENT_TIME, GrabMode::ASYNC, GrabMode::ASYNC)?.reply()?;
    println!("Keyboard grabbed successfully.");

    loop {
        let event = conn.wait_for_event()?;
        match event {
            Event::KeyPress(event) => {
                let keycode = event.detail;
                xkb_state.update_key(keycode.into(), xkbcommon::xkb::KeyDirection::Down);

                if keycode == 9 { // ESC
                    println!("ESC pressed. Emergency exit.");
                    conn.ungrab_keyboard(CURRENT_TIME)?;
                    conn.flush()?;
                    std::process::exit(0);
                }

                let mut is_toggle = false;
                let is_ctrl = xkb_state.mod_name_is_active(
                    &xkbcommon::xkb::MOD_NAME_CTRL,
                    xkbcommon::xkb::STATE_MODS_DEPRESSED,
                );
                let keysym = xkb_state.key_get_one_sym(keycode.into());
                if is_ctrl && keysym == xkbcommon::xkb::keysyms::KEY_space.into() {
                    is_toggle = true;
                }

                if is_toggle {
                    if is_vietnamese_enabled {
                        if let Some(Action::Commit(buffer)) =
                            engine.set_input_method(engine.get_input_method())
                        {
                            let text = String::from_iter(buffer.as_slice());
                            println!("Output: {}", text);

                            // Inject text using clipboard MVP hack on toggle out
                            if let (Some(shift_l), Some(insert)) = (shift_l_keycode, insert_keycode) {
                                if let Ok(mut clipboard) = arboard::Clipboard::new() {
                                    let _ = clipboard.set_text(text);
                                    let _ = conn.ungrab_keyboard(CURRENT_TIME);
                                    let _ = conn.flush();

                                    let _ = conn.xtest_fake_input(2, shift_l, CURRENT_TIME, root, 0, 0, 0);
                                    let _ = conn.xtest_fake_input(2, insert, CURRENT_TIME, root, 0, 0, 0);
                                    let _ = conn.xtest_fake_input(3, insert, CURRENT_TIME, root, 0, 0, 0);
                                    let _ = conn.xtest_fake_input(3, shift_l, CURRENT_TIME, root, 0, 0, 0);

                                    let _ = conn.flush();
                                    std::thread::sleep(std::time::Duration::from_millis(20));

                                    if let Ok(cookie) = conn.grab_keyboard(false, root, CURRENT_TIME, GrabMode::ASYNC, GrabMode::ASYNC) {
                                        let _ = cookie.reply();
                                    }
                                    let _ = conn.flush();
                                }
                            }
                        }
                    }
                    is_vietnamese_enabled = !is_vietnamese_enabled;
                    current_preedit_len = 0;
                    intercepted_keys.insert(keycode);
                    continue;
                }

                if !is_vietnamese_enabled {
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
                            let text = String::from_iter(buffer.as_slice());

                            if let (Some(shift_l), Some(insert), Some(backspace)) = (shift_l_keycode, insert_keycode, backspace_keycode) {
                                if let Ok(mut clipboard) = arboard::Clipboard::new() {
                                    let _ = clipboard.set_text(text.clone());

                                    // Ungrab
                                    let _ = conn.ungrab_keyboard(CURRENT_TIME);
                                    let _ = conn.flush();

                                    // Backspace simulation
                                    for _ in 0..current_preedit_len {
                                        let _ = conn.xtest_fake_input(2, backspace, CURRENT_TIME, root, 0, 0, 0);
                                        let _ = conn.xtest_fake_input(3, backspace, CURRENT_TIME, root, 0, 0, 0);
                                    }

                                    // Shift_L Down (type_ = 2)
                                    let _ = conn.xtest_fake_input(2, shift_l, CURRENT_TIME, root, 0, 0, 0);
                                    // Insert Down (type_ = 2)
                                    let _ = conn.xtest_fake_input(2, insert, CURRENT_TIME, root, 0, 0, 0);
                                    // Insert Up (type_ = 3)
                                    let _ = conn.xtest_fake_input(3, insert, CURRENT_TIME, root, 0, 0, 0);
                                    // Shift_L Up (type_ = 3)
                                    let _ = conn.xtest_fake_input(3, shift_l, CURRENT_TIME, root, 0, 0, 0);

                                    let _ = conn.flush();

                                    // Short sleep to allow X clients to process the events
                                    std::thread::sleep(std::time::Duration::from_millis(20));

                                    // Regrab
                                    if let Ok(cookie) = conn.grab_keyboard(false, root, CURRENT_TIME, GrabMode::ASYNC, GrabMode::ASYNC) {
                                        let _ = cookie.reply();
                                    }
                                    let _ = conn.flush();
                                }
                            }

                            current_preedit_len = text.chars().count();
                            intercepted_keys.insert(keycode);
                        }
                        Action::Commit(buffer) => {
                            let text = String::from_iter(buffer.as_slice());
                            println!("Output: {}", text);

                            // Inject text using clipboard MVP hack
                            if let (Some(shift_l), Some(insert), Some(backspace)) = (shift_l_keycode, insert_keycode, backspace_keycode) {
                                if let Ok(mut clipboard) = arboard::Clipboard::new() {
                                    let _ = clipboard.set_text(text);

                                    // Ungrab
                                    let _ = conn.ungrab_keyboard(CURRENT_TIME);
                                    let _ = conn.flush();

                                    // Backspace simulation
                                    for _ in 0..current_preedit_len {
                                        let _ = conn.xtest_fake_input(2, backspace, CURRENT_TIME, root, 0, 0, 0);
                                        let _ = conn.xtest_fake_input(3, backspace, CURRENT_TIME, root, 0, 0, 0);
                                    }

                                    // Shift_L Down (type_ = 2)
                                    let _ = conn.xtest_fake_input(2, shift_l, CURRENT_TIME, root, 0, 0, 0);
                                    // Insert Down (type_ = 2)
                                    let _ = conn.xtest_fake_input(2, insert, CURRENT_TIME, root, 0, 0, 0);
                                    // Insert Up (type_ = 3)
                                    let _ = conn.xtest_fake_input(3, insert, CURRENT_TIME, root, 0, 0, 0);
                                    // Shift_L Up (type_ = 3)
                                    let _ = conn.xtest_fake_input(3, shift_l, CURRENT_TIME, root, 0, 0, 0);

                                    let _ = conn.flush();

                                    // Short sleep to allow X clients to process the paste
                                    std::thread::sleep(std::time::Duration::from_millis(20));

                                    // Regrab
                                    if let Ok(cookie) = conn.grab_keyboard(false, root, CURRENT_TIME, GrabMode::ASYNC, GrabMode::ASYNC) {
                                        let _ = cookie.reply();
                                    }
                                    let _ = conn.flush();
                                }
                            }

                            current_preedit_len = 0;
                            intercepted_keys.insert(keycode);
                        }
                        Action::PassThrough => {
                            current_preedit_len = 0;
                            pass_through_key(&conn, root, keycode, true)?;
                        }
                    }
                } else {
                    current_preedit_len = 0;
                    pass_through_key(&conn, root, keycode, true)?;
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
