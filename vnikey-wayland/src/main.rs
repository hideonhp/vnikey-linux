use wayland_client::{Connection, Dispatch, QueueHandle, protocol::{wl_registry, wl_seat}};
use wayland_protocols_misc::zwp_virtual_keyboard_v1::client::{
    zwp_virtual_keyboard_manager_v1::ZwpVirtualKeyboardManagerV1,
    zwp_virtual_keyboard_v1::ZwpVirtualKeyboardV1,
};
use wayland_protocols_misc::zwp_input_method_v2::client::{
    zwp_input_method_manager_v2::ZwpInputMethodManagerV2,
    zwp_input_method_v2::ZwpInputMethodV2,
    zwp_input_method_keyboard_grab_v2::ZwpInputMethodKeyboardGrabV2,
};

struct State {
    vk_mgr: Option<ZwpVirtualKeyboardManagerV1>,
    im_mgr: Option<ZwpInputMethodManagerV2>,
    seat: Option<wl_seat::WlSeat>,
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
        _state: &mut Self,
        _proxy: &ZwpInputMethodKeyboardGrabV2,
        _event: <ZwpInputMethodKeyboardGrabV2 as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {}
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
    };

    event_queue.roundtrip(&mut state).expect("Failed to roundtrip event queue");

    if state.vk_mgr.is_none() || state.im_mgr.is_none() || state.seat.is_none() {
        panic!("Compositor does not support required IME protocols or seat!");
    }

    println!("Successfully connected to Wayland and bound IME protocols!");
}
