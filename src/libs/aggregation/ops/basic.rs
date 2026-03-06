use crate::libs::aggregation::math;
use crate::libs::aggregation::ops::parse_float;
use crate::libs::aggregation::{Aggregator, Calculator};
use crate::libs::number::format_float;
use crate::libs::tsv::record::Row;

pub struct Count;

impl Calculator for Count {
    fn update(&self, agg: &mut Aggregator, _row: &dyn Row) {
        agg.count += 1;
    }

    fn format(&self, _agg: &Aggregator) -> String {
        _agg.count.to_string()
    }
}

pub struct Sum {
    pub field_idx: usize,
    pub sum_slot: usize,
    pub precision: Option<usize>,
    pub missing_val: Option<f64>,
    pub exclude_missing: bool,
}

impl Calculator for Sum {
    fn update(&self, agg: &mut Aggregator, row: &dyn Row) {
        if let Some(val) =
            parse_float(row, self.field_idx, self.missing_val, self.exclude_missing)
        {
            agg.sums[self.sum_slot] += val;
        }
    }

    fn format(&self, agg: &Aggregator) -> String {
        format_float(agg.sums[self.sum_slot], self.precision)
    }
}

pub struct Min {
    pub field_idx: usize,
    pub min_slot: usize,
    pub precision: Option<usize>,
    pub missing_val: Option<f64>,
    pub exclude_missing: bool,
}

impl Calculator for Min {
    fn update(&self, agg: &mut Aggregator, row: &dyn Row) {
        if let Some(val) =
            parse_float(row, self.field_idx, self.missing_val, self.exclude_missing)
        {
            if val < agg.mins[self.min_slot] {
                agg.mins[self.min_slot] = val;
            }
        }
    }

    fn format(&self, agg: &Aggregator) -> String {
        let val = agg.mins[self.min_slot];
        if val == f64::INFINITY {
            "nan".to_string()
        } else {
            format_float(val, self.precision)
        }
    }
}

pub struct Max {
    pub field_idx: usize,
    pub max_slot: usize,
    pub precision: Option<usize>,
    pub missing_val: Option<f64>,
    pub exclude_missing: bool,
}

impl Calculator for Max {
    fn update(&self, agg: &mut Aggregator, row: &dyn Row) {
        if let Some(val) =
            parse_float(row, self.field_idx, self.missing_val, self.exclude_missing)
        {
            if val > agg.maxs[self.max_slot] {
                agg.maxs[self.max_slot] = val;
            }
        }
    }

    fn format(&self, agg: &Aggregator) -> String {
        let val = agg.maxs[self.max_slot];
        if val == f64::NEG_INFINITY {
            "nan".to_string()
        } else {
            format_float(val, self.precision)
        }
    }
}

pub struct Range {
    pub field_idx: usize,
    pub min_slot: usize,
    pub max_slot: usize,
    pub precision: Option<usize>,
    pub missing_val: Option<f64>,
    pub exclude_missing: bool,
}

impl Calculator for Range {
    fn update(&self, agg: &mut Aggregator, row: &dyn Row) {
        if let Some(val) =
            parse_float(row, self.field_idx, self.missing_val, self.exclude_missing)
        {
            if val < agg.mins[self.min_slot] {
                agg.mins[self.min_slot] = val;
            }
            if val > agg.maxs[self.max_slot] {
                agg.maxs[self.max_slot] = val;
            }
        }
    }

    fn format(&self, agg: &Aggregator) -> String {
        let min = agg.mins[self.min_slot];
        let max = agg.maxs[self.max_slot];
        let res = math::range(min, max);
        format_float(res, self.precision)
    }
}

pub struct MissingCount {
    pub field_idx: usize,
    pub count_slot: usize,
}

impl Calculator for MissingCount {
    fn update(&self, agg: &mut Aggregator, row: &dyn Row) {
        let is_missing = match row.get_bytes(self.field_idx + 1) {
            None => true,
            Some(bytes) => bytes.is_empty(),
        };
        if is_missing {
            agg.field_counts[self.count_slot] += 1;
        }
    }

    fn format(&self, agg: &Aggregator) -> String {
        agg.field_counts[self.count_slot].to_string()
    }
}

pub struct NotMissingCount {
    pub field_idx: usize,
    pub count_slot: usize,
}

impl Calculator for NotMissingCount {
    fn update(&self, agg: &mut Aggregator, row: &dyn Row) {
        let is_missing = match row.get_bytes(self.field_idx + 1) {
            None => true,
            Some(bytes) => bytes.is_empty(),
        };
        if !is_missing {
            agg.field_counts[self.count_slot] += 1;
        }
    }

    fn format(&self, agg: &Aggregator) -> String {
        agg.field_counts[self.count_slot].to_string()
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
            sums: vec![0.0],
            mins: vec![f64::INFINITY],
            maxs: vec![f64::NEG_INFINITY],
            field_counts: vec![0],
            sum_sqs: vec![],
            sum_logs: vec![],
            sum_invs: vec![],
            firsts: vec![],
            lasts: vec![],
            string_values: vec![],
            values: vec![],
            value_counts: vec![],
        }
    }

    #[test]
    fn test_count() {
        let mut agg = new_agg();
        let calc = Count;
        let row = StrSliceRow { fields: &[] };
        calc.update(&mut agg, &row);
        assert_eq!(agg.count, 1);
        assert_eq!(calc.format(&agg), "1");
    }

    #[test]
    fn test_sum() {
        let mut agg = new_agg();
        let calc = Sum {
            field_idx: 0,
            sum_slot: 0,
            precision: None,
            missing_val: None,
            exclude_missing: false,
        };
        let row = StrSliceRow { fields: &["10.5"] };
        calc.update(&mut agg, &row);
        assert_eq!(agg.sums[0], 10.5);
        assert_eq!(calc.format(&agg), "10.5");
    }

    #[test]
    fn test_min_max() {
        let mut agg = new_agg();
        let min_calc = Min {
            field_idx: 0,
            min_slot: 0,
            precision: None,
            missing_val: None,
            exclude_missing: false,
        };
        let max_calc = Max {
            field_idx: 0,
            max_slot: 0,
            precision: None,
            missing_val: None,
            exclude_missing: false,
        };

        let row1 = StrSliceRow { fields: &["10"] };
        let row2 = StrSliceRow { fields: &["5"] };
        let row3 = StrSliceRow { fields: &["20"] };

        min_calc.update(&mut agg, &row1);
        max_calc.update(&mut agg, &row1);
        min_calc.update(&mut agg, &row2);
        max_calc.update(&mut agg, &row2);
        min_calc.update(&mut agg, &row3);
        max_calc.update(&mut agg, &row3);

        assert_eq!(agg.mins[0], 5.0);
        assert_eq!(agg.maxs[0], 20.0);
        assert_eq!(min_calc.format(&agg), "5");
        assert_eq!(max_calc.format(&agg), "20");
    }

    #[test]
    fn test_min_max_empty() {
        let agg = new_agg();
        let min_calc = Min {
            field_idx: 0,
            min_slot: 0,
            precision: None,
            missing_val: None,
            exclude_missing: false,
        };
        let max_calc = Max {
            field_idx: 0,
            max_slot: 0,
            precision: None,
            missing_val: None,
            exclude_missing: false,
        };

        assert_eq!(min_calc.format(&agg), "nan");
        assert_eq!(max_calc.format(&agg), "nan");
    }

    #[test]
    fn test_range() {
        let mut agg = new_agg();
        let calc = Range {
            field_idx: 0,
            min_slot: 0,
            max_slot: 0,
            precision: None,
            missing_val: None,
            exclude_missing: false,
        };

        calc.update(&mut agg, &StrSliceRow { fields: &["10"] });
        calc.update(&mut agg, &StrSliceRow { fields: &["30"] });

        assert_eq!(agg.mins[0], 10.0);
        assert_eq!(agg.maxs[0], 30.0);
        assert_eq!(calc.format(&agg), "20");
    }

    #[test]
    fn test_missing_counts() {
        let mut agg = new_agg();
        let m_calc = MissingCount {
            field_idx: 0,
            count_slot: 0,
        };
        let nm_calc = NotMissingCount {
            field_idx: 0,
            count_slot: 0, // Reuse same slot for test simplicity
        };

        // Note: row.get_bytes uses 1-based index internally, but calc uses 0-based field_idx.
        // The implementation does `field_idx + 1`.

        // "val" -> Present
        m_calc.update(&mut agg, &StrSliceRow { fields: &["val"] });
        assert_eq!(agg.field_counts[0], 0); // Not missing

        nm_calc.update(&mut agg, &StrSliceRow { fields: &["val"] });
        assert_eq!(agg.field_counts[0], 1); // Not missing -> increment

        // "" -> Missing (empty string)
        m_calc.update(&mut agg, &StrSliceRow { fields: &[""] });
        assert_eq!(agg.field_counts[0], 2); // Missing -> increment (now 2 because we reuse slot)

        // Missing field (index out of bounds)
        // StrSliceRow with empty fields
        m_calc.update(&mut agg, &StrSliceRow { fields: &[] });
        assert_eq!(agg.field_counts[0], 3); // Missing -> increment
    }
}
