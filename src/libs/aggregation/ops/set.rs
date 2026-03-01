use crate::libs::aggregation::{Aggregator, Calculator};
use crate::libs::aggregation::ops::get_str;
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

pub struct Unique {
    pub field_idx: usize,
    pub value_counts_slot: usize,
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
                .join(",")
        }
    }
}
