use vnikey_core::engine::Engine;
fn main() {
    let mut e = Engine::new();
    e.process_key('h');
    e.process_key('o');
    e.process_key('a');
    let act = e.process_key('s');
    println!("{:?}", act);
}
