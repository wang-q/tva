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
