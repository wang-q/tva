//! 2D binning algorithms for data visualization.
//!
//! This module implements 2D binning (similar to ggplot2's geom_bin2d) for
//! visualizing density distributions of two-dimensional data.

/// Configuration for 2D binning.
pub struct Bin2dConfig {
    pub width: u16,
    pub height: u16,
    pub x_bins: usize,
    pub y_bins: usize,
    pub x_binwidth: Option<f64>,
    pub y_binwidth: Option<f64>,
    pub x_label: String,
    pub y_label: String,
}

/// A 2D bin with count information.
#[derive(Debug)]
pub struct Bin2d {
    pub x_min: f64,
    pub x_max: f64,
    pub y_min: f64,
    pub y_max: f64,
    pub count: usize,
}

/// Compute 2D bins from data points.
pub fn compute_bins_2d(
    x_values: &[f64],
    y_values: &[f64],
    x_bins: usize,
    y_bins: usize,
    x_binwidth: Option<f64>,
    y_binwidth: Option<f64>,
) -> (Vec<Bin2d>, f64, f64, f64, f64) {
    // Compute data bounds
    let x_min = x_values.iter().copied().fold(f64::INFINITY, f64::min);
    let x_max = x_values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let y_min = y_values.iter().copied().fold(f64::INFINITY, f64::min);
    let y_max = y_values.iter().copied().fold(f64::NEG_INFINITY, f64::max);

    // Compute actual bin counts and widths
    let (actual_x_bins, actual_x_binwidth) = match x_binwidth {
        Some(width) => {
            let bins = ((x_max - x_min) / width).ceil() as usize;
            (bins.max(1), width)
        }
        None => {
            let width = (x_max - x_min) / x_bins as f64;
            (x_bins, width)
        }
    };

    let (actual_y_bins, actual_y_binwidth) = match y_binwidth {
        Some(width) => {
            let bins = ((y_max - y_min) / width).ceil() as usize;
            (bins.max(1), width)
        }
        None => {
            let width = (y_max - y_min) / y_bins as f64;
            (y_bins, width)
        }
    };

    // Initialize bins
    let mut bins: Vec<Vec<usize>> = vec![vec![0; actual_y_bins]; actual_x_bins];

    // Count points in each bin
    for (x, y) in x_values.iter().zip(y_values.iter()) {
        let x_bin = ((x - x_min) / actual_x_binwidth)
            .floor()
            .clamp(0.0, (actual_x_bins - 1) as f64) as usize;
        let y_bin = ((y - y_min) / actual_y_binwidth)
            .floor()
            .clamp(0.0, (actual_y_bins - 1) as f64) as usize;
        bins[x_bin][y_bin] += 1;
    }

    // Convert to Bin2d structures
    let mut bin2d_list = Vec::new();
    for i in 0..actual_x_bins {
        for j in 0..actual_y_bins {
            bin2d_list.push(Bin2d {
                x_min: x_min + i as f64 * actual_x_binwidth,
                x_max: x_min + (i + 1) as f64 * actual_x_binwidth,
                y_min: y_min + j as f64 * actual_y_binwidth,
                y_max: y_min + (j + 1) as f64 * actual_y_binwidth,
                count: bins[i][j],
            });
        }
    }

    (bin2d_list, x_min, x_max, y_min, y_max)
}

/// Compute optimal bin count using Freedman-Diaconis rule.
pub fn freedman_diaconis_bins(values: &[f64]) -> usize {
    if values.len() < 4 {
        return 10;
    }

    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let q1_idx = (sorted.len() as f64 * 0.25).floor() as usize;
    let q3_idx = (sorted.len() as f64 * 0.75).floor() as usize;
    let q1 = sorted[q1_idx.min(sorted.len() - 1)];
    let q3 = sorted[q3_idx.min(sorted.len() - 1)];
    let iqr = q3 - q1;

    if iqr <= 0.0 {
        return 10;
    }

    let bin_width = 2.0 * iqr / (sorted.len() as f64).cbrt();
    let range = sorted.last().unwrap() - sorted.first().unwrap();

    (range / bin_width).ceil() as usize
}

/// Compute optimal bin count using Sturges' rule.
pub fn sturges_bins(n: usize) -> usize {
    (1.0 + (n as f64).log2()).ceil() as usize
}

/// Compute optimal bin count using square root rule.
pub fn sqrt_bins(n: usize) -> usize {
    (n as f64).sqrt().ceil() as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_bins_2d_basic() {
        let x_values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y_values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let (bins, x_min, x_max, y_min, y_max) =
            compute_bins_2d(&x_values, &y_values, 3, 3, None, None);

        assert_eq!(x_min, 1.0);
        assert_eq!(x_max, 5.0);
        assert_eq!(y_min, 1.0);
        assert_eq!(y_max, 5.0);
        assert_eq!(bins.len(), 9); // 3x3 grid
    }

    #[test]
    fn test_compute_bins_2d_with_binwidth() {
        let x_values = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0];
        let y_values = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0];
        let (bins, x_min, x_max, y_min, y_max) =
            compute_bins_2d(&x_values, &y_values, 10, 10, Some(1.0), Some(1.0));

        assert_eq!(x_min, 0.0);
        assert_eq!(x_max, 5.0);
        assert_eq!(y_min, 0.0);
        assert_eq!(y_max, 5.0);
        // With binwidth 1.0, range 5.0, we get 6 bins (0-1, 1-2, 2-3, 3-4, 4-5, 5-6)
        assert!(bins.len() >= 6);
    }

    #[test]
    fn test_compute_bins_2d_empty_data() {
        let x_values: Vec<f64> = vec![];
        let y_values: Vec<f64> = vec![];
        let (bins, x_min, x_max, _y_min, _y_max) =
            compute_bins_2d(&x_values, &y_values, 3, 3, None, None);

        assert_eq!(bins.len(), 9); // Still creates bins with inf bounds
        assert_eq!(x_min, f64::INFINITY);
        assert_eq!(x_max, f64::NEG_INFINITY);
    }

    #[test]
    fn test_compute_bins_2d_single_point() {
        let x_values = vec![5.0];
        let y_values = vec![5.0];
        let (bins, x_min, x_max, y_min, y_max) =
            compute_bins_2d(&x_values, &y_values, 3, 3, None, None);

        assert_eq!(x_min, 5.0);
        assert_eq!(x_max, 5.0);
        assert_eq!(y_min, 5.0);
        assert_eq!(y_max, 5.0);
        // All points fall into one bin
        let total_count: usize = bins.iter().map(|b| b.count).sum();
        assert_eq!(total_count, 1);
    }

    #[test]
    fn test_compute_bins_2d_clamping() {
        // Test that points outside the computed range are clamped
        let x_values = vec![0.0, 10.0, 20.0];
        let y_values = vec![0.0, 10.0, 20.0];
        let (bins, _x_min, _x_max, _y_min, _y_max) =
            compute_bins_2d(&x_values, &y_values, 2, 2, None, None);

        let total_count: usize = bins.iter().map(|b| b.count).sum();
        assert_eq!(total_count, 3);
    }

    #[test]
    fn test_freedman_diaconis_bins() {
        // Normal case
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let bins = freedman_diaconis_bins(&values);
        assert!(bins > 0);

        // Too few values
        let few_values = vec![1.0, 2.0, 3.0];
        assert_eq!(freedman_diaconis_bins(&few_values), 10);

        // Zero IQR (all same values)
        let same_values = vec![5.0, 5.0, 5.0, 5.0, 5.0];
        assert_eq!(freedman_diaconis_bins(&same_values), 10);
    }

    #[test]
    fn test_sturges_bins() {
        assert_eq!(sturges_bins(1), 1);
        assert_eq!(sturges_bins(2), 2);
        assert_eq!(sturges_bins(10), 5); // 1 + log2(10) ≈ 4.32, ceil = 5
        assert_eq!(sturges_bins(100), 8);
        assert_eq!(sturges_bins(1000), 11);
    }

    #[test]
    fn test_sqrt_bins() {
        assert_eq!(sqrt_bins(1), 1);
        assert_eq!(sqrt_bins(4), 2);
        assert_eq!(sqrt_bins(9), 3);
        assert_eq!(sqrt_bins(10), 4);
        assert_eq!(sqrt_bins(100), 10);
    }
}
