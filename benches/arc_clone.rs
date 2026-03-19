use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use std::hint::black_box;
use std::sync::Arc;
use std::time::Duration;

/// Benchmark to measure the performance benefit of using Arc for large data cloning
/// This simulates the scenario in expression evaluation where values are cloned frequently

const ITERATIONS: usize = 100000;

// Simulate a large string (typical cell content in TSV)
fn create_large_string() -> String {
    "x".repeat(1000)
}

// Simulate a list of strings
fn create_string_list() -> Vec<String> {
    (0..100).map(|i| format!("field_{}_{}", i, "x".repeat(50))).collect()
}

fn benchmark_string_clone(c: &mut Criterion) {
    let large_string = create_large_string();
    let data_size = large_string.len() * ITERATIONS;

    let mut group = c.benchmark_group("string_clone");
    group.throughput(Throughput::Bytes(data_size as u64));
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(3));

    // Baseline: Clone String directly (deep copy)
    group.bench_function("string_direct_clone", |b| {
        b.iter(|| {
            for _ in 0..ITERATIONS {
                let cloned = large_string.clone();
                black_box(cloned);
            }
        })
    });

    // Optimized: Clone Arc<String> (just increments ref count)
    let arc_string = Arc::new(create_large_string());
    group.bench_function("arc_string_clone", |b| {
        b.iter(|| {
            for _ in 0..ITERATIONS {
                let cloned = arc_string.clone();
                black_box(cloned);
            }
        })
    });

    group.finish();
}

fn benchmark_list_clone(c: &mut Criterion) {
    let string_list = create_string_list();
    let data_size: usize = string_list.iter().map(|s| s.len()).sum::<usize>() * ITERATIONS;

    let mut group = c.benchmark_group("list_clone");
    group.throughput(Throughput::Bytes(data_size as u64));
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(3));

    // Baseline: Clone Vec<String> directly (deep copy of all strings)
    group.bench_function("vec_string_direct_clone", |b| {
        b.iter(|| {
            for _ in 0..ITERATIONS / 10 {
                // Fewer iterations because this is much slower
                let cloned = string_list.clone();
                black_box(cloned);
            }
        })
    });

    // Optimized: Clone Arc<Vec<String>> (just increments ref count)
    let arc_list = Arc::new(create_string_list());
    group.bench_function("arc_vec_string_clone", |b| {
        b.iter(|| {
            for _ in 0..ITERATIONS {
                let cloned = arc_list.clone();
                black_box(cloned);
            }
        })
    });

    group.finish();
}

fn benchmark_mixed_scenario(c: &mut Criterion) {
    // Simulate real expression evaluation scenario with multiple values
    let iterations = 10000;

    let mut group = c.benchmark_group("mixed_scenario");
    group.throughput(Throughput::Elements(iterations as u64));
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(3));

    // Scenario: Simulating expression evaluation context cloning
    // Current tva approach: clone values directly
    group.bench_function("direct_value_clone", |b| {
        let large_str = create_large_string();
        let list = create_string_list();

        b.iter(|| {
            for _ in 0..iterations {
                // Simulate cloning context with multiple values
                let _str_clone = large_str.clone();
                let _list_clone = list.clone();
                black_box(&_str_clone);
                black_box(&_list_clone);
            }
        })
    });

    // Optimized approach: using Arc
    group.bench_function("arc_value_clone", |b| {
        let large_str = Arc::new(create_large_string());
        let list = Arc::new(create_string_list());

        b.iter(|| {
            for _ in 0..iterations {
                // Simulate cloning context with Arc values
                let _str_clone = large_str.clone();
                let _list_clone = list.clone();
                black_box(&_str_clone);
                black_box(&_list_clone);
            }
        })
    });

    group.finish();
}

fn benchmark_memory_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_overhead");
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(3));

    // Measure the overhead of Arc wrapper itself
    group.bench_function("arc_overhead_small_string", |b| {
        let small = Arc::new("small".to_string());
        b.iter(|| {
            for _ in 0..ITERATIONS {
                let cloned = small.clone();
                black_box(cloned);
            }
        })
    });

    group.bench_function("direct_clone_small_string", |b| {
        let small = "small".to_string();
        b.iter(|| {
            for _ in 0..ITERATIONS {
                let cloned = small.clone();
                black_box(cloned);
            }
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_string_clone,
    benchmark_list_clone,
    benchmark_mixed_scenario,
    benchmark_memory_overhead
);
criterion_main!(benches);
