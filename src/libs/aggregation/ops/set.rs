use crate::libs::aggregation::ops::get_str;
use crate::libs::aggregation::{Aggregator, Calculator};
use crate::libs::tsv::record::Row;

pub struct NUnique {
    pub field_idx: usize,
    pub value_counts_slot: usize,
}

impl Calculator for NUnique {
    fn update(&self, agg: &mut Aggregator, row: &dyn Row) {
        let val = get_str(row, self.field_idx);
        *agg.value_counts[self.value_counts_slot]
            .entry(val)
            .or_insert(0) += 1;
    }

    fn format(&self, agg: &Aggregator) -> String {
        agg.value_counts[self.value_counts_slot].len().to_string()
    }
}

pub struct Mode {
    pub field_idx: usize,
    pub value_counts_slot: usize,
}

impl Calculator for Mode {
    fn update(&self, agg: &mut Aggregator, row: &dyn Row) {
        let val = get_str(row, self.field_idx);
        *agg.value_counts[self.value_counts_slot]
            .entry(val)
            .or_insert(0) += 1;
    }

    fn format(&self, agg: &Aggregator) -> String {
        let counts = &agg.value_counts[self.value_counts_slot];
        if counts.is_empty() {
            "".to_string()
        } else {
            let mut count_vec: Vec<(&String, &usize)> = counts.iter().collect();
            count_vec.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));
            count_vec[0].0.clone()
        }
    }
}

pub struct ModeCount {
    pub field_idx: usize,
    pub value_counts_slot: usize,
}

impl Calculator for ModeCount {
    fn update(&self, agg: &mut Aggregator, row: &dyn Row) {
        let val = get_str(row, self.field_idx);
        *agg.value_counts[self.value_counts_slot]
            .entry(val)
            .or_insert(0) += 1;
    }

    fn format(&self, agg: &Aggregator) -> String {
        let counts = &agg.value_counts[self.value_counts_slot];
        if counts.is_empty() {
            "0".to_string()
        } else {
            let mut count_vec: Vec<(&String, &usize)> = counts.iter().collect();
            count_vec.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));
            count_vec[0].1.to_string()
        }
    }
}

pub struct Unique {
    pub field_idx: usize,
    pub value_counts_slot: usize,
    pub delimiter: String,
}

impl Calculator for Unique {
    fn update(&self, agg: &mut Aggregator, row: &dyn Row) {
        let val = get_str(row, self.field_idx);
        *agg.value_counts[self.value_counts_slot]
            .entry(val)
            .or_insert(0) += 1;
    }

    fn format(&self, agg: &Aggregator) -> String {
        let counts = &agg.value_counts[self.value_counts_slot];
        if counts.is_empty() {
            "".to_string()
        } else {
            let mut keys: Vec<&String> = counts.keys().collect();
            keys.sort();
            keys.into_iter()
                .map(|s| s.as_str())
                .collect::<Vec<&str>>()
                .join(&self.delimiter)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::libs::tsv::record::StrSliceRow;
    use std::collections::HashMap;

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
            firsts: vec![],
            lasts: vec![],
            string_values: vec![],
            values: vec![],
            value_counts: vec![HashMap::new()], // One slot
        }
    }

    #[test]
    fn test_nunique() {
        let mut agg = new_agg();
        let calc = NUnique {
            field_idx: 0,
            value_counts_slot: 0,
        };

        calc.update(&mut agg, &StrSliceRow { fields: &["A"] });
        calc.update(&mut agg, &StrSliceRow { fields: &["B"] });
        calc.update(&mut agg, &StrSliceRow { fields: &["A"] });

        assert_eq!(calc.format(&agg), "2");
    }

    #[test]
    fn test_mode() {
        let mut agg = new_agg();
        let calc = Mode {
            field_idx: 0,
            value_counts_slot: 0,
        };

        calc.update(&mut agg, &StrSliceRow { fields: &["A"] });
        calc.update(&mut agg, &StrSliceRow { fields: &["B"] });
        calc.update(&mut agg, &StrSliceRow { fields: &["B"] });

        assert_eq!(calc.format(&agg), "B");
    }

    #[test]
    fn test_mode_count() {
        let mut agg = new_agg();
        let calc = ModeCount {
            field_idx: 0,
            value_counts_slot: 0,
        };

        calc.update(&mut agg, &StrSliceRow { fields: &["A"] });
        calc.update(&mut agg, &StrSliceRow { fields: &["B"] });
        calc.update(&mut agg, &StrSliceRow { fields: &["B"] });

        assert_eq!(calc.format(&agg), "2");
    }

    #[test]
    fn test_unique() {
        let mut agg = new_agg();
        let calc = Unique {
            field_idx: 0,
            value_counts_slot: 0,
            delimiter: ",".to_string(),
        };

        calc.update(&mut agg, &StrSliceRow { fields: &["A"] });
        calc.update(&mut agg, &StrSliceRow { fields: &["C"] });
        calc.update(&mut agg, &StrSliceRow { fields: &["B"] });
        calc.update(&mut agg, &StrSliceRow { fields: &["A"] });

        // Output should be sorted: A,B,C
        assert_eq!(calc.format(&agg), "A,B,C");
    }

    #[test]
    fn test_empty_mode() {
        let agg = new_agg();
        let calc = Mode {
            field_idx: 0,
            value_counts_slot: 0,
        };
        assert_eq!(calc.format(&agg), "");

        let calc_count = ModeCount {
            field_idx: 0,
            value_counts_slot: 0,
        };
        assert_eq!(calc_count.format(&agg), "0");
    }
}
