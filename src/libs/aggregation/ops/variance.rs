use crate::libs::aggregation::math;
use crate::libs::aggregation::ops::parse_float;
use crate::libs::aggregation::{Aggregator, Calculator};
use crate::libs::number::format_float;
use crate::libs::tsv::record::Row;

pub struct Variance {
    pub field_idx: usize,
    pub sum_slot: usize,
    pub sum_sq_slot: usize,
    pub count_slot: usize,
    pub precision: Option<usize>,
    pub missing_val: Option<f64>,
    pub exclude_missing: bool,
}

impl Calculator for Variance {
    fn update(&self, agg: &mut Aggregator, row: &dyn Row) {
        if let Some(val) =
            parse_float(row, self.field_idx, self.missing_val, self.exclude_missing)
        {
            agg.sums[self.sum_slot] += val;
            agg.sum_sqs[self.sum_sq_slot] += val * val;
            agg.field_counts[self.count_slot] += 1;
        }
    }

    fn format(&self, agg: &Aggregator) -> String {
        let count = agg.field_counts[self.count_slot];
        let sum = agg.sums[self.sum_slot];
        let sum_sq = agg.sum_sqs[self.sum_sq_slot];
        let res = math::variance(sum_sq, sum, count);
        format_float(res, self.precision)
    }
}

pub struct Stdev {
    pub field_idx: usize,
    pub sum_slot: usize,
    pub sum_sq_slot: usize,
    pub count_slot: usize,
    pub precision: Option<usize>,
    pub missing_val: Option<f64>,
    pub exclude_missing: bool,
}

impl Calculator for Stdev {
    fn update(&self, agg: &mut Aggregator, row: &dyn Row) {
        if let Some(val) =
            parse_float(row, self.field_idx, self.missing_val, self.exclude_missing)
        {
            agg.sums[self.sum_slot] += val;
            agg.sum_sqs[self.sum_sq_slot] += val * val;
            agg.field_counts[self.count_slot] += 1;
        }
    }

    fn format(&self, agg: &Aggregator) -> String {
        let count = agg.field_counts[self.count_slot];
        let sum = agg.sums[self.sum_slot];
        let sum_sq = agg.sum_sqs[self.sum_sq_slot];
        let res = math::stdev(sum_sq, sum, count);
        format_float(res, self.precision)
    }
}

pub struct CV {
    pub field_idx: usize,
    pub sum_slot: usize,
    pub sum_sq_slot: usize,
    pub count_slot: usize,
    pub precision: Option<usize>,
    pub missing_val: Option<f64>,
    pub exclude_missing: bool,
}

impl Calculator for CV {
    fn update(&self, agg: &mut Aggregator, row: &dyn Row) {
        if let Some(val) =
            parse_float(row, self.field_idx, self.missing_val, self.exclude_missing)
        {
            agg.sums[self.sum_slot] += val;
            agg.sum_sqs[self.sum_sq_slot] += val * val;
            agg.field_counts[self.count_slot] += 1;
        }
    }

    fn format(&self, agg: &Aggregator) -> String {
        let count = agg.field_counts[self.count_slot];
        let sum = agg.sums[self.sum_slot];
        let sum_sq = agg.sum_sqs[self.sum_sq_slot];
        let res = math::cv(sum_sq, sum, count);
        format_float(res, self.precision)
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
            mins: vec![],
            maxs: vec![],
            field_counts: vec![0],
            sum_sqs: vec![0.0],
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
    fn test_variance() {
        let mut agg = new_agg();
        let calc = Variance {
            field_idx: 0,
            sum_slot: 0,
            sum_sq_slot: 0,
            count_slot: 0,
            precision: None,
            missing_val: None,
            exclude_missing: false,
        };

        // 2, 4, 6. Mean=4. Var = ((2-4)^2 + (4-4)^2 + (6-4)^2) / (3-1) = (4+0+4)/2 = 4.
        calc.update(&mut agg, &StrSliceRow { fields: &["2"] });
        calc.update(&mut agg, &StrSliceRow { fields: &["4"] });
        calc.update(&mut agg, &StrSliceRow { fields: &["6"] });

        assert_eq!(calc.format(&agg), "4");
    }

    #[test]
    fn test_stdev() {
        let mut agg = new_agg();
        let calc = Stdev {
            field_idx: 0,
            sum_slot: 0,
            sum_sq_slot: 0,
            count_slot: 0,
            precision: None,
            missing_val: None,
            exclude_missing: false,
        };

        // 2, 4, 6. Var=4. Stdev=2.
        calc.update(&mut agg, &StrSliceRow { fields: &["2"] });
        calc.update(&mut agg, &StrSliceRow { fields: &["4"] });
        calc.update(&mut agg, &StrSliceRow { fields: &["6"] });

        assert_eq!(calc.format(&agg), "2");
    }

    #[test]
    fn test_cv() {
        let mut agg = new_agg();
        let calc = CV {
            field_idx: 0,
            sum_slot: 0,
            sum_sq_slot: 0,
            count_slot: 0,
            precision: None,
            missing_val: None,
            exclude_missing: false,
        };

        // 2, 4, 6. Mean=4. Stdev=2. CV = 2/4 = 0.5.
        calc.update(&mut agg, &StrSliceRow { fields: &["2"] });
        calc.update(&mut agg, &StrSliceRow { fields: &["4"] });
        calc.update(&mut agg, &StrSliceRow { fields: &["6"] });

        assert_eq!(calc.format(&agg), "0.5");
    }
}
