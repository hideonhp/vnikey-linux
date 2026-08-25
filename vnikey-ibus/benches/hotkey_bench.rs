use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

// A mock of what happens in the hot path (current/baseline)
fn is_toggle_hotkey_current(
    state: u32,
    key_name: &str,
    config_mod: &str,
    config_key: &str,
) -> bool {
    let has_ctrl = (state & (1 << 2)) != 0;

    let mod_match = if config_mod.is_empty() {
        true
    } else {
        has_ctrl && (config_mod.contains("control") || "control".contains(config_mod))
    };

    let key_match = key_name == config_key || key_name.contains(config_key);

    mod_match && key_match
}

// A mock of the optimized hot path
fn is_toggle_hotkey_optimized(
    state: u32,
    key_name: &str,
    config_mod: &str,
    config_key: &str,
) -> bool {
    let has_ctrl = (state & (1 << 2)) != 0;

    let mod_match = if config_mod.is_empty() {
        true
    } else {
        has_ctrl && (config_mod.contains("control") || "control".contains(config_mod))
    };

    let key_match = key_name.eq_ignore_ascii_case(config_key)
        || (key_name.len() >= config_key.len()
            && key_name
                .as_bytes()
                .windows(config_key.len())
                .any(|window| window.eq_ignore_ascii_case(config_key.as_bytes())));

    mod_match && key_match
}

fn bench_hotkey_check_current(c: &mut Criterion) {
    let config_modifier = "control".to_string();
    let config_key = "space".to_string();
    let raw_key_name = "Space";

    c.bench_function("hotkey check current (with to_lowercase)", |b| {
        b.iter(|| {
            let normalized_key_name = black_box(&raw_key_name).to_lowercase();
            let normalized_mod = black_box(&config_modifier);
            let normalized_key = black_box(&config_key);
            is_toggle_hotkey_current(4, &normalized_key_name, normalized_mod, normalized_key)
        })
    });
}

fn bench_hotkey_check_optimized(c: &mut Criterion) {
    let config_modifier = "control".to_string();
    let config_key = "space".to_string();
    let raw_key_name = "Space";

    c.bench_function("hotkey check optimized (allocation-free)", |b| {
        b.iter(|| {
            let normalized_mod = black_box(&config_modifier);
            let normalized_key = black_box(&config_key);
            is_toggle_hotkey_optimized(4, black_box(raw_key_name), normalized_mod, normalized_key)
        })
    });
}

criterion_group!(
    benches,
    bench_hotkey_check_current,
    bench_hotkey_check_optimized
);
criterion_main!(benches);
