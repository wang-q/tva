use rapidhash::RapidRng;
use std::cmp::{Ordering, Reverse};
use std::collections::BinaryHeap;
use std::io::Write;

pub trait Sampler {
    fn process<W: Write>(
        &mut self,
        record: &[u8],
        writer: &mut W,
        rng: &mut RapidRng,
    ) -> anyhow::Result<()>;
    fn finalize<W: Write>(
        &mut self,
        writer: &mut W,
        rng: &mut RapidRng,
        print_random: bool,
    ) -> anyhow::Result<()>;
}

pub const INV_U64_MAX_PLUS_1: f64 = 1.0 / (u64::MAX as f64 + 1.0);

#[derive(Debug)]
pub struct WeightedItem {
    pub key: f64,
    pub record: Vec<u8>,
}

impl PartialEq for WeightedItem {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl Eq for WeightedItem {}

impl PartialOrd for WeightedItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.key.partial_cmp(&other.key)
    }
}

impl Ord for WeightedItem {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

fn write_with_optional_random<W: std::io::Write>(
    writer: &mut W,
    row: &[u8],
    rng: &mut RapidRng,
    print_random: bool,
    random_value: Option<f64>,
) -> anyhow::Result<()> {
    if print_random {
        let v = match random_value {
            Some(x) => x,
            None => rng.next() as f64 * INV_U64_MAX_PLUS_1,
        };
        let mut buffer = ryu::Buffer::new();
        let printed = buffer.format(v);
        writer.write_all(printed.as_bytes())?;
        writer.write_all(b"\t")?;
    }
    writer.write_all(row)?;
    writer.write_all(b"\n")?;
    Ok(())
}

pub struct BernoulliSampler {
    pub prob: f64,
    pub print_random: bool,
    pub skip_counter: usize,
}

impl Sampler for BernoulliSampler {
    fn process<W: Write>(
        &mut self,
        record: &[u8],
        writer: &mut W,
        rng: &mut RapidRng,
    ) -> anyhow::Result<()> {
        if self.skip_counter > 0 {
            self.skip_counter -= 1;
            return Ok(());
        }

        // Process current record (selected)
        let r = rng.next() as f64 * INV_U64_MAX_PLUS_1;
        
        // If print_random is true, we need a random value for the output.
        // Even though selection was decided by skip_counter, we generate 'r' here
        // to maintain consistency if the user requested the random column.
        write_with_optional_random(writer, record, rng, self.print_random, Some(r))?;

        // Generate next skip interval using Geometric distribution
        // Variate generation: floor(ln(u) / ln(1-p))
        if self.prob >= 1.0 {
            self.skip_counter = 0;
        } else {
            let u = rng.next() as f64 * INV_U64_MAX_PLUS_1;
            // Avoid log(0)
            let u = if u < 1e-10 { 1e-10 } else { u };
            let val = u.ln() / (1.0 - self.prob).ln();
            self.skip_counter = val.floor() as usize;
        }

        Ok(())
    }
    fn finalize<W: Write>(
        &mut self,
        _writer: &mut W,
        _rng: &mut RapidRng,
        _print_random: bool,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}

pub struct ReservoirSampler {
    pub k: usize,
    pub reservoir: Vec<Vec<u8>>,
    pub count: usize,
}

impl Sampler for ReservoirSampler {
    fn process<W: Write>(
        &mut self,
        record: &[u8],
        _writer: &mut W,
        rng: &mut RapidRng,
    ) -> anyhow::Result<()> {
        if self.count < self.k {
            self.reservoir.push(record.to_vec());
        } else {
            let j = rng.next() as usize % (self.count + 1);
            if j < self.k {
                self.reservoir[j] = record.to_vec();
            }
        }
        self.count += 1;
        Ok(())
    }
    fn finalize<W: Write>(
        &mut self,
        writer: &mut W,
        rng: &mut RapidRng,
        print_random: bool,
    ) -> anyhow::Result<()> {
        let len = self.reservoir.len();
        // Shuffle reservoir (optional but matches previous behavior)
        for i in (1..len).rev() {
            let j = (rng.next() as usize) % (i + 1);
            self.reservoir.swap(i, j);
        }
        for row in &self.reservoir {
            write_with_optional_random(writer, row, rng, print_random, None)?;
        }
        Ok(())
    }
}

pub struct WeightedReservoirSampler {
    pub k: usize,
    pub weight_field_idx: usize,
    // Use a Min-Heap (via Reverse) to store the top-K items with largest keys.
    // The root of the heap is the item with the smallest key among the top-K.
    pub heap: BinaryHeap<Reverse<WeightedItem>>,
}

impl Sampler for WeightedReservoirSampler {
    fn process<W: Write>(
        &mut self,
        record: &[u8],
        _writer: &mut W,
        rng: &mut RapidRng,
    ) -> anyhow::Result<()> {
        let mut weight_bytes = None;

        // Scan for nth field (1-based index)
        if self.weight_field_idx == 1 {
            let end = memchr::memchr(b'\t', record).unwrap_or(record.len());
            weight_bytes = Some(&record[0..end]);
        } else {
            let mut iter = memchr::memchr_iter(b'\t', record);
            for _ in 0..self.weight_field_idx - 2 {
                if iter.next().is_none() {
                    break;
                }
            }
            if let Some(start_pos) = iter.next() {
                let start = start_pos + 1;
                let end = iter.next().unwrap_or(record.len());
                weight_bytes = Some(&record[start..end]);
            }
        }

        if let Some(w_bytes) = weight_bytes {
            if let Ok(w_str) = std::str::from_utf8(w_bytes) {
                if let Ok(w) = w_str.trim().parse::<f64>() {
                    if w > 0.0 {
                        let u = rng.next() as f64 * INV_U64_MAX_PLUS_1;
                        // A-Res Key: k = u^(1/w) <=> ln(k) = ln(u)/w
                        // We use ln(u)/w as the key.
                        let key = u.ln() / w;

                        if self.heap.len() < self.k {
                            self.heap.push(Reverse(WeightedItem {
                                key,
                                record: record.to_vec(),
                            }));
                        } else {
                            // Replace the smallest key in heap if new key is larger
                            if let Some(Reverse(min_item)) = self.heap.peek() {
                                if key > min_item.key {
                                    self.heap.pop();
                                    self.heap.push(Reverse(WeightedItem {
                                        key,
                                        record: record.to_vec(),
                                    }));
                                }
                            }
                        }
                    }
                } else {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("weight value `{}` is not a valid number", w_str),
                    )
                    .into());
                }
            }
        } else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("weight field index {} out of range", self.weight_field_idx),
            )
            .into());
        }
        Ok(())
    }

    fn finalize<W: Write>(
        &mut self,
        writer: &mut W,
        rng: &mut RapidRng,
        print_random: bool,
    ) -> anyhow::Result<()> {
        if self.heap.is_empty() {
            return Ok(());
        }
        
        // Extract items and sort by key descending (highest probability first)
        let mut items: Vec<WeightedItem> = self.heap.drain().map(|Reverse(item)| item).collect();
        items.sort_by(|a, b| b.key.partial_cmp(&a.key).unwrap_or(Ordering::Equal));

        for item in items {
            write_with_optional_random(writer, &item.record, rng, print_random, None)?;
        }
        Ok(())
    }
}

pub struct DistinctBernoulliSampler {
    pub prob: f64,
    pub key_field_indices: Vec<usize>,
    pub print_random: bool,
    pub decisions: std::collections::HashMap<Vec<u8>, (bool, f64)>,
}

impl Sampler for DistinctBernoulliSampler {
    fn process<W: Write>(
        &mut self,
        record: &[u8],
        writer: &mut W,
        rng: &mut RapidRng,
    ) -> anyhow::Result<()> {
        let key = if self.key_field_indices.is_empty() {
            record.to_vec()
        } else {
            let mut parts = Vec::new();
            
            // Extract specific fields for key
            let mut tab_iter = memchr::memchr_iter(b'\t', record);
            let mut last_pos = 0;

            let mut field_idx = 1; // 1-based
            let mut next_tab = tab_iter.next();

            for (i, &target_idx) in self.key_field_indices.iter().enumerate() {
                if i > 0 {
                    parts.push(0x1f);
                }

                // Advance to target_idx
                while field_idx < target_idx {
                    if let Some(pos) = next_tab {
                        last_pos = pos + 1;
                        next_tab = tab_iter.next();
                        field_idx += 1;
                    } else {
                        // End of record reached before target field
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            format!("key field index {} out of range", target_idx),
                        )
                        .into());
                    }
                }

                // Now at target_idx
                let end = next_tab.unwrap_or(record.len());
                parts.extend_from_slice(&record[last_pos..end]);
            }
            parts
        };

        let (keep, r) = self.decisions.entry(key).or_insert_with(|| {
            let r = rng.next() as f64 * INV_U64_MAX_PLUS_1;
            (r < self.prob, r)
        });

        if *keep {
            write_with_optional_random(
                writer,
                record,
                rng,
                self.print_random,
                Some(*r),
            )?;
        }
        Ok(())
    }

    fn finalize<W: Write>(
        &mut self,
        _writer: &mut W,
        _rng: &mut RapidRng,
        _print_random: bool,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}

pub struct ShuffleSampler {
    pub rows: Vec<Vec<u8>>,
}

impl Sampler for ShuffleSampler {
    fn process<W: Write>(
        &mut self,
        record: &[u8],
        _writer: &mut W,
        _rng: &mut RapidRng,
    ) -> anyhow::Result<()> {
        self.rows.push(record.to_vec());
        Ok(())
    }
    fn finalize<W: Write>(
        &mut self,
        writer: &mut W,
        rng: &mut RapidRng,
        print_random: bool,
    ) -> anyhow::Result<()> {
        let len = self.rows.len();
        for i in (1..len).rev() {
            let j = (rng.next() as usize) % (i + 1);
            self.rows.swap(i, j);
        }
        for row in &self.rows {
            write_with_optional_random(writer, row, rng, print_random, None)?;
        }
        Ok(())
    }
}

pub struct InorderSampler {
    pub k: usize,
    pub rows: Vec<Vec<u8>>,
}

impl Sampler for InorderSampler {
    fn process<W: Write>(
        &mut self,
        record: &[u8],
        _writer: &mut W,
        _rng: &mut RapidRng,
    ) -> anyhow::Result<()> {
        self.rows.push(record.to_vec());
        Ok(())
    }
    fn finalize<W: Write>(
        &mut self,
        writer: &mut W,
        rng: &mut RapidRng,
        print_random: bool,
    ) -> anyhow::Result<()> {
        let n = self.rows.len();
        if self.k == 0 || n == 0 {
            return Ok(());
        }
        if self.k >= n {
            for row in &self.rows {
                write_with_optional_random(writer, row, rng, print_random, None)?;
            }
            return Ok(());
        }

        let mut indices: Vec<usize> = (0..n).collect();
        for i in (1..n).rev() {
            let j = (rng.next() as usize) % (i + 1);
            indices.swap(i, j);
        }
        indices.truncate(self.k);
        indices.sort_unstable();

        for idx in indices {
            let row = &self.rows[idx];
            write_with_optional_random(writer, row, rng, print_random, None)?;
        }
        Ok(())
    }
}

pub struct ReplacementSampler {
    pub k: usize,
    pub rows: Vec<Vec<u8>>,
}

impl Sampler for ReplacementSampler {
    fn process<W: Write>(
        &mut self,
        record: &[u8],
        _writer: &mut W,
        _rng: &mut RapidRng,
    ) -> anyhow::Result<()> {
        self.rows.push(record.to_vec());
        Ok(())
    }
    fn finalize<W: Write>(
        &mut self,
        writer: &mut W,
        rng: &mut RapidRng,
        _print_random: bool,
    ) -> anyhow::Result<()> {
        if self.k == 0 || self.rows.is_empty() {
            return Ok(());
        }
        let n = self.rows.len();
        for _ in 0..self.k {
            let idx = (rng.next() as usize) % n;
            let row = &self.rows[idx];
            writer.write_all(row)?;
            writer.write_all(b"\n")?;
        }
        Ok(())
    }
}

pub struct CompatRandomSampler {
    pub k: usize,
    pub rows: Vec<Vec<u8>>,
}

impl Sampler for CompatRandomSampler {
    fn process<W: Write>(
        &mut self,
        record: &[u8],
        _writer: &mut W,
        _rng: &mut RapidRng,
    ) -> anyhow::Result<()> {
        self.rows.push(record.to_vec());
        Ok(())
    }
    fn finalize<W: Write>(
        &mut self,
        writer: &mut W,
        rng: &mut RapidRng,
        print_random: bool,
    ) -> anyhow::Result<()> {
        let n = self.rows.len();
        if n == 0 {
            return Ok(());
        }
        let sample_size = if self.k == 0 || self.k >= n {
            n
        } else {
            self.k
        };

        let mut keyed_indices: Vec<(f64, usize)> = Vec::with_capacity(n);
        for idx in 0..n {
            let r = rng.next() as f64 * INV_U64_MAX_PLUS_1;
            keyed_indices.push((r, idx));
        }
        keyed_indices.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        for (r, idx) in keyed_indices.into_iter().take(sample_size) {
            let row = &self.rows[idx];
            write_with_optional_random(writer, row, rng, print_random, Some(r))?;
        }
        Ok(())
    }
}
