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
use xkbcommon::xkb::{Context, Keymap, State as XkbState, CONTEXT_NO_FLAGS};

struct State {
    vk_mgr: Option<ZwpVirtualKeyboardManagerV1>,
    im_mgr: Option<ZwpInputMethodManagerV2>,
    seat: Option<wl_seat::WlSeat>,
    vk: Option<ZwpVirtualKeyboardV1>,
    im: Option<ZwpInputMethodV2>,
    grab: Option<ZwpInputMethodKeyboardGrabV2>,
    engine: Engine,
    intercepted_keys: HashSet<u32>,
    xkb_context: Context,
    xkb_state: Option<XkbState>,
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

use std::fs::File;
use std::io::Read;

impl Dispatch<ZwpInputMethodKeyboardGrabV2, ()> for State {
    fn event(
        state: &mut Self,
        _proxy: &ZwpInputMethodKeyboardGrabV2,
        event: <ZwpInputMethodKeyboardGrabV2 as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        match event {
            zwp_input_method_keyboard_grab_v2::Event::Keymap { format: _, fd, size: _ } => {
                let mut file = File::from(fd);
                let mut string = String::new();
                if file.read_to_string(&mut string).is_ok() {
                    let string = string.trim_end_matches('\0').to_string();
                    if let Some(keymap) = Keymap::new_from_string(
                        &state.xkb_context,
                        string,
                        xkbcommon::xkb::KEYMAP_FORMAT_TEXT_V1,
                        xkbcommon::xkb::KEYMAP_COMPILE_NO_FLAGS,
                    ) {
                        state.xkb_state = Some(XkbState::new(&keymap));
                    }
                }
            }
            zwp_input_method_keyboard_grab_v2::Event::Modifiers {
                serial: _,
                mods_depressed,
                mods_latched,
                mods_locked,
                group,
            } => {
                if let Some(xkb_state) = &mut state.xkb_state {
                    xkb_state.update_mask(
                        mods_depressed,
                        mods_latched,
                        mods_locked,
                        0,
                        0,
                        group,
                    );
                }
            }
            zwp_input_method_keyboard_grab_v2::Event::Key {
                serial: _,
                time,
                key,
                state: key_state,
            } => {
            // Wayland key states are typically: 1 for pressed, 0 for released
            let key_state_u32: u32 = key_state.into();
            let is_pressed = key_state_u32 == 1;

            if is_pressed {
                // Pressed
                let xkb_keycode = key + 8;
                let c = state.xkb_state.as_ref().and_then(|xkb_state| {
                    let utf8 = xkb_state.key_get_utf8(xkb_keycode.into());
                    if utf8.is_empty() {
                        None
                    } else {
                        utf8.chars().next()
                    }
                });

                if let Some(c) = c {
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
            _ => {}
        }
    }
}


fn main() {
    let conn = Connection::connect_to_env().expect("Failed to connect to Wayland");
    let display = conn.display();
    let mut event_queue = conn.new_event_queue();
    let qh = event_queue.handle();

    let _registry = display.get_registry(&qh, ());

    let xkb_context = Context::new(CONTEXT_NO_FLAGS);
    let mut state = State {
        vk_mgr: None,
        im_mgr: None,
        seat: None,
        vk: None,
        im: None,
        grab: None,
        engine: Engine::new(InputMethod::Telex),
        intercepted_keys: HashSet::new(),
        xkb_context,
        xkb_state: None,
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

