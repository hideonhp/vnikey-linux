use notify::{EventKind, RecursiveMode, Watcher};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use vnikey_config::Config;
use vnikey_core::engine::{Action, Engine};
use vnikey_core::window_state::WindowStateManager;

const _IBUS_CAP_PREEDIT_TEXT: u32 = 1 << 0;
const IBUS_CAP_SURROUNDING_TEXT: u32 = 1 << 3;

const IBUS_RELEASE_MASK: u32 = 1 << 30;
const IBUS_SHIFT_MASK: u32 = 1 << 0;
const IBUS_CONTROL_MASK: u32 = 1 << 2;
const IBUS_MOD1_MASK: u32 = 1 << 3;
const IBUS_SUPER_MASK: u32 = 1 << 26;

struct EngineState {
    engine: Engine,
    capabilities: u32,
}

struct IBusEngine {
    state: Arc<Mutex<EngineState>>,
    config_lock: Arc<RwLock<Config>>,
    is_vietnamese_enabled: Arc<AtomicBool>,
    window_state: Arc<RwLock<WindowStateManager<String>>>,
    tx_state: tokio::sync::mpsc::UnboundedSender<bool>,
}

struct StateIntegration {
    is_vietnamese_enabled: Arc<AtomicBool>,
}

#[zbus::interface(name = "org.vnikey.State")]
impl StateIntegration {
    #[zbus(name = "GetState")]
    async fn get_state(&self) -> bool {
        self.is_vietnamese_enabled.load(Ordering::SeqCst)
    }

    #[zbus(name = "ToggleState")]
    async fn toggle_state(&self) {
        let current = self.is_vietnamese_enabled.load(Ordering::SeqCst);
        let new_state = !current;
        self.is_vietnamese_enabled
            .store(new_state, Ordering::SeqCst);
    }

    #[zbus(signal, name = "StateChanged")]
    async fn state_changed(
        signal_context: &zbus::SignalContext<'_>,
        state: bool,
    ) -> zbus::Result<()>;
}

struct WaylandIntegration {
    window_state: Arc<RwLock<WindowStateManager<String>>>,
    is_vietnamese_enabled: Arc<AtomicBool>,
    tx_state: tokio::sync::mpsc::UnboundedSender<bool>,
}

#[zbus::interface(name = "org.vnikey.WaylandIntegration")]
impl WaylandIntegration {
    async fn notify_active_window(&self, app_id: String) {
        if let Ok(mut state_manager) = self.window_state.write() {
            state_manager.set_active_window(app_id);
            if let Some(saved_state) = state_manager.get_state_for_current_window() {
                self.is_vietnamese_enabled
                    .store(saved_state, Ordering::SeqCst);
                let _ = self.tx_state.send(saved_state);
            }
        }
    }
}

fn keyval_to_char(keyval: u32) -> Option<char> {
    if (0x0020..=0x007E).contains(&keyval) {
        return char::from_u32(keyval);
    }
    if keyval >= 0x01000000 {
        return char::from_u32(keyval - 0x01000000);
    }
    None
}

fn is_toggle_hotkey(state: u32, key_name: &str, config_mod: &str, config_key: &str) -> bool {
    let has_ctrl = (state & IBUS_CONTROL_MASK) != 0;
    let has_shift = (state & IBUS_SHIFT_MASK) != 0;
    let has_alt = (state & IBUS_MOD1_MASK) != 0;
    let has_super = (state & IBUS_SUPER_MASK) != 0;

    let mod_match = if config_mod.is_empty() {
        true
    } else {
        (has_ctrl && (config_mod.contains("control") || "control".contains(config_mod)))
            || (has_shift && (config_mod.contains("shift") || "shift".contains(config_mod)))
            || (has_alt && (config_mod.contains("alt") || "alt".contains(config_mod)))
            || (has_super && (config_mod.contains("super") || "super".contains(config_mod)))
    };

    let key_match = key_name.eq_ignore_ascii_case(config_key)
        || (key_name.len() >= config_key.len()
            && key_name
                .as_bytes()
                .windows(config_key.len())
                .any(|window| window.eq_ignore_ascii_case(config_key.as_bytes())));

    mod_match && key_match
}

fn is_nav_key(keyval: u32) -> bool {
    matches!(
        keyval,
        0xFF08 | 0xFF09 | 0xFF1B | 0xFF50..=0xFF58 | 0xFF63 | 0xFFFF
    )
}

fn make_ibus_text(text: &str) -> zbus::zvariant::Value<'static> {
    // Tạo array rỗng chứa các variant (chữ ký "v")
    let empty_array = zbus::zvariant::Array::new(
        zbus::zvariant::Signature::try_from("v").expect("Valid signature"),
    );

    let attr_list = zbus::zvariant::Value::from((
        "IBusAttrList",
        std::collections::HashMap::<String, zbus::zvariant::Value<'static>>::new(),
        empty_array,
    ));

    // IBusText
    zbus::zvariant::Value::from((
        "IBusText",
        std::collections::HashMap::<String, zbus::zvariant::Value<'static>>::new(),
        text.to_string(),
        zbus::zvariant::Value::Value(Box::new(attr_list)),
    ))
}

impl IBusEngine {
    fn with_state<R, F: FnOnce(&EngineState) -> R>(&self, f: F) -> R {
        let st = self.state.lock().unwrap();
        f(&st)
    }

    fn with_state_mut<R, F: FnOnce(&mut EngineState) -> R>(&self, f: F) -> R {
        let mut st = self.state.lock().unwrap();
        f(&mut st)
    }

    fn flush_engine_text(&self) -> Option<String> {
        self.with_state_mut(|st| {
            if let Some(Action::Commit(buf)) = st.engine.flush() {
                Some(buf.to_string())
            } else {
                None
            }
        })
    }
}

#[zbus::interface(name = "org.freedesktop.IBus.Engine")]
impl IBusEngine {
    // IBus daemon gọi khi engine được enable
    async fn enable(&self) {
        eprintln!("[vnikey-ibus] Enable");
        // Reset engine context khi enable để tránh stale state
        if let Ok(mut st) = self.state.lock() {
            st.engine.reset_context();
        }
    }

    // IBus daemon gọi khi engine bị disable
    async fn disable(&self, #[zbus(signal_context)] ctx: zbus::SignalContext<'_>) {
        eprintln!("[vnikey-ibus] Disable");
        // Flush bất kỳ preedit còn đang dở
        let text_to_commit = self.flush_engine_text();
        if let Some(text) = text_to_commit {
            let _ = Self::commit_text(&ctx, make_ibus_text(&text)).await;
        }
        let _ = Self::hide_preedit_text(&ctx).await;
    }

    // Focus vào text field
    async fn focus_in(&self) {
        eprintln!("[vnikey-ibus] FocusIn");
    }

    // Rời text field
    async fn focus_out(&self, #[zbus(signal_context)] ctx: zbus::SignalContext<'_>) {
        eprintln!("[vnikey-ibus] FocusOut");
        self.with_state_mut(|st| {
            st.engine.reset_context();
        });
        let _ = Self::hide_preedit_text(&ctx).await;
    }

    // Reset engine state
    async fn reset(&self, #[zbus(signal_context)] ctx: zbus::SignalContext<'_>) {
        eprintln!("[vnikey-ibus] Reset");
        let text_to_commit = self.flush_engine_text();

        if let Some(text) = text_to_commit {
            let _ = Self::commit_text(&ctx, make_ibus_text(&text)).await;
        }
        let _ = Self::hide_preedit_text(&ctx).await;
    }

    // App thông báo capability (preedit, surrounding text, etc.)
    async fn set_capabilities(&self, caps: u32) {
        eprintln!("[vnikey-ibus] SetCapabilities: {:#010x}", caps);
        if let Ok(mut st) = self.state.lock() {
            st.capabilities = caps;
        }
    }

    // KEY EVENT HANDLER — QUAN TRỌNG NHẤT
    // IBus gọi method này cho mỗi phím bấm
    // keyval: X11 keysym (ví dụ: 0x0061 = 'a')
    // keycode: hardware keycode
    // state: modifier bitmask (shift, ctrl, etc.)
    // Trả về: true = engine đã xử lý (nuốt phím), false = pass-through
    async fn process_key_event(
        &self,
        keyval: u32,
        _keycode: u32,
        state: u32,
        #[zbus(signal_context)] ctx: zbus::SignalContext<'_>,
    ) -> bool {
        if state & IBUS_RELEASE_MASK != 0 {
            return false;
        }

        let current_config = self
            .config_lock
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .clone();

        let key_name = xkbcommon::xkb::keysym_get_name(keyval.into());

        let config_mod = current_config.get_toggle_modifier_normalized();
        let config_key = current_config.get_toggle_key_normalized();

        let cycle_mod = current_config.get_cycle_method_modifier_normalized();
        let cycle_key = current_config.get_cycle_method_key_normalized();
        if !cycle_key.is_empty() && is_toggle_hotkey(state, &key_name, cycle_mod, cycle_key) {
            let mut config_to_save = vnikey_config::Config::load();
            let new_method = match config_to_save.get_input_method() {
                vnikey_core::engine::InputMethod::Telex => "vni",
                vnikey_core::engine::InputMethod::Vni => "viqr",
                vnikey_core::engine::InputMethod::Viqr => "telex",
            };
            config_to_save.input_method = new_method.to_string();
            if let Err(e) = config_to_save.save() {
                eprintln!("Failed to cycle input method: {}", e);
            }
            return true;
        }

        if is_toggle_hotkey(state, &key_name, config_mod, config_key) {
            let is_enabled = self.is_vietnamese_enabled.load(Ordering::SeqCst);
            if is_enabled {
                let text_to_commit = self.flush_engine_text();
                if let Some(text) = text_to_commit {
                    let _ = Self::commit_text(&ctx, make_ibus_text(&text)).await;
                    let _ = Self::hide_preedit_text(&ctx).await;
                }
            }

            let new_state = !is_enabled;
            self.is_vietnamese_enabled
                .store(new_state, Ordering::SeqCst);

            if current_config.per_window_state
                && let Ok(mut w_state) = self.window_state.write()
            {
                w_state.save_state_for_current_window(new_state);
            }
            let _ = self.tx_state.send(new_state);
            return true;
        }

        if keyval == 0xFF1B && current_config.vim_mode {
            let is_enabled = self.is_vietnamese_enabled.load(Ordering::SeqCst);
            if is_enabled {
                let text_to_commit = self.flush_engine_text();
                if let Some(text) = text_to_commit {
                    let _ = Self::commit_text(&ctx, make_ibus_text(&text)).await;
                    let _ = Self::hide_preedit_text(&ctx).await;
                }

                self.is_vietnamese_enabled.store(false, Ordering::SeqCst);

                if current_config.per_window_state
                    && let Ok(mut w_state) = self.window_state.write()
                {
                    w_state.save_state_for_current_window(false);
                }
                let _ = self.tx_state.send(false);
            }
            return false;
        }

        if !self.is_vietnamese_enabled.load(Ordering::SeqCst) {
            return false;
        }

        if state & (IBUS_CONTROL_MASK | IBUS_MOD1_MASK) != 0 {
            let text_to_commit = self.flush_engine_text();
            if let Some(text) = text_to_commit {
                let _ = Self::commit_text(&ctx, make_ibus_text(&text)).await;
                let _ = Self::hide_preedit_text(&ctx).await;
            }
            return false;
        }

        let is_nav = is_nav_key(keyval);
        let is_backspace = keyval == 0xFF08;

        if is_nav && !is_backspace {
            let text_to_commit = self.flush_engine_text();
            if let Some(text) = text_to_commit {
                let _ = Self::commit_text(&ctx, make_ibus_text(&text)).await;
                let _ = Self::hide_preedit_text(&ctx).await;
            }
            return false;
        }

        let ch = if is_backspace {
            Some('\x08')
        } else if keyval == 0xFF0D {
            Some('\n')
        } else {
            keyval_to_char(keyval)
        };

        if ch.is_none() {
            return false;
        }

        match ch {
            None => false,
            Some(c) => {
                let action = self.with_state_mut(|st| st.engine.process_key(c));

                match action {
                    Action::Preedit(buf) => {
                        let text = buf.to_string();
                        let char_count = text.chars().count() as u32;
                        let _ = Self::update_preedit_text(
                            &ctx,
                            make_ibus_text(&text),
                            char_count,
                            !text.is_empty(),
                        )
                        .await;
                        true
                    }
                    Action::Commit(buf) => {
                        let text = buf.to_string();
                        let _ = Self::commit_text(&ctx, make_ibus_text(&text)).await;
                        let _ = Self::hide_preedit_text(&ctx).await;
                        true
                    }
                    Action::CommitAndPassThrough(buf) => {
                        let text = buf.to_string();
                        let _ = Self::commit_text(&ctx, make_ibus_text(&text)).await;
                        let _ = Self::hide_preedit_text(&ctx).await;
                        false
                    }
                    Action::PassThrough => false,
                    Action::SurroundingRecompose {
                        preedit,
                        delete_count,
                        ..
                    } => {
                        let caps = self.with_state(|st| st.capabilities);

                        if caps & IBUS_CAP_SURROUNDING_TEXT != 0 {
                            let _ = Self::delete_surrounding_text(
                                &ctx,
                                -(delete_count as i32),
                                delete_count as u32,
                            )
                            .await;

                            let text = preedit.to_string();
                            if text.is_empty() {
                                let _ = Self::hide_preedit_text(&ctx).await;
                            } else {
                                let char_count = text.chars().count() as u32;
                                let _ = Self::update_preedit_text(
                                    &ctx,
                                    make_ibus_text(&text),
                                    char_count,
                                    true,
                                )
                                .await;
                            }
                            true
                        } else {
                            false
                        }
                    }
                }
            }
        }
    }

    // IBus gọi khi cần set surrounding text context
    async fn set_surrounding_text(
        &self,
        _text: zbus::zvariant::Value<'_>,
        cursor_pos: u32,
        anchor_pos: u32,
    ) {
        eprintln!(
            "[vnikey-ibus] SetSurroundingText cursor={} anchor={}",
            cursor_pos, anchor_pos
        );
    }

    // Signals mà engine emit NGƯỢC LẠI cho daemon
    // Khai báo ở đây để zbus biết, implementation ở step 2
    #[zbus(signal)]
    async fn commit_text(
        signal_ctx: &zbus::SignalContext<'_>,
        text: zbus::zvariant::Value<'_>,
    ) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn delete_surrounding_text(
        signal_ctx: &zbus::SignalContext<'_>,
        offset: i32,
        n_chars: u32,
    ) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn update_preedit_text(
        signal_ctx: &zbus::SignalContext<'_>,
        text: zbus::zvariant::Value<'_>,
        cursor_pos: u32,
        visible: bool,
    ) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn hide_preedit_text(signal_ctx: &zbus::SignalContext<'_>) -> zbus::Result<()>;
}

async fn get_ibus_address() -> String {
    // IBus daemon set env var $IBUS_ADDRESS khi spawn engine
    // Fallback: đọc file ~/.config/ibus/bus/<machine-id>-unix-<display>-<screen>
    if let Ok(addr) = std::env::var("IBUS_ADDRESS") {
        return addr;
    }

    // Try reading from file: ~/.config/ibus/bus/
    // File name format: <machine-id>-unix-0-0 (thường là thế)
    let home = std::env::var("HOME").unwrap_or_default();

    let machine_id = match tokio::fs::read_to_string("/var/lib/dbus/machine-id").await {
        Ok(s) => s,
        Err(_) => tokio::fs::read_to_string("/etc/machine-id")
            .await
            .unwrap_or_default(),
    }
    .trim()
    .to_string();

    let display = std::env::var("DISPLAY").unwrap_or_else(|_| ":0".to_string());
    let display_num = display
        .trim_start_matches(':')
        .split('.')
        .next()
        .unwrap_or("0");

    let fname = format!("{home}/.config/ibus/bus/{machine_id}-unix-{display_num}-0");

    // Parse file để tìm IBUS_ADDRESS=unix:...
    tokio::fs::read_to_string(&fname)
        .await
        .unwrap_or_default()
        .lines()
        .find(|l| l.starts_with("IBUS_ADDRESS="))
        .and_then(|l| l.strip_prefix("IBUS_ADDRESS="))
        .unwrap_or("")
        .to_string()
}

fn main() {
    if std::env::args().any(|a| a == "--version" || a == "-V") {
        println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        return;
    }

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed to build tokio runtime");
    rt.block_on(async_main());
}

async fn async_main() {
    let config = Config::load();
    let start_enabled = config.start_enabled;
    let initial_input_method = config.get_input_method();

    let config_lock = Arc::new(RwLock::new(config));

    let is_vietnamese_enabled = Arc::new(AtomicBool::new(start_enabled));
    let window_state = Arc::new(RwLock::new(WindowStateManager::new()));

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

    let initial_macros = {
        let guard = config_lock.read().unwrap();
        guard.macros.clone()
    };
    let mut engine = Engine::new(initial_input_method, true);
    engine.set_macros(initial_macros);

    let engine_state = Arc::new(Mutex::new(EngineState {
        engine,
        capabilities: 0,
    }));

    let watcher_config = Arc::clone(&config_lock);
    let watcher_engine = Arc::clone(&engine_state);
    let mut watcher =
        notify::recommended_watcher(move |res: notify::Result<notify::Event>| match res {
            Ok(event) => {
                if let EventKind::Modify(_) | EventKind::Create(_) = event.kind {
                    let new_config = Config::load();
                    let new_im = new_config.get_input_method();
                    let new_spell_check = new_config.spell_check;

                    let _new_im_val = match new_config.get_input_method() {
                        vnikey_core::engine::InputMethod::Vni => 1,
                        vnikey_core::engine::InputMethod::Viqr => 2,
                        _ => 0,
                    };

                    if let Ok(mut lock) = watcher_config.write() {
                        *lock = new_config;
                    }
                    if let Ok(mut st) = watcher_engine.lock() {
                        if st.engine.get_input_method() != new_im {
                            st.engine.set_input_method(new_im);
                        }
                        st.engine.spell_check = new_spell_check;

                        let current_macros = {
                            let guard = watcher_config.read().unwrap();
                            guard.macros.clone()
                        };
                        st.engine.set_macros(current_macros);

                        eprintln!("[vnikey-ibus] Config reloaded: {:?}", new_im);
                    }
                }
            }
            Err(e) => eprintln!("[vnikey-ibus] watch error: {:?}", e),
        })
        .expect("Failed to create config watcher");

    if let Some(proj_dirs) = directories::ProjectDirs::from("", "", "vnikey") {
        let config_dir = proj_dirs.config_dir().to_path_buf();
        if config_dir.exists()
            && let Err(e) = watcher.watch(&config_dir, RecursiveMode::NonRecursive)
        {
            eprintln!("[vnikey-ibus] Warning: failed to watch config dir: {}", e);
        }
    }

    let ibus_address = get_ibus_address().await;
    eprintln!("[vnikey-ibus] Connecting to IBus at: {}", ibus_address);

    let engine_obj_path = "/org/freedesktop/IBus/Engine/VNIKey";

    let conn = zbus::connection::Builder::address(ibus_address.as_str())
        .expect("Invalid IBus address")
        .serve_at(
            engine_obj_path,
            IBusEngine {
                state: Arc::clone(&engine_state),
                config_lock: Arc::clone(&config_lock),
                is_vietnamese_enabled: Arc::clone(&is_vietnamese_enabled),
                window_state: Arc::clone(&window_state),
                tx_state: tx.clone(),
            },
        )
        .expect("Failed to serve IBusEngine")
        .build()
        .await
        .expect("Failed to connect to IBus daemon");

    let ibus_proxy = zbus::Proxy::new(
        &conn,
        "org.freedesktop.IBus",  // destination
        "/org/freedesktop/IBus", // object path
        "org.freedesktop.IBus",  // interface
    )
    .await
    .expect("Failed to create IBus proxy");

    ibus_proxy
        .call_method("CreateEngine", &("VNIKey",))
        .await
        .expect("Failed to call CreateEngine on IBus daemon");

    eprintln!("[vnikey-ibus] Engine registered with IBus daemon!");

    let state_integration = StateIntegration {
        is_vietnamese_enabled: Arc::clone(&is_vietnamese_enabled),
    };

    let _session_conn = zbus::connection::Builder::session()
        .expect("Failed to connect to D-Bus session bus")
        .name("org.vnikey.State")
        .expect("Failed to request D-Bus name")
        .serve_at("/org/vnikey/State", state_integration)
        .expect("Failed to serve D-Bus object")
        .build()
        .await
        .expect("Failed to build D-Bus connection");

    let iface_ref = _session_conn
        .object_server()
        .interface::<_, StateIntegration>("/org/vnikey/State")
        .await
        .unwrap();

    let wayland_integration = WaylandIntegration {
        window_state: Arc::clone(&window_state),
        is_vietnamese_enabled: Arc::clone(&is_vietnamese_enabled),
        tx_state: tx.clone(),
    };

    let _wayland_conn = zbus::connection::Builder::session()
        .expect("Failed to connect to D-Bus session bus")
        .name("org.vnikey.WaylandIntegration")
        .expect("Failed to request D-Bus name")
        .serve_at("/org/vnikey/WaylandIntegration", wayland_integration)
        .expect("Failed to serve D-Bus object")
        .build()
        .await
        .expect("Failed to build D-Bus connection");

    tokio::spawn(async move {
        while let Some(new_state) = rx.recv().await {
            let _ = StateIntegration::state_changed(iface_ref.signal_context(), new_state).await;
        }
    });

    let _watcher = watcher;
    std::future::pending::<()>().await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyval_to_char() {
        assert_eq!(keyval_to_char(0x0061), Some('a'));
        assert_eq!(keyval_to_char(0x0020), Some(' '));
        assert_eq!(keyval_to_char(0x010001B0), Some('ư'));
        assert_eq!(keyval_to_char(0x010001A1), Some('ơ'));
        assert_eq!(keyval_to_char(0xFF08), None);
        assert_eq!(keyval_to_char(0xFF1B), None);
    }

    #[test]
    fn test_is_toggle_hotkey() {
        assert!(is_toggle_hotkey(
            IBUS_CONTROL_MASK,
            "space",
            "control",
            "space"
        ));
        assert!(is_toggle_hotkey(
            IBUS_CONTROL_MASK,
            "Space",
            "control",
            "space"
        ));
        assert!(!is_toggle_hotkey(
            IBUS_SHIFT_MASK,
            "space",
            "control",
            "space"
        ));
        assert!(is_toggle_hotkey(IBUS_MOD1_MASK, "z", "alt", "z"));
        assert!(is_toggle_hotkey(IBUS_MOD1_MASK, "Z", "alt", "z"));
        assert!(is_toggle_hotkey(0, "shift_l", "", "shift_l"));
        assert!(is_toggle_hotkey(0, "Shift_L", "", "shift_l"));
    }

    #[test]
    fn test_is_nav_key() {
        assert!(is_nav_key(0xFF51));
        assert!(is_nav_key(0xFF52));
        assert!(is_nav_key(0xFF53));
        assert!(is_nav_key(0xFF54));
        assert!(is_nav_key(0xFF50));
        assert!(is_nav_key(0xFF57));
        assert!(is_nav_key(0xFFFF));
        assert!(is_nav_key(0xFF08));
        assert!(is_nav_key(0xFF09));
        assert!(is_nav_key(0xFF1B));

        assert!(!is_nav_key(0x0061));
        assert!(!is_nav_key(0x0020));
        assert!(!is_nav_key(0x010001B0));
    }

    struct MockIBusHandler {
        pub engine: Engine,
        pub is_vietnamese_enabled: bool,
        pub preedits: Vec<String>,
        pub commits: Vec<String>,
        pub vim_mode: bool,
    }

    impl MockIBusHandler {
        fn new() -> Self {
            Self {
                engine: Engine::new(vnikey_core::engine::InputMethod::Telex, true),
                is_vietnamese_enabled: false,
                preedits: vec![],
                commits: vec![],
                vim_mode: false,
            }
        }

        fn process_key_event(
            &mut self,
            keyval: u32,
            state: u32,
            config_mod: &str,
            config_key: &str,
        ) -> bool {
            if state & IBUS_RELEASE_MASK != 0 {
                return false;
            }

            let key_name = xkbcommon::xkb::keysym_get_name(keyval.into());

            if is_toggle_hotkey(state, &key_name, config_mod, config_key) {
                if self.is_vietnamese_enabled {
                    if let Some(Action::Commit(buf)) = self.engine.flush() {
                        self.commits.push(buf.to_string());
                        self.preedits.push("".to_string());
                    }
                }
                self.is_vietnamese_enabled = !self.is_vietnamese_enabled;
                return true;
            }

            if keyval == 0xFF1B && self.vim_mode {
                if self.is_vietnamese_enabled {
                    if let Some(Action::Commit(buf)) = self.engine.flush() {
                        self.commits.push(buf.to_string());
                        self.preedits.push("".to_string());
                    }
                    self.is_vietnamese_enabled = false;
                }
                return false;
            }

            if !self.is_vietnamese_enabled {
                return false;
            }

            if state & (IBUS_CONTROL_MASK | IBUS_MOD1_MASK) != 0 {
                if let Some(Action::Commit(buf)) = self.engine.flush() {
                    self.commits.push(buf.to_string());
                    self.preedits.push("".to_string());
                }
                return false;
            }

            let is_nav = is_nav_key(keyval);
            let is_backspace = keyval == 0xFF08;

            if is_nav && !is_backspace {
                if let Some(Action::Commit(buf)) = self.engine.flush() {
                    self.commits.push(buf.to_string());
                    self.preedits.push("".to_string());
                }
                return false;
            }

            let ch = if is_backspace {
                Some('\x08')
            } else if keyval == 0xFF0D {
                Some('\n')
            } else {
                keyval_to_char(keyval)
            };

            if let Some(c) = ch {
                let action = self.engine.process_key(c);
                match action {
                    Action::Preedit(buf) => {
                        self.preedits.push(buf.to_string());
                        true
                    }
                    Action::Commit(buf) => {
                        self.commits.push(buf.to_string());
                        self.preedits.push("".to_string());
                        true
                    }
                    Action::CommitAndPassThrough(buf) => {
                        self.commits.push(buf.to_string());
                        self.preedits.push("".to_string());
                        false
                    }
                    Action::PassThrough => false,
                    Action::SurroundingRecompose { preedit, .. } => {
                        self.preedits.push(preedit.to_string());
                        true
                    }
                }
            } else {
                false
            }
        }
    }

    #[test]
    fn test_flow_go_tieng_viet_co_ban() {
        let mut handler = MockIBusHandler::new();
        handler.is_vietnamese_enabled = true;

        let keyvals = vec![0x0076, 0x0069, 0x0065, 0x0065, 0x0074, 0x006A, 0x0020];
        for k in keyvals {
            handler.process_key_event(k, 0, "none", "unmatched");
        }

        assert_eq!(handler.engine.state, vnikey_core::engine::State::Idle);
        let commit_text = handler.commits.join("");
        assert_eq!(commit_text, "việt ");
    }

    #[test]
    fn test_flow_toggle_qua_phim_tat() {
        let mut handler = MockIBusHandler::new();

        assert!(!handler.is_vietnamese_enabled);

        let handled = handler.process_key_event(0x0020, IBUS_CONTROL_MASK, "control", "space");
        assert!(handled);
        assert!(handler.is_vietnamese_enabled);

        let handled2 = handler.process_key_event(0x0020, IBUS_CONTROL_MASK, "control", "space");
        assert!(handled2);
        assert!(!handler.is_vietnamese_enabled);
    }

    #[test]
    fn test_flow_vim_mode() {
        let mut handler = MockIBusHandler::new();
        handler.is_vietnamese_enabled = true;
        handler.vim_mode = true;

        handler.process_key_event(0x0076, 0, "none", "unmatched");
        handler.process_key_event(0x0069, 0, "none", "unmatched");
        handler.process_key_event(0xFF1B, 0, "none", "unmatched");

        assert!(!handler.is_vietnamese_enabled);

        let commit_text = handler.commits.join("");
        assert_eq!(commit_text, "vi");

        let last_preedit = handler.preedits.last().map(|s| s.as_str()).unwrap_or("");
        assert_eq!(last_preedit, "");
    }
}
