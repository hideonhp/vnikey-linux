use vnikey_core::engine::{Engine, InputMethod};

fn main() {
    let mut engine = Engine::new(InputMethod::Vni);
    for c in "hoang".chars() {
        println!("Process '{}' -> {:?}", c, engine.process_key(c));
    }
}
