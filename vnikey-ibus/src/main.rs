struct IBusEngine {
    // Sẽ chứa vnikey Engine ở step 2
    // Hiện tại để trống
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
    async fn focus_out(&self) {
        eprintln!("[vnikey-ibus] FocusOut");
    }

    // Reset engine state
    async fn reset(&self) {
        eprintln!("[vnikey-ibus] Reset");
    }

    // App thông báo capability (preedit, surrounding text, etc.)
    async fn set_capabilities(&self, caps: u32) {
        eprintln!("[vnikey-ibus] SetCapabilities: {:#010x}", caps);
    }

    // KEY EVENT HANDLER — QUAN TRỌNG NHẤT
    // IBus gọi method này cho mỗi phím bấm
    // keyval: X11 keysym (ví dụ: 0x0061 = 'a')
    // keycode: hardware keycode
    // state: modifier bitmask (shift, ctrl, etc.)
    // Trả về: true = engine đã xử lý (nuốt phím), false = pass-through
    async fn process_key_event(&self, keyval: u32, keycode: u32, state: u32) -> bool {
        eprintln!(
            "[vnikey-ibus] ProcessKeyEvent: keyval={:#06x} keycode={} state={:#010x}",
            keyval, keycode, state
        );
        // STUB: luôn pass-through, step 2 sẽ implement thật
        false
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
    let ibus_address = get_ibus_address();
    eprintln!("[vnikey-ibus] Connecting to IBus at: {}", ibus_address);

    let engine_obj_path = "/org/freedesktop/IBus/Engine/VNIKey";

    let conn = zbus::connection::Builder::address(ibus_address.as_str())
        .expect("Invalid IBus address")
        .serve_at(engine_obj_path, IBusEngine {})
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

    std::future::pending::<()>().await;
}
