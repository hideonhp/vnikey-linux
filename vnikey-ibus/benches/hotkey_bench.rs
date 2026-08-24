use criterion::{Criterion, black_box, criterion_group, criterion_main};

// A mock of what happens in the hot path
fn is_toggle_hotkey(state: u32, key_name: &str, config_mod: &str, config_key: &str) -> bool {
    let has_ctrl = (state & (1 << 2)) != 0;

    let mod_match = if config_mod.is_empty() {
        true
    } else {
        has_ctrl && (config_mod.contains("control") || "control".contains(config_mod))
    };

    let key_match = key_name == config_key || key_name.contains(config_key);

    mod_match && key_match
}

fn bench_hotkey_check_current(c: &mut Criterion) {
    let config_modifier = "Control".to_string();
    let config_key = "Space".to_string();

    c.bench_function("hotkey check current (with to_lowercase)", |b| {
        b.iter(|| {
            let normalized_mod = black_box(&config_modifier).to_lowercase();
            let normalized_key = black_box(&config_key).to_lowercase();
            is_toggle_hotkey(4, "space", &normalized_mod, &normalized_key)
        })
    });
}

fn bench_hotkey_check_optimized(c: &mut Criterion) {
    // Simulated normalized config
    let config_modifier = "control".to_string();
    let config_key = "space".to_string();

    c.bench_function("hotkey check optimized (pre-normalized)", |b| {
        b.iter(|| {
            let normalized_mod = black_box(&config_modifier);
            let normalized_key = black_box(&config_key);
            is_toggle_hotkey(4, "space", normalized_mod, normalized_key)
        })
    });
}

criterion_group!(
    benches,
    bench_hotkey_check_current,
    bench_hotkey_check_optimized
);
criterion_main!(benches);
