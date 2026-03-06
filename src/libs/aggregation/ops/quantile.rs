use crate::libs::aggregation::math;
use crate::libs::aggregation::ops::parse_float;
use crate::libs::aggregation::{Aggregator, Calculator};
use crate::libs::number::format_float;
use crate::libs::tsv::record::Row;

pub struct Quantile {
    pub field_idx: usize,
    pub values_slot: usize,
    pub precision: Option<usize>,
    pub probability: f64,
    pub missing_val: Option<f64>,
    pub exclude_missing: bool,
}

impl Calculator for Quantile {
    fn update(&self, agg: &mut Aggregator, row: &dyn Row) {
        if let Some(val) =
            parse_float(row, self.field_idx, self.missing_val, self.exclude_missing)
        {
            agg.values[self.values_slot].push(val);
        }
    }

    fn format(&self, agg: &Aggregator) -> String {
        let vals = &agg.values[self.values_slot];
        if !vals.is_empty() {
            let mut sorted_vals = vals.clone();
            sorted_vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let res = math::quantile(&sorted_vals, self.probability);
            format_float(res, self.precision)
        } else {
            "nan".to_string()
        }
    }
}

pub struct Median {
    pub field_idx: usize,
    pub values_slot: usize,
    pub precision: Option<usize>,
    pub missing_val: Option<f64>,
    pub exclude_missing: bool,
}

impl Calculator for Median {
    fn update(&self, agg: &mut Aggregator, row: &dyn Row) {
        if let Some(val) =
            parse_float(row, self.field_idx, self.missing_val, self.exclude_missing)
        {
            agg.values[self.values_slot].push(val);
        }
    }

    fn format(&self, agg: &Aggregator) -> String {
        let vals = &agg.values[self.values_slot];
        if !vals.is_empty() {
            let mut sorted_vals = vals.clone();
            sorted_vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let res = math::quantile(&sorted_vals, 0.5);
            format_float(res, self.precision)
        } else {
            "nan".to_string()
        }
    }
}

pub struct Q1 {
    pub field_idx: usize,
    pub values_slot: usize,
    pub precision: Option<usize>,
    pub missing_val: Option<f64>,
    pub exclude_missing: bool,
}

impl Calculator for Q1 {
    fn update(&self, agg: &mut Aggregator, row: &dyn Row) {
        if let Some(val) =
            parse_float(row, self.field_idx, self.missing_val, self.exclude_missing)
        {
            agg.values[self.values_slot].push(val);
        }
    }

    fn format(&self, agg: &Aggregator) -> String {
        let vals = &agg.values[self.values_slot];
        if !vals.is_empty() {
            let mut sorted_vals = vals.clone();
            sorted_vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let res = math::quantile(&sorted_vals, 0.25);
            format_float(res, self.precision)
        } else {
            "nan".to_string()
        }
    }
}

pub struct Q3 {
    pub field_idx: usize,
    pub values_slot: usize,
    pub precision: Option<usize>,
    pub missing_val: Option<f64>,
    pub exclude_missing: bool,
}

impl Calculator for Q3 {
    fn update(&self, agg: &mut Aggregator, row: &dyn Row) {
        if let Some(val) =
            parse_float(row, self.field_idx, self.missing_val, self.exclude_missing)
        {
            agg.values[self.values_slot].push(val);
        }
    }

    fn format(&self, agg: &Aggregator) -> String {
        let vals = &agg.values[self.values_slot];
        if !vals.is_empty() {
            let mut sorted_vals = vals.clone();
            sorted_vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let res = math::quantile(&sorted_vals, 0.75);
            format_float(res, self.precision)
        } else {
            "nan".to_string()
        }
    }
}

pub struct IQR {
    pub field_idx: usize,
    pub values_slot: usize,
    pub precision: Option<usize>,
    pub missing_val: Option<f64>,
    pub exclude_missing: bool,
}

impl Calculator for IQR {
    fn update(&self, agg: &mut Aggregator, row: &dyn Row) {
        if let Some(val) =
            parse_float(row, self.field_idx, self.missing_val, self.exclude_missing)
        {
            agg.values[self.values_slot].push(val);
        }
    }

    fn format(&self, agg: &Aggregator) -> String {
        let vals = &agg.values[self.values_slot];
        if !vals.is_empty() {
            let mut sorted_vals = vals.clone();
            sorted_vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let q1 = math::quantile(&sorted_vals, 0.25);
            let q3 = math::quantile(&sorted_vals, 0.75);
            let res = q3 - q1;
            format_float(res, self.precision)
        } else {
            "nan".to_string()
        }
    }
}

pub struct Mad {
    pub field_idx: usize,
    pub values_slot: usize,
    pub precision: Option<usize>,
    pub missing_val: Option<f64>,
    pub exclude_missing: bool,
}

impl Calculator for Mad {
    fn update(&self, agg: &mut Aggregator, row: &dyn Row) {
        if let Some(val) =
            parse_float(row, self.field_idx, self.missing_val, self.exclude_missing)
        {
            agg.values[self.values_slot].push(val);
        }
    }

    fn format(&self, agg: &Aggregator) -> String {
        let vals = &agg.values[self.values_slot];
        if !vals.is_empty() {
            let mut sorted_vals = vals.clone();
            sorted_vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let res = math::mad(&sorted_vals);
            format_float(res, self.precision)
        } else {
            "nan".to_string()
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
            firsts: vec![],
            lasts: vec![],
            string_values: vec![],
            values: vec![vec![]], // One slot for values
            value_counts: vec![],
        }
    }

    #[test]
    fn test_median() {
        let mut agg = new_agg();
        let calc = Median {
            field_idx: 0,
            values_slot: 0,
            precision: None,
            missing_val: None,
            exclude_missing: false,
        };

        // 1, 3, 5 -> Median 3
        calc.update(&mut agg, &StrSliceRow { fields: &["1"] });
        calc.update(&mut agg, &StrSliceRow { fields: &["5"] });
        calc.update(&mut agg, &StrSliceRow { fields: &["3"] });

        assert_eq!(calc.format(&agg), "3");
    }

    #[test]
    fn test_quantile() {
        let mut agg = new_agg();
        let calc = Quantile {
            field_idx: 0,
            values_slot: 0,
            precision: None,
            probability: 0.9,
            missing_val: None,
            exclude_missing: false,
        };

        // 0..10. 0.9 quantile -> 9
        for i in 0..=10 {
            calc.update(
                &mut agg,
                &StrSliceRow {
                    fields: &[i.to_string().as_str()],
                },
            );
        }

        assert_eq!(calc.format(&agg), "9");
    }

    #[test]
    fn test_iqr() {
        let mut agg = new_agg();
        let calc = IQR {
            field_idx: 0,
            values_slot: 0,
            precision: None,
            missing_val: None,
            exclude_missing: false,
        };

        // 1, 2, 3, 4, 5, 6, 7, 8
        // Q1 (25%) = 2.75? Or depending on implementation.
        // math::quantile usually does linear interpolation or similar.
        // Let's use simple case: 0, 10, 20, 30, 40.
        // Q1(25%) = 10. Q3(75%) = 30. IQR = 20.

        calc.update(&mut agg, &StrSliceRow { fields: &["0"] });
        calc.update(&mut agg, &StrSliceRow { fields: &["10"] });
        calc.update(&mut agg, &StrSliceRow { fields: &["20"] });
        calc.update(&mut agg, &StrSliceRow { fields: &["30"] });
        calc.update(&mut agg, &StrSliceRow { fields: &["40"] });

        assert_eq!(calc.format(&agg), "20");
    }

    #[test]
    fn test_mad() {
        let mut agg = new_agg();
        let calc = Mad {
            field_idx: 0,
            values_slot: 0,
            precision: None,
            missing_val: None,
            exclude_missing: false,
        };

        // 1, 1, 2, 2, 4, 6, 9. Median = 2.
        // Abs diffs: 1, 1, 0, 0, 2, 4, 7.
        // Sorted diffs: 0, 0, 1, 1, 2, 4, 7.
        // Median diff = 1.
        // MAD usually includes a scaling factor (1.4826) to estimate sigma for normal distribution.
        // math::mad implementation seems to apply this factor.
        // 1 * 1.4826 = 1.4826.

        calc.update(&mut agg, &StrSliceRow { fields: &["1"] });
        calc.update(&mut agg, &StrSliceRow { fields: &["1"] });
        calc.update(&mut agg, &StrSliceRow { fields: &["2"] });
        calc.update(&mut agg, &StrSliceRow { fields: &["2"] });
        calc.update(&mut agg, &StrSliceRow { fields: &["4"] });
        calc.update(&mut agg, &StrSliceRow { fields: &["6"] });
        calc.update(&mut agg, &StrSliceRow { fields: &["9"] });

        // Check if result starts with 1.48
        let res = calc.format(&agg);
        assert!(res.starts_with("1.48"));
    }

    #[test]
    fn test_empty_values() {
        let agg = new_agg();
        let calc = Median {
            field_idx: 0,
            values_slot: 0,
            precision: None,
            missing_val: None,
            exclude_missing: false,
        };
        assert_eq!(calc.format(&agg), "nan");
    }
}
