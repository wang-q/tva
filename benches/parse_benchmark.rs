use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use std::io::{BufRead, Read};
use std::time::Duration;

// A small sample of TSV data to repeat
const DATA: &str = "1\tJohn\tDoe\t30\tNew York\n2\tJane\tSmith\t25\tLos Angeles\n3\tBob\tJohnson\t40\tChicago\n";
// Number of repetitions to make the benchmark meaningful
const REPETITIONS: usize = 1000;

fn create_tsv_data() -> String {
    let mut s = String::with_capacity(DATA.len() * REPETITIONS);
    for _ in 0..REPETITIONS {
        s.push_str(DATA);
    }
    s
}

fn benchmark_parsing(c: &mut Criterion) {
    let data = create_tsv_data();
    let mut group = c.benchmark_group("tsv_parsing");
    group.throughput(Throughput::Bytes(data.len() as u64));
    group.warm_up_time(Duration::from_secs(3));
    group.measurement_time(Duration::from_secs(8));

    // 1. csv crate (The Gold Standard for Correctness)
    // Uses a highly optimized DFA state machine.
    // Handles quotes, escapes, etc.
    group.bench_function("csv_crate", |b| {
        b.iter(|| {
            let mut rdr = csv::ReaderBuilder::new()
                .delimiter(b'\t')
                .has_headers(false)
                .from_reader(data.as_bytes());

            let mut record = csv::ByteRecord::new();
            while rdr.read_byte_record(&mut record).unwrap() {
                black_box(&record);
            }
        })
    });

    // 2. simd-csv crate (The Speed Limit)
    // Uses SIMD to process blocks of data.
    group.bench_function("simd_csv_crate", |b| {
        b.iter(|| {
            let mut rdr = simd_csv::ReaderBuilder::new()
                .delimiter(b'\t')
                .has_headers(false)
                .from_reader(data.as_bytes());

            let mut record = simd_csv::ByteRecord::new();
            while let Ok(has_more) = rdr.read_byte_record(&mut record) {
                if !has_more {
                    break;
                }
                black_box(&record);
            }
        })
    });

    // 3. Naive Split (Original TVA)
    // Allocates a String for each line (lines())
    // Allocates a Vec<&str> for fields (collect())
    group.bench_function("algo_naive_split_collect", |b| {
        b.iter(|| {
            let cursor = std::io::Cursor::new(data.as_bytes());
            let reader = std::io::BufReader::new(cursor);

            for line in reader.lines() {
                let line = line.unwrap();
                let fields: Vec<&str> = line.split('\t').collect();
                black_box(fields);
            }
        })
    });

    // 4. Optimized Split (Iterator)
    // Allocates a String for each line (lines())
    // Uses lazy iterator for fields (no Vec allocation)
    group.bench_function("algo_std_split_iter", |b| {
        b.iter(|| {
            let cursor = std::io::Cursor::new(data.as_bytes());
            let reader = std::io::BufReader::new(cursor);

            for line in reader.lines() {
                let line = line.unwrap();
                for field in line.split('\t') {
                    black_box(field);
                }
            }
        })
    });

    // 5. Manual Byte Loop (Scalar)
    // Allocates a String for each line (lines())
    // Manually iterates bytes to find tabs.
    // Shows if std::split has overhead.
    group.bench_function("algo_manual_byte_loop", |b| {
        b.iter(|| {
            let cursor = std::io::Cursor::new(data.as_bytes());
            let reader = std::io::BufReader::new(cursor);

            for line in reader.lines() {
                let line = line.unwrap();
                let bytes = line.as_bytes();
                let mut last_pos = 0;
                for (i, &byte) in bytes.iter().enumerate() {
                    if byte == b'\t' {
                        let field = &bytes[last_pos..i];
                        black_box(field);
                        last_pos = i + 1;
                    }
                }
                let field = &bytes[last_pos..];
                black_box(field);
            }
        })
    });

    // 6. Memchr Inline (Current TVA Select)
    // Allocates a String for each line (lines())
    // Uses SIMD (memchr) to find tabs.
    group.bench_function("algo_memchr_inline_loop", |b| {
        b.iter(|| {
            let cursor = std::io::Cursor::new(data.as_bytes());
            let reader = std::io::BufReader::new(cursor);

            for line in reader.lines() {
                let line = line.unwrap();
                let line_bytes = line.as_bytes();
                let mut iter = memchr::memchr_iter(b'\t', line_bytes);
                let mut last_pos = 0;
                loop {
                    match iter.next() {
                        Some(pos) => {
                            let field =
                                unsafe { line_bytes.get_unchecked(last_pos..pos) };
                            black_box(field);
                            last_pos = pos + 1;
                        }
                        None => {
                            let field = unsafe { line_bytes.get_unchecked(last_pos..) };
                            black_box(field);
                            break;
                        }
                    }
                }
            }
        })
    });

    // 7. Memchr + Reused Buffer (Future Optimization)
    // Reuses a single Vec<u8> buffer (read_until).
    // Uses SIMD (memchr) to find tabs.
    // No String allocation per line!
    group.bench_function("algo_memchr_reused_buffer", |b| {
        b.iter(|| {
            let cursor = std::io::Cursor::new(data.as_bytes());
            let mut reader = std::io::BufReader::new(cursor);
            let mut buf = Vec::with_capacity(1024);

            while reader.read_until(b'\n', &mut buf).unwrap() > 0 {
                // Strip newline logic (simplified)
                let mut len = buf.len();
                if len > 0 && buf[len - 1] == b'\n' {
                    len -= 1;
                    if len > 0 && buf[len - 1] == b'\r' {
                        len -= 1;
                    }
                }
                let line_bytes = &buf[..len];

                let mut iter = memchr::memchr_iter(b'\t', line_bytes);
                let mut last_pos = 0;
                loop {
                    match iter.next() {
                        Some(pos) => {
                            let field =
                                unsafe { line_bytes.get_unchecked(last_pos..pos) };
                            black_box(field);
                            last_pos = pos + 1;
                        }
                        None => {
                            let field = unsafe { line_bytes.get_unchecked(last_pos..) };
                            black_box(field);
                            break;
                        }
                    }
                }
                buf.clear();
            }
        })
    });

    // 8. TsvRecord (Struct with pre-allocation)
    // Simulates tva::libs::tsv::record::TsvRecord
    group.bench_function("tsv_record_struct", |b| {
        b.iter(|| {
            let cursor = std::io::Cursor::new(data.as_bytes());
            let mut reader = std::io::BufReader::new(cursor);
            let mut buf = Vec::with_capacity(1024);
            let mut record = tva::libs::tsv::record::TsvRecord::with_capacity(1024, 20);

            while reader.read_until(b'\n', &mut buf).unwrap() > 0 {
                // Strip newline
                let mut len = buf.len();
                if len > 0 && buf[len - 1] == b'\n' {
                    len -= 1;
                    if len > 0 && buf[len - 1] == b'\r' {
                        len -= 1;
                    }
                }

                record.parse_line(&buf[..len], b'\t');

                // Access fields to simulate work
                for field in record.iter() {
                    black_box(field);
                }

                buf.clear();
            }
        })
    });

    // 9. Multi-char SIMD (memchr2)
    // Searches for \t and \n simultaneously.
    // Simulates what a specialized reader would do.
    group.bench_function("algo_memchr2_simd_loop", |b| {
        b.iter(|| {
            // In a real implementation, we would read chunks.
            // Here we just operate on the whole buffer to test the SIMD loop speed.
            // (Assuming data fits in memory, or we map it)
            let bytes = data.as_bytes();
            let iter = memchr::memchr2_iter(b'\t', b'\n', bytes);
            let mut last_pos = 0;
            for pos in iter {
                // SAFETY: memchr returns valid indices
                let field = unsafe { bytes.get_unchecked(last_pos..pos) };
                black_box(field);
                last_pos = pos + 1;
            }
            let field = unsafe { bytes.get_unchecked(last_pos..) };
            black_box(field);
        })
    });

    // 10. Portable SIMD (std::simd)
    // Manually implemented SIMD scan using std::simd (nightly only, but we can simulate with manual masking logic)
    // Or we can use SWAR (SIMD Within A Register) for a safe Rust comparison.
    // SWAR is usually slower than memchr (which uses hand-tuned assembly/intrinsics), but good to check.
    // Let's skip SWAR as memchr is the gold standard for byte search.

    // 11. Chunked Reader Simulation
    // Reads fixed size chunks (e.g. 8KB) and processes them.
    // Handles fields crossing chunk boundaries.
    // This is the most realistic model for a high-performance reader.
    group.bench_function("algo_chunked_reader_sim", |b| {
        b.iter(|| {
            let mut cursor = std::io::Cursor::new(data.as_bytes());
            let mut buf = [0u8; 8 * 1024]; // 8KB chunk
            let mut leftover_len = 0;

            loop {
                // Fill buffer starting after leftover
                let read_len = cursor.read(&mut buf[leftover_len..]).unwrap();
                if read_len == 0 {
                    break;
                }
                let end = leftover_len + read_len;
                let active_buf = &buf[..end];

                // Find last newline to know where to stop processing
                // We process up to the last complete record.
                let mut valid_end = end;
                // Reverse search for \n
                let mut i = end;
                while i > 0 {
                    i -= 1;
                    if active_buf[i] == b'\n' {
                        valid_end = i + 1;
                        break;
                    }
                }

                // If no newline found, we need a bigger buffer or just keep extending.
                // For this benchmark, data is simple, so valid_end is likely found unless chunk is tiny.
                if valid_end == 0 && read_len > 0 {
                    // Pathological case: line larger than buffer.
                    // In real impl, grow buffer. Here, just panic or skip.
                    break;
                }

                let process_buf = &active_buf[..valid_end];
                let iter = memchr::memchr2_iter(b'\t', b'\n', process_buf);
                let mut last_pos = 0;
                for pos in iter {
                    let field = unsafe { process_buf.get_unchecked(last_pos..pos) };
                    black_box(field);
                    last_pos = pos + 1;
                }
                let field = unsafe { process_buf.get_unchecked(last_pos..) };
                black_box(field);

                // Move leftover to start
                leftover_len = end - valid_end;
                if leftover_len > 0 {
                    // Use copy_within (memmove)
                    buf.copy_within(valid_end..end, 0);
                }
            }
        })
    });

    // 12. TVA TsvReader (Zero-copy, SIMD)
    // Uses internal buffer + memchr for zero-copy iteration.
    // Avoids String/Vec allocation per record.
    group.bench_function("tva_tsv_reader", |b| {
        b.iter(|| {
            let cursor = std::io::Cursor::new(data.as_bytes());
            let mut reader = tva::libs::tsv::reader::TsvReader::new(cursor);
            reader
                .for_each_record(|record| {
                    // Simulate field processing to be fair with other benchmarks
                    let mut iter = memchr::memchr_iter(b'\t', record);
                    let mut last_pos = 0;
                    loop {
                        match iter.next() {
                            Some(pos) => {
                                let field =
                                    unsafe { record.get_unchecked(last_pos..pos) };
                                black_box(field);
                                last_pos = pos + 1;
                            }
                            None => {
                                let field = unsafe { record.get_unchecked(last_pos..) };
                                black_box(field);
                                break;
                            }
                        }
                    }
                    Ok(())
                })
                .unwrap();
        })
    });

    group.finish();
}

criterion_group!(benches, benchmark_parsing);
criterion_main!(benches);
