/// Performance benchmarks for ferrous-opencc, mirroring the official `OpenCC` C++ benchmark
/// <https://github.com/BYVoid/OpenCC/blob/4d23eff614fcb9c2fe4460d9e610f93efb35ff11/src/benchmark/Performance.cpp> for cross-implementation comparison.
use std::{
    fmt::Write,
    path::PathBuf,
};

use criterion::{
    BenchmarkId,
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

fn read_zuozhuan() -> String {
    let path = benchmark_data_dir().join("zuozhuan.txt");
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e))
}

/// Generate the same repeated string used in the C++ `BM_Convert` benchmark.
fn generate_text(iterations: usize) -> String {
    let mut s = String::new();
    for i in 0..iterations {
        let _ = writeln!(s, "Open Chinese Convert 開放中文轉換{i}");
    }
    s
}

/// Initialization benchmark (mirrors `BM_Initialization`)
fn bench_initialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("initialization");

    macro_rules! bench_init {
        ($name:literal, $cfg:expr) => {
            group.bench_function($name, |b| {
                b.iter(|| OpenCC::from_config($cfg).expect("init failed"));
            });
        };
    }

    #[cfg(feature = "t2s-conversion")]
    bench_init!("hk2s", BuiltinConfig::Hk2s);
    #[cfg(feature = "t2s-conversion")]
    bench_init!("hk2t", BuiltinConfig::Hk2t);
    #[cfg(feature = "japanese-conversion")]
    bench_init!("jp2t", BuiltinConfig::Jp2t);
    #[cfg(feature = "s2t-conversion")]
    bench_init!("s2hk", BuiltinConfig::S2hk);
    #[cfg(feature = "s2t-conversion")]
    bench_init!("s2t", BuiltinConfig::S2t);
    #[cfg(feature = "s2t-conversion")]
    bench_init!("s2tw", BuiltinConfig::S2tw);
    #[cfg(feature = "s2t-conversion")]
    bench_init!("s2twp", BuiltinConfig::S2twp);
    #[cfg(feature = "s2t-conversion")]
    bench_init!("t2hk", BuiltinConfig::T2hk);
    #[cfg(feature = "japanese-conversion")]
    bench_init!("t2jp", BuiltinConfig::T2jp);
    #[cfg(feature = "t2s-conversion")]
    bench_init!("t2s", BuiltinConfig::T2s);
    #[cfg(feature = "t2s-conversion")]
    bench_init!("t2tw", BuiltinConfig::T2tw);
    #[cfg(feature = "t2s-conversion")]
    bench_init!("tw2s", BuiltinConfig::Tw2s);
    #[cfg(feature = "t2s-conversion")]
    bench_init!("tw2sp", BuiltinConfig::Tw2sp);
    #[cfg(feature = "s2t-conversion")]
    bench_init!("tw2t", BuiltinConfig::Tw2t);

    group.finish();
}

/// Long-text benchmark (mirrors `BM_ConvertLongText`)
fn bench_convert_long_text(c: &mut Criterion) {
    let text = read_zuozhuan();
    let bytes = text.len() as u64;
    let mut group = c.benchmark_group("convert_long_text");

    #[cfg(feature = "s2t-conversion")]
    {
        let converter = OpenCC::from_config(BuiltinConfig::S2t).expect("init failed");
        group.throughput(Throughput::Bytes(bytes));
        group.bench_function("s2t", |b| {
            b.iter(|| converter.convert(&text));
        });
    }

    #[cfg(feature = "s2t-conversion")]
    {
        let converter = OpenCC::from_config(BuiltinConfig::S2twp).expect("init failed");
        group.throughput(Throughput::Bytes(bytes));
        group.bench_function("s2twp", |b| {
            b.iter(|| converter.convert(&text));
        });
    }

    group.finish();
}

/// Convert benchmark (mirrors `BM_Convert`)
fn bench_convert(c: &mut Criterion) {
    const SIZES: &[usize] = &[100, 1_000, 10_000, 100_000];

    let mut group = c.benchmark_group("convert");

    #[cfg(feature = "s2t-conversion")]
    {
        let s2t = OpenCC::from_config(BuiltinConfig::S2t).expect("init failed");
        for &n in SIZES {
            let text = generate_text(n);
            group.throughput(Throughput::Bytes(text.len() as u64));
            group.bench_with_input(BenchmarkId::new("s2t", n), &text, |b, t| {
                b.iter(|| s2t.convert(t));
            });
        }
    }

    #[cfg(feature = "s2t-conversion")]
    {
        let s2twp = OpenCC::from_config(BuiltinConfig::S2twp).expect("init failed");
        for &n in SIZES {
            let text = generate_text(n);
            group.throughput(Throughput::Bytes(text.len() as u64));
            group.bench_with_input(BenchmarkId::new("s2twp", n), &text, |b, t| {
                b.iter(|| s2twp.convert(t));
            });
        }
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_initialization,
    bench_convert_long_text,
    bench_convert
);
criterion_main!(benches);
