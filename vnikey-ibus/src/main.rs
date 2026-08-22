use notify::{EventKind, RecursiveMode, Watcher};
use std::sync::{Arc, Mutex, RwLock};
use vnikey_config::Config;
use vnikey_core::engine::{Action, Engine};


const IBUS_CAP_PREEDIT_TEXT: u32 = 1 << 0;
const IBUS_CAP_SURROUNDING_TEXT: u32 = 1 << 3;

struct EngineState {
    engine: Engine,
    capabilities: u32,
}

struct IBusEngine {
    state: Arc<Mutex<EngineState>>,
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

fn make_ibus_text(text: &str) -> zbus::zvariant::Value<'static> {
    let attr = zbus::zvariant::Value::from((
        "IBusAttribute",
        std::collections::HashMap::<String, zbus::zvariant::Value<'static>>::new(),
        1u32, // TYPE_UNDERLINE
        1u32, // UNDERLINE_SINGLE
        0u32, // start_index
        text.len() as u32, // end_index (byte count)
    ));

    let attr_array = zbus::zvariant::Array::try_from(vec![attr]).unwrap();

    let attr_list = zbus::zvariant::Value::from((
        "IBusAttrList",
        std::collections::HashMap::<String, zbus::zvariant::Value<'static>>::new(),
        attr_array,
    ));

    // IBusText
    zbus::zvariant::Value::from((
        "IBusText",
        std::collections::HashMap::<String, zbus::zvariant::Value<'static>>::new(),
        text.to_string(),
        zbus::zvariant::Value::Value(Box::new(attr_list)),
    ))
}

#[zbus::interface(name = "org.freedesktop.IBus.Engine")]
impl IBusEngine {
    // IBus daemon gọi khi engine được enable
    async fn enable(&self) {
        eprintln!("[vnikey-ibus] Enable");
    }

    // IBus daemon gọi khi engine bị disable
    async fn disable(&self) {
        eprintln!("[vnikey-ibus] Disable");
    }

    // Focus vào text field
    async fn focus_in(&self) {
        eprintln!("[vnikey-ibus] FocusIn");
    }

    // Rời text field
    async fn focus_out(&self, #[zbus(signal_context)] ctx: zbus::SignalContext<'_>) {
        eprintln!("[vnikey-ibus] FocusOut");
        {
            let mut st = self.state.lock().unwrap();
            st.engine.reset_context();
        }
        let _ = Self::hide_preedit_text(&ctx).await;
    }

    // Reset engine state
    async fn reset(&self, #[zbus(signal_context)] ctx: zbus::SignalContext<'_>) {
        eprintln!("[vnikey-ibus] Reset");
        let text_to_commit = {
            let mut st = self.state.lock().unwrap();
            if let Some(Action::Commit(buf)) = st.engine.flush() {
                Some(buf.to_string())
            } else {
                None
            }
        };

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
        const IBUS_RELEASE_MASK: u32 = 1 << 30;
        const IBUS_CONTROL_MASK: u32 = 1 << 2;
        const IBUS_MOD1_MASK: u32 = 1 << 3;

        if state & IBUS_RELEASE_MASK != 0 {
            return false;
        }

        if state & (IBUS_CONTROL_MASK | IBUS_MOD1_MASK) != 0 {
            let text_to_commit = {
                let mut st = self.state.lock().unwrap();
                if let Some(Action::Commit(buf)) = st.engine.flush() {
                    Some(buf.to_string())
                } else {
                    None
                }
            };
            if let Some(text) = text_to_commit {
                let _ = Self::commit_text(&ctx, make_ibus_text(&text)).await;
                let _ = Self::hide_preedit_text(&ctx).await;
            }
            return false;
        }

        let is_nav = matches!(
            keyval,
            0xFF08 | 0xFF09 | 0xFF0D | 0xFF1B | 0xFF50..=0xFF58 | 0xFF63 | 0xFFFF
        );
        let is_backspace = keyval == 0xFF08;

        if is_nav && !is_backspace {
            let text_to_commit = {
                let mut st = self.state.lock().unwrap();
                if let Some(Action::Commit(buf)) = st.engine.flush() {
                    Some(buf.to_string())
                } else {
                    None
                }
            };
            if let Some(text) = text_to_commit {
                let _ = Self::commit_text(&ctx, make_ibus_text(&text)).await;
                let _ = Self::hide_preedit_text(&ctx).await;
            }
            return false;
        }

        let ch = if is_backspace {
            Some('')
        } else {
            keyval_to_char(keyval)
        };

        if ch.is_none() {
            return false;
        }

        match ch {
            None => false,
            Some(c) => {
                let action = {
                    let mut st = self.state.lock().unwrap();
                    st.engine.process_key(c)
                };

                match action {
                    Action::Preedit(buf) => {
                        let text = buf.to_string();
                        let byte_len = text.len() as u32;
                        let _ = Self::update_preedit_text(
                            &ctx,
                            make_ibus_text(&text),
                            byte_len,
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
                    Action::SurroundingRecompose { preedit, delete_count, .. } => {
                        let caps = {
                            let st = self.state.lock().unwrap();
                            st.capabilities
                        };

                        if caps & IBUS_CAP_SURROUNDING_TEXT != 0 {
                            let _ = Self::delete_surrounding_text(
                                &ctx,
                                -(delete_count as i32),
                                delete_count as u32,
                            ).await;

                            let text = preedit.to_string();
                            let byte_len = text.len() as u32;
                            let _ = Self::update_preedit_text(
                                &ctx,
                                make_ibus_text(&text),
                                byte_len,
                                !text.is_empty(),
                            )
                            .await;
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

fn get_ibus_address() -> String {
    // IBus daemon set env var $IBUS_ADDRESS khi spawn engine
    // Fallback: đọc file ~/.config/ibus/bus/<machine-id>-unix-<display>-<screen>
    std::env::var("IBUS_ADDRESS").unwrap_or_else(|_| {
        // Try reading from file: ~/.config/ibus/bus/
        // File name format: <machine-id>-unix-0-0 (thường là thế)
        let home = std::env::var("HOME").unwrap_or_default();
        let machine_id = std::fs::read_to_string("/var/lib/dbus/machine-id")
            .or_else(|_| std::fs::read_to_string("/etc/machine-id"))
            .unwrap_or_default()
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
        std::fs::read_to_string(&fname)
            .unwrap_or_default()
            .lines()
            .find(|l| l.starts_with("IBUS_ADDRESS="))
            .and_then(|l| l.strip_prefix("IBUS_ADDRESS="))
            .unwrap_or("")
            .to_string()
    })
}

fn main() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed to build tokio runtime");
    rt.block_on(async_main());
}

async fn async_main() {
    let config_lock = Arc::new(RwLock::new(Config::load()));
    let initial_input_method = {
        let cfg = config_lock.read().unwrap();
        cfg.get_input_method()
    };

    let engine_state = Arc::new(Mutex::new(EngineState {
        engine: Engine::new(initial_input_method, true),
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
                    if let Ok(mut lock) = watcher_config.write() {
                        *lock = new_config;
                    }
                    if let Ok(mut st) = watcher_engine.lock() {
                        if st.engine.get_input_method() != new_im {
                            st.engine.set_input_method(new_im);
                        }
                        st.engine.spell_check = new_spell_check;
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

    let ibus_address = get_ibus_address();
    eprintln!("[vnikey-ibus] Connecting to IBus at: {}", ibus_address);

    let engine_obj_path = "/org/freedesktop/IBus/Engine/VNIKey";

    let conn = zbus::connection::Builder::address(ibus_address.as_str())
        .expect("Invalid IBus address")
        .serve_at(
            engine_obj_path,
            IBusEngine {
                state: Arc::clone(&engine_state),
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

    let _watcher = watcher;
    std::future::pending::<()>().await;
}
