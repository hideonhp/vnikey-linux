fn main() {
    let space_keycode: u32 = 57; // evdev space keycode
    let keysym = xkbcommon::xkb::keysyms::KEY_space;
    println!("Evdev Space: {}", space_keycode);
    println!("Keysym Space: {:?}", keysym);
}
