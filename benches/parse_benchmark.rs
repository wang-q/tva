use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use std::io::BufRead;

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

    // 1. csv crate
    group.bench_function("csv", |b| {
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

    // 2. simd-csv crate
    group.bench_function("simd-csv", |b| {
        b.iter(|| {
            let mut rdr = simd_csv::ReaderBuilder::new()
                .delimiter(b'\t')
                .has_headers(false)
                .from_reader(data.as_bytes());
            
            let mut record = simd_csv::ByteRecord::new();
            while let Ok(has_more) = rdr.read_byte_record(&mut record) {
                if !has_more { break; }
                black_box(&record);
            }
        })
    });

    // 3. Naive split (tva current implementation style)
    // Note: This allocates a String for each line and a Vec<&str> for fields.
    group.bench_function("naive_split_collect", |b| {
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
    
    // 4. Optimized Split (Iterator based, no Vec allocation per line)
    // This represents what tsv-select (D) does - lazy iteration
    group.bench_function("optimized_split_iter", |b| {
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

    group.finish();
}

criterion_group!(benches, benchmark_parsing);
criterion_main!(benches);
