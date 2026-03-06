use super::traits::{write_with_optional_random, Sampler, INV_U64_MAX_PLUS_1};
use rapidhash::RapidRng;
use std::io::Write;

pub struct BernoulliSampler {
    pub prob: f64,
    pub print_random: bool,
    pub skip_counter: usize,
    pub compatibility_mode: bool,
}

impl Sampler for BernoulliSampler {
    fn process<W: Write>(
        &mut self,
        record: &[u8],
        writer: &mut W,
        rng: &mut RapidRng,
    ) -> anyhow::Result<()> {
        if self.compatibility_mode {
            // In compatibility mode, we generate a random number for *every* row
            // to ensure that if we run with a higher probability, it selects a superset.
            let r = rng.next() as f64 * INV_U64_MAX_PLUS_1;

            if r < self.prob {
                write_with_optional_random(
                    writer,
                    record,
                    rng,
                    self.print_random,
                    Some(r),
                )?;
            }
            return Ok(());
        }

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
        } else if self.prob <= 0.0 {
            self.skip_counter = usize::MAX;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bernoulli_always() {
        let mut sampler = BernoulliSampler {
            prob: 1.0,
            print_random: false,
            skip_counter: 0,
            compatibility_mode: false,
        };
        let mut rng = RapidRng::new(123);
        let mut writer = Vec::new();
        let record = b"test";

        sampler.process(record, &mut writer, &mut rng).unwrap();
        assert_eq!(writer, b"test\n");
    }

    #[test]
    fn test_bernoulli_never() {
        // Initialize skip_counter to a non-zero value to prevent the first item from being selected
        let mut sampler = BernoulliSampler {
            prob: 0.0,
            print_random: false,
            skip_counter: 10,
            compatibility_mode: false,
        };
        let mut rng = RapidRng::new(123);
        let mut writer = Vec::new();
        let record = b"test";

        // First call should be skipped
        sampler.process(record, &mut writer, &mut rng).unwrap();
        assert_eq!(writer, b"");
        assert_eq!(sampler.skip_counter, 9);

        // Reset skip_counter to 0. It should output once, then set skip_counter to MAX.
        sampler.skip_counter = 0;
        writer.clear();
        sampler.process(record, &mut writer, &mut rng).unwrap();
        assert_eq!(writer, b"test\n");

        // Verify next skip is huge (effectively never select again)
        assert_eq!(sampler.skip_counter, usize::MAX);

        // Next call should be skipped
        writer.clear();
        sampler.process(record, &mut writer, &mut rng).unwrap();
        assert_eq!(writer, b"");
    }

    #[test]
    fn test_bernoulli_compatibility() {
        let mut sampler = BernoulliSampler {
            prob: 0.5,
            print_random: false,
            skip_counter: 0,
            compatibility_mode: true,
        };
        let mut rng = RapidRng::new(123);
        let mut writer = Vec::new();

        // Should select ~50%
        let mut count = 0;
        for _ in 0..100 {
            writer.clear();
            sampler.process(b"row", &mut writer, &mut rng).unwrap();
            if !writer.is_empty() {
                count += 1;
            }
        }
        assert!(count > 30 && count < 70);
    }

    #[test]
    fn test_distinct_bernoulli_basic() {
        // Prob 1.0 -> always select
        let mut sampler = DistinctBernoulliSampler::new(1.0, vec![], false);
        let mut rng = RapidRng::new(123);
        let mut writer = Vec::new();

        sampler.process(b"test", &mut writer, &mut rng).unwrap();
        assert_eq!(writer, b"test\n");
    }

    #[test]
    fn test_distinct_bernoulli_key() {
        // Prob 0.5 -> buckets = 2.
        // Key "a" hash % 2 vs Key "b" hash % 2.
        // We need deterministic behavior.
        // rapidhash is seeded? The wrapper uses default seed.

        let mut sampler = DistinctBernoulliSampler::new(0.5, vec![1], false);
        let mut rng = RapidRng::new(123);
        let mut writer = Vec::new();

        // Same key should always have same result
        let record1 = b"key1\tdata1";

        writer.clear();
        sampler.process(record1, &mut writer, &mut rng).unwrap();
        let result1 = writer.clone();

        writer.clear();
        sampler.process(record1, &mut writer, &mut rng).unwrap();
        assert_eq!(writer, result1);
    }

    #[test]
    fn test_distinct_bernoulli_key_out_of_range() {
        let mut sampler = DistinctBernoulliSampler::new(0.5, vec![2], false);
        let mut rng = RapidRng::new(123);
        let mut writer = Vec::new();

        let record = b"col1"; // Only 1 column
        let res = sampler.process(record, &mut writer, &mut rng);
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("out of range"));
    }
}

pub struct DistinctBernoulliSampler {
    pub prob: f64,
    pub key_field_indices: Vec<usize>,
    pub print_random: bool,
    pub key_buffer: Vec<u8>,
    pub num_buckets: u64,
}

impl DistinctBernoulliSampler {
    pub fn new(prob: f64, key_field_indices: Vec<usize>, print_random: bool) -> Self {
        // Calculate number of buckets: 1 / prob
        // E.g., prob=0.1 -> 10 buckets. Key hash % 10 == 0 -> selected.
        // We use u64 for bucket calculation.
        // Clamp to avoid division by zero or overflow
        let num_buckets = if prob <= 0.0 {
            u64::MAX // effectively never select
        } else if prob >= 1.0 {
            1 // always select (mod 1 == 0)
        } else {
            (1.0 / prob).round() as u64
        };

        Self {
            prob,
            key_field_indices,
            print_random,
            key_buffer: Vec::new(),
            num_buckets,
        }
    }
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

        // Use rapidhash for O(1) hashing
        let hash = rapidhash::rapidhash(&self.key_buffer);
        let selected = (hash % self.num_buckets) == 0;

        if selected {
            // Reconstruct consistent random value if needed?
            // tsv-sample behavior:
            // "Distinct sampling: An integer, zero and up, representing a selection group."
            // "if (hasher.get % numBuckets == 0) { ... if (printRandom) outputStream.put('0'); ... }"
            // So it just prints '0' if print_random is true.
            let r = 0.0;
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
