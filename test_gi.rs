use vnikey_core::telex::find_tone_target_index;

fn main() {
    println!("{:?}", find_tone_target_index(&['g', 'i', 'e', 'n', 'g']));
    println!("{:?}", find_tone_target_index(&['g', 'i', 'ê', 'n', 'g']));
}
