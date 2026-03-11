//! Statistical utilities for plotting.
//!
//! Provides functions for calculating box plot statistics used in data visualization.
//! Uses aggregation/math for core statistical calculations.

use crate::libs::aggregation::math;

/// Box plot statistics
#[derive(Debug, Clone)]
pub struct BoxStats {
    pub min: f64,
    pub q1: f64,
    pub median: f64,
    pub q3: f64,
    pub max: f64,
    pub outliers: Vec<f64>,
    pub count: usize,
}

impl BoxStats {
    /// Calculate box plot statistics from a list of values
    ///
    /// Uses Tukey's box plot method:
    /// - Q1: 25th percentile
    /// - Median: 50th percentile
    /// - Q3: 75th percentile
    /// - Whiskers: 1.5 * IQR rule
    /// - Outliers: Values beyond whiskers
    ///
    /// # Arguments
    /// * `values` - Slice of f64 values
    ///
    /// # Returns
    /// * `Some(BoxStats)` if values is not empty
    /// * `None` if values is empty
    pub fn calculate(values: &[f64]) -> Option<Self> {
        if values.is_empty() {
            return None;
        }

        let mut sorted = values.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let count = sorted.len();

        // Use aggregation/math for quantile calculations
        let median = math::quantile(&sorted, 0.5);
        let q1 = math::quantile(&sorted, 0.25);
        let q3 = math::quantile(&sorted, 0.75);

        // Calculate whiskers using 1.5 * IQR rule
        let iqr = q3 - q1;
        let lower_fence = q1 - 1.5 * iqr;
        let upper_fence = q3 + 1.5 * iqr;

        // Find whisker ends (last/first non-outlier values)
        let min_val = sorted[0];
        let max_val = sorted[count - 1];
        let mut lower_whisker = min_val;
        let mut upper_whisker = max_val;
        let mut outliers = Vec::new();

        for &v in &sorted {
            if v < lower_fence {
                outliers.push(v);
            } else {
                lower_whisker = v;
                break;
            }
        }

        for &v in sorted.iter().rev() {
            if v > upper_fence {
                outliers.push(v);
            } else {
                upper_whisker = v;
                break;
            }
        }

        Some(BoxStats {
            min: lower_whisker,
            q1,
            median,
            q3,
            max: upper_whisker,
            outliers,
            count,
        })
    }
}

/// Calculate bounds from a collection of BoxStats
///
/// Returns (min, max) including outliers if present, with 5% padding
pub fn calculate_bounds_from_stats<'a, I>(stats_iter: I) -> (f64, f64)
where
    I: Iterator<Item = &'a BoxStats>,
{
    let mut y_min = f64::INFINITY;
    let mut y_max = f64::NEG_INFINITY;

    for stats in stats_iter {
        y_min = y_min.min(stats.min);
        y_max = y_max.max(stats.max);
        // Include outliers if present
        for &outlier in &stats.outliers {
            y_min = y_min.min(outlier);
            y_max = y_max.max(outlier);
        }
    }

    // Add some padding
    let range = y_max - y_min;
    if range > 0.0 {
        y_min -= range * 0.05;
        y_max += range * 0.05;
    } else {
        y_min -= 1.0;
        y_max += 1.0;
    }

    (y_min, y_max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_box_stats_calculate() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let stats = BoxStats::calculate(&data).unwrap();

        assert_eq!(stats.min, 1.0);
        assert_eq!(stats.max, 10.0);
        assert_eq!(stats.median, 5.5);
        assert!((stats.q1 - 3.25).abs() < 1e-10);
        assert!((stats.q3 - 7.75).abs() < 1e-10);
    }

    #[test]
    fn test_box_stats_with_outliers() {
        // Data with clear outliers
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 100.0];
        let stats = BoxStats::calculate(&data).unwrap();

        // 100 should be detected as outlier
        assert!(stats.outliers.contains(&100.0));
        // Max should be the largest non-outlier
        assert!(stats.max < 100.0);
    }

    #[test]
    fn test_calculate_bounds_from_stats() {
        let stats1 = BoxStats::calculate(&[1.0, 2.0, 3.0, 4.0, 5.0]).unwrap();
        let stats2 = BoxStats::calculate(&[10.0, 20.0, 30.0]).unwrap();

        let (min, max) = calculate_bounds_from_stats([&stats1, &stats2].into_iter());

        // Should include padding
        assert!(min < 1.0);
        assert!(max > 30.0);
    }

    #[test]
    fn test_box_stats_empty() {
        let data: Vec<f64> = vec![];
        assert!(BoxStats::calculate(&data).is_none());
    }

    #[test]
    fn test_box_stats_single_value() {
        let data = vec![5.0];
        let stats = BoxStats::calculate(&data).unwrap();

        assert_eq!(stats.min, 5.0);
        assert_eq!(stats.max, 5.0);
        assert_eq!(stats.median, 5.0);
        assert_eq!(stats.q1, 5.0);
        assert_eq!(stats.q3, 5.0);
        assert!(stats.outliers.is_empty());
    }
}
