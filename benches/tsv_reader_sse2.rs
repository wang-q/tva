//! Benchmark comparing TsvReader with SSE2 vs standard implementation

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

fn benchmark_tsv_reader(c: &mut Criterion) {
    let mut group = c.benchmark_group("tsv_reader_sse2");

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

        // next_row implementation (auto-selects SSE2 on x86_64)
        group.bench_with_input(
            BenchmarkId::new("next_row", &bench_id),
            &data,
            |b, data| {
                b.iter(|| {
                    let cursor = Cursor::new(std::hint::black_box(data));
                    let mut reader = TsvReader::new(cursor);
                    let mut count = 0;

                    while let Ok(Some(row)) = reader.next_row(b'\t') {
                        // Iterate through fields using get_bytes
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

        // for_each_record_legacy (two-pass approach)
        group.bench_with_input(
            BenchmarkId::new("for_each_record_legacy", &bench_id),
            &data,
            |b, data| {
                b.iter(|| {
                    let cursor = Cursor::new(std::hint::black_box(data));
                    let mut reader = TsvReader::new(cursor);
                    let mut count = 0;

                    reader
                        .for_each_record_legacy(|record| {
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

        // for_each_record (single-pass with SIMD)
        group.bench_with_input(
            BenchmarkId::new("for_each_record", &bench_id),
            &data,
            |b, data| {
                b.iter(|| {
                    let cursor = Cursor::new(std::hint::black_box(data));
                    let mut reader = TsvReader::new(cursor);
                    let mut count = 0;

                    reader
                        .for_each_record(|record| {
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
    }

    group.finish();
}

criterion_group!(benches, benchmark_tsv_reader);
criterion_main!(benches);
