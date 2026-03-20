//! Benchmark comparing next_row vs next_line performance
//!
//! This benchmark tests the overhead of:
//! - next_row: searches \t, \n, \r and allocates ends array
//! - next_line (simulated): searches \n, \r only, no allocation

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

fn benchmark_next_line(c: &mut Criterion) {
    let mut group = c.benchmark_group("next_row_vs_line");

    // Test different data sizes
    let configs = vec![
        (1000, 5),   // Small: 1K rows, 5 cols
        (10000, 5),  // Medium: 10K rows, 5 cols
        (1000, 50),  // Wide: 1K rows, 50 cols
        (10000, 50), // Large wide: 10K rows, 50 cols
    ];

    for (rows, cols) in configs {
        let data = generate_tsv_data(rows, cols);
        let data_size = data.len();
        group.throughput(Throughput::Bytes(data_size as u64));

        let bench_id = format!("{}rows_{}cols", rows, cols);

        // next_row with \t - baseline (allocates ends array)
        group.bench_with_input(
            BenchmarkId::new("next_row_tab", &bench_id),
            &data,
            |b, data| {
                b.iter(|| {
                    let cursor = Cursor::new(std::hint::black_box(data));
                    let mut reader = TsvReader::new(cursor);
                    let mut count = 0;

                    while let Ok(Some(row)) = reader.next_row(b'\t') {
                        // Access all fields to simulate real usage
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

        // next_row with 0xFF (impossible byte) - simulates next_line
        // This searches for a delimiter that never appears, effectively only finding \n and \r
        // But still allocates ends array (which will have only 1 element per row)
        group.bench_with_input(
            BenchmarkId::new("next_row_no_tab", &bench_id),
            &data,
            |b, data| {
                b.iter(|| {
                    let cursor = Cursor::new(std::hint::black_box(data));
                    let mut reader = TsvReader::new(cursor);
                    let mut count = 0;

                    // Use 0xFF as delimiter - it won't appear in normal text
                    // This tests the overhead of searching for \t vs not searching
                    while let Ok(Some(row)) = reader.next_row(0xFF) {
                        // Only access the single "field" (whole line)
                        if let Some(field) = row.get_bytes(1) {
                            std::hint::black_box(field);
                        }
                        count += 1;
                    }

                    std::hint::black_box(count);
                });
            },
        );

        // for_each_line - current implementation using next_row internally
        group.bench_with_input(
            BenchmarkId::new("for_each_line", &bench_id),
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

// Additional benchmark to measure allocation overhead specifically
fn benchmark_allocation_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("allocation_overhead");

    // Test with very wide table (many columns = large ends array)
    let configs = vec![
        (1000, 100),  // 1K rows, 100 cols
        (1000, 500),  // 1K rows, 500 cols
        (1000, 1000), // 1K rows, 1000 cols
    ];

    for (rows, cols) in configs {
        let data = generate_tsv_data(rows, cols);
        let data_size = data.len();
        group.throughput(Throughput::Bytes(data_size as u64));

        let bench_id = format!("{}rows_{}cols", rows, cols);

        // next_row with \t - allocates large ends array
        group.bench_with_input(
            BenchmarkId::new("with_ends_alloc", &bench_id),
            &data,
            |b, data| {
                b.iter(|| {
                    let cursor = Cursor::new(std::hint::black_box(data));
                    let mut reader = TsvReader::new(cursor);
                    let mut count = 0;

                    while let Ok(Some(row)) = reader.next_row(b'\t') {
                        // Just count, don't access fields
                        count += 1;
                    }

                    std::hint::black_box(count);
                });
            },
        );

        // next_row with 0xFF - allocates small ends array (1 element per row)
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
    }

    group.finish();
}

criterion_group!(benches, benchmark_next_line, benchmark_allocation_overhead);
criterion_main!(benches);
