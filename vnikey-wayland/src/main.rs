use notify::{EventKind, RecursiveMode, Watcher};
use std::collections::HashSet;
use std::os::fd::AsFd;
use std::sync::RwLock;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use wayland_client::{
    Connection, Dispatch, QueueHandle,
    protocol::{wl_registry, wl_seat},
};
use wayland_protocols_misc::zwp_virtual_keyboard_v1::client::{
    zwp_virtual_keyboard_manager_v1::ZwpVirtualKeyboardManagerV1,
    zwp_virtual_keyboard_v1::ZwpVirtualKeyboardV1,
};

use wayland_protocols_misc::zwp_input_method_v2::client::{
    zwp_input_method_keyboard_grab_v2::{self, ZwpInputMethodKeyboardGrabV2},
    zwp_input_method_manager_v2::ZwpInputMethodManagerV2,
    zwp_input_method_v2::{self, ZwpInputMethodV2},
};

use vnikey_core::engine::{Action, Engine};
use xkbcommon::xkb::{CONTEXT_NO_FLAGS, Context, Keymap, State as XkbState};

use vnikey_config::Config;

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
    is_vietnamese_enabled: Arc<AtomicBool>,
    tray_handle: ksni::blocking::Handle<vnikey_tray::VnikeyTray>,
    config: Arc<RwLock<Config>>,
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
        if let wl_registry::Event::Global {
            name,
            interface,
            version: _,
        } = event
        {
            if interface == "zwp_virtual_keyboard_manager_v1" {
                state.vk_mgr =
                    Some(proxy.bind::<ZwpVirtualKeyboardManagerV1, _, _>(name, 1, qhandle, ()));
            } else if interface == "zwp_input_method_manager_v2" {
                state.im_mgr =
                    Some(proxy.bind::<ZwpInputMethodManagerV2, _, _>(name, 1, qhandle, ()));
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
    ) {
    }
}

impl Dispatch<ZwpVirtualKeyboardManagerV1, ()> for State {
    fn event(
        _state: &mut Self,
        _proxy: &ZwpVirtualKeyboardManagerV1,
        _event: <ZwpVirtualKeyboardManagerV1 as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZwpVirtualKeyboardV1, ()> for State {
    fn event(
        _state: &mut Self,
        _proxy: &ZwpVirtualKeyboardV1,
        _event: <ZwpVirtualKeyboardV1 as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZwpInputMethodManagerV2, ()> for State {
    fn event(
        _state: &mut Self,
        _proxy: &ZwpInputMethodManagerV2,
        _event: <ZwpInputMethodManagerV2 as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZwpInputMethodV2, ()> for State {
    fn event(
        state: &mut Self,
        _proxy: &ZwpInputMethodV2,
        event: <ZwpInputMethodV2 as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        match event {
            zwp_input_method_v2::Event::Deactivate | zwp_input_method_v2::Event::Activate => {
                state.engine.reset_context();
            }
            _ => {}
        }
    }
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
            zwp_input_method_keyboard_grab_v2::Event::Keymap {
                format: _,
                fd,
                size: _,
            } => {
                let mut file = File::from(fd);
                let mut string = String::new();
                if file.read_to_string(&mut string).is_ok() {
                    let trim_len = string.trim_end_matches('\x00').len();
                    string.truncate(trim_len);
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
                    xkb_state.update_mask(mods_depressed, mods_latched, mods_locked, 0, 0, group);
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
                    let current_config = state.config.read().unwrap();
                    let new_im = current_config.get_input_method();
                    if new_im != state.engine.get_input_method() {
                        state.engine.set_input_method(new_im);
                    }
                    if current_config.spell_check != state.engine.spell_check {
                        state.engine.spell_check = current_config.spell_check;
                    }

                    let mut is_toggle = false;
                    if let Some(xkb_state) = state.xkb_state.as_ref() {
                        let mut active_mods: Vec<String> = Vec::new();
                        if xkb_state.mod_name_is_active(
                            &xkbcommon::xkb::MOD_NAME_CTRL,
                            xkbcommon::xkb::STATE_MODS_DEPRESSED,
                        ) {
                            active_mods.push("control".to_string());
                        }
                        if xkb_state.mod_name_is_active(
                            &xkbcommon::xkb::MOD_NAME_SHIFT,
                            xkbcommon::xkb::STATE_MODS_DEPRESSED,
                        ) {
                            active_mods.push("shift".to_string());
                        }
                        if xkb_state.mod_name_is_active(
                            &xkbcommon::xkb::MOD_NAME_ALT,
                            xkbcommon::xkb::STATE_MODS_DEPRESSED,
                        ) {
                            active_mods.push("alt".to_string());
                        }
                        if xkb_state.mod_name_is_active(
                            &xkbcommon::xkb::MOD_NAME_LOGO,
                            xkbcommon::xkb::STATE_MODS_DEPRESSED,
                        ) {
                            active_mods.push("super".to_string());
                        }

                        let keysym = xkb_state.key_get_one_sym(xkb_keycode.into());
                        let key_name = xkbcommon::xkb::keysym_get_name(keysym).to_lowercase();

                        let config_mod = current_config.get_toggle_modifier_normalized();
                        let config_key = current_config.get_toggle_key_normalized();

                        let mod_match = if config_mod.is_empty() {
                            true
                        } else {
                            active_mods
                                .iter()
                                .any(|m| config_mod.contains(m) || m.contains(&config_mod))
                        };

                        let key_match = key_name == config_key || key_name.contains(&config_key);

                        if mod_match && key_match {
                            is_toggle = true;
                        }
                    }

                    if is_toggle {
                        let is_enabled = state.is_vietnamese_enabled.load(Ordering::SeqCst);
                        if is_enabled
                            && let Some(Action::Commit(buffer)) = state
                                .engine
                                .set_input_method(state.engine.get_input_method())
                        {
                            let text = String::from_iter(buffer.as_slice());
                            if let Some(im) = state.im.as_ref() {
                                im.commit_string(text);
                                im.commit(0);
                            } else {
                                eprintln!("Warning: im is None during toggle commit");
                            }
                        }
                        state
                            .is_vietnamese_enabled
                            .store(!is_enabled, Ordering::SeqCst);
                        state.tray_handle.update(|_| {});
                        state.intercepted_keys.insert(key);
                        return;
                    }

                    let is_enabled = state.is_vietnamese_enabled.load(Ordering::SeqCst);
                    if !is_enabled {
                        if let Some(vk) = state.vk.as_ref() {
                            vk.key(time, key, key_state_u32);
                        } else {
                            eprintln!("Warning: vk is None during PassThrough");
                        }
                        return;
                    }
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
                                if let Some(im) = state.im.as_ref() {
                                    let byte_len = text.len() as i32;
                                    im.set_preedit_string(text, byte_len, byte_len);
                                    im.commit(0);
                                } else {
                                    eprintln!("Warning: im is None during Preedit");
                                }
                                state.intercepted_keys.insert(key);
                            }
                            Action::Commit(buffer) => {
                                let text = String::from_iter(buffer.as_slice());
                                if let Some(im) = state.im.as_ref() {
                                    im.commit_string(text);
                                    im.commit(0);
                                } else {
                                    eprintln!("Warning: im is None during Commit");
                                }
                                state.intercepted_keys.insert(key);
                            }
                            Action::CommitAndPassThrough(buffer) => {
                                let text = String::from_iter(buffer.as_slice());
                                if let Some(im) = state.im.as_ref() {
                                    im.commit_string(text);
                                    im.commit(0);
                                } else {
                                    eprintln!("Warning: im is None during CommitAndPassThrough");
                                }
                                if let Some(vk) = state.vk.as_ref() {
                                    vk.key(time, key, key_state_u32);
                                } else {
                                    eprintln!("Warning: vk is None during CommitAndPassThrough");
                                }
                            }
                            Action::PassThrough => {
                                if let Some(vk) = state.vk.as_ref() {
                                    vk.key(time, key, key_state_u32);
                                } else {
                                    eprintln!("Warning: vk is None during PassThrough");
                                }
                            }
                            Action::SurroundingRecompose {
                                preedit,
                                delete_count: _,
                                delete_byte_len,
                            } => {
                                if let Some(im) = state.im.as_ref() {
                                    if preedit.is_empty() {
                                        im.delete_surrounding_text(delete_byte_len as u32, 0);
                                        im.commit(0);
                                    } else {
                                        let text = String::from_iter(preedit.as_slice());
                                        let byte_len = text.len() as i32;
                                        im.delete_surrounding_text(delete_byte_len as u32, 0);
                                        im.set_preedit_string(text, byte_len, byte_len);
                                        im.commit(0);
                                    }
                                }
                                state.intercepted_keys.insert(key);
                            }
                        }
                    } else {
                        // Non-character key (Arrow, Home, End, etc.)
                        let is_modifier = state.xkb_state.as_ref().is_some_and(|xkb_state| {
                            let keysym = xkb_state.key_get_one_sym((key + 8).into()).raw();
                            #[allow(non_upper_case_globals)]
                            {
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
                            }
                        });

                        if is_modifier {
                            if let Some(vk) = state.vk.as_ref() {
                                vk.key(time, key, key_state_u32);
                            }
                        } else {
                            // Navigation/function keys: flush engine to finalize preedit and avoid ghost text
                            if let Some(Action::Commit(buffer)) = state.engine.flush() {
                                let text = String::from_iter(buffer.as_slice());
                                if let Some(im) = state.im.as_ref() {
                                    im.commit_string(text);
                                    im.commit(0);
                                }
                            }
                            // Pass through the navigation key
                            if let Some(vk) = state.vk.as_ref() {
                                vk.key(time, key, key_state_u32);
                            }
                        }
                    }
                } else {
                    // Released
                    if state.intercepted_keys.contains(&key) {
                        state.intercepted_keys.remove(&key);
                    } else {
                        if let Some(vk) = state.vk.as_ref() {
                            vk.key(time, key, key_state_u32);
                        } else {
                            eprintln!("Warning: vk is None during key release");
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

fn main() {
    let config = Config::load();
    let start_enabled = config.start_enabled;
    let initial_input_method = config.get_input_method();

    let config_lock = Arc::new(RwLock::new(config));

    // Setup notify watcher
    let watcher_config_lock = Arc::clone(&config_lock);
    let mut watcher =
        notify::recommended_watcher(move |res: notify::Result<notify::Event>| match res {
            Ok(event) => {
                if let EventKind::Modify(_) | EventKind::Create(_) = event.kind {
                    let new_config = Config::load();
                    if let Ok(mut lock) = watcher_config_lock.write() {
                        *lock = new_config;
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
            if let Err(e) = std::fs::create_dir_all(&config_dir) {
                eprintln!("Warning: Failed to create config directory at {:?}: {}", config_dir, e);
            }
        }
        if config_dir.exists() {
            if let Err(e) = watcher.watch(&config_dir, RecursiveMode::NonRecursive) {
                eprintln!("Warning: Failed to watch config directory at {:?}: {}", config_dir, e);
            }
        } else {
            eprintln!("Warning: Config directory does not exist at {:?}, skipping watcher setup.", config_dir);
        }
    }

    let is_vietnamese_enabled = Arc::new(AtomicBool::new(start_enabled));
    let tray_handle = vnikey_tray::spawn_tray(Arc::clone(&is_vietnamese_enabled));

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
        engine: Engine::new(initial_input_method, true),
        intercepted_keys: HashSet::new(),
        xkb_context,
        xkb_state: None,
        is_vietnamese_enabled,
        tray_handle,
        config: Arc::clone(&config_lock),
    };

    event_queue
        .roundtrip(&mut state)
        .expect("Failed to roundtrip event queue");

    if state.vk_mgr.is_none() || state.im_mgr.is_none() || state.seat.is_none() {
        panic!("Compositor does not support required IME protocols or seat!");
    }

    println!("Successfully connected to Wayland and bound IME protocols!");

    let seat = state.seat.as_ref().unwrap();

    let vk = state
        .vk_mgr
        .as_ref()
        .unwrap()
        .create_virtual_keyboard(seat, &qh, ());
    state.vk = Some(vk.clone());

    let im = state
        .im_mgr
        .as_ref()
        .unwrap()
        .get_input_method(seat, &qh, ());
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
        event_queue
            .blocking_dispatch(&mut state)
            .expect("Failed to dispatch Wayland events");
    }
}
