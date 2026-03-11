//! Linear regression utilities for plotting.

/// Calculate linear regression (least squares) for a set of points.
/// Returns (slope, intercept) for the line y = slope * x + intercept
pub fn calculate_regression(points: &[(f64, f64)]) -> Option<(f64, f64)> {
    let n = points.len() as f64;
    if n < 2.0 {
        return None;
    }

    let sum_x: f64 = points.iter().map(|p| p.0).sum();
    let sum_y: f64 = points.iter().map(|p| p.1).sum();
    let sum_xy: f64 = points.iter().map(|p| p.0 * p.1).sum();
    let sum_x2: f64 = points.iter().map(|p| p.0 * p.0).sum();

    let denominator = n * sum_x2 - sum_x * sum_x;
    if denominator.abs() < f64::EPSILON {
        return None; // Vertical line or no variation in x
    }

    let slope = (n * sum_xy - sum_x * sum_y) / denominator;
    let intercept = (sum_y - slope * sum_x) / n;

    Some((slope, intercept))
}

/// Format regression equation for display in legend.
/// Shows y = mx + b with appropriate precision.
pub fn format_regression_equation(slope: f64, intercept: f64) -> String {
    // Determine appropriate precision based on magnitude
    let format_coefficient = |v: f64| -> String {
        if v.abs() >= 100.0 {
            format!("{:.0}", v)
        } else if v.abs() >= 10.0 {
            format!("{:.1}", v)
        } else if v.abs() >= 1.0 {
            format!("{:.2}", v)
        } else if v.abs() >= 0.1 {
            format!("{:.3}", v)
        } else {
            format!("{:.4}", v)
        }
    };

    let slope_str = format_coefficient(slope);
    let intercept_str = format_coefficient(intercept);

    if intercept >= 0.0 {
        format!("y = {}x + {}", slope_str, intercept_str)
    } else {
        format!(
            "y = {}x - {}",
            slope_str,
            format_coefficient(intercept.abs())
        )
    }
}

/// Liang-Barsky line clipping algorithm.
/// Clips a line segment to a rectangular boundary.
/// Returns Some((x1, y1, x2, y2)) if the line intersects the rectangle, None otherwise.
fn liang_barsky_clip(
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    x_min: f64,
    y_min: f64,
    x_max: f64,
    y_max: f64,
) -> Option<(f64, f64, f64, f64)> {
    let dx = x2 - x1;
    let dy = y2 - y1;

    let p = [-dx, dx, -dy, dy];
    let q = [x1 - x_min, x_max - x1, y1 - y_min, y_max - y1];

    let mut u1 = 0.0;
    let mut u2 = 1.0;

    for i in 0..4 {
        if p[i] == 0.0 {
            // Line is parallel to this boundary
            if q[i] < 0.0 {
                return None; // Line is outside and parallel
            }
            // Line is inside and parallel, continue
        } else {
            let t = q[i] / p[i];
            if p[i] < 0.0 {
                // Potentially entering
                if t > u1 {
                    u1 = t;
                }
            } else {
                // Potentially leaving
                if t < u2 {
                    u2 = t;
                }
            }
        }
    }

    if u1 > u2 {
        return None; // Line is completely outside
    }

    let clipped_x1 = x1 + u1 * dx;
    let clipped_y1 = y1 + u1 * dy;
    let clipped_x2 = x1 + u2 * dx;
    let clipped_y2 = y1 + u2 * dy;

    Some((clipped_x1, clipped_y1, clipped_x2, clipped_y2))
}

/// Generate regression line points for plotting.
/// Uses Liang-Barsky clipping to ensure the line stays within chart boundaries.
pub fn generate_regression_points(
    slope: f64,
    intercept: f64,
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
) -> Vec<(f64, f64)> {
    // Calculate the line at x boundaries
    let x1 = x_min;
    let y1 = slope * x1 + intercept;
    let x2 = x_max;
    let y2 = slope * x2 + intercept;

    // Clip the line to the chart boundaries
    if let Some((cx1, cy1, cx2, cy2)) =
        liang_barsky_clip(x1, y1, x2, y2, x_min, y_min, x_max, y_max)
    {
        vec![(cx1, cy1), (cx2, cy2)]
    } else {
        // Line is completely outside the chart
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_regression_linear() {
        // y = 2x
        let points = vec![(0.0, 0.0), (1.0, 2.0), (2.0, 4.0), (3.0, 6.0)];
        let (slope, intercept) = calculate_regression(&points).unwrap();
        assert!((slope - 2.0).abs() < 0.001);
        assert!(intercept.abs() < 0.001);
    }

    #[test]
    fn test_calculate_regression_with_intercept() {
        // y = 0.5x + 1
        let points = vec![(0.0, 1.0), (2.0, 2.0), (4.0, 3.0)];
        let (slope, intercept) = calculate_regression(&points).unwrap();
        assert!((slope - 0.5).abs() < 0.001);
        assert!((intercept - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_calculate_regression_insufficient_points() {
        let points = vec![(1.0, 1.0)];
        assert!(calculate_regression(&points).is_none());
    }

    #[test]
    fn test_format_regression_equation_positive() {
        assert_eq!(format_regression_equation(0.5, 1.0), "y = 0.500x + 1.00");
    }

    #[test]
    fn test_format_regression_equation_negative() {
        assert_eq!(format_regression_equation(0.5, -1.0), "y = 0.500x - 1.00");
    }

    #[test]
    fn test_format_regression_equation_large_values() {
        assert_eq!(format_regression_equation(150.5, 200.3), "y = 150x + 200");
    }

    #[test]
    fn test_generate_regression_points() {
        // y = 2x, within bounds
        let points = generate_regression_points(2.0, 0.0, 0.0, 10.0, 0.0, 25.0);
        assert_eq!(points, vec![(0.0, 0.0), (10.0, 20.0)]);
    }

    #[test]
    fn test_generate_regression_points_with_clipping() {
        // y = 10x + 5, steep slope that goes out of y bounds
        // At x=0, y=5; At x=10, y=105
        // With y_max=50, should clip at y=50
        let points = generate_regression_points(10.0, 5.0, 0.0, 10.0, 0.0, 50.0);
        assert_eq!(points.len(), 2);
        // First point should be at x=0, y=5 (within bounds)
        assert!((points[0].0 - 0.0).abs() < 0.001);
        assert!((points[0].1 - 5.0).abs() < 0.001);
        // Second point should be clipped at y=50
        assert!((points[1].1 - 50.0).abs() < 0.001);
    }

    #[test]
    fn test_generate_regression_points_completely_outside() {
        // Line completely above the chart
        let points = generate_regression_points(1.0, 100.0, 0.0, 10.0, 0.0, 50.0);
        assert!(points.is_empty());

        // Line completely below the chart
        let points = generate_regression_points(1.0, -100.0, 0.0, 10.0, 0.0, 50.0);
        assert!(points.is_empty());
    }

    #[test]
    fn test_liang_barsky_clip() {
        // Line completely inside
        let result = liang_barsky_clip(1.0, 1.0, 5.0, 5.0, 0.0, 0.0, 10.0, 10.0);
        assert!(result.is_some());
        let (x1, y1, x2, y2) = result.unwrap();
        assert!((x1 - 1.0).abs() < 0.001);
        assert!((y1 - 1.0).abs() < 0.001);
        assert!((x2 - 5.0).abs() < 0.001);
        assert!((y2 - 5.0).abs() < 0.001);

        // Line crossing top boundary
        let result = liang_barsky_clip(0.0, 0.0, 10.0, 20.0, 0.0, 0.0, 10.0, 10.0);
        assert!(result.is_some());
        let (_, y1, _, y2) = result.unwrap();
        assert!((y1 - 0.0).abs() < 0.001);
        assert!((y2 - 10.0).abs() < 0.001);

        // Line completely outside
        let result = liang_barsky_clip(0.0, 20.0, 10.0, 30.0, 0.0, 0.0, 10.0, 10.0);
        assert!(result.is_none());
    }
}
