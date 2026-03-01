use super::{OpKind, Operation, StatsProcessor};
use crate::libs::tsv::record::Row;

// Helper for testing Row trait
struct TestRow {
    fields: Vec<String>,
}
impl Row for TestRow {
    fn get_bytes(&self, idx: usize) -> Option<&[u8]> {
        if idx == 0 || idx > self.fields.len() {
            None
        } else {
            Some(self.fields[idx - 1].as_bytes())
        }
    }
}

#[test]
fn test_mean_nan() {
    let ops = vec![Operation {
        kind: OpKind::Mean,
        field_idx: Some(0),
    }];
    let processor = StatsProcessor::new(ops);
    let agg = processor.create_aggregator();
    // Mean needs count > 0 to not be nan
    let results = processor.format_results(&agg);
    assert_eq!(results[0], "nan");
}

#[test]
fn test_mad_nan_no_entry() {
    let ops = vec![Operation {
        kind: OpKind::Mad,
        field_idx: Some(0),
    }];
    let processor = StatsProcessor::new(ops);
    let agg = processor.create_aggregator();
    let results = processor.format_results(&agg);
    assert_eq!(results[0], "nan");
}

#[test]
fn test_stdev_nan() {
    let ops = vec![Operation {
        kind: OpKind::Stdev,
        field_idx: Some(0),
    }];
    let processor = StatsProcessor::new(ops);
    let mut agg = processor.create_aggregator();
    // Stdev requires count > 1
    // Manually hack state
    // We can't access private fields directly in tests unless we make them public or use pub(crate)
    // For now, let's use public update method with a single value, count=1
    let row = TestRow {
        fields: vec!["10".to_string()],
    };
    processor.update(&mut agg, &row);

    let results = processor.format_results(&agg);
    assert_eq!(results[0], "nan");
}

#[test]
fn test_min_max_nan() {
    let ops = vec![
        Operation {
            kind: OpKind::Min,
            field_idx: Some(0),
        },
        Operation {
            kind: OpKind::Max,
            field_idx: Some(0),
        },
    ];
    let processor = StatsProcessor::new(ops);
    let agg = processor.create_aggregator();
    let results = processor.format_results(&agg);
    assert_eq!(results[0], "nan");
    assert_eq!(results[1], "nan");
}

#[test]
fn test_basic_stats() {
    let ops = vec![
        Operation {
            kind: OpKind::Count,
            field_idx: None,
        },
        Operation {
            kind: OpKind::Sum,
            field_idx: Some(0),
        },
        Operation {
            kind: OpKind::Mean,
            field_idx: Some(0),
        },
        Operation {
            kind: OpKind::Min,
            field_idx: Some(0),
        },
        Operation {
            kind: OpKind::Max,
            field_idx: Some(0),
        },
    ];
    let processor = StatsProcessor::new(ops);
    let mut agg = processor.create_aggregator();

    // Data: 10, 20, 30
    let rows = vec![
        TestRow {
            fields: vec!["10".to_string()],
        },
        TestRow {
            fields: vec!["20".to_string()],
        },
        TestRow {
            fields: vec!["30".to_string()],
        },
    ];

    for row in &rows {
        processor.update(&mut agg, row);
    }

    let results = processor.format_results(&agg);
    assert_eq!(results[0], "3"); // Count
    assert_eq!(results[1], "60"); // Sum
    assert_eq!(results[2], "20"); // Mean
    assert_eq!(results[3], "10"); // Min
    assert_eq!(results[4], "30"); // Max
}

#[test]
fn test_variance_stdev_cv() {
    let ops = vec![
        Operation {
            kind: OpKind::Variance,
            field_idx: Some(0),
        },
        Operation {
            kind: OpKind::Stdev,
            field_idx: Some(0),
        },
        Operation {
            kind: OpKind::CV,
            field_idx: Some(0),
        },
    ];
    let processor = StatsProcessor::new(ops);
    let mut agg = processor.create_aggregator();

    // Data: 2, 4, 4, 4, 5, 5, 7, 9
    // Mean: 5
    // Variance: 4.571428...
    // Stdev: 2.138089...
    // CV: 0.427617...
    let data = vec![2, 4, 4, 4, 5, 5, 7, 9];
    for v in data {
        let row = TestRow {
            fields: vec![v.to_string()],
        };
        processor.update(&mut agg, &row);
    }

    let results = processor.format_results(&agg);
    let var: f64 = results[0].parse().unwrap();
    let stdev: f64 = results[1].parse().unwrap();
    let cv: f64 = results[2].parse().unwrap();

    assert!((var - 4.571428).abs() < 1e-5);
    assert!((stdev - 2.138089).abs() < 1e-5);
    assert!((cv - 0.427617).abs() < 1e-5);
}

#[test]
fn test_quantiles() {
    let ops = vec![
        Operation {
            kind: OpKind::Median,
            field_idx: Some(0),
        },
        Operation {
            kind: OpKind::Q1,
            field_idx: Some(0),
        },
        Operation {
            kind: OpKind::Q3,
            field_idx: Some(0),
        },
        Operation {
            kind: OpKind::IQR,
            field_idx: Some(0),
        },
    ];
    let processor = StatsProcessor::new(ops);
    let mut agg = processor.create_aggregator();

    // Data: 1, 2, 3, 4, 5
    // Median: 3
    // Q1: 2
    // Q3: 4
    // IQR: 2
    for i in 1..=5 {
        let row = TestRow {
            fields: vec![i.to_string()],
        };
        processor.update(&mut agg, &row);
    }

    let results = processor.format_results(&agg);
    assert_eq!(results[0], "3");
    assert_eq!(results[1], "2");
    assert_eq!(results[2], "4");
    assert_eq!(results[3], "2");
}

#[test]
fn test_geomean_harmmean() {
    let ops = vec![
        Operation {
            kind: OpKind::GeoMean,
            field_idx: Some(0),
        },
        Operation {
            kind: OpKind::HarmMean,
            field_idx: Some(0),
        },
    ];
    let processor = StatsProcessor::new(ops);
    let mut agg = processor.create_aggregator();

    // Data: 2, 8
    // GeoMean: 4
    // HarmMean: 3.2
    let data = vec![2, 8];
    for v in data {
        let row = TestRow {
            fields: vec![v.to_string()],
        };
        processor.update(&mut agg, &row);
    }

    let results = processor.format_results(&agg);
    assert_eq!(results[0], "4");
    assert_eq!(results[1], "3.2");
}

#[test]
fn test_mode_nunique() {
    let ops = vec![
        Operation {
            kind: OpKind::Mode,
            field_idx: Some(0),
        },
        Operation {
            kind: OpKind::NUnique,
            field_idx: Some(0),
        },
        Operation {
            kind: OpKind::Unique,
            field_idx: Some(0),
        },
    ];
    let processor = StatsProcessor::new(ops);
    let mut agg = processor.create_aggregator();

    // Data: A, A, B
    let data = vec!["A", "A", "B"];
    for v in data {
        let row = TestRow {
            fields: vec![v.to_string()],
        };
        processor.update(&mut agg, &row);
    }

    let results = processor.format_results(&agg);
    assert_eq!(results[0], "A"); // Mode
    assert_eq!(results[1], "2"); // NUnique
                                 // Unique order is sorted: A,B
    assert_eq!(results[2], "A,B");
}

#[test]
fn test_mad_basic() {
    let ops = vec![Operation {
        kind: OpKind::Mad,
        field_idx: Some(0),
    }];
    let processor = StatsProcessor::new(ops);
    let mut agg = processor.create_aggregator();

    // Data: 1, 2, 3, 4, 5
    // Median = 3
    // Deviations = |1-3|, |2-3|, |3-3|, |4-3|, |5-3|
    //            = 2, 1, 0, 1, 2
    // Sorted Deviations = 0, 1, 1, 2, 2
    // Median Deviation = 1
    // MAD = 1 * 1.4826 = 1.4826
    for i in 1..=5 {
        let row = TestRow {
            fields: vec![i.to_string()],
        };
        processor.update(&mut agg, &row);
    }

    let results = processor.format_results(&agg);
    assert_eq!(results[0], "1.4826");
}

#[test]
fn test_mad_even() {
    let ops = vec![Operation {
        kind: OpKind::Mad,
        field_idx: Some(0),
    }];
    let processor = StatsProcessor::new(ops);
    let mut agg = processor.create_aggregator();

    // Data: 1, 2, 3, 4
    // Median = (2+3)/2 = 2.5
    // Deviations = |1-2.5|, |2-2.5|, |3-2.5|, |4-2.5|
    //            = 1.5, 0.5, 0.5, 1.5
    // Sorted Deviations = 0.5, 0.5, 1.5, 1.5
    // Median Deviation = (0.5 + 1.5) / 2 = 1.0
    // MAD = 1.0 * 1.4826 = 1.4826
    for i in 1..=4 {
        let row = TestRow {
            fields: vec![i.to_string()],
        };
        processor.update(&mut agg, &row);
    }

    let results = processor.format_results(&agg);
    assert_eq!(results[0], "1.4826");
}

#[test]
fn test_mad_constant() {
    let ops = vec![Operation {
        kind: OpKind::Mad,
        field_idx: Some(0),
    }];
    let processor = StatsProcessor::new(ops);
    let mut agg = processor.create_aggregator();

    // Data: 5, 5, 5
    // Median = 5
    // Deviations = 0, 0, 0
    // MAD = 0
    for _ in 0..3 {
        let row = TestRow {
            fields: vec!["5".to_string()],
        };
        processor.update(&mut agg, &row);
    }

    let results = processor.format_results(&agg);
    assert_eq!(results[0], "0");
}

#[test]
fn test_first_last_range() {
    let ops = vec![
        Operation {
            kind: OpKind::First,
            field_idx: Some(0),
        },
        Operation {
            kind: OpKind::Last,
            field_idx: Some(0),
        },
        Operation {
            kind: OpKind::Range,
            field_idx: Some(0),
        },
    ];
    let processor = StatsProcessor::new(ops);
    let mut agg = processor.create_aggregator();

    // Data: 10, 5, 20
    let data = vec!["10", "5", "20"];
    for v in data {
        let row = TestRow {
            fields: vec![v.to_string()],
        };
        processor.update(&mut agg, &row);
    }

    let results = processor.format_results(&agg);
    assert_eq!(results[0], "10"); // First
    assert_eq!(results[1], "20"); // Last
    assert_eq!(results[2], "15"); // Range (20 - 5)
}

#[test]
fn test_collapse_rand() {
    let ops = vec![
        Operation {
            kind: OpKind::Collapse,
            field_idx: Some(0),
        },
        Operation {
            kind: OpKind::Rand,
            field_idx: Some(0),
        },
    ];
    let processor = StatsProcessor::new(ops);
    let mut agg = processor.create_aggregator();

    // Data: A, B, C
    let data = vec!["A", "B", "C"];
    for v in &data {
        let row = TestRow {
            fields: vec![v.to_string()],
        };
        processor.update(&mut agg, &row);
    }

    let results = processor.format_results(&agg);
    assert_eq!(results[0], "A,B,C"); // Collapse
    assert!(data.contains(&results[1].as_str())); // Rand
}

#[test]
fn test_empty_input() {
    let ops = vec![
        Operation {
            kind: OpKind::Sum,
            field_idx: Some(0),
        },
        Operation {
            kind: OpKind::Mean,
            field_idx: Some(0),
        },
        Operation {
            kind: OpKind::Min,
            field_idx: Some(0),
        },
    ];
    let processor = StatsProcessor::new(ops);
    let agg = processor.create_aggregator();

    let results = processor.format_results(&agg);
    assert_eq!(results[0], "0"); // Sum default 0
    assert_eq!(results[1], "nan"); // Mean
    assert_eq!(results[2], "nan"); // Min
}

#[test]
fn test_single_value_stats() {
    let ops = vec![
        Operation {
            kind: OpKind::Stdev,
            field_idx: Some(0),
        },
        Operation {
            kind: OpKind::Variance,
            field_idx: Some(0),
        },
        Operation {
            kind: OpKind::Mad,
            field_idx: Some(0),
        },
    ];
    let processor = StatsProcessor::new(ops);
    let mut agg = processor.create_aggregator();

    // Data: 5
    let row = TestRow {
        fields: vec!["5".to_string()],
    };
    processor.update(&mut agg, &row);

    let results = processor.format_results(&agg);
    assert_eq!(results[0], "nan"); // Stdev (needs count > 1)
    assert_eq!(results[1], "nan"); // Variance (needs count > 1)
    assert_eq!(results[2], "0"); // MAD (deviation is 0)
}
