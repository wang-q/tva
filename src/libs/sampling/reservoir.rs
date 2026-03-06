use super::traits::{
    write_with_optional_random, Sampler, WeightedItem, INV_U64_MAX_PLUS_1,
};
use rapidhash::RapidRng;
use std::cmp::{Ordering, Reverse};
use std::collections::BinaryHeap;
use std::io::Write;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reservoir_sampler_basic() {
        let mut sampler = ReservoirSampler {
            k: 2,
            reservoir: Vec::new(),
            count: 0,
        };
        let mut rng = RapidRng::new(123);
        let mut writer = Vec::new();

        // Add 3 items
        sampler.process(b"1", &mut writer, &mut rng).unwrap();
        sampler.process(b"2", &mut writer, &mut rng).unwrap();
        sampler.process(b"3", &mut writer, &mut rng).unwrap();

        assert_eq!(sampler.count, 3);
        assert_eq!(sampler.reservoir.len(), 2);

        sampler.finalize(&mut writer, &mut rng, false).unwrap();
        let s = String::from_utf8(writer).unwrap();
        let lines: Vec<&str> = s.lines().collect();
        assert_eq!(lines.len(), 2);
    }

    #[test]
    fn test_weighted_reservoir_sampler_basic() {
        let mut sampler = WeightedReservoirSampler {
            k: 1,
            weight_field_idx: 2,
            heap: BinaryHeap::new(),
            inorder: false,
            current_index: 0,
        };
        let mut rng = RapidRng::new(123);
        let mut writer = Vec::new();

        // 100 with weight 1, 200 with weight 100
        sampler.process(b"100\t1", &mut writer, &mut rng).unwrap();
        sampler.process(b"200\t100", &mut writer, &mut rng).unwrap();

        sampler.finalize(&mut writer, &mut rng, false).unwrap();
        // High probability that 200 is selected due to weight 100 vs 1
        assert_eq!(writer, b"200\t100\n");
    }

    #[test]
    fn test_weighted_reservoir_sampler_invalid_weight() {
        let mut sampler = WeightedReservoirSampler {
            k: 1,
            weight_field_idx: 2,
            heap: BinaryHeap::new(),
            inorder: false,
            current_index: 0,
        };
        let mut rng = RapidRng::new(123);
        let mut writer = Vec::new();

        let res = sampler.process(b"100\tbad", &mut writer, &mut rng);
        assert!(res.is_err());
    }

    #[test]
    fn test_weighted_reservoir_sampler_missing_weight() {
        let mut sampler = WeightedReservoirSampler {
            k: 1,
            weight_field_idx: 3,
            heap: BinaryHeap::new(),
            inorder: false,
            current_index: 0,
        };
        let mut rng = RapidRng::new(123);
        let mut writer = Vec::new();

        let res = sampler.process(b"100\t1", &mut writer, &mut rng);
        assert!(res.is_err());
    }
}

pub struct WeightedReservoirSampler {
    pub k: usize,
    pub weight_field_idx: usize,
    // Use a Min-Heap (via Reverse) to store the top-K items with largest keys.
    // The root of the heap is the item with the smallest key among the top-K.
    pub heap: BinaryHeap<Reverse<WeightedItem>>,
    pub inorder: bool,
    pub current_index: usize,
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
                                original_index: self.current_index,
                            }));
                        } else {
                            // Replace the smallest key in heap if new key is larger
                            if let Some(Reverse(min_item)) = self.heap.peek() {
                                if key > min_item.key {
                                    self.heap.pop();
                                    self.heap.push(Reverse(WeightedItem {
                                        key,
                                        record: record.to_vec(),
                                        original_index: self.current_index,
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
        self.current_index += 1;
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

        // Extract items
        let mut items: Vec<WeightedItem> =
            self.heap.drain().map(|Reverse(item)| item).collect();

        if self.inorder {
            items.sort_by_key(|item| item.original_index);
        } else {
            // Sort by key descending (highest probability first)
            items.sort_by(|a, b| b.key.partial_cmp(&a.key).unwrap_or(Ordering::Equal));
        }

        for item in items {
            write_with_optional_random(
                writer,
                &item.record,
                rng,
                print_random,
                Some(item.key),
            )?;
        }
        Ok(())
    }
}
