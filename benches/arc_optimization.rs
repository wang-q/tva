use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use std::hint::black_box;
use std::sync::Arc;
use std::time::Duration;

const ITERATIONS: usize = 10000;
const LARGE_ITERATIONS: usize = 100000;

// ============================================================================
// Value Type Definitions
// ============================================================================

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

// ============================================================================
// Test Data Generators
// ============================================================================

fn create_large_string() -> String {
    "x".repeat(1000)
}

fn create_split_string() -> String {
    (0..100)
        .map(|i| format!("part{}", i))
        .collect::<Vec<_>>()
        .join(",")
}

fn create_large_list() -> Vec<ValueCurrent> {
    (0..100)
        .map(|i| ValueCurrent::String(format!("field_{}_{}", i, "x".repeat(50))))
        .collect()
}

fn create_large_list_optimized() -> Vec<ValueOptimized> {
    (0..100)
        .map(|i| {
            ValueOptimized::String(Arc::new(format!("field_{}_{}", i, "x".repeat(50))))
        })
        .collect()
}

fn create_numeric_list() -> Vec<ValueCurrent> {
    (0..1000).map(ValueCurrent::Int).collect()
}

fn create_numeric_list_optimized() -> Vec<ValueOptimized> {
    (0..1000).map(ValueOptimized::Int).collect()
}

fn create_string_list() -> Vec<String> {
    (0..100)
        .map(|i| format!("field_{}_{}", i, "x".repeat(50)))
        .collect()
}

// ============================================================================
// String Operation Implementations (Current)
// ============================================================================

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
        (
            ValueCurrent::String(s),
            ValueCurrent::String(from),
            ValueCurrent::String(to),
        ) => ValueCurrent::String(s.replace(from, to)),
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

// ============================================================================
// String Operation Implementations (Optimized with Arc)
// ============================================================================

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
        (
            ValueOptimized::String(s),
            ValueOptimized::String(from),
            ValueOptimized::String(to),
        ) => ValueOptimized::String(Arc::new(s.replace(from.as_str(), to.as_str()))),
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

// ============================================================================
// List Operation Implementations (Current)
// ============================================================================

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

// ============================================================================
// List Operation Implementations (Optimized with Arc)
// ============================================================================

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

// ============================================================================
// Benchmark: Basic Clone Operations
// ============================================================================

fn benchmark_string_clone(c: &mut Criterion) {
    let large_string = create_large_string();
    let data_size = large_string.len() * LARGE_ITERATIONS;

    let mut group = c.benchmark_group("arc_basic/string_clone");
    group.throughput(Throughput::Bytes(data_size as u64));
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(3));

    // Baseline: Clone String directly (deep copy)
    group.bench_function("direct_clone", |b| {
        b.iter(|| {
            for _ in 0..LARGE_ITERATIONS {
                let cloned = large_string.clone();
                black_box(cloned);
            }
        })
    });

    // Optimized: Clone Arc<String> (just increments ref count)
    let arc_string = Arc::new(create_large_string());
    group.bench_function("arc_clone", |b| {
        b.iter(|| {
            for _ in 0..LARGE_ITERATIONS {
                let cloned = arc_string.clone();
                black_box(cloned);
            }
        })
    });

    group.finish();
}

fn benchmark_list_clone(c: &mut Criterion) {
    let string_list = create_string_list();
    let data_size: usize =
        string_list.iter().map(|s| s.len()).sum::<usize>() * LARGE_ITERATIONS;

    let mut group = c.benchmark_group("arc_basic/list_clone");
    group.throughput(Throughput::Bytes(data_size as u64));
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(3));

    // Baseline: Clone Vec<String> directly (deep copy of all strings)
    group.bench_function("vec_direct_clone", |b| {
        b.iter(|| {
            for _ in 0..LARGE_ITERATIONS / 10 {
                let cloned = string_list.clone();
                black_box(cloned);
            }
        })
    });

    // Optimized: Clone Arc<Vec<String>> (just increments ref count)
    let arc_list = Arc::new(create_string_list());
    group.bench_function("arc_vec_clone", |b| {
        b.iter(|| {
            for _ in 0..LARGE_ITERATIONS {
                let cloned = arc_list.clone();
                black_box(cloned);
            }
        })
    });

    group.finish();
}

fn benchmark_value_clone(c: &mut Criterion) {
    let mut group = c.benchmark_group("arc_basic/value_clone");
    group.throughput(Throughput::Elements(ITERATIONS as u64));
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(3));

    let large_str = create_large_string();
    let val_current = ValueCurrent::String(large_str.clone());
    group.bench_function("current_string", |b| {
        b.iter(|| {
            for _ in 0..ITERATIONS {
                let cloned = val_current.clone();
                black_box(cloned);
            }
        })
    });

    let val_optimized = ValueOptimized::String(Arc::new(large_str));
    group.bench_function("optimized_string", |b| {
        b.iter(|| {
            for _ in 0..ITERATIONS {
                let cloned = val_optimized.clone();
                black_box(cloned);
            }
        })
    });

    let list_current = create_large_list();
    let val_list_current = ValueCurrent::List(list_current);
    group.bench_function("current_list", |b| {
        b.iter(|| {
            for _ in 0..ITERATIONS / 100 {
                let cloned = val_list_current.clone();
                black_box(cloned);
            }
        })
    });

    let list_optimized = create_large_list_optimized();
    let val_list_optimized = ValueOptimized::List(Arc::new(list_optimized));
    group.bench_function("optimized_list", |b| {
        b.iter(|| {
            for _ in 0..ITERATIONS {
                let cloned = val_list_optimized.clone();
                black_box(cloned);
            }
        })
    });

    group.finish();
}

fn benchmark_mixed_scenario(c: &mut Criterion) {
    let iterations = 10000;

    let mut group = c.benchmark_group("arc_basic/mixed_scenario");
    group.throughput(Throughput::Elements(iterations as u64));
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(3));

    group.bench_function("direct_value_clone", |b| {
        let large_str = create_large_string();
        let list = create_string_list();

        b.iter(|| {
            for _ in 0..iterations {
                let _str_clone = large_str.clone();
                let _list_clone = list.clone();
                black_box(&_str_clone);
                black_box(&_list_clone);
            }
        })
    });

    group.bench_function("arc_value_clone", |b| {
        let large_str = Arc::new(create_large_string());
        let list = Arc::new(create_string_list());

        b.iter(|| {
            for _ in 0..iterations {
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
    let mut group = c.benchmark_group("arc_basic/memory_overhead");
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(3));

    group.bench_function("arc_overhead_small", |b| {
        let small = Arc::new("small".to_string());
        b.iter(|| {
            for _ in 0..LARGE_ITERATIONS {
                let cloned = small.clone();
                black_box(cloned);
            }
        })
    });

    group.bench_function("direct_clone_small", |b| {
        let small = "small".to_string();
        b.iter(|| {
            for _ in 0..LARGE_ITERATIONS {
                let cloned = small.clone();
                black_box(cloned);
            }
        })
    });

    group.finish();
}

// ============================================================================
// Benchmark: String Operations
// ============================================================================

fn benchmark_split_function(c: &mut Criterion) {
    let mut group = c.benchmark_group("arc_string/split");
    group.throughput(Throughput::Elements(ITERATIONS as u64));
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(3));

    let split_str = create_split_string();
    let args_current = vec![
        ValueCurrent::String(split_str.clone()),
        ValueCurrent::String(",".to_string()),
    ];
    group.bench_function("current", |b| {
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
    group.bench_function("optimized", |b| {
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
    let mut group = c.benchmark_group("arc_string/replace");
    group.throughput(Throughput::Elements(ITERATIONS as u64));
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(3));

    let large_str = create_large_string();
    let args_current = vec![
        ValueCurrent::String(large_str.clone()),
        ValueCurrent::String("x".to_string()),
        ValueCurrent::String("y".to_string()),
    ];
    group.bench_function("current", |b| {
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
    group.bench_function("optimized", |b| {
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
    let mut group = c.benchmark_group("arc_string/concat");
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
    group.bench_function("current", |b| {
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
    group.bench_function("optimized", |b| {
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
    let mut group = c.benchmark_group("arc_string/upper");
    group.throughput(Throughput::Elements(ITERATIONS as u64));
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(3));

    let large_str = create_large_string();
    let args_current = vec![ValueCurrent::String(large_str.clone())];
    group.bench_function("current", |b| {
        b.iter(|| {
            for _ in 0..ITERATIONS / 100 {
                let args = args_current.clone();
                let result = upper_current(&args);
                black_box(result);
            }
        })
    });

    let args_optimized = vec![ValueOptimized::String(Arc::new(large_str))];
    group.bench_function("optimized", |b| {
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
    let mut group = c.benchmark_group("arc_string/take_str");
    group.throughput(Throughput::Elements(ITERATIONS as u64));
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(3));

    let large_str = create_large_string();
    let args_current = vec![
        ValueCurrent::String(large_str.clone()),
        ValueCurrent::Int(100),
    ];
    group.bench_function("current", |b| {
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
    group.bench_function("optimized", |b| {
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
    let mut group = c.benchmark_group("arc_string/substr");
    group.throughput(Throughput::Elements(ITERATIONS as u64));
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(3));

    let large_str = create_large_string();
    let args_current = vec![
        ValueCurrent::String(large_str.clone()),
        ValueCurrent::Int(100),
        ValueCurrent::Int(200),
    ];
    group.bench_function("current", |b| {
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
    group.bench_function("optimized", |b| {
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

// ============================================================================
// Benchmark: List Operations
// ============================================================================

fn benchmark_take_function(c: &mut Criterion) {
    let mut group = c.benchmark_group("arc_list/take");
    group.throughput(Throughput::Elements(ITERATIONS as u64));
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(3));

    let list_current = create_large_list();
    let args_current = vec![ValueCurrent::List(list_current), ValueCurrent::Int(10)];
    group.bench_function("current", |b| {
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
    group.bench_function("optimized", |b| {
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
    let mut group = c.benchmark_group("arc_list/reverse");
    group.throughput(Throughput::Elements(ITERATIONS as u64));
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(3));

    let list_current = create_large_list();
    let args_current = vec![ValueCurrent::List(list_current)];
    group.bench_function("current", |b| {
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
    group.bench_function("optimized", |b| {
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
    let mut group = c.benchmark_group("arc_list/slice");
    group.throughput(Throughput::Elements(ITERATIONS as u64));
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(3));

    let list_current = create_large_list();
    let args_current = vec![
        ValueCurrent::List(list_current),
        ValueCurrent::Int(10),
        ValueCurrent::Int(50),
    ];
    group.bench_function("current", |b| {
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
    group.bench_function("optimized", |b| {
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
    let mut group = c.benchmark_group("arc_list/sort");
    group.throughput(Throughput::Elements(ITERATIONS as u64));
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(3));

    let list_current = create_numeric_list();
    let args_current = vec![ValueCurrent::List(list_current)];
    group.bench_function("current", |b| {
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
    group.bench_function("optimized", |b| {
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
    let mut group = c.benchmark_group("arc_list/unique");
    group.throughput(Throughput::Elements(ITERATIONS as u64));
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(3));

    let list_current = create_large_list();
    let args_current = vec![ValueCurrent::List(list_current)];
    group.bench_function("current", |b| {
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
    group.bench_function("optimized", |b| {
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
    let mut group = c.benchmark_group("arc_list/filter");
    group.throughput(Throughput::Elements(ITERATIONS as u64));
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(3));

    let list_current = create_numeric_list();
    let args_current = vec![ValueCurrent::List(list_current), ValueCurrent::Int(500)];
    group.bench_function("current", |b| {
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
    group.bench_function("optimized", |b| {
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
    let mut group = c.benchmark_group("arc_list/map");
    group.throughput(Throughput::Elements(ITERATIONS as u64));
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(3));

    let list_current = create_numeric_list();
    let args_current = vec![ValueCurrent::List(list_current), ValueCurrent::Int(2)];
    group.bench_function("current", |b| {
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
    group.bench_function("optimized", |b| {
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

// ============================================================================
// Criterion Groups
// ============================================================================

criterion_group!(
    arc_basic,
    benchmark_string_clone,
    benchmark_list_clone,
    benchmark_value_clone,
    benchmark_mixed_scenario,
    benchmark_memory_overhead
);

criterion_group!(
    arc_string,
    benchmark_split_function,
    benchmark_replace_function,
    benchmark_concat_function,
    benchmark_upper_function,
    benchmark_take_str_function,
    benchmark_substr_function
);

criterion_group!(
    arc_list,
    benchmark_take_function,
    benchmark_reverse_function,
    benchmark_slice_function,
    benchmark_sort_function,
    benchmark_unique_function,
    benchmark_filter_function,
    benchmark_map_function
);

criterion_main!(arc_basic, arc_string, arc_list);
