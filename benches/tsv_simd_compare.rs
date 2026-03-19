//! Benchmark comparing SIMD implementations (SSE2/NEON) vs memchr2
//!
//! This benchmark compares the performance of:
//! - x86_64: SSE2 vs memchr2
//! - aarch64: NEON vs memchr2

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

fn benchmark_simd_compare(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd_vs_memchr");

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

        // memchr2 implementation (baseline)
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

        // Auto-selected implementation (for reference)
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

criterion_group!(benches, benchmark_simd_compare);
criterion_main!(benches);
