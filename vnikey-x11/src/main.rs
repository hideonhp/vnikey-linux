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

    let mut engine = Engine::new(InputMethod::Telex);
    let mut intercepted_keys = HashSet::new();
    let mut is_vietnamese_enabled = true;

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
                        }
                    }
                    is_vietnamese_enabled = !is_vietnamese_enabled;
                    intercepted_keys.insert(keycode);
                    continue;
                }

                if !is_vietnamese_enabled {
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
                        Action::Preedit(buffer) | Action::Commit(buffer) => {
                            let text = String::from_iter(buffer.as_slice());
                            println!("Output: {}", text);
                            intercepted_keys.insert(keycode);
                        }
                        Action::PassThrough => {
                            pass_through_key(&conn, root, keycode, true)?;
                        }
                    }
                } else {
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
