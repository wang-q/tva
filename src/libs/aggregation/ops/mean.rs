use crate::libs::aggregation::math;
use crate::libs::aggregation::ops::parse_float;
use crate::libs::aggregation::{Aggregator, Calculator};
use crate::libs::number::format_float;
use crate::libs::tsv::record::Row;

pub struct Mean {
    pub field_idx: usize,
    pub sum_slot: usize,
    pub count_slot: usize,
    pub precision: Option<usize>,
    pub missing_val: Option<f64>,
    pub exclude_missing: bool,
}

impl Calculator for Mean {
    fn update(&self, agg: &mut Aggregator, row: &dyn Row) {
        if let Some(val) =
            parse_float(row, self.field_idx, self.missing_val, self.exclude_missing)
        {
            agg.sums[self.sum_slot] += val;
            agg.field_counts[self.count_slot] += 1;
        }
    }

    fn format(&self, agg: &Aggregator) -> String {
        let count = agg.field_counts[self.count_slot];
        let sum = agg.sums[self.sum_slot];
        let res = math::mean(sum, count);
        format_float(res, self.precision)
    }
}

pub struct GeoMean {
    pub field_idx: usize,
    pub sum_log_slot: usize,
    pub count_slot: usize,
    pub precision: Option<usize>,
    pub missing_val: Option<f64>,
    pub exclude_missing: bool,
}

impl Calculator for GeoMean {
    fn update(&self, agg: &mut Aggregator, row: &dyn Row) {
        if let Some(val) =
            parse_float(row, self.field_idx, self.missing_val, self.exclude_missing)
        {
            if val > 0.0 {
                agg.sum_logs[self.sum_log_slot] += val.ln();
                agg.field_counts[self.count_slot] += 1;
            }
        }
    }

    fn format(&self, agg: &Aggregator) -> String {
        let count = agg.field_counts[self.count_slot];
        let sum_log = agg.sum_logs[self.sum_log_slot];
        let res = math::geomean(sum_log, count);
        format_float(res, self.precision)
    }
}

pub struct HarmMean {
    pub field_idx: usize,
    pub sum_inv_slot: usize,
    pub count_slot: usize,
    pub precision: Option<usize>,
    pub missing_val: Option<f64>,
    pub exclude_missing: bool,
}

impl Calculator for HarmMean {
    fn update(&self, agg: &mut Aggregator, row: &dyn Row) {
        if let Some(val) =
            parse_float(row, self.field_idx, self.missing_val, self.exclude_missing)
        {
            if val != 0.0 {
                agg.sum_invs[self.sum_inv_slot] += 1.0 / val;
                agg.field_counts[self.count_slot] += 1;
            }
        }
    }

    fn format(&self, agg: &Aggregator) -> String {
        let count = agg.field_counts[self.count_slot];
        let sum_inv = agg.sum_invs[self.sum_inv_slot];
        let res = math::harmmean(sum_inv, count);
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
            sum_sqs: vec![],
            sum_logs: vec![0.0],
            sum_invs: vec![0.0],
            firsts: vec![],
            lasts: vec![],
            string_values: vec![],
            values: vec![],
            value_counts: vec![],
        }
    }

    #[test]
    fn test_mean() {
        let mut agg = new_agg();
        let calc = Mean {
            field_idx: 0,
            sum_slot: 0,
            count_slot: 0,
            precision: None,
            missing_val: None,
            exclude_missing: false,
        };

        // 10, 20. Mean = 15.
        calc.update(&mut agg, &StrSliceRow { fields: &["10"] });
        calc.update(&mut agg, &StrSliceRow { fields: &["20"] });

        assert_eq!(agg.sums[0], 30.0);
        assert_eq!(agg.field_counts[0], 2);
        assert_eq!(calc.format(&agg), "15");
    }

    #[test]
    fn test_geo_mean() {
        let mut agg = new_agg();
        let calc = GeoMean {
            field_idx: 0,
            sum_log_slot: 0,
            count_slot: 0,
            precision: None,
            missing_val: None,
            exclude_missing: false,
        };

        // 2, 8. GeoMean = sqrt(2*8) = 4.
        calc.update(&mut agg, &StrSliceRow { fields: &["2"] });
        calc.update(&mut agg, &StrSliceRow { fields: &["8"] });

        assert_eq!(calc.format(&agg), "4");
    }

    #[test]
    fn test_harm_mean() {
        let mut agg = new_agg();
        let calc = HarmMean {
            field_idx: 0,
            sum_inv_slot: 0,
            count_slot: 0,
            precision: None,
            missing_val: None,
            exclude_missing: false,
        };

        // 2, 4. HarmMean = 2 / (1/2 + 1/4) = 2 / (0.75) = 2.666...
        calc.update(&mut agg, &StrSliceRow { fields: &["2"] });
        calc.update(&mut agg, &StrSliceRow { fields: &["4"] });

        let res = calc.format(&agg);
        // Default precision format might vary, but should be close
        assert!(res.starts_with("2.666"));
    }
}
