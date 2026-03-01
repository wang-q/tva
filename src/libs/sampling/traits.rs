use rapidhash::RapidRng;
use std::cmp::Ordering;
use std::io::Write;

pub const INV_U64_MAX_PLUS_1: f64 = 1.0 / (u64::MAX as f64 + 1.0);

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

#[derive(Debug)]
pub struct WeightedItem {
    pub key: f64,
    pub record: Vec<u8>,
    pub original_index: usize,
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

pub fn write_with_optional_random<W: std::io::Write>(
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
