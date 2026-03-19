use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use std::hint::black_box;
use std::sync::Arc;
use std::time::Duration;

/// Benchmark to measure the performance benefit of using Arc for TVA's Value type
/// Tests the actual take() function and Value cloning scenarios

const ITERATIONS: usize = 10000;

/// Current TVA Value type (without Arc)
#[derive(Debug, Clone, PartialEq)]
pub enum ValueCurrent {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    List(Vec<ValueCurrent>),
}

/// Optimized Value type with Arc for large data
#[derive(Debug, Clone, PartialEq)]
pub enum ValueOptimized {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(Arc<String>),
    List(Arc<Vec<ValueOptimized>>),
}

/// Create a large string value
fn create_large_string() -> String {
    "x".repeat(1000)
}

/// Create a list with many elements
fn create_large_list() -> Vec<ValueCurrent> {
    (0..100)
        .map(|i| ValueCurrent::String(format!("field_{}_{}", i, "x".repeat(50))))
        .collect()
}

/// Create a list with many elements (optimized version)
fn create_large_list_optimized() -> Vec<ValueOptimized> {
    (0..100)
        .map(|i| {
            ValueOptimized::String(Arc::new(format!("field_{}_{}", i, "x".repeat(50))))
        })
        .collect()
}

/// Create a numeric list
fn create_numeric_list() -> Vec<ValueCurrent> {
    (0..1000).map(ValueCurrent::Int).collect()
}

fn create_numeric_list_optimized() -> Vec<ValueOptimized> {
    (0..1000).map(ValueOptimized::Int).collect()
}

// ==================== Current implementations ====================

fn take_current(args: &[ValueCurrent]) -> ValueCurrent {
    match &args[0] {
        ValueCurrent::List(list) => {
            let n = match &args[1] {
                ValueCurrent::Int(i) => (*i).max(0) as usize,
                _ => 10,
            };
            let end = n.min(list.len());
            ValueCurrent::List(list[..end].to_vec())
        }
        _ => ValueCurrent::Null,
    }
}

fn reverse_current(args: &[ValueCurrent]) -> ValueCurrent {
    match &args[0] {
        ValueCurrent::List(list) => {
            let mut reversed = list.clone();
            reversed.reverse();
            ValueCurrent::List(reversed)
        }
        _ => ValueCurrent::Null,
    }
}

fn slice_current(args: &[ValueCurrent]) -> ValueCurrent {
    match &args[0] {
        ValueCurrent::List(list) => {
            let start = match &args[1] {
                ValueCurrent::Int(i) => *i as usize,
                _ => 0,
            };
            let end = match &args[2] {
                ValueCurrent::Int(i) => *i as usize,
                _ => list.len(),
            };
            let start = start.min(list.len());
            let end = end.min(list.len());
            ValueCurrent::List(list[start..end].to_vec())
        }
        _ => ValueCurrent::Null,
    }
}

fn sort_current(args: &[ValueCurrent]) -> ValueCurrent {
    match &args[0] {
        ValueCurrent::List(list) => {
            let mut sorted = list.clone();
            sorted.sort_by(|a, b| match (a, b) {
                (ValueCurrent::Int(a), ValueCurrent::Int(b)) => a.cmp(b),
                _ => std::cmp::Ordering::Equal,
            });
            ValueCurrent::List(sorted)
        }
        _ => ValueCurrent::Null,
    }
}

fn unique_current(args: &[ValueCurrent]) -> ValueCurrent {
    match &args[0] {
        ValueCurrent::List(list) => {
            let mut seen = std::collections::HashSet::new();
            let mut result = Vec::new();
            for item in list {
                let key = format!("{:?}", item);
                if seen.insert(key) {
                    result.push(item.clone());
                }
            }
            ValueCurrent::List(result)
        }
        _ => ValueCurrent::Null,
    }
}

fn filter_current(args: &[ValueCurrent]) -> ValueCurrent {
    match &args[0] {
        ValueCurrent::List(list) => {
            let threshold = match &args[1] {
                ValueCurrent::Int(i) => *i,
                _ => 50,
            };
            let mut result = Vec::new();
            for item in list.iter() {
                // Simulate filter condition: keep items > threshold
                if let ValueCurrent::Int(i) = item {
                    if *i > threshold {
                        result.push(item.clone());
                    }
                }
            }
            ValueCurrent::List(result)
        }
        _ => ValueCurrent::Null,
    }
}

fn map_current(args: &[ValueCurrent]) -> ValueCurrent {
    match &args[0] {
        ValueCurrent::List(list) => {
            let multiplier = match &args[1] {
                ValueCurrent::Int(i) => *i,
                _ => 2,
            };
            let mut result = Vec::with_capacity(list.len());
            for item in list.iter() {
                if let ValueCurrent::Int(i) = item {
                    result.push(ValueCurrent::Int(i * multiplier));
                }
            }
            ValueCurrent::List(result)
        }
        _ => ValueCurrent::Null,
    }
}

// ==================== Optimized implementations ====================

fn take_optimized(args: &[ValueOptimized]) -> ValueOptimized {
    match &args[0] {
        ValueOptimized::List(list) => {
            let n = match &args[1] {
                ValueOptimized::Int(i) => (*i).max(0) as usize,
                _ => 10,
            };
            let end = n.min(list.len());
            let sliced: Vec<ValueOptimized> = list[..end].iter().cloned().collect();
            ValueOptimized::List(Arc::new(sliced))
        }
        _ => ValueOptimized::Null,
    }
}

fn reverse_optimized(args: &[ValueOptimized]) -> ValueOptimized {
    match &args[0] {
        ValueOptimized::List(list) => {
            let mut reversed: Vec<ValueOptimized> = list.iter().cloned().collect();
            reversed.reverse();
            ValueOptimized::List(Arc::new(reversed))
        }
        _ => ValueOptimized::Null,
    }
}

fn slice_optimized(args: &[ValueOptimized]) -> ValueOptimized {
    match &args[0] {
        ValueOptimized::List(list) => {
            let start = match &args[1] {
                ValueOptimized::Int(i) => *i as usize,
                _ => 0,
            };
            let end = match &args[2] {
                ValueOptimized::Int(i) => *i as usize,
                _ => list.len(),
            };
            let start = start.min(list.len());
            let end = end.min(list.len());
            let sliced: Vec<ValueOptimized> = list[start..end].iter().cloned().collect();
            ValueOptimized::List(Arc::new(sliced))
        }
        _ => ValueOptimized::Null,
    }
}

fn sort_optimized(args: &[ValueOptimized]) -> ValueOptimized {
    match &args[0] {
        ValueOptimized::List(list) => {
            let mut sorted: Vec<ValueOptimized> = list.iter().cloned().collect();
            sorted.sort_by(|a, b| match (a, b) {
                (ValueOptimized::Int(a), ValueOptimized::Int(b)) => a.cmp(b),
                _ => std::cmp::Ordering::Equal,
            });
            ValueOptimized::List(Arc::new(sorted))
        }
        _ => ValueOptimized::Null,
    }
}

fn unique_optimized(args: &[ValueOptimized]) -> ValueOptimized {
    match &args[0] {
        ValueOptimized::List(list) => {
            let mut seen = std::collections::HashSet::new();
            let mut result = Vec::new();
            for item in list.iter() {
                let key = format!("{:?}", item);
                if seen.insert(key) {
                    result.push(item.clone());
                }
            }
            ValueOptimized::List(Arc::new(result))
        }
        _ => ValueOptimized::Null,
    }
}

fn filter_optimized(args: &[ValueOptimized]) -> ValueOptimized {
    match &args[0] {
        ValueOptimized::List(list) => {
            let threshold = match &args[1] {
                ValueOptimized::Int(i) => *i,
                _ => 50,
            };
            let mut result = Vec::new();
            for item in list.iter() {
                if let ValueOptimized::Int(i) = item {
                    if *i > threshold {
                        result.push(item.clone());
                    }
                }
            }
            ValueOptimized::List(Arc::new(result))
        }
        _ => ValueOptimized::Null,
    }
}

fn map_optimized(args: &[ValueOptimized]) -> ValueOptimized {
    match &args[0] {
        ValueOptimized::List(list) => {
            let multiplier = match &args[1] {
                ValueOptimized::Int(i) => *i,
                _ => 2,
            };
            let mut result = Vec::with_capacity(list.len());
            for item in list.iter() {
                if let ValueOptimized::Int(i) = item {
                    result.push(ValueOptimized::Int(i * multiplier));
                }
            }
            ValueOptimized::List(Arc::new(result))
        }
        _ => ValueOptimized::Null,
    }
}

// ==================== Benchmarks ====================

fn benchmark_value_clone(c: &mut Criterion) {
    let mut group = c.benchmark_group("value_clone");
    group.throughput(Throughput::Elements(ITERATIONS as u64));
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(3));

    let large_str = create_large_string();
    let val_current = ValueCurrent::String(large_str.clone());
    group.bench_function("current_string_clone", |b| {
        b.iter(|| {
            for _ in 0..ITERATIONS {
                let cloned = val_current.clone();
                black_box(cloned);
            }
        })
    });

    let val_optimized = ValueOptimized::String(Arc::new(large_str));
    group.bench_function("optimized_string_clone", |b| {
        b.iter(|| {
            for _ in 0..ITERATIONS {
                let cloned = val_optimized.clone();
                black_box(cloned);
            }
        })
    });

    let list_current = create_large_list();
    let val_list_current = ValueCurrent::List(list_current);
    group.bench_function("current_list_clone", |b| {
        b.iter(|| {
            for _ in 0..ITERATIONS / 100 {
                let cloned = val_list_current.clone();
                black_box(cloned);
            }
        })
    });

    let list_optimized = create_large_list_optimized();
    let val_list_optimized = ValueOptimized::List(Arc::new(list_optimized));
    group.bench_function("optimized_list_clone", |b| {
        b.iter(|| {
            for _ in 0..ITERATIONS {
                let cloned = val_list_optimized.clone();
                black_box(cloned);
            }
        })
    });

    group.finish();
}

fn benchmark_take_function(c: &mut Criterion) {
    let mut group = c.benchmark_group("take_function");
    group.throughput(Throughput::Elements(ITERATIONS as u64));
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(3));

    let list_current = create_large_list();
    let args_current = vec![ValueCurrent::List(list_current), ValueCurrent::Int(10)];
    group.bench_function("current_take", |b| {
        b.iter(|| {
            for _ in 0..ITERATIONS {
                let args = args_current.clone();
                let result = take_current(&args);
                black_box(result);
            }
        })
    });

    let list_optimized = create_large_list_optimized();
    let args_optimized = vec![
        ValueOptimized::List(Arc::new(list_optimized)),
        ValueOptimized::Int(10),
    ];
    group.bench_function("optimized_take", |b| {
        b.iter(|| {
            for _ in 0..ITERATIONS {
                let args = args_optimized.clone();
                let result = take_optimized(&args);
                black_box(result);
            }
        })
    });

    group.finish();
}

fn benchmark_reverse_function(c: &mut Criterion) {
    let mut group = c.benchmark_group("reverse_function");
    group.throughput(Throughput::Elements(ITERATIONS as u64));
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(3));

    let list_current = create_large_list();
    let args_current = vec![ValueCurrent::List(list_current)];
    group.bench_function("current_reverse", |b| {
        b.iter(|| {
            for _ in 0..ITERATIONS / 100 {
                let args = args_current.clone();
                let result = reverse_current(&args);
                black_box(result);
            }
        })
    });

    let list_optimized = create_large_list_optimized();
    let args_optimized = vec![ValueOptimized::List(Arc::new(list_optimized))];
    group.bench_function("optimized_reverse", |b| {
        b.iter(|| {
            for _ in 0..ITERATIONS {
                let args = args_optimized.clone();
                let result = reverse_optimized(&args);
                black_box(result);
            }
        })
    });

    group.finish();
}

fn benchmark_slice_function(c: &mut Criterion) {
    let mut group = c.benchmark_group("slice_function");
    group.throughput(Throughput::Elements(ITERATIONS as u64));
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(3));

    let list_current = create_large_list();
    let args_current = vec![
        ValueCurrent::List(list_current),
        ValueCurrent::Int(10),
        ValueCurrent::Int(50),
    ];
    group.bench_function("current_slice", |b| {
        b.iter(|| {
            for _ in 0..ITERATIONS {
                let args = args_current.clone();
                let result = slice_current(&args);
                black_box(result);
            }
        })
    });

    let list_optimized = create_large_list_optimized();
    let args_optimized = vec![
        ValueOptimized::List(Arc::new(list_optimized)),
        ValueOptimized::Int(10),
        ValueOptimized::Int(50),
    ];
    group.bench_function("optimized_slice", |b| {
        b.iter(|| {
            for _ in 0..ITERATIONS {
                let args = args_optimized.clone();
                let result = slice_optimized(&args);
                black_box(result);
            }
        })
    });

    group.finish();
}

fn benchmark_sort_function(c: &mut Criterion) {
    let mut group = c.benchmark_group("sort_function");
    group.throughput(Throughput::Elements(ITERATIONS as u64));
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(3));

    let list_current = create_numeric_list();
    let args_current = vec![ValueCurrent::List(list_current)];
    group.bench_function("current_sort", |b| {
        b.iter(|| {
            for _ in 0..ITERATIONS / 100 {
                let args = args_current.clone();
                let result = sort_current(&args);
                black_box(result);
            }
        })
    });

    let list_optimized = create_numeric_list_optimized();
    let args_optimized = vec![ValueOptimized::List(Arc::new(list_optimized))];
    group.bench_function("optimized_sort", |b| {
        b.iter(|| {
            for _ in 0..ITERATIONS {
                let args = args_optimized.clone();
                let result = sort_optimized(&args);
                black_box(result);
            }
        })
    });

    group.finish();
}

fn benchmark_unique_function(c: &mut Criterion) {
    let mut group = c.benchmark_group("unique_function");
    group.throughput(Throughput::Elements(ITERATIONS as u64));
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(3));

    let list_current = create_large_list();
    let args_current = vec![ValueCurrent::List(list_current)];
    group.bench_function("current_unique", |b| {
        b.iter(|| {
            for _ in 0..ITERATIONS / 100 {
                let args = args_current.clone();
                let result = unique_current(&args);
                black_box(result);
            }
        })
    });

    let list_optimized = create_large_list_optimized();
    let args_optimized = vec![ValueOptimized::List(Arc::new(list_optimized))];
    group.bench_function("optimized_unique", |b| {
        b.iter(|| {
            for _ in 0..ITERATIONS {
                let args = args_optimized.clone();
                let result = unique_optimized(&args);
                black_box(result);
            }
        })
    });

    group.finish();
}

fn benchmark_filter_function(c: &mut Criterion) {
    let mut group = c.benchmark_group("filter_function");
    group.throughput(Throughput::Elements(ITERATIONS as u64));
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(3));

    let list_current = create_numeric_list();
    let args_current = vec![ValueCurrent::List(list_current), ValueCurrent::Int(500)];
    group.bench_function("current_filter", |b| {
        b.iter(|| {
            for _ in 0..ITERATIONS / 100 {
                let args = args_current.clone();
                let result = filter_current(&args);
                black_box(result);
            }
        })
    });

    let list_optimized = create_numeric_list_optimized();
    let args_optimized = vec![
        ValueOptimized::List(Arc::new(list_optimized)),
        ValueOptimized::Int(500),
    ];
    group.bench_function("optimized_filter", |b| {
        b.iter(|| {
            for _ in 0..ITERATIONS {
                let args = args_optimized.clone();
                let result = filter_optimized(&args);
                black_box(result);
            }
        })
    });

    group.finish();
}

fn benchmark_map_function(c: &mut Criterion) {
    let mut group = c.benchmark_group("map_function");
    group.throughput(Throughput::Elements(ITERATIONS as u64));
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(3));

    let list_current = create_numeric_list();
    let args_current = vec![ValueCurrent::List(list_current), ValueCurrent::Int(2)];
    group.bench_function("current_map", |b| {
        b.iter(|| {
            for _ in 0..ITERATIONS / 100 {
                let args = args_current.clone();
                let result = map_current(&args);
                black_box(result);
            }
        })
    });

    let list_optimized = create_numeric_list_optimized();
    let args_optimized = vec![
        ValueOptimized::List(Arc::new(list_optimized)),
        ValueOptimized::Int(2),
    ];
    group.bench_function("optimized_map", |b| {
        b.iter(|| {
            for _ in 0..ITERATIONS {
                let args = args_optimized.clone();
                let result = map_optimized(&args);
                black_box(result);
            }
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_value_clone,
    benchmark_take_function,
    benchmark_reverse_function,
    benchmark_slice_function,
    benchmark_sort_function,
    benchmark_unique_function,
    benchmark_filter_function,
    benchmark_map_function
);
criterion_main!(benches);
