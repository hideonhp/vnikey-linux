use std::collections::HashSet;
use vnikey_core::engine::{Action, Engine, InputMethod};
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{ConnectionExt as _, GrabMode};
use x11rb::protocol::xtest::ConnectionExt as _;
use x11rb::protocol::Event;
use x11rb::CURRENT_TIME;

fn map_keycode_to_char(keycode: u8) -> Option<char> {
    match keycode {
        24 => Some('q'),
        25 => Some('w'),
        26 => Some('e'),
        27 => Some('r'),
        28 => Some('t'),
        29 => Some('y'),
        30 => Some('u'),
        31 => Some('i'),
        32 => Some('o'),
        33 => Some('p'),
        38 => Some('a'),
        39 => Some('s'),
        40 => Some('d'),
        41 => Some('f'),
        42 => Some('g'),
        43 => Some('h'),
        44 => Some('j'),
        45 => Some('k'),
        46 => Some('l'),
        52 => Some('z'),
        53 => Some('x'),
        54 => Some('c'),
        55 => Some('v'),
        56 => Some('b'),
        57 => Some('n'),
        58 => Some('m'),
        10 => Some('1'),
        11 => Some('2'),
        12 => Some('3'),
        13 => Some('4'),
        14 => Some('5'),
        15 => Some('6'),
        16 => Some('7'),
        17 => Some('8'),
        18 => Some('9'),
        19 => Some('0'),
        _ => None,
    }
}

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
    let (conn, screen_num) = x11rb::connect(None).expect("Panic: Cannot connect to X11 server.");
    let setup = conn.setup();
    let screen = &setup.roots[screen_num];
    let root = screen.root;

    println!("Successfully connected to X11 Server!");

    // Verify xtest extension exists
    let _xtest = conn.xtest_get_version(2, 2)?.reply()?;

    let mut engine = Engine::new(InputMethod::Telex);
    let mut intercepted_keys = HashSet::new();

    conn.grab_keyboard(false, root, CURRENT_TIME, GrabMode::ASYNC, GrabMode::ASYNC)?.reply()?;
    println!("Keyboard grabbed successfully.");

    loop {
        let event = conn.wait_for_event()?;
        match event {
            Event::KeyPress(event) => {
                let keycode = event.detail;
                if keycode == 9 { // ESC
                    println!("ESC pressed. Emergency exit.");
                    conn.ungrab_keyboard(CURRENT_TIME)?;
                    conn.flush()?;
                    std::process::exit(0);
                }

                if let Some(c) = map_keycode_to_char(keycode) {
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
