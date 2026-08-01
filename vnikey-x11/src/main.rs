use x11rb::connect;

fn main() {
    let (_conn, _screen_num) = connect(None).expect("Panic: Cannot connect to X11 server.");
    println!("Successfully connected to X11 Server!");
}
