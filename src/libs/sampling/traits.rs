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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weighted_item_ordering() {
        let item1 = WeightedItem {
            key: 1.0,
            record: vec![],
            original_index: 0,
        };
        let item2 = WeightedItem {
            key: 2.0,
            record: vec![],
            original_index: 1,
        };
        let item3 = WeightedItem {
            key: 1.0,
            record: vec![],
            original_index: 2,
        };

        assert!(item1 < item2);
        assert!(item2 > item1);
        assert!(item1 == item3);
        assert!(item1 <= item3);
        assert!(item1 >= item3);

        // Test Ord explicitly to cover cmp() (L44-47)
        assert_eq!(item1.cmp(&item2), Ordering::Less);
        assert_eq!(item2.cmp(&item1), Ordering::Greater);
        assert_eq!(item1.cmp(&item3), Ordering::Equal);

        // Test NaN handling in cmp (fallback to Equal)
        // Although in standard f64 PartialOrd, NaN != NaN.
        // partial_cmp(NaN, NaN) returns None.
        // unwrap_or(Equal) should return Equal.
        let item_nan1 = WeightedItem {
            key: f64::NAN,
            record: vec![],
            original_index: 3,
        };
        let item_nan2 = WeightedItem {
            key: f64::NAN,
            record: vec![],
            original_index: 4,
        };
        assert_eq!(item_nan1.cmp(&item_nan2), Ordering::Equal);

        // Compare normal with NaN
        // partial_cmp(1.0, NaN) -> None -> Equal
        assert_eq!(item1.cmp(&item_nan1), Ordering::Equal);
    }

    #[test]
    fn test_write_with_optional_random() {
        let mut writer = Vec::new();
        let mut rng = RapidRng::new(42);
        let row = b"test";

        // Without random
        write_with_optional_random(&mut writer, row, &mut rng, false, None).unwrap();
        assert_eq!(writer, b"test\n");

        // With random, provided value
        writer.clear();
        write_with_optional_random(&mut writer, row, &mut rng, true, Some(0.123))
            .unwrap();
        let s = String::from_utf8(writer.clone()).unwrap();
        assert!(s.starts_with("0.123\ttest\n"));

        // With random, generated value
        writer.clear();
        write_with_optional_random(&mut writer, row, &mut rng, true, None).unwrap();
        let s = String::from_utf8(writer).unwrap();
        assert!(s.ends_with("\ttest\n"));
        // Check if starts with a number
        let parts: Vec<&str> = s.split('\t').collect();
        assert_eq!(parts.len(), 2);
        assert!(parts[0].parse::<f64>().is_ok());
    }
}
