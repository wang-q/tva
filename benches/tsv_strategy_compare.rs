//! Comprehensive TSV parsing strategy comparison benchmark
//!
//! Compares different parsing approaches:
//! 1. Hand-written SSE2 SIMD searcher (from simd-csv)
//! 2. memchr single-pass (memchr2)
//! 3. memchr two-pass (memchr + memchr_iter)
//! 4. Current TsvReader implementations

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use memchr::{memchr, memchr2, memchr_iter};

// ============================================================================
// Hand-written SSE2 SIMD Searcher (adapted from simd-csv)
// ============================================================================

#[cfg(target_arch = "x86_64")]
mod sse2 {
    use core::arch::x86_64::{
        __m128i, _mm_cmpeq_epi8, _mm_loadu_si128, _mm_movemask_epi8, _mm_or_si128, _mm_set1_epi8,
    };

    pub struct Sse2Searcher {
        v_tab: __m128i,
        v_newline: __m128i,
        v_cr: __m128i,
    }

    impl Sse2Searcher {
        #[inline]
        pub unsafe fn new(tab: u8, newline: u8, cr: u8) -> Self {
            Self {
                v_tab: _mm_set1_epi8(tab as i8),
                v_newline: _mm_set1_epi8(newline as i8),
                v_cr: _mm_set1_epi8(cr as i8),
            }
        }

        /// Search for tab, newline, or CR in haystack
        /// Returns iterator of positions
        #[inline(always)]
        pub fn search<'a>(&'a self, haystack: &'a [u8]) -> Sse2Iter<'a> {
            Sse2Iter::new(self, haystack)
        }
    }

    pub struct Sse2Iter<'a> {
        searcher: &'a Sse2Searcher,
        haystack: &'a [u8],
        pos: usize,
        mask: u32,
    }

    impl<'a> Sse2Iter<'a> {
        #[inline]
        fn new(searcher: &'a Sse2Searcher, haystack: &'a [u8]) -> Self {
            Self {
                searcher,
                haystack,
                pos: 0,
                mask: 0,
            }
        }

        #[inline(always)]
        fn next_mask(&mut self) -> Option<usize> {
            const STEP: usize = 16;

            loop {
                // Process current mask
                if self.mask != 0 {
                    let offset = self.pos - STEP + self.mask.trailing_zeros() as usize;
                    self.mask &= self.mask - 1; // Clear least significant bit
                    return Some(offset);
                }

                // Main SIMD loop
                let remaining = self.haystack.len() - self.pos;
                if remaining >= STEP {
                    let chunk = unsafe {
                        _mm_loadu_si128(self.haystack.as_ptr().add(self.pos) as *const __m128i)
                    };
                    let cmp1 = unsafe { _mm_cmpeq_epi8(chunk, self.searcher.v_tab) };
                    let cmp2 = unsafe { _mm_cmpeq_epi8(chunk, self.searcher.v_newline) };
                    let cmp3 = unsafe { _mm_cmpeq_epi8(chunk, self.searcher.v_cr) };
                    let cmp = unsafe { _mm_or_si128(cmp1, cmp2) };
                    let cmp = unsafe { _mm_or_si128(cmp, cmp3) };
                    self.mask = unsafe { _mm_movemask_epi8(cmp) } as u32;
                    self.pos += STEP;

                    if self.mask != 0 {
                        continue;
                    }
                } else {
                    // Linear scan for remaining bytes
                    while self.pos < self.haystack.len() {
                        let byte = self.haystack[self.pos];
                        if byte == b'\t' || byte == b'\n' || byte == b'\r' {
                            let offset = self.pos;
                            self.pos += 1;
                            return Some(offset);
                        }
                        self.pos += 1;
                    }
                    return None;
                }
            }
        }
    }

    impl Iterator for Sse2Iter<'_> {
        type Item = usize;

        #[inline(always)]
        fn next(&mut self) -> Option<Self::Item> {
            self.next_mask()
        }
    }
}

// ============================================================================
// Parsing Strategies
// ============================================================================

/// Strategy 1: Two-pass with memchr (current TsvReader approach)
/// Pass 1: Find newline with memchr
/// Pass 2: Split fields with memchr_iter
#[inline(always)]
fn parse_two_pass_memchr(data: &[u8]) -> usize {
    let mut count = 0;
    let mut start = 0;

    loop {
        match memchr(b'\n', &data[start..]) {
            Some(offset) => {
                let line = &data[start..start + offset];
                // Second pass: split fields
                for field in memchr_iter(b'\t', line) {
                    black_box(field);
                    count += 1;
                }
                black_box(line.len()); // Last field
                count += 1;
                start += offset + 1;
            }
            None => {
                // Last line without newline
                if start < data.len() {
                    let line = &data[start..];
                    for field in memchr_iter(b'\t', line) {
                        black_box(field);
                        count += 1;
                    }
                    black_box(line.len());
                    count += 1;
                }
                break;
            }
        }
    }

    count
}

/// Strategy 2: Single-pass with memchr2
/// Use memchr2 to find both tab and newline in one scan
#[inline(always)]
fn parse_single_pass_memchr2(data: &[u8]) -> usize {
    let mut count = 0;
    let mut pos = 0;
    let mut line_start = 0;
    let mut field_start = 0;

    loop {
        if pos >= data.len() {
            // Handle last line
            if field_start < data.len() {
                black_box(&data[field_start..]);
                count += 1;
            }
            break;
        }

        match memchr2(b'\t', b'\n', &data[pos..]) {
            Some(offset) => {
                let abs_pos = pos + offset;
                let byte = data[abs_pos];

                if byte == b'\t' {
                    // Field delimiter
                    black_box(&data[field_start..abs_pos]);
                    count += 1;
                    field_start = abs_pos + 1;
                    pos = abs_pos + 1;
                } else {
                    // Newline
                    black_box(&data[field_start..abs_pos]);
                    count += 1;
                    pos = abs_pos + 1;
                    line_start = pos;
                    field_start = pos;
                }
            }
            None => {
                // No more delimiters
                if field_start < data.len() {
                    black_box(&data[field_start..]);
                    count += 1;
                }
                break;
            }
        }
    }

    count
}

/// Strategy 3: Single-pass with SSE2 SIMD searcher
/// Use hand-written SSE2 to find tab, newline, CR in one scan
#[cfg(target_arch = "x86_64")]
#[inline(always)]
fn parse_single_pass_sse2(data: &[u8]) -> usize {
    use sse2::*;

    let mut count = 0;
    let mut pos = 0;
    let mut field_start = 0;

    unsafe {
        let searcher = Sse2Searcher::new(b'\t', b'\n', b'\r');
        let mut iter = searcher.search(data);

        while let Some(offset) = iter.next() {
            let byte = data[offset];

            if byte == b'\t' {
                // Field delimiter
                black_box(&data[field_start..offset]);
                count += 1;
                field_start = offset + 1;
            } else if byte == b'\n' {
                // Newline (handle CR+LF)
                let end = if offset > 0 && data[offset - 1] == b'\r' {
                    offset - 1
                } else {
                    offset
                };
                if field_start < end {
                    black_box(&data[field_start..end]);
                    count += 1;
                }
                field_start = offset + 1;
            }
            // Skip CR (handled with LF)
        }

        // Last field
        if field_start < data.len() {
            black_box(&data[field_start..]);
            count += 1;
        }
    }

    count
}

/// Strategy 4: Two-pass with SSE2
/// Pass 1: Find newlines with SSE2
/// Pass 2: Split fields with memchr_iter
#[cfg(target_arch = "x86_64")]
#[inline(always)]
fn parse_two_pass_sse2(data: &[u8]) -> usize {
    use sse2::*;

    let mut count = 0;
    let mut line_start = 0;

    unsafe {
        let searcher = Sse2Searcher::new(b'\t', b'\n', b'\r');
        let mut iter = searcher.search(data);
        let mut last_offset = 0;

        while let Some(offset) = iter.next() {
            let byte = data[offset];

            if byte == b'\n' {
                // Found newline - process the line
                let line_end = if offset > 0 && data[offset - 1] == b'\r' {
                    offset - 1
                } else {
                    offset
                };
                let line = &data[line_start..line_end];

                // Second pass: split fields
                for field in memchr_iter(b'\t', line) {
                    black_box(field);
                    count += 1;
                }
                black_box(line.len()); // Last field
                count += 1;

                line_start = offset + 1;
                last_offset = offset;
            }
        }

        // Last line
        if line_start < data.len() {
            let line = &data[line_start..];
            for field in memchr_iter(b'\t', line) {
                black_box(field);
                count += 1;
            }
            black_box(line.len());
            count += 1;
        }
    }

    count
}

/// Strategy 5: Naive byte-by-byte parsing (baseline)
#[inline(always)]
fn parse_naive(data: &[u8]) -> usize {
    let mut count = 0;
    let mut field_start = 0;

    for (pos, &byte) in data.iter().enumerate() {
        if byte == b'\t' {
            black_box(&data[field_start..pos]);
            count += 1;
            field_start = pos + 1;
        } else if byte == b'\n' {
            black_box(&data[field_start..pos]);
            count += 1;
            field_start = pos + 1;
        }
    }

    // Last field
    if field_start < data.len() {
        black_box(&data[field_start..]);
        count += 1;
    }

    count
}

// ============================================================================
// Benchmark
// ============================================================================

fn generate_tsv_data(num_rows: usize, num_cols: usize) -> Vec<u8> {
    let mut data = Vec::with_capacity(num_rows * num_cols * 10);

    for row in 0..num_rows {
        for col in 0..num_cols {
            // Generate field like "row123_col456"
            data.extend_from_slice(format!("row{}_col{}", row, col).as_bytes());
            if col < num_cols - 1 {
                data.push(b'\t');
            }
        }
        data.push(b'\n');
    }

    data
}

fn benchmark_strategies(c: &mut Criterion) {
    let mut group = c.benchmark_group("tsv_parsing_strategies");

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

        // Naive baseline
        group.bench_with_input(
            BenchmarkId::new("naive_byte_by_byte", &bench_id),
            &data,
            |b, data| {
                b.iter(|| parse_naive(black_box(data)));
            },
        );

        // Two-pass with memchr (current approach)
        group.bench_with_input(
            BenchmarkId::new("two_pass_memchr", &bench_id),
            &data,
            |b, data| {
                b.iter(|| parse_two_pass_memchr(black_box(data)));
            },
        );

        // Single-pass with memchr2
        group.bench_with_input(
            BenchmarkId::new("single_pass_memchr2", &bench_id),
            &data,
            |b, data| {
                b.iter(|| parse_single_pass_memchr2(black_box(data)));
            },
        );

        // SSE2 variants (x86_64 only)
        #[cfg(target_arch = "x86_64")]
        {
            group.bench_with_input(
                BenchmarkId::new("single_pass_sse2", &bench_id),
                &data,
                |b, data| {
                    b.iter(|| parse_single_pass_sse2(black_box(data)));
                },
            );

            group.bench_with_input(
                BenchmarkId::new("two_pass_sse2", &bench_id),
                &data,
                |b, data| {
                    b.iter(|| parse_two_pass_sse2(black_box(data)));
                },
            );
        }
    }

    group.finish();
}

criterion_group!(benches, benchmark_strategies);
criterion_main!(benches);
