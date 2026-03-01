use super::traits::{write_with_optional_random, Sampler, INV_U64_MAX_PLUS_1};
use rapidhash::RapidRng;
use std::io::Write;

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
