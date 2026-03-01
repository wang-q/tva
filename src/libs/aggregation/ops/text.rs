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
}

impl Calculator for Collapse {
    fn update(&self, agg: &mut Aggregator, row: &dyn Row) {
        agg.string_values[self.string_values_slot].push(get_str(row, self.field_idx));
    }

    fn format(&self, agg: &Aggregator) -> String {
        agg.string_values[self.string_values_slot].join(",")
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
