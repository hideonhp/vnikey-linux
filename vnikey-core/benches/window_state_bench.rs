use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use vnikey_core::window_state::WindowStateManager;

fn bench_save_state(c: &mut Criterion) {
    let mut manager = WindowStateManager::new();
    let window_id = "test_app_id_wayland_org_gnome_terminal".to_string();
    manager.set_active_window(window_id.clone());

    c.bench_function("save_state_for_current_window", |b| {
        b.iter(|| {
            manager.save_state_for_current_window(black_box(true));
        })
    });
}

criterion_group!(benches, bench_save_state);
criterion_main!(benches);
