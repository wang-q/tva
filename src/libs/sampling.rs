use ahash::AHashMap;
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
        let mut items: Vec<WeightedItem> =
            self.heap.drain().map(|Reverse(item)| item).collect();
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
    pub decisions: AHashMap<Vec<u8>, (bool, f64)>,
    pub key_buffer: Vec<u8>,
}

impl Sampler for DistinctBernoulliSampler {
    fn process<W: Write>(
        &mut self,
        record: &[u8],
        writer: &mut W,
        rng: &mut RapidRng,
    ) -> anyhow::Result<()> {
        self.key_buffer.clear();

        if self.key_field_indices.is_empty() {
            self.key_buffer.extend_from_slice(record);
        } else {
            // Extract specific fields for key
            let mut tab_iter = memchr::memchr_iter(b'\t', record);
            let mut last_pos = 0;

            let mut field_idx = 1; // 1-based
            let mut next_tab = tab_iter.next();

            for (i, &target_idx) in self.key_field_indices.iter().enumerate() {
                if i > 0 {
                    self.key_buffer.push(0x1f); // Unit Separator as internal delimiter
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
                self.key_buffer.extend_from_slice(&record[last_pos..end]);
            }
        }

        let (keep, r) = if let Some(&(keep, r)) = self.decisions.get(&self.key_buffer) {
            (keep, r)
        } else {
            let r = rng.next() as f64 * INV_U64_MAX_PLUS_1;
            let keep = r < self.prob;
            self.decisions.insert(self.key_buffer.clone(), (keep, r));
            (keep, r)
        };

        if keep {
            write_with_optional_random(writer, record, rng, self.print_random, Some(r))?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use rapidhash::RapidRng;

    fn get_rng(seed: u64) -> RapidRng {
        RapidRng::new(seed)
    }

    #[test]
    fn test_bernoulli_sampler() {
        let mut rng = get_rng(42);
        let mut output = Vec::new();
        // prob=0.5
        let mut sampler = BernoulliSampler {
            prob: 0.5,
            print_random: false,
            skip_counter: 0,
        };

        let inputs: Vec<String> = (0..10).map(|i| format!("row{}", i)).collect();
        for row in &inputs {
            sampler
                .process(row.as_bytes(), &mut output, &mut rng)
                .unwrap();
        }
        // Finalize does nothing for Bernoulli
        sampler.finalize(&mut output, &mut rng, false).unwrap();

        let out_str = String::from_utf8(output).unwrap();
        let lines: Vec<&str> = out_str
            .trim()
            .split('\n')
            .filter(|s| !s.is_empty())
            .collect();

        // Deterministic check with seed 42
        // It selects some rows.
        assert!(!lines.is_empty());
        assert!(lines.len() <= 10);
    }

    #[test]
    fn test_reservoir_sampler() {
        let mut rng = get_rng(42);
        let mut output = Vec::new();
        let mut sampler = ReservoirSampler {
            k: 3,
            reservoir: Vec::new(),
            count: 0,
        };

        let inputs: Vec<String> = (0..10).map(|i| format!("row{}", i)).collect();
        for row in &inputs {
            sampler
                .process(row.as_bytes(), &mut output, &mut rng)
                .unwrap();
        }
        sampler.finalize(&mut output, &mut rng, false).unwrap();

        let out_str = String::from_utf8(output).unwrap();
        let lines: Vec<&str> = out_str
            .trim()
            .split('\n')
            .filter(|s| !s.is_empty())
            .collect();

        assert_eq!(lines.len(), 3);
        // Verify unique lines
        let mut sorted = lines.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), 3);
    }

    #[test]
    fn test_weighted_reservoir_sampler() {
        let mut rng = get_rng(12345);
        let mut output = Vec::new();
        let mut sampler = WeightedReservoirSampler {
            k: 2,
            weight_field_idx: 2,
            heap: BinaryHeap::new(),
        };

        // item\tweight
        // Large weights should be preferred
        let inputs = vec![
            "A\t100000.0",
            "B\t0.0001",
            "C\t100000.0",
            "D\t0.0001",
            "E\t0.0001",
        ];

        for row in &inputs {
            sampler
                .process(row.as_bytes(), &mut output, &mut rng)
                .unwrap();
        }
        sampler.finalize(&mut output, &mut rng, false).unwrap();

        let out_str = String::from_utf8(output).unwrap();
        let lines: Vec<&str> = out_str
            .trim()
            .split('\n')
            .filter(|s| !s.is_empty())
            .collect();

        assert_eq!(lines.len(), 2);
        // A and C should be selected
        assert!(out_str.contains("A\t"));
        assert!(out_str.contains("C\t"));
    }

    #[test]
    fn test_distinct_bernoulli_sampler() {
        let mut rng = get_rng(42);
        let mut output = Vec::new();
        let mut sampler = DistinctBernoulliSampler {
            prob: 0.5,
            key_field_indices: vec![1], // Key is 1st field
            print_random: false,
            decisions: AHashMap::new(),
            key_buffer: Vec::new(),
        };

        // Same keys should have same decision
        let inputs = vec![
            "k1\tval1", "k2\tval2", "k1\tval3", // Should match k1 decision
            "k3\tval4", "k2\tval5", // Should match k2 decision
        ];

        for row in &inputs {
            sampler
                .process(row.as_bytes(), &mut output, &mut rng)
                .unwrap();
        }

        let out_str = String::from_utf8(output).unwrap();
        let lines: Vec<&str> = out_str
            .trim()
            .split('\n')
            .filter(|s| !s.is_empty())
            .collect();

        // Check consistency
        let k1_in = lines.iter().any(|l| l.starts_with("k1"));
        let k2_in = lines.iter().any(|l| l.starts_with("k2"));

        // Count occurrences
        let k1_count = lines.iter().filter(|l| l.starts_with("k1")).count();
        let k2_count = lines.iter().filter(|l| l.starts_with("k2")).count();

        // If k1 is kept, both k1 lines should be present
        if k1_in {
            assert_eq!(k1_count, 2);
        } else {
            assert_eq!(k1_count, 0);
        }

        if k2_in {
            assert_eq!(k2_count, 2);
        } else {
            assert_eq!(k2_count, 0);
        }
    }

    #[test]
    fn test_shuffle_sampler() {
        let mut rng = get_rng(42);
        let mut output = Vec::new();
        let mut sampler = ShuffleSampler { rows: Vec::new() };

        let inputs: Vec<String> = (0..10).map(|i| format!("row{}", i)).collect();
        for row in &inputs {
            sampler
                .process(row.as_bytes(), &mut output, &mut rng)
                .unwrap();
        }
        sampler.finalize(&mut output, &mut rng, false).unwrap();

        let out_str = String::from_utf8(output).unwrap();
        let lines: Vec<&str> = out_str
            .trim()
            .split('\n')
            .filter(|s| !s.is_empty())
            .collect();

        assert_eq!(lines.len(), 10);
        // Order should likely change, but small chance it doesn't.
        // With seed 42, we can check if it changed.
        let is_same_order = lines.iter().zip(inputs.iter()).all(|(a, b)| a == b);
        assert!(!is_same_order);

        // Content should be same
        let mut sorted_out = lines.clone();
        sorted_out.sort();
        let mut sorted_in = inputs.iter().map(|s| s.as_str()).collect::<Vec<_>>();
        sorted_in.sort();
        assert_eq!(sorted_out, sorted_in);
    }

    #[test]
    fn test_inorder_sampler() {
        let mut rng = get_rng(42);
        let mut output = Vec::new();
        let mut sampler = InorderSampler {
            k: 5,
            rows: Vec::new(),
        };

        let inputs: Vec<String> = (0..10).map(|i| format!("row{:02}", i)).collect();
        for row in &inputs {
            sampler
                .process(row.as_bytes(), &mut output, &mut rng)
                .unwrap();
        }
        sampler.finalize(&mut output, &mut rng, false).unwrap();

        let out_str = String::from_utf8(output).unwrap();
        let lines: Vec<&str> = out_str
            .trim()
            .split('\n')
            .filter(|s| !s.is_empty())
            .collect();

        assert_eq!(lines.len(), 5);

        // Order must be preserved (increasing indices)
        // Since input is sorted, output must be sorted
        let mut sorted_out = lines.clone();
        sorted_out.sort();
        assert_eq!(lines, sorted_out);
    }

    #[test]
    fn test_replacement_sampler() {
        let mut rng = get_rng(42);
        let mut output = Vec::new();
        let mut sampler = ReplacementSampler {
            k: 20,
            rows: Vec::new(),
        };

        let inputs: Vec<String> = (0..5).map(|i| format!("row{}", i)).collect();
        for row in &inputs {
            sampler
                .process(row.as_bytes(), &mut output, &mut rng)
                .unwrap();
        }
        sampler.finalize(&mut output, &mut rng, false).unwrap();

        let out_str = String::from_utf8(output).unwrap();
        let lines: Vec<&str> = out_str
            .trim()
            .split('\n')
            .filter(|s| !s.is_empty())
            .collect();

        assert_eq!(lines.len(), 20);
        // Since k=20 and n=5, there MUST be duplicates (pigeonhole principle)
        let mut unique = lines.clone();
        unique.sort();
        unique.dedup();
        assert!(unique.len() <= 5);
        assert!(lines.len() > unique.len());
    }

    #[test]
    fn test_compat_random_sampler() {
        let mut rng = get_rng(42);
        let mut output = Vec::new();
        let mut sampler = CompatRandomSampler {
            k: 3,
            rows: Vec::new(),
        };

        let inputs: Vec<String> = (0..10).map(|i| format!("row{}", i)).collect();
        for row in &inputs {
            sampler
                .process(row.as_bytes(), &mut output, &mut rng)
                .unwrap();
        }
        sampler.finalize(&mut output, &mut rng, false).unwrap();

        let out_str = String::from_utf8(output).unwrap();
        let lines: Vec<&str> = out_str
            .trim()
            .split('\n')
            .filter(|s| !s.is_empty())
            .collect();

        assert_eq!(lines.len(), 3);
    }

    #[test]
    fn test_weighted_reservoir_sampler_invalid_weight() {
        let mut rng = get_rng(42);
        let mut output = Vec::new();
        let mut sampler = WeightedReservoirSampler {
            k: 2,
            weight_field_idx: 2,
            heap: BinaryHeap::new(),
        };

        // Negative weights should be ignored (Ok), non-numeric should be error
        let row_neg = "A\t-1.0";
        let res_neg = sampler.process(row_neg.as_bytes(), &mut output, &mut rng);
        assert!(res_neg.is_ok());

        let row_invalid = "B\tnot_a_number";
        let res_invalid = sampler.process(row_invalid.as_bytes(), &mut output, &mut rng);
        assert!(res_invalid.is_err());
    }

    #[test]
    fn test_weighted_reservoir_sampler_field_out_of_bounds() {
        let mut rng = get_rng(42);
        let mut output = Vec::new();
        // Index 3, but input has only 2 fields
        let mut sampler = WeightedReservoirSampler {
            k: 2,
            weight_field_idx: 3,
            heap: BinaryHeap::new(),
        };

        let inputs = vec!["A\t1.0"];

        for row in &inputs {
            let res = sampler.process(row.as_bytes(), &mut output, &mut rng);
            assert!(res.is_err());
        }
    }

    #[test]
    fn test_distinct_bernoulli_sampler_multiple_keys() {
        let mut rng = get_rng(42);
        let mut output = Vec::new();
        // Key is fields 1 and 3.
        let mut sampler = DistinctBernoulliSampler {
            prob: 0.5,
            key_field_indices: vec![1, 3],
            print_random: false,
            decisions: AHashMap::new(),
            key_buffer: Vec::new(),
        };

        // k1\tv1\tk2
        let inputs = vec![
            "A\tval\tX",
            "A\tval\tY", // Different key (A, Y)
            "A\tval\tX", // Same key as first (A, X)
            "B\tval\tX", // Different key (B, X)
        ];

        for row in &inputs {
            sampler
                .process(row.as_bytes(), &mut output, &mut rng)
                .unwrap();
        }

        let out_str = String::from_utf8(output).unwrap();
        let lines: Vec<&str> = out_str
            .trim()
            .split('\n')
            .filter(|s| !s.is_empty())
            .collect();

        // (A, X) should appear twice or zero times
        let ax_count = lines.iter().filter(|l| l.contains("A\tval\tX")).count();
        if ax_count > 0 {
            assert_eq!(ax_count, 2);
        }
    }

    #[test]
    fn test_reservoir_sampler_k_greater_than_n() {
        let mut rng = get_rng(42);
        let mut output = Vec::new();
        let mut sampler = ReservoirSampler {
            k: 20, // > 5 items
            reservoir: Vec::new(),
            count: 0,
        };

        let inputs: Vec<String> = (0..5).map(|i| format!("row{}", i)).collect();
        for row in &inputs {
            sampler
                .process(row.as_bytes(), &mut output, &mut rng)
                .unwrap();
        }
        sampler.finalize(&mut output, &mut rng, false).unwrap();

        let out_str = String::from_utf8(output).unwrap();
        let lines: Vec<&str> = out_str
            .trim()
            .split('\n')
            .filter(|s| !s.is_empty())
            .collect();

        assert_eq!(lines.len(), 5);
    }

    #[test]
    fn test_reservoir_sampler_k_zero() {
        let mut rng = get_rng(42);
        let mut output = Vec::new();
        let mut sampler = ReservoirSampler {
            k: 0,
            reservoir: Vec::new(),
            count: 0,
        };

        let inputs: Vec<String> = (0..5).map(|i| format!("row{}", i)).collect();
        for row in &inputs {
            sampler
                .process(row.as_bytes(), &mut output, &mut rng)
                .unwrap();
        }
        sampler.finalize(&mut output, &mut rng, false).unwrap();

        let out_str = String::from_utf8(output).unwrap();
        assert!(out_str.is_empty());
    }

    #[test]
    fn test_bernoulli_sampler_prob_one() {
        let mut rng = get_rng(42);
        let mut output = Vec::new();
        let mut sampler = BernoulliSampler {
            prob: 1.0,
            print_random: false,
            skip_counter: 0,
        };

        let inputs: Vec<String> = (0..10).map(|i| format!("row{}", i)).collect();
        for row in &inputs {
            sampler
                .process(row.as_bytes(), &mut output, &mut rng)
                .unwrap();
        }
        sampler.finalize(&mut output, &mut rng, false).unwrap();

        let out_str = String::from_utf8(output).unwrap();
        let lines: Vec<&str> = out_str
            .trim()
            .split('\n')
            .filter(|s| !s.is_empty())
            .collect();
        assert_eq!(lines.len(), 10);
    }

    #[test]
    fn test_bernoulli_sampler_prob_zero() {
        let mut rng = get_rng(42);
        let mut output = Vec::new();
        let mut sampler = BernoulliSampler {
            prob: 0.0,
            print_random: false,
            // If p=0, skip_counter should prevent any selection.
            // We set it high initially.
            skip_counter: usize::MAX,
        };

        let inputs: Vec<String> = (0..10).map(|i| format!("row{}", i)).collect();
        for row in &inputs {
            sampler
                .process(row.as_bytes(), &mut output, &mut rng)
                .unwrap();
        }

        let out_str = String::from_utf8(output).unwrap();
        assert!(out_str.is_empty());
    }
}
