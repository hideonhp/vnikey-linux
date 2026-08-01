use wayland_client::{Connection, Dispatch, QueueHandle, protocol::{wl_registry, wl_seat}};
use wayland_protocols_misc::zwp_virtual_keyboard_v1::client::{
    zwp_virtual_keyboard_manager_v1::ZwpVirtualKeyboardManagerV1,
    zwp_virtual_keyboard_v1::ZwpVirtualKeyboardV1,
};
use std::collections::HashSet;
use std::os::fd::AsFd;

use wayland_protocols_misc::zwp_input_method_v2::client::{
    zwp_input_method_manager_v2::ZwpInputMethodManagerV2,
    zwp_input_method_v2::ZwpInputMethodV2,
    zwp_input_method_keyboard_grab_v2::{self, ZwpInputMethodKeyboardGrabV2},
};

use vnikey_core::engine::{Action, Engine, InputMethod};

struct State {
    vk_mgr: Option<ZwpVirtualKeyboardManagerV1>,
    im_mgr: Option<ZwpInputMethodManagerV2>,
    seat: Option<wl_seat::WlSeat>,
    vk: Option<ZwpVirtualKeyboardV1>,
    im: Option<ZwpInputMethodV2>,
    grab: Option<ZwpInputMethodKeyboardGrabV2>,
    engine: Engine,
    intercepted_keys: HashSet<u32>,
}

impl Dispatch<wl_registry::WlRegistry, ()> for State {
    fn event(
        state: &mut Self,
        proxy: &wl_registry::WlRegistry,
        event: <wl_registry::WlRegistry as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        qhandle: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global { name, interface, version: _ } = event {
            if interface == "zwp_virtual_keyboard_manager_v1" {
                state.vk_mgr = Some(proxy.bind::<ZwpVirtualKeyboardManagerV1, _, _>(name, 1, qhandle, ()));
            } else if interface == "zwp_input_method_manager_v2" {
                state.im_mgr = Some(proxy.bind::<ZwpInputMethodManagerV2, _, _>(name, 1, qhandle, ()));
            } else if interface == "wl_seat" {
                state.seat = Some(proxy.bind::<wl_seat::WlSeat, _, _>(name, 1, qhandle, ()));
            }
        }
    }
}

impl Dispatch<wl_seat::WlSeat, ()> for State {
    fn event(
        _state: &mut Self,
        _proxy: &wl_seat::WlSeat,
        _event: <wl_seat::WlSeat as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {}
}

impl Dispatch<ZwpVirtualKeyboardManagerV1, ()> for State {
    fn event(
        _state: &mut Self,
        _proxy: &ZwpVirtualKeyboardManagerV1,
        _event: <ZwpVirtualKeyboardManagerV1 as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {}
}

impl Dispatch<ZwpVirtualKeyboardV1, ()> for State {
    fn event(
        _state: &mut Self,
        _proxy: &ZwpVirtualKeyboardV1,
        _event: <ZwpVirtualKeyboardV1 as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {}
}

impl Dispatch<ZwpInputMethodManagerV2, ()> for State {
    fn event(
        _state: &mut Self,
        _proxy: &ZwpInputMethodManagerV2,
        _event: <ZwpInputMethodManagerV2 as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {}
}

impl Dispatch<ZwpInputMethodV2, ()> for State {
    fn event(
        _state: &mut Self,
        _proxy: &ZwpInputMethodV2,
        _event: <ZwpInputMethodV2 as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {}
}

impl Dispatch<ZwpInputMethodKeyboardGrabV2, ()> for State {
    fn event(
        state: &mut Self,
        _proxy: &ZwpInputMethodKeyboardGrabV2,
        event: <ZwpInputMethodKeyboardGrabV2 as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        if let zwp_input_method_keyboard_grab_v2::Event::Key {
            serial: _,
            time,
            key,
            state: key_state,
        } = event
        {
            // Wayland key states are typically: 1 for pressed, 0 for released
            let key_state_u32: u32 = key_state.into();
            let is_pressed = key_state_u32 == 1;

            if is_pressed {
                // Pressed
                if let Some(c) = translate_keycode(key) {
                    let action = state.engine.process_key(c);
                    match action {
                        Action::Preedit(buffer) => {
                            let text = String::from_iter(buffer.as_slice());
                            state.im.as_ref().unwrap().set_preedit_string(text, 0, 0);
                            state.im.as_ref().unwrap().commit(0);
                            state.intercepted_keys.insert(key);
                        }
                        Action::Commit(buffer) => {
                            let text = String::from_iter(buffer.as_slice());
                            state.im.as_ref().unwrap().commit_string(text);
                            state.im.as_ref().unwrap().commit(0);
                            state.intercepted_keys.insert(key);
                        }
                        Action::PassThrough => {
                            state.vk.as_ref().unwrap().key(time, key, key_state_u32);
                        }
                    }
                } else {
                    state.vk.as_ref().unwrap().key(time, key, key_state_u32);
                }
            } else {
                // Released
                if state.intercepted_keys.contains(&key) {
                    state.intercepted_keys.remove(&key);
                } else {
                    state.vk.as_ref().unwrap().key(time, key, key_state_u32);
                }
            }
        }
    }
}


fn main() {
    let conn = Connection::connect_to_env().expect("Failed to connect to Wayland");
    let display = conn.display();
    let mut event_queue = conn.new_event_queue();
    let qh = event_queue.handle();

    let _registry = display.get_registry(&qh, ());

    let mut state = State {
        vk_mgr: None,
        im_mgr: None,
        seat: None,
        vk: None,
        im: None,
        grab: None,
        engine: Engine::new(InputMethod::Telex),
        intercepted_keys: HashSet::new(),
    };

    event_queue.roundtrip(&mut state).expect("Failed to roundtrip event queue");

    if state.vk_mgr.is_none() || state.im_mgr.is_none() || state.seat.is_none() {
        panic!("Compositor does not support required IME protocols or seat!");
    }

    println!("Successfully connected to Wayland and bound IME protocols!");

    let seat = state.seat.as_ref().unwrap();

    let vk = state.vk_mgr.as_ref().unwrap().create_virtual_keyboard(seat, &qh, ());
    state.vk = Some(vk.clone());

    let im = state.im_mgr.as_ref().unwrap().get_input_method(seat, &qh, ());
    state.im = Some(im.clone());

    // Initialize keymap for virtual keyboard
    let keymap = r#"xkb_keymap { xkb_keycodes { include "evdev+aliases(qwerty)" }; xkb_types { include "complete" }; xkb_compat { include "complete" }; xkb_symbols { include "pc+us+inet(evdev)" }; };"#;

    let fd = rustix::fs::memfd_create("vnikey-xkb", rustix::fs::MemfdFlags::CLOEXEC)
        .expect("Failed to create memfd for keymap");

    rustix::io::write(&fd, keymap.as_bytes()).expect("Failed to write keymap to memfd");

    vk.keymap(
        1, // WL_KEYBOARD_KEYMAP_FORMAT_XKB_V1
        fd.as_fd(),
        keymap.len() as u32,
    );

    let grab = im.grab_keyboard(&qh, ());
    state.grab = Some(grab);

    println!("Starting input interception loop...");
    loop {
        event_queue.blocking_dispatch(&mut state).expect("Failed to dispatch Wayland events");
    }
}

// Minimal hardcoded evdev-to-qwerty map
fn translate_keycode(keycode: u32) -> Option<char> {
    match keycode {
        16 => Some('q'),
        17 => Some('w'),
        18 => Some('e'),
        19 => Some('r'),
        20 => Some('t'),
        21 => Some('y'),
        22 => Some('u'),
        23 => Some('i'),
        24 => Some('o'),
        25 => Some('p'),
        26 => Some('['),
        27 => Some(']'),
        30 => Some('a'),
        31 => Some('s'),
        32 => Some('d'),
        33 => Some('f'),
        34 => Some('g'),
        35 => Some('h'),
        36 => Some('j'),
        37 => Some('k'),
        38 => Some('l'),
        39 => Some(';'),
        40 => Some('\''),
        44 => Some('z'),
        45 => Some('x'),
        46 => Some('c'),
        47 => Some('v'),
        48 => Some('b'),
        49 => Some('n'),
        50 => Some('m'),
        51 => Some(','),
        52 => Some('.'),
        53 => Some('/'),
        2 => Some('1'),
        3 => Some('2'),
        4 => Some('3'),
        5 => Some('4'),
        6 => Some('5'),
        7 => Some('6'),
        8 => Some('7'),
        9 => Some('8'),
        10 => Some('9'),
        11 => Some('0'),
        14 => Some('\x08'), // Backspace
        57 => Some(' '),
        _ => None,
    }
}
