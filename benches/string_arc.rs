use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use std::hint::black_box;
use std::sync::Arc;
use std::time::Duration;

/// Benchmark to measure the performance benefit of using Arc for string operations
/// Tests complex string functions like split, replace, concat, fmt

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

/// Create a string with multiple parts for split
fn create_split_string() -> String {
    (0..100).map(|i| format!("part{}", i)).collect::<Vec<_>>().join(",")
}

// ==================== Current implementations ====================

fn split_current(args: &[ValueCurrent]) -> ValueCurrent {
    match &args[0] {
        ValueCurrent::String(s) => {
            let delim = match &args[1] {
                ValueCurrent::String(d) => d,
                _ => ",",
            };
            let parts: Vec<ValueCurrent> = s
                .split(delim)
                .map(|p| ValueCurrent::String(p.to_string()))
                .collect();
            ValueCurrent::List(parts)
        }
        _ => ValueCurrent::Null,
    }
}

fn replace_current(args: &[ValueCurrent]) -> ValueCurrent {
    match (&args[0], &args[1], &args[2]) {
        (ValueCurrent::String(s), ValueCurrent::String(from), ValueCurrent::String(to)) => {
            ValueCurrent::String(s.replace(from, to))
        }
        _ => ValueCurrent::Null,
    }
}

fn concat_current(args: &[ValueCurrent]) -> ValueCurrent {
    let mut result = String::new();
    for arg in args {
        match arg {
            ValueCurrent::String(s) => result.push_str(s),
            v => result.push_str(&format!("{:?}", v)),
        }
    }
    ValueCurrent::String(result)
}

fn upper_current(args: &[ValueCurrent]) -> ValueCurrent {
    match &args[0] {
        ValueCurrent::String(s) => ValueCurrent::String(s.to_uppercase()),
        _ => ValueCurrent::Null,
    }
}

fn take_str_current(args: &[ValueCurrent]) -> ValueCurrent {
    match &args[0] {
        ValueCurrent::String(s) => {
            let n = match &args[1] {
                ValueCurrent::Int(i) => (*i).max(0) as usize,
                _ => 10,
            };
            let end = n.min(s.len());
            ValueCurrent::String(s[..end].to_string())
        }
        _ => ValueCurrent::Null,
    }
}

fn substr_current(args: &[ValueCurrent]) -> ValueCurrent {
    match &args[0] {
        ValueCurrent::String(s) => {
            let start = match &args[1] {
                ValueCurrent::Int(i) => *i as usize,
                _ => 0,
            };
            let len = match &args[2] {
                ValueCurrent::Int(i) => *i as usize,
                _ => s.len(),
            };
            if start >= s.len() {
                return ValueCurrent::String(String::new());
            }
            let end = (start + len).min(s.len());
            ValueCurrent::String(s[start..end].to_string())
        }
        _ => ValueCurrent::Null,
    }
}

// ==================== Optimized implementations ====================

fn split_optimized(args: &[ValueOptimized]) -> ValueOptimized {
    match &args[0] {
        ValueOptimized::String(s) => {
            let delim = match &args[1] {
                ValueOptimized::String(d) => d.as_str(),
                _ => ",",
            };
            let parts: Vec<ValueOptimized> = s
                .split(delim)
                .map(|p| ValueOptimized::String(Arc::new(p.to_string())))
                .collect();
            ValueOptimized::List(Arc::new(parts))
        }
        _ => ValueOptimized::Null,
    }
}

fn replace_optimized(args: &[ValueOptimized]) -> ValueOptimized {
    match (&args[0], &args[1], &args[2]) {
        (ValueOptimized::String(s), ValueOptimized::String(from), ValueOptimized::String(to)) => {
            ValueOptimized::String(Arc::new(s.replace(from.as_str(), to.as_str())))
        }
        _ => ValueOptimized::Null,
    }
}

fn concat_optimized(args: &[ValueOptimized]) -> ValueOptimized {
    let mut result = String::new();
    for arg in args {
        match arg {
            ValueOptimized::String(s) => result.push_str(s),
            v => result.push_str(&format!("{:?}", v)),
        }
    }
    ValueOptimized::String(Arc::new(result))
}

fn upper_optimized(args: &[ValueOptimized]) -> ValueOptimized {
    match &args[0] {
        ValueOptimized::String(s) => ValueOptimized::String(Arc::new(s.to_uppercase())),
        _ => ValueOptimized::Null,
    }
}

fn take_str_optimized(args: &[ValueOptimized]) -> ValueOptimized {
    match &args[0] {
        ValueOptimized::String(s) => {
            let n = match &args[1] {
                ValueOptimized::Int(i) => (*i).max(0) as usize,
                _ => 10,
            };
            let end = n.min(s.len());
            ValueOptimized::String(Arc::new(s[..end].to_string()))
        }
        _ => ValueOptimized::Null,
    }
}

fn substr_optimized(args: &[ValueOptimized]) -> ValueOptimized {
    match &args[0] {
        ValueOptimized::String(s) => {
            let start = match &args[1] {
                ValueOptimized::Int(i) => *i as usize,
                _ => 0,
            };
            let len = match &args[2] {
                ValueOptimized::Int(i) => *i as usize,
                _ => s.len(),
            };
            if start >= s.len() {
                return ValueOptimized::String(Arc::new(String::new()));
            }
            let end = (start + len).min(s.len());
            ValueOptimized::String(Arc::new(s[start..end].to_string()))
        }
        _ => ValueOptimized::Null,
    }
}

// ==================== Benchmarks ====================

fn benchmark_string_clone(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_clone");
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

    group.finish();
}

fn benchmark_split_function(c: &mut Criterion) {
    let mut group = c.benchmark_group("split_function");
    group.throughput(Throughput::Elements(ITERATIONS as u64));
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(3));

    let split_str = create_split_string();
    let args_current = vec![
        ValueCurrent::String(split_str.clone()),
        ValueCurrent::String(",".to_string()),
    ];
    group.bench_function("current_split", |b| {
        b.iter(|| {
            for _ in 0..ITERATIONS / 10 {
                let args = args_current.clone();
                let result = split_current(&args);
                black_box(result);
            }
        })
    });

    let args_optimized = vec![
        ValueOptimized::String(Arc::new(split_str)),
        ValueOptimized::String(Arc::new(",".to_string())),
    ];
    group.bench_function("optimized_split", |b| {
        b.iter(|| {
            for _ in 0..ITERATIONS {
                let args = args_optimized.clone();
                let result = split_optimized(&args);
                black_box(result);
            }
        })
    });

    group.finish();
}

fn benchmark_replace_function(c: &mut Criterion) {
    let mut group = c.benchmark_group("replace_function");
    group.throughput(Throughput::Elements(ITERATIONS as u64));
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(3));

    let large_str = create_large_string();
    let args_current = vec![
        ValueCurrent::String(large_str.clone()),
        ValueCurrent::String("x".to_string()),
        ValueCurrent::String("y".to_string()),
    ];
    group.bench_function("current_replace", |b| {
        b.iter(|| {
            for _ in 0..ITERATIONS / 100 {
                let args = args_current.clone();
                let result = replace_current(&args);
                black_box(result);
            }
        })
    });

    let args_optimized = vec![
        ValueOptimized::String(Arc::new(large_str)),
        ValueOptimized::String(Arc::new("x".to_string())),
        ValueOptimized::String(Arc::new("y".to_string())),
    ];
    group.bench_function("optimized_replace", |b| {
        b.iter(|| {
            for _ in 0..ITERATIONS / 100 {
                let args = args_optimized.clone();
                let result = replace_optimized(&args);
                black_box(result);
            }
        })
    });

    group.finish();
}

fn benchmark_concat_function(c: &mut Criterion) {
    let mut group = c.benchmark_group("concat_function");
    group.throughput(Throughput::Elements(ITERATIONS as u64));
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(3));

    let str1 = "Hello ".to_string();
    let str2 = "World ".to_string();
    let str3 = "!".to_string();
    let args_current = vec![
        ValueCurrent::String(str1.clone()),
        ValueCurrent::String(str2.clone()),
        ValueCurrent::String(str3.clone()),
    ];
    group.bench_function("current_concat", |b| {
        b.iter(|| {
            for _ in 0..ITERATIONS {
                let args = args_current.clone();
                let result = concat_current(&args);
                black_box(result);
            }
        })
    });

    let args_optimized = vec![
        ValueOptimized::String(Arc::new(str1)),
        ValueOptimized::String(Arc::new(str2)),
        ValueOptimized::String(Arc::new(str3)),
    ];
    group.bench_function("optimized_concat", |b| {
        b.iter(|| {
            for _ in 0..ITERATIONS {
                let args = args_optimized.clone();
                let result = concat_optimized(&args);
                black_box(result);
            }
        })
    });

    group.finish();
}

fn benchmark_upper_function(c: &mut Criterion) {
    let mut group = c.benchmark_group("upper_function");
    group.throughput(Throughput::Elements(ITERATIONS as u64));
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(3));

    let large_str = create_large_string();
    let args_current = vec![ValueCurrent::String(large_str.clone())];
    group.bench_function("current_upper", |b| {
        b.iter(|| {
            for _ in 0..ITERATIONS / 100 {
                let args = args_current.clone();
                let result = upper_current(&args);
                black_box(result);
            }
        })
    });

    let args_optimized = vec![ValueOptimized::String(Arc::new(large_str))];
    group.bench_function("optimized_upper", |b| {
        b.iter(|| {
            for _ in 0..ITERATIONS / 100 {
                let args = args_optimized.clone();
                let result = upper_optimized(&args);
                black_box(result);
            }
        })
    });

    group.finish();
}

fn benchmark_take_str_function(c: &mut Criterion) {
    let mut group = c.benchmark_group("take_str_function");
    group.throughput(Throughput::Elements(ITERATIONS as u64));
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(3));

    let large_str = create_large_string();
    let args_current = vec![
        ValueCurrent::String(large_str.clone()),
        ValueCurrent::Int(100),
    ];
    group.bench_function("current_take_str", |b| {
        b.iter(|| {
            for _ in 0..ITERATIONS {
                let args = args_current.clone();
                let result = take_str_current(&args);
                black_box(result);
            }
        })
    });

    let args_optimized = vec![
        ValueOptimized::String(Arc::new(large_str)),
        ValueOptimized::Int(100),
    ];
    group.bench_function("optimized_take_str", |b| {
        b.iter(|| {
            for _ in 0..ITERATIONS {
                let args = args_optimized.clone();
                let result = take_str_optimized(&args);
                black_box(result);
            }
        })
    });

    group.finish();
}

fn benchmark_substr_function(c: &mut Criterion) {
    let mut group = c.benchmark_group("substr_function");
    group.throughput(Throughput::Elements(ITERATIONS as u64));
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(3));

    let large_str = create_large_string();
    let args_current = vec![
        ValueCurrent::String(large_str.clone()),
        ValueCurrent::Int(100),
        ValueCurrent::Int(200),
    ];
    group.bench_function("current_substr", |b| {
        b.iter(|| {
            for _ in 0..ITERATIONS {
                let args = args_current.clone();
                let result = substr_current(&args);
                black_box(result);
            }
        })
    });

    let args_optimized = vec![
        ValueOptimized::String(Arc::new(large_str)),
        ValueOptimized::Int(100),
        ValueOptimized::Int(200),
    ];
    group.bench_function("optimized_substr", |b| {
        b.iter(|| {
            for _ in 0..ITERATIONS {
                let args = args_optimized.clone();
                let result = substr_optimized(&args);
                black_box(result);
            }
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_string_clone,
    benchmark_split_function,
    benchmark_replace_function,
    benchmark_concat_function,
    benchmark_upper_function,
    benchmark_take_str_function,
    benchmark_substr_function
);
criterion_main!(benches);
