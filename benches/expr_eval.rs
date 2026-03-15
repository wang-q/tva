use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use std::hint::black_box;
use std::time::Duration;

/// Benchmark expression evaluation performance
/// Tests various aspects of the expression engine to identify bottlenecks

fn create_test_row() -> Vec<String> {
    vec![
        "100".to_string(),
        "3.14".to_string(),
        "hello world".to_string(),
        "true".to_string(),
        "42".to_string(),
    ]
}

fn create_test_headers() -> Vec<String> {
    vec![
        "id".to_string(),
        "value".to_string(),
        "name".to_string(),
        "active".to_string(),
        "count".to_string(),
    ]
}

fn benchmark_expression_eval(c: &mut Criterion) {
    let row = create_test_row();
    let headers = create_test_headers();
    let iterations = 10000;

    let mut group = c.benchmark_group("expression_eval");
    group.throughput(Throughput::Elements(iterations as u64));
    group.warm_up_time(Duration::from_secs(2));
    group.measurement_time(Duration::from_secs(5));

    // 1. Simple column access by index
    // Baseline: just accessing a column
    group.bench_function("col_access_by_index", |b| {
        let expr = tva::libs::expr::parser::parse("@1").unwrap();
        b.iter(|| {
            for _ in 0..iterations {
                let mut ctx = tva::libs::expr::runtime::EvalContext::new(&row);
                let result = tva::libs::expr::runtime::eval(&expr, &mut ctx).unwrap();
                black_box(result);
            }
        })
    });

    // 2. Column access by name
    // Tests the overhead of name-to-index resolution
    group.bench_function("col_access_by_name", |b| {
        let expr = tva::libs::expr::parser::parse("@id").unwrap();
        b.iter(|| {
            for _ in 0..iterations {
                let mut ctx =
                    tva::libs::expr::runtime::EvalContext::with_headers(&row, &headers);
                let result = tva::libs::expr::runtime::eval(&expr, &mut ctx).unwrap();
                black_box(result);
            }
        })
    });

    // 3. Arithmetic expression
    // Tests basic arithmetic operations
    group.bench_function("arithmetic_simple", |b| {
        let expr = tva::libs::expr::parser::parse("@1 + @5 * 2").unwrap();
        b.iter(|| {
            for _ in 0..iterations {
                let mut ctx = tva::libs::expr::runtime::EvalContext::new(&row);
                let result = tva::libs::expr::runtime::eval(&expr, &mut ctx).unwrap();
                black_box(result);
            }
        })
    });

    // 4. Complex arithmetic
    // Tests multiple operations
    group.bench_function("arithmetic_complex", |b| {
        let expr = tva::libs::expr::parser::parse("(@1 + @5) * 3 - @5 / 2").unwrap();
        b.iter(|| {
            for _ in 0..iterations {
                let mut ctx = tva::libs::expr::runtime::EvalContext::new(&row);
                let result = tva::libs::expr::runtime::eval(&expr, &mut ctx).unwrap();
                black_box(result);
            }
        })
    });

    // 5. String concatenation
    // Tests string operations
    group.bench_function("string_concat", |b| {
        let expr = tva::libs::expr::parser::parse("@3 ++ ' suffix'").unwrap();
        b.iter(|| {
            for _ in 0..iterations {
                let mut ctx = tva::libs::expr::runtime::EvalContext::new(&row);
                let result = tva::libs::expr::runtime::eval(&expr, &mut ctx).unwrap();
                black_box(result);
            }
        })
    });

    // 6. Function call - trim
    // Tests function call overhead (creates FunctionRegistry each time)
    group.bench_function("func_call_trim", |b| {
        let expr = tva::libs::expr::parser::parse("trim(@3)").unwrap();
        b.iter(|| {
            for _ in 0..iterations {
                let mut ctx = tva::libs::expr::runtime::EvalContext::new(&row);
                let result = tva::libs::expr::runtime::eval(&expr, &mut ctx).unwrap();
                black_box(result);
            }
        })
    });

    // 7. Function call - len
    // Tests another common function
    group.bench_function("func_call_len", |b| {
        let expr = tva::libs::expr::parser::parse("len(@3)").unwrap();
        b.iter(|| {
            for _ in 0..iterations {
                let mut ctx = tva::libs::expr::runtime::EvalContext::new(&row);
                let result = tva::libs::expr::runtime::eval(&expr, &mut ctx).unwrap();
                black_box(result);
            }
        })
    });

    // 8. Comparison operation
    // Tests comparison operators
    group.bench_function("comparison", |b| {
        let expr = tva::libs::expr::parser::parse("@1 > @5").unwrap();
        b.iter(|| {
            for _ in 0..iterations {
                let mut ctx = tva::libs::expr::runtime::EvalContext::new(&row);
                let result = tva::libs::expr::runtime::eval(&expr, &mut ctx).unwrap();
                black_box(result);
            }
        })
    });

    // 9. Logical operation with short-circuit
    // Tests logical operators (should short-circuit)
    group.bench_function("logical_short_circuit", |b| {
        let expr = tva::libs::expr::parser::parse("@1 > 0 and @5 < 100").unwrap();
        b.iter(|| {
            for _ in 0..iterations {
                let mut ctx = tva::libs::expr::runtime::EvalContext::new(&row);
                let result = tva::libs::expr::runtime::eval(&expr, &mut ctx).unwrap();
                black_box(result);
            }
        })
    });

    // 10. Pipe expression
    // Tests pipe operator overhead
    group.bench_function("pipe_simple", |b| {
        let expr = tva::libs::expr::parser::parse("@3 | trim() | len()").unwrap();
        b.iter(|| {
            for _ in 0..iterations {
                let mut ctx = tva::libs::expr::runtime::EvalContext::new(&row);
                let result = tva::libs::expr::runtime::eval(&expr, &mut ctx).unwrap();
                black_box(result);
            }
        })
    });

    // 11. List literal
    // Tests list creation
    group.bench_function("list_literal", |b| {
        let expr = tva::libs::expr::parser::parse("[@1, @2, @3]").unwrap();
        b.iter(|| {
            for _ in 0..iterations {
                let mut ctx = tva::libs::expr::runtime::EvalContext::new(&row);
                let result = tva::libs::expr::runtime::eval(&expr, &mut ctx).unwrap();
                black_box(result);
            }
        })
    });

    // 12. Method call
    // Tests method call syntax (object.method(args))
    group.bench_function("method_call", |b| {
        let expr = tva::libs::expr::parser::parse("@3.trim()").unwrap();
        b.iter(|| {
            for _ in 0..iterations {
                let mut ctx = tva::libs::expr::runtime::EvalContext::new(&row);
                let result = tva::libs::expr::runtime::eval(&expr, &mut ctx).unwrap();
                black_box(result);
            }
        })
    });

    // 13. Variable binding
    // Tests 'as' binding
    group.bench_function("variable_bind", |b| {
        let expr =
            tva::libs::expr::parser::parse("@1 + @5 as @total; @total * 2").unwrap();
        b.iter(|| {
            for _ in 0..iterations {
                let mut ctx = tva::libs::expr::runtime::EvalContext::new(&row);
                let result = tva::libs::expr::runtime::eval(&expr, &mut ctx).unwrap();
                black_box(result);
            }
        })
    });

    // 14. Parse + Eval (combined)
    // Tests the full pipeline (worst case - parsing every time)
    group.bench_function("parse_and_eval", |b| {
        let expr_str = "@1 + @5 * 2";
        b.iter(|| {
            for _ in 0..iterations {
                let expr = tva::libs::expr::parser::parse(expr_str).unwrap();
                let mut ctx = tva::libs::expr::runtime::EvalContext::new(&row);
                let result = tva::libs::expr::runtime::eval(&expr, &mut ctx).unwrap();
                black_box(result);
            }
        })
    });

    // 15. Pre-parsed only (best case)
    // Tests evaluation only (parsing done outside loop)
    group.bench_function("eval_only", |b| {
        let expr = tva::libs::expr::parser::parse("@1 + @5 * 2").unwrap();
        b.iter(|| {
            for _ in 0..iterations {
                let mut ctx = tva::libs::expr::runtime::EvalContext::new(&row);
                let result = tva::libs::expr::runtime::eval(&expr, &mut ctx).unwrap();
                black_box(result);
            }
        })
    });

    // 16. Cached parse (new optimization)
    // Tests parse caching effectiveness
    group.bench_function("parse_cached", |b| {
        // Clear cache before test
        tva::libs::expr::clear_cache();
        let expr_str = "@1 + @5 * 2";
        // First call to populate cache
        let _ = tva::libs::expr::parse_cached(expr_str).unwrap();

        b.iter(|| {
            for _ in 0..iterations {
                let expr = tva::libs::expr::parse_cached(expr_str).unwrap();
                let mut ctx = tva::libs::expr::runtime::EvalContext::new(&row);
                let result = tva::libs::expr::runtime::eval(&expr, &mut ctx).unwrap();
                black_box(result);
            }
        })
    });

    // 17. Constant folding (new optimization)
    // Tests constant expression pre-computation
    group.bench_function("constant_folded", |b| {
        // Parse expression with constants
        let mut expr = tva::libs::expr::parser::parse("2 + 3 * 4 - 5").unwrap();
        // Fold constants
        tva::libs::expr::fold_constants(&mut expr);

        b.iter(|| {
            for _ in 0..iterations {
                let mut ctx = tva::libs::expr::runtime::EvalContext::new(&row);
                let result = tva::libs::expr::runtime::eval(&expr, &mut ctx).unwrap();
                black_box(result);
            }
        })
    });

    // 18. Constant folding with column access
    // Tests mixed constant and column expressions
    group.bench_function("constant_folded_mixed", |b| {
        // Parse expression: @1 + 100 * 2 (100 * 2 should fold to 200)
        let mut expr = tva::libs::expr::parser::parse("@1 + 100 * 2").unwrap();
        // Fold constants
        tva::libs::expr::fold_constants(&mut expr);

        b.iter(|| {
            for _ in 0..iterations {
                let mut ctx = tva::libs::expr::runtime::EvalContext::new(&row);
                let result = tva::libs::expr::runtime::eval(&expr, &mut ctx).unwrap();
                black_box(result);
            }
        })
    });

    group.finish();
}

fn benchmark_function_registry(c: &mut Criterion) {
    let mut group = c.benchmark_group("function_registry");
    group.warm_up_time(Duration::from_secs(2));
    group.measurement_time(Duration::from_secs(5));

    // 16. FunctionRegistry creation overhead
    // This is the current bottleneck - creating registry for each call
    group.bench_function("registry_create", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                let registry = tva::libs::expr::functions::FunctionRegistry::new();
                black_box(registry);
            }
        })
    });

    // 17. Function lookup by name
    // Tests HashMap lookup performance
    group.bench_function("registry_lookup", |b| {
        let registry = tva::libs::expr::functions::FunctionRegistry::new();
        b.iter(|| {
            for _ in 0..10000 {
                let func = registry.get("trim");
                black_box(func);
            }
        })
    });

    group.finish();
}

fn benchmark_column_resolution(c: &mut Criterion) {
    let row = create_test_row();
    let headers = create_test_headers();
    let iterations = 10000;

    let mut group = c.benchmark_group("column_resolution");
    group.throughput(Throughput::Elements(iterations as u64));
    group.warm_up_time(Duration::from_secs(2));
    group.measurement_time(Duration::from_secs(5));

    // 18. Column index access (fast path)
    // Using expression evaluation to test column access
    group.bench_function("by_index", |b| {
        let expr = tva::libs::expr::parser::parse("@1").unwrap();
        b.iter(|| {
            for _ in 0..iterations {
                let mut ctx = tva::libs::expr::runtime::EvalContext::new(&row);
                let result = tva::libs::expr::runtime::eval(&expr, &mut ctx).unwrap();
                black_box(result);
            }
        })
    });

    // 19. Column name access (slow path - linear search)
    group.bench_function("by_name_linear", |b| {
        let expr = tva::libs::expr::parser::parse("@id").unwrap();
        b.iter(|| {
            for _ in 0..iterations {
                let mut ctx =
                    tva::libs::expr::runtime::EvalContext::with_headers(&row, &headers);
                let result = tva::libs::expr::runtime::eval(&expr, &mut ctx).unwrap();
                black_box(result);
            }
        })
    });

    // 20. Column name with pre-resolution (optimization)
    // Parse once, resolve names to indices, then evaluate
    group.bench_function("by_name_resolved", |b| {
        // Parse and resolve once outside the loop
        let mut expr = tva::libs::expr::parser::parse("@id").unwrap();
        tva::libs::expr::resolve_columns(&mut expr, &headers);

        b.iter(|| {
            for _ in 0..iterations {
                // Evaluate with pre-resolved indices
                let mut ctx =
                    tva::libs::expr::runtime::EvalContext::with_headers(&row, &headers);
                let result = tva::libs::expr::runtime::eval(&expr, &mut ctx).unwrap();
                black_box(result);
            }
        })
    });

    group.finish();
}

fn benchmark_concrete_expr(c: &mut Criterion) {
    let row = create_test_row();
    let headers = create_test_headers();
    let iterations = 10000;

    let mut group = c.benchmark_group("concrete_expr");
    group.throughput(Throughput::Elements(iterations as u64));
    group.warm_up_time(Duration::from_secs(2));
    group.measurement_time(Duration::from_secs(5));

    // 21. ConcreteExpr: Column access by index
    // Baseline comparison with regular eval
    group.bench_function("col_access_concrete", |b| {
        // Compile once
        let expr = tva::libs::expr::parser::parse("@1").unwrap();
        let (concrete, var_count) = tva::libs::expr::compile(&expr, &headers).unwrap();

        b.iter(|| {
            for _ in 0..iterations {
                let result =
                    tva::libs::expr::eval_compiled(&concrete, &row, var_count).unwrap();
                black_box(result);
            }
        })
    });

    // 22. ConcreteExpr vs Regular: Column access comparison
    group.bench_function("col_access_regular", |b| {
        // Parse and resolve once
        let mut expr = tva::libs::expr::parser::parse("@1").unwrap();
        tva::libs::expr::resolve_columns(&mut expr, &headers);

        b.iter(|| {
            for _ in 0..iterations {
                let mut ctx =
                    tva::libs::expr::runtime::EvalContext::with_headers(&row, &headers);
                let result = tva::libs::expr::runtime::eval(&expr, &mut ctx).unwrap();
                black_box(result);
            }
        })
    });

    // 23. ConcreteExpr: Arithmetic expression
    group.bench_function("arithmetic_concrete", |b| {
        let expr = tva::libs::expr::parser::parse("@1 + @5 * 2").unwrap();
        let (concrete, var_count) = tva::libs::expr::compile(&expr, &headers).unwrap();

        b.iter(|| {
            for _ in 0..iterations {
                let result =
                    tva::libs::expr::eval_compiled(&concrete, &row, var_count).unwrap();
                black_box(result);
            }
        })
    });

    // 24. ConcreteExpr vs Regular: Arithmetic comparison
    group.bench_function("arithmetic_regular", |b| {
        let mut expr = tva::libs::expr::parser::parse("@1 + @5 * 2").unwrap();
        tva::libs::expr::resolve_columns(&mut expr, &headers);

        b.iter(|| {
            for _ in 0..iterations {
                let mut ctx =
                    tva::libs::expr::runtime::EvalContext::with_headers(&row, &headers);
                let result = tva::libs::expr::runtime::eval(&expr, &mut ctx).unwrap();
                black_box(result);
            }
        })
    });

    // 25. ConcreteExpr: Function call
    group.bench_function("func_call_concrete", |b| {
        let expr = tva::libs::expr::parser::parse("len(@3)").unwrap();
        let (concrete, var_count) = tva::libs::expr::compile(&expr, &headers).unwrap();

        b.iter(|| {
            for _ in 0..iterations {
                let result =
                    tva::libs::expr::eval_compiled(&concrete, &row, var_count).unwrap();
                black_box(result);
            }
        })
    });

    // 26. ConcreteExpr vs Regular: Function call comparison
    group.bench_function("func_call_regular", |b| {
        let mut expr = tva::libs::expr::parser::parse("len(@3)").unwrap();
        tva::libs::expr::resolve_columns(&mut expr, &headers);

        b.iter(|| {
            for _ in 0..iterations {
                let mut ctx =
                    tva::libs::expr::runtime::EvalContext::with_headers(&row, &headers);
                let result = tva::libs::expr::runtime::eval(&expr, &mut ctx).unwrap();
                black_box(result);
            }
        })
    });

    // 27. ConcreteExpr: Complex expression with multiple operations
    group.bench_function("complex_concrete", |b| {
        let expr = tva::libs::expr::parser::parse("(@1 + @5) * 10 + len(@3)").unwrap();
        let (concrete, var_count) = tva::libs::expr::compile(&expr, &headers).unwrap();

        b.iter(|| {
            for _ in 0..iterations {
                let result =
                    tva::libs::expr::eval_compiled(&concrete, &row, var_count).unwrap();
                black_box(result);
            }
        })
    });

    // 28. ConcreteExpr vs Regular: Complex expression comparison
    group.bench_function("complex_regular", |b| {
        let mut expr =
            tva::libs::expr::parser::parse("(@1 + @5) * 10 + len(@3)").unwrap();
        tva::libs::expr::resolve_columns(&mut expr, &headers);

        b.iter(|| {
            for _ in 0..iterations {
                let mut ctx =
                    tva::libs::expr::runtime::EvalContext::with_headers(&row, &headers);
                let result = tva::libs::expr::runtime::eval(&expr, &mut ctx).unwrap();
                black_box(result);
            }
        })
    });

    // 29. ConcreteExpr: Pipe operation
    group.bench_function("pipe_concrete", |b| {
        let expr = tva::libs::expr::parser::parse("@3 | upper() | len()").unwrap();
        let (concrete, var_count) = tva::libs::expr::compile(&expr, &headers).unwrap();

        b.iter(|| {
            for _ in 0..iterations {
                let result =
                    tva::libs::expr::eval_compiled(&concrete, &row, var_count).unwrap();
                black_box(result);
            }
        })
    });

    // 30. ConcreteExpr vs Regular: Pipe operation comparison
    group.bench_function("pipe_regular", |b| {
        let mut expr = tva::libs::expr::parser::parse("@3 | upper() | len()").unwrap();
        tva::libs::expr::resolve_columns(&mut expr, &headers);

        b.iter(|| {
            for _ in 0..iterations {
                let mut ctx =
                    tva::libs::expr::runtime::EvalContext::with_headers(&row, &headers);
                let result = tva::libs::expr::runtime::eval(&expr, &mut ctx).unwrap();
                black_box(result);
            }
        })
    });

    // 31. Full pipeline: Parse + Compile + Eval (ConcreteExpr)
    group.bench_function("full_pipeline_concrete", |b| {
        b.iter(|| {
            for _ in 0..100 {
                let expr = tva::libs::expr::parser::parse("@1 + @5 * 2").unwrap();
                let (concrete, var_count) =
                    tva::libs::expr::compile(&expr, &headers).unwrap();
                let result =
                    tva::libs::expr::eval_compiled(&concrete, &row, var_count).unwrap();
                black_box(result);
            }
        })
    });

    // 32. Full pipeline: Parse + Resolve + Eval (Regular)
    group.bench_function("full_pipeline_regular", |b| {
        b.iter(|| {
            for _ in 0..100 {
                let mut expr = tva::libs::expr::parser::parse("@1 + @5 * 2").unwrap();
                tva::libs::expr::resolve_columns(&mut expr, &headers);
                let mut ctx =
                    tva::libs::expr::runtime::EvalContext::with_headers(&row, &headers);
                let result = tva::libs::expr::runtime::eval(&expr, &mut ctx).unwrap();
                black_box(result);
            }
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_expression_eval,
    benchmark_function_registry,
    benchmark_column_resolution,
    benchmark_concrete_expr
);
criterion_main!(benches);
