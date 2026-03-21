//! TsvReader comprehensive benchmark
//!
//! This benchmark tests various aspects of TsvReader:
//! 1. SIMD implementation comparison (SSE2/NEON vs memchr2)
//! 2. Reader method comparison (for_each_line variants)
//! 3. Allocation overhead comparison

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::io::Cursor;
use tva::libs::tsv::reader::TsvReader;
use tva::libs::tsv::record::Row;

fn generate_tsv_data(num_rows: usize, num_cols: usize) -> Vec<u8> {
    let mut data = Vec::with_capacity(num_rows * num_cols * 10);

    for row in 0..num_rows {
        for col in 0..num_cols {
            data.extend_from_slice(format!("row{}_col{}", row, col).as_bytes());
            if col < num_cols - 1 {
                data.push(b'\t');
            }
        }
        data.push(b'\n');
    }

    data
}

// ============================================================================
// Benchmark Group 1: SIMD Implementation Comparison
// ============================================================================

fn benchmark_simd_compare(c: &mut Criterion) {
    let mut group = c.benchmark_group("tsv_reader/simd_compare");

    let configs = vec![(1000, 5), (10000, 5), (1000, 50), (10000, 50)];

    for (rows, cols) in configs {
        let data = generate_tsv_data(rows, cols);
        let data_size = data.len();
        group.throughput(Throughput::Bytes(data_size as u64));

        let bench_id = format!("{}rows_{}cols", rows, cols);

        // memchr2 implementation
        group.bench_with_input(
            BenchmarkId::new("memchr2", &bench_id),
            &data,
            |b, data| {
                b.iter(|| {
                    let cursor = Cursor::new(std::hint::black_box(data));
                    let mut reader = TsvReader::new(cursor);
                    let mut count = 0;

                    while let Ok(Some(row)) = reader.next_row_memchr2(b'\t') {
                        for i in 1..=row.ends.len() {
                            if let Some(field) = row.get_bytes(i) {
                                std::hint::black_box(field);
                            }
                        }
                        count += 1;
                    }

                    std::hint::black_box(count);
                });
            },
        );

        // SSE2 implementation (x86_64 only)
        #[cfg(target_arch = "x86_64")]
        {
            group.bench_with_input(
                BenchmarkId::new("sse2", &bench_id),
                &data,
                |b, data| {
                    b.iter(|| {
                        let cursor = Cursor::new(std::hint::black_box(data));
                        let mut reader = TsvReader::new(cursor);
                        let mut count = 0;

                        unsafe {
                            while let Ok(Some(row)) =
                                reader.next_row_sse2_internal(b'\t')
                            {
                                for i in 1..=row.ends.len() {
                                    if let Some(field) = row.get_bytes(i) {
                                        std::hint::black_box(field);
                                    }
                                }
                                count += 1;
                            }
                        }

                        std::hint::black_box(count);
                    });
                },
            );
        }

        // NEON implementation (aarch64 only)
        #[cfg(target_arch = "aarch64")]
        {
            group.bench_with_input(
                BenchmarkId::new("neon", &bench_id),
                &data,
                |b, data| {
                    b.iter(|| {
                        let cursor = Cursor::new(std::hint::black_box(data));
                        let mut reader = TsvReader::new(cursor);
                        let mut count = 0;

                        unsafe {
                            while let Ok(Some(row)) =
                                reader.next_row_neon_internal(b'\t')
                            {
                                for i in 1..=row.ends.len() {
                                    if let Some(field) = row.get_bytes(i) {
                                        std::hint::black_box(field);
                                    }
                                }
                                count += 1;
                            }
                        }

                        std::hint::black_box(count);
                    });
                },
            );
        }

        // Auto-selected implementation (next_row with default backend)
        group.bench_with_input(BenchmarkId::new("auto", &bench_id), &data, |b, data| {
            b.iter(|| {
                let cursor = Cursor::new(std::hint::black_box(data));
                let mut reader = TsvReader::new(cursor);
                let mut count = 0;

                while let Ok(Some(row)) = reader.next_row(b'\t') {
                    for i in 1..=row.ends.len() {
                        if let Some(field) = row.get_bytes(i) {
                            std::hint::black_box(field);
                        }
                    }
                    count += 1;
                }

                std::hint::black_box(count);
            });
        });
    }

    group.finish();
}

// ============================================================================
// Benchmark Group 2: Reader Method Comparison
// Compares different approaches to reading TSV data
// ============================================================================

fn benchmark_reader_methods(c: &mut Criterion) {
    let mut group = c.benchmark_group("tsv_reader/methods");

    let configs = vec![(1000, 5), (10000, 5), (1000, 50), (10000, 50)];

    for (rows, cols) in configs {
        let data = generate_tsv_data(rows, cols);
        let data_size = data.len();
        group.throughput(Throughput::Bytes(data_size as u64));

        let bench_id = format!("{}rows_{}cols", rows, cols);

        // for_each_line_legacy (two-pass: find lines, then split fields)
        group.bench_with_input(
            BenchmarkId::new("for_each_line_legacy", &bench_id),
            &data,
            |b, data| {
                b.iter(|| {
                    let cursor = Cursor::new(std::hint::black_box(data));
                    let mut reader = TsvReader::new(cursor);
                    let mut count = 0;

                    reader
                        .for_each_line_legacy(|record| {
                            use memchr::memchr_iter;
                            for field in memchr_iter(b'\t', record) {
                                std::hint::black_box(field);
                            }
                            count += 1;
                            Ok(())
                        })
                        .unwrap();

                    std::hint::black_box(count);
                });
            },
        );

        // for_each_line (single-pass with SIMD, then split fields)
        group.bench_with_input(
            BenchmarkId::new("for_each_line_with_split", &bench_id),
            &data,
            |b, data| {
                b.iter(|| {
                    let cursor = Cursor::new(std::hint::black_box(data));
                    let mut reader = TsvReader::new(cursor);
                    let mut count = 0;

                    reader
                        .for_each_line(|record| {
                            use memchr::memchr_iter;
                            for field in memchr_iter(b'\t', record) {
                                std::hint::black_box(field);
                            }
                            count += 1;
                            Ok(())
                        })
                        .unwrap();

                    std::hint::black_box(count);
                });
            },
        );

        // for_each_line (single-pass, no field splitting - just line access)
        group.bench_with_input(
            BenchmarkId::new("for_each_line_only", &bench_id),
            &data,
            |b, data| {
                b.iter(|| {
                    let cursor = Cursor::new(std::hint::black_box(data));
                    let mut reader = TsvReader::new(cursor);
                    let mut count = 0;

                    reader
                        .for_each_line(|line| {
                            std::hint::black_box(line);
                            count += 1;
                            Ok(())
                        })
                        .unwrap();

                    std::hint::black_box(count);
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// Benchmark Group 3: Allocation Overhead
// Tests the cost of allocating the ends array for field positions
// ============================================================================

fn benchmark_allocation_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("tsv_reader/allocation_overhead");

    // Test with very wide tables (many columns = large ends array)
    let configs = vec![(1000, 100), (1000, 500), (1000, 1000)];

    for (rows, cols) in configs {
        let data = generate_tsv_data(rows, cols);
        let data_size = data.len();
        group.throughput(Throughput::Bytes(data_size as u64));

        let bench_id = format!("{}rows_{}cols", rows, cols);

        // next_row with \t - allocates large ends array (many field positions)
        group.bench_with_input(
            BenchmarkId::new("with_ends_alloc", &bench_id),
            &data,
            |b, data| {
                b.iter(|| {
                    let cursor = Cursor::new(std::hint::black_box(data));
                    let mut reader = TsvReader::new(cursor);
                    let mut count = 0;

                    while let Ok(Some(_row)) = reader.next_row(b'\t') {
                        count += 1;
                    }

                    std::hint::black_box(count);
                });
            },
        );

        // next_row with 0xFF (impossible byte) - allocates small ends array
        // This simulates line-only reading without field tracking
        group.bench_with_input(
            BenchmarkId::new("without_ends_alloc", &bench_id),
            &data,
            |b, data| {
                b.iter(|| {
                    let cursor = Cursor::new(std::hint::black_box(data));
                    let mut reader = TsvReader::new(cursor);
                    let mut count = 0;

                    while let Ok(Some(_row)) = reader.next_row(0xFF) {
                        count += 1;
                    }

                    std::hint::black_box(count);
                });
            },
        );

        // for_each_line - no ends array allocation at all
        group.bench_with_input(
            BenchmarkId::new("for_each_line", &bench_id),
            &data,
            |b, data| {
                b.iter(|| {
                    let cursor = Cursor::new(std::hint::black_box(data));
                    let mut reader = TsvReader::new(cursor);
                    let mut count = 0;

                    reader
                        .for_each_line(|_line| {
                            count += 1;
                            Ok(())
                        })
                        .unwrap();

                    std::hint::black_box(count);
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_simd_compare,
    benchmark_reader_methods,
    benchmark_allocation_overhead
);
criterion_main!(benches);
