use std::{
    path::PathBuf,
    time::Duration,
};

use criterion::{
    Criterion,
    Throughput,
    criterion_group,
    criterion_main,
};
use ferrous_opencc::{
    OpenCC,
    config::BuiltinConfig,
};

fn benchmark_data_dir() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("benches");
    p.push("data");
    p
}

fn read_data3185k() -> String {
    let path = benchmark_data_dir().join("data3185k.txt");
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e))
}

fn bench_s2twp_data3185k(c: &mut Criterion) {
    let mut group = c.benchmark_group("s2twp_data3185k");

    #[cfg(feature = "s2t-conversion")]
    {
        let text = read_data3185k();
        let converter = OpenCC::from_config(BuiltinConfig::S2twp).expect("S2twp init failed");
        group.throughput(Throughput::Bytes(text.len() as u64));
        group.sample_size(10);
        group.warm_up_time(Duration::from_secs(1));
        group.measurement_time(Duration::from_secs(15));
        group.bench_function("s2twp", |b| {
            b.iter(|| converter.convert(&text));
        });
    }

    group.finish();
}

criterion_group!(benches, bench_s2twp_data3185k);
criterion_main!(benches);
