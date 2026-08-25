use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

// The original sync version
fn get_ibus_address_sync() -> String {
    std::env::var("IBUS_ADDRESS").unwrap_or_else(|_| {
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
        std::fs::read_to_string(&fname)
            .unwrap_or_default()
            .lines()
            .find(|l| l.starts_with("IBUS_ADDRESS="))
            .and_then(|l| l.strip_prefix("IBUS_ADDRESS="))
            .unwrap_or("")
            .to_string()
    })
}

// The new async version
async fn get_ibus_address_async() -> String {
    if let Ok(addr) = std::env::var("IBUS_ADDRESS") {
        return addr;
    }

    let home = std::env::var("HOME").unwrap_or_default();

    let machine_id = match tokio::fs::read_to_string("/var/lib/dbus/machine-id").await {
        Ok(s) => s,
        Err(_) => tokio::fs::read_to_string("/etc/machine-id")
            .await
            .unwrap_or_default(),
    }
    .trim()
    .to_string();

    let display = std::env::var("DISPLAY").unwrap_or_else(|_| ":0".to_string());
    let display_num = display
        .trim_start_matches(':')
        .split('.')
        .next()
        .unwrap_or("0");

    let fname = format!("{home}/.config/ibus/bus/{machine_id}-unix-{display_num}-0");

    tokio::fs::read_to_string(&fname)
        .await
        .unwrap_or_default()
        .lines()
        .find(|l| l.starts_with("IBUS_ADDRESS="))
        .and_then(|l| l.strip_prefix("IBUS_ADDRESS="))
        .unwrap_or("")
        .to_string()
}

fn bench_ibus_address(c: &mut Criterion) {
    let mut group = c.benchmark_group("get_ibus_address");

    // Force fallback execution by unsetting the env variable
    unsafe { std::env::remove_var("IBUS_ADDRESS") };

    group.bench_function("sync", |b| {
        b.iter(|| {
            black_box(get_ibus_address_sync());
        });
    });

    let rt = tokio::runtime::Runtime::new().unwrap();
    group.bench_function("async", |b| {
        b.to_async(&rt).iter(|| async {
            black_box(get_ibus_address_async().await);
        });
    });

    group.finish();
}

criterion_group!(benches, bench_ibus_address);
criterion_main!(benches);
