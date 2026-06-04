use crate::libs::aggregation::ops::get_str;
use crate::libs::aggregation::{Aggregator, Calculator};
use crate::libs::tsv::record::Row;

pub struct First {
    pub field_idx: usize,
    pub first_slot: usize,
}

impl Calculator for First {
    fn update(&self, agg: &mut Aggregator, row: &dyn Row) {
        if agg.firsts[self.first_slot].is_empty() {
            agg.firsts[self.first_slot] = get_str(row, self.field_idx);
        }
    }

    fn format(&self, agg: &Aggregator) -> String {
        agg.firsts[self.first_slot].clone()
    }
}

pub struct Last {
    pub field_idx: usize,
    pub last_slot: usize,
}

impl Calculator for Last {
    fn update(&self, agg: &mut Aggregator, row: &dyn Row) {
        agg.lasts[self.last_slot] = get_str(row, self.field_idx);
    }

    fn format(&self, agg: &Aggregator) -> String {
        agg.lasts[self.last_slot].clone()
    }
}

pub struct Collapse {
    pub field_idx: usize,
    pub string_values_slot: usize,
    pub delimiter: String,
    pub exclude_missing: bool,
    pub missing_val: Option<String>,
}

impl Calculator for Collapse {
    fn update(&self, agg: &mut Aggregator, row: &dyn Row) {
        let val = get_str(row, self.field_idx);
        if self.exclude_missing && val.is_empty() {
            return;
        }
        agg.string_values[self.string_values_slot].push(val);
    }

    fn format(&self, agg: &Aggregator) -> String {
        let vals = &agg.string_values[self.string_values_slot];
        if let Some(ref replacement) = self.missing_val {
            vals.iter()
                .map(|v| if v.is_empty() { replacement.as_str() } else { v.as_str() })
                .collect::<Vec<&str>>()
                .join(&self.delimiter)
        } else {
            vals.join(&self.delimiter)
        }
    }
}

pub struct Rand {
    pub field_idx: usize,
    pub string_values_slot: usize,
}

impl Calculator for Rand {
    fn update(&self, agg: &mut Aggregator, row: &dyn Row) {
        agg.string_values[self.string_values_slot].push(get_str(row, self.field_idx));
    }

    fn format(&self, agg: &Aggregator) -> String {
        let vals = &agg.string_values[self.string_values_slot];
        if vals.is_empty() {
            "".to_string()
        } else {
            let seed = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64;
            let mut x = seed;
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            let index = (x as usize) % vals.len();
            vals[index].clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::libs::tsv::record::StrSliceRow;

    // Helper to create a dummy aggregator
    fn new_agg() -> Aggregator {
        Aggregator {
            count: 0,
            sums: vec![],
            mins: vec![],
            maxs: vec![],
            field_counts: vec![],
            sum_sqs: vec![],
            sum_logs: vec![],
            sum_invs: vec![],
            firsts: vec!["".to_string()], // One slot initialized
            lasts: vec!["".to_string()],  // One slot initialized
            string_values: vec![vec![]],  // One slot for values
            values: vec![],
            value_counts: vec![],
        }
    }

    #[test]
    fn test_first() {
        let mut agg = new_agg();
        let calc = First {
            field_idx: 0,
            first_slot: 0,
        };

        calc.update(&mut agg, &StrSliceRow { fields: &["A"] });
        calc.update(&mut agg, &StrSliceRow { fields: &["B"] });

        assert_eq!(calc.format(&agg), "A");
    }

    #[test]
    fn test_last() {
        let mut agg = new_agg();
        let calc = Last {
            field_idx: 0,
            last_slot: 0,
        };

        calc.update(&mut agg, &StrSliceRow { fields: &["A"] });
        calc.update(&mut agg, &StrSliceRow { fields: &["B"] });

        assert_eq!(calc.format(&agg), "B");
    }

    #[test]
    fn test_collapse() {
        let mut agg = new_agg();
        let calc = Collapse {
            field_idx: 0,
            string_values_slot: 0,
            delimiter: ",".to_string(),
            exclude_missing: false,
            missing_val: None,
        };

        calc.update(&mut agg, &StrSliceRow { fields: &["A"] });
        calc.update(&mut agg, &StrSliceRow { fields: &["B"] });

        assert_eq!(calc.format(&agg), "A,B");
    }

    #[test]
    fn test_collapse_exclude_missing() {
        let mut agg = new_agg();
        let calc = Collapse {
            field_idx: 0,
            string_values_slot: 0,
            delimiter: ",".to_string(),
            exclude_missing: true,
            missing_val: None,
        };

        calc.update(&mut agg, &StrSliceRow { fields: &["A"] });
        calc.update(&mut agg, &StrSliceRow { fields: &[""] });
        calc.update(&mut agg, &StrSliceRow { fields: &["B"] });

        assert_eq!(calc.format(&agg), "A,B");
    }

    #[test]
    fn test_collapse_replace_missing() {
        let mut agg = new_agg();
        let calc = Collapse {
            field_idx: 0,
            string_values_slot: 0,
            delimiter: ",".to_string(),
            exclude_missing: false,
            missing_val: Some("NA".to_string()),
        };

        calc.update(&mut agg, &StrSliceRow { fields: &["A"] });
        calc.update(&mut agg, &StrSliceRow { fields: &[""] });
        calc.update(&mut agg, &StrSliceRow { fields: &["B"] });

        assert_eq!(calc.format(&agg), "A,NA,B");
    }

    #[test]
    fn test_rand() {
        let mut agg = new_agg();
        let calc = Rand {
            field_idx: 0,
            string_values_slot: 0,
        };

        calc.update(&mut agg, &StrSliceRow { fields: &["A"] });

        // Only one value, so it must be "A"
        assert_eq!(calc.format(&agg), "A");

        calc.update(&mut agg, &StrSliceRow { fields: &["B"] });
        let res = calc.format(&agg);
        assert!(res == "A" || res == "B");
    }

    #[test]
    fn test_empty_rand() {
        let agg = new_agg();
        let calc = Rand {
            field_idx: 0,
            string_values_slot: 0,
        };
        assert_eq!(calc.format(&agg), "");
    }
}
