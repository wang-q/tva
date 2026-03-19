//! Efficient field selection logic.
//!
//! Implements the `SelectPlan` which pre-computes how to extract a set of fields
//! from a line, handling reordering and repetition.

use std::collections::HashMap;
use std::collections::HashSet;
use std::io;
use std::io::Write;
use std::ops::Range;

use crate::libs::tsv::record::TsvRow;

pub struct SelectPlan {
    targets: Vec<(usize, Vec<usize>)>,
    output_len: usize,
}

impl SelectPlan {
    pub fn new(indices: &[usize]) -> Self {
        let mut mapping: HashMap<usize, Vec<usize>> = HashMap::new();
        for (pos, &idx) in indices.iter().enumerate() {
            mapping.entry(idx).or_default().push(pos);
        }
        let mut targets: Vec<(usize, Vec<usize>)> = mapping.into_iter().collect();
        targets.sort_unstable_by_key(|k| k.0);
        Self {
            targets,
            output_len: indices.len(),
        }
    }

    pub fn output_len(&self) -> usize {
        self.output_len
    }

    /// Extract selected fields from a TsvRow into ranges.
    ///
    /// This method fills `output_ranges` with byte ranges corresponding to the selected fields.
    /// The ranges are ordered according to the selection plan (e.g. if plan is 3,1, ranges will be [field3, field1]).
    /// `output_ranges` is resized to match `output_len`.
    ///
    /// Returns `Ok(())` if all fields were found.
    /// Returns `Err(missing_idx)` if a requested field index was not found in the line.
    #[inline(always)]
    pub fn extract_ranges(
        &self,
        row: &TsvRow<'_, '_>,
        output_ranges: &mut Vec<Range<usize>>,
    ) -> Result<(), usize> {
        let output_len = self.output_len;
        if output_ranges.len() != output_len {
            output_ranges.resize(output_len, 0..0);
        } else {
            output_ranges.fill(0..0);
        }

        let targets = &self.targets;
        let ends = row.ends;
        let _line = row.line;

        let mut target_idx = 0usize;
        let targets_len = targets.len();

        for (field_idx, &end_pos) in ends.iter().enumerate() {
            let current_col_idx = field_idx + 1;
            let start_pos = if field_idx == 0 {
                0
            } else {
                ends[field_idx - 1] + 1
            };

            while target_idx < targets_len {
                let (target_col, positions) = &targets[target_idx];
                if *target_col == current_col_idx {
                    let range = start_pos..end_pos;
                    for &pos in positions {
                        output_ranges[pos] = range.clone();
                    }
                    target_idx += 1;
                    break;
                } else if *target_col > current_col_idx {
                    break;
                } else {
                    target_idx += 1;
                }
            }

            if target_idx >= targets_len {
                break;
            }
        }

        if target_idx < targets_len {
            return Err(targets[target_idx].0);
        }

        Ok(())
    }
}

pub fn write_selected_from_bytes(
    writer: &mut dyn Write,
    row: &TsvRow<'_, '_>,
    delimiter: u8,
    plan: &SelectPlan,
    output_ranges: &mut Vec<Range<usize>>,
) -> io::Result<()> {
    match plan.extract_ranges(row, output_ranges) {
        Ok(_) => {
            let mut first = true;
            for range in output_ranges.iter() {
                if !first {
                    writer.write_all(&[delimiter])?;
                }
                writer.write_all(&row.line[range.clone()])?;
                first = false;
            }
            writer.write_all(b"\n")?;
            Ok(())
        }
        Err(missing_idx) => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "Not enough fields in line. Field index {} out of range",
                missing_idx
            ),
        )),
    }
}

pub fn write_excluding_from_bytes(
    writer: &mut dyn Write,
    row: &TsvRow<'_, '_>,
    delimiter: u8,
    exclude_set: &HashSet<usize>,
) -> io::Result<()> {
    let ends = row.ends;
    let line = row.line;
    let mut first_output = true;

    for (field_idx, &end_pos) in ends.iter().enumerate() {
        let current_col_idx = field_idx + 1;
        let start_pos = if field_idx == 0 {
            0
        } else {
            ends[field_idx - 1] + 1
        };

        if !exclude_set.contains(&current_col_idx) {
            if !first_output {
                writer.write_all(&[delimiter])?;
            }
            writer.write_all(&line[start_pos..end_pos])?;
            first_output = false;
        }
    }

    writer.write_all(b"\n")?;
    Ok(())
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RestMode {
    None,
    First,
    Last,
}

pub fn write_with_rest(
    writer: &mut dyn Write,
    row: &TsvRow<'_, '_>,
    delimiter: u8,
    selected_indices: &[usize], // Explicitly selected fields
    excluded_set: Option<&HashSet<usize>>, // Fields to exclude from rest
    rest_mode: RestMode,
) -> io::Result<()> {
    let ends = row.ends;
    let line = row.line;
    let total_fields = ends.len();
    let mut first_output = true;

    let mut write_field = |field_idx: usize| -> io::Result<()> {
        if !first_output {
            writer.write_all(&[delimiter])?;
        }
        let start = if field_idx == 0 {
            0
        } else {
            ends[field_idx - 1] + 1
        };
        let end = ends[field_idx];
        writer.write_all(&line[start..end])?;
        first_output = false;
        Ok(())
    };

    // selected_set for fast lookup of what is "selected" (so we don't output it in rest)
    let selected_set: HashSet<usize> = selected_indices.iter().cloned().collect();

    let is_rest = |idx: usize| -> bool {
        if selected_set.contains(&idx) {
            return false;
        }
        if let Some(ex) = excluded_set {
            if ex.contains(&idx) {
                return false;
            }
        }
        true
    };

    if rest_mode == RestMode::First {
        for i in 1..=total_fields {
            if is_rest(i) {
                write_field(i - 1)?;
            }
        }
    }

    for &idx in selected_indices {
        if idx >= 1 && idx <= total_fields {
            write_field(idx - 1)?;
        } else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Not enough fields in line. Field index {} out of range",
                    idx
                ),
            ));
        }
    }

    if rest_mode == RestMode::Last {
        for i in 1..=total_fields {
            if is_rest(i) {
                write_field(i - 1)?;
            }
        }
    }

    writer.write_all(b"\n")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::libs::tsv::record::TsvRow;

    #[test]
    fn test_select_plan_basic() {
        let plan = SelectPlan::new(&[1, 2, 3]);
        let line = b"a\tb\tc";
        let ends = vec![1, 3, 5];
        let row = TsvRow { line, ends: &ends };
        let mut ranges = Vec::new();
        plan.extract_ranges(&row, &mut ranges).unwrap();

        assert_eq!(ranges.len(), 3);
        assert_eq!(&line[ranges[0].clone()], b"a");
        assert_eq!(&line[ranges[1].clone()], b"b");
        assert_eq!(&line[ranges[2].clone()], b"c");
    }

    #[test]
    fn test_select_plan_reorder() {
        // Select 3, 1
        let plan = SelectPlan::new(&[3, 1]);
        let line = b"a\tb\tc";
        let ends = vec![1, 3, 5];
        let row = TsvRow { line, ends: &ends };
        let mut ranges = Vec::new();
        plan.extract_ranges(&row, &mut ranges).unwrap();

        assert_eq!(ranges.len(), 2);
        // output_ranges[0] corresponds to index 3 ("c")
        // output_ranges[1] corresponds to index 1 ("a")
        assert_eq!(&line[ranges[0].clone()], b"c");
        assert_eq!(&line[ranges[1].clone()], b"a");
    }

    #[test]
    fn test_select_plan_repeat() {
        // Select 2, 2
        let plan = SelectPlan::new(&[2, 2]);
        let line = b"a\tb\tc";
        let ends = vec![1, 3, 5];
        let row = TsvRow { line, ends: &ends };
        let mut ranges = Vec::new();
        plan.extract_ranges(&row, &mut ranges).unwrap();

        assert_eq!(ranges.len(), 2);
        assert_eq!(&line[ranges[0].clone()], b"b");
        assert_eq!(&line[ranges[1].clone()], b"b");
    }

    #[test]
    fn test_select_plan_missing_field() {
        // Select 4 (out of bounds)
        let plan = SelectPlan::new(&[4]);
        let line = b"a\tb\tc";
        let ends = vec![1, 3, 5];
        let row = TsvRow { line, ends: &ends };
        let mut ranges = Vec::new();
        let err = plan.extract_ranges(&row, &mut ranges).unwrap_err();
        assert_eq!(err, 4);
    }

    #[test]
    fn test_select_plan_empty_line() {
        let plan = SelectPlan::new(&[1]);
        let line = b""; // 1 field (empty)
        let ends = vec![0];
        let row = TsvRow { line, ends: &ends };
        let mut ranges = Vec::new();
        plan.extract_ranges(&row, &mut ranges).unwrap();
        assert_eq!(&line[ranges[0].clone()], b"");
    }

    #[test]
    fn test_select_plan_empty_line_missing() {
        let plan = SelectPlan::new(&[2]);
        let line = b""; // 1 field (empty)
        let ends = vec![0];
        let row = TsvRow { line, ends: &ends };
        let mut ranges = Vec::new();
        let err = plan.extract_ranges(&row, &mut ranges).unwrap_err();
        assert_eq!(err, 2);
    }

    #[test]
    fn test_write_selected_basic() {
        let plan = SelectPlan::new(&[2, 3]);
        let line = b"a\tb\tc";
        let ends = vec![1, 3, 5];
        let row = TsvRow { line, ends: &ends };
        let mut output = Vec::new();
        let mut ranges = Vec::new();
        write_selected_from_bytes(&mut output, &row, b'\t', &plan, &mut ranges).unwrap();
        assert_eq!(output, b"b\tc\n");
    }

    #[test]
    fn test_write_selected_error() {
        let plan = SelectPlan::new(&[4]);
        let line = b"a\tb\tc";
        let ends = vec![1, 3, 5];
        let row = TsvRow { line, ends: &ends };
        let mut output = Vec::new();
        let mut ranges = Vec::new();
        let err =
            write_selected_from_bytes(&mut output, &row, b'\t', &plan, &mut ranges)
                .unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
        assert!(err.to_string().contains("Field index 4 out of range"));
    }

    #[test]
    fn test_write_excluding_basic() {
        let mut exclude = HashSet::new();
        exclude.insert(2);
        let line = b"a\tb\tc";
        let ends = vec![1, 3, 5];
        let row = TsvRow { line, ends: &ends };
        let mut output = Vec::new();
        write_excluding_from_bytes(&mut output, &row, b'\t', &exclude).unwrap();
        assert_eq!(output, b"a\tc\n");
    }

    #[test]
    fn test_write_excluding_all() {
        let mut exclude = HashSet::new();
        exclude.insert(1);
        exclude.insert(2);
        exclude.insert(3);
        let line = b"a\tb\tc";
        let ends = vec![1, 3, 5];
        let row = TsvRow { line, ends: &ends };
        let mut output = Vec::new();
        write_excluding_from_bytes(&mut output, &row, b'\t', &exclude).unwrap();
        assert_eq!(output, b"\n");
    }

    #[test]
    fn test_write_excluding_none() {
        let exclude = HashSet::new();
        let line = b"a\tb\tc";
        let ends = vec![1, 3, 5];
        let row = TsvRow { line, ends: &ends };
        let mut output = Vec::new();
        write_excluding_from_bytes(&mut output, &row, b'\t', &exclude).unwrap();
        assert_eq!(output, b"a\tb\tc\n");
    }

    #[test]
    fn test_write_with_rest_none() {
        let line = b"a\tb\tc";
        let ends = vec![1, 3, 5];
        let row = TsvRow { line, ends: &ends };
        let mut output = Vec::new();
        write_with_rest(&mut output, &row, b'\t', &[2], None, RestMode::None).unwrap();
        assert_eq!(output, b"b\n");
    }

    #[test]
    fn test_write_with_rest_first() {
        let line = b"a\tb\tc";
        let ends = vec![1, 3, 5];
        let row = TsvRow { line, ends: &ends };
        let mut output = Vec::new();
        write_with_rest(&mut output, &row, b'\t', &[2], None, RestMode::First).unwrap();
        // Rest: 1, 3 -> a, c
        // Selected: 2 -> b
        // Output: a, c, b
        assert_eq!(output, b"a\tc\tb\n");
    }

    #[test]
    fn test_write_with_rest_last() {
        let line = b"a\tb\tc";
        let ends = vec![1, 3, 5];
        let row = TsvRow { line, ends: &ends };
        let mut output = Vec::new();
        write_with_rest(&mut output, &row, b'\t', &[2], None, RestMode::Last).unwrap();
        // Selected: 2 -> b
        // Rest: 1, 3 -> a, c
        // Output: b, a, c
        assert_eq!(output, b"b\ta\tc\n");
    }

    #[test]
    fn test_write_with_rest_exclude_from_rest() {
        let line = b"a\tb\tc\td";
        let ends = vec![1, 3, 5, 7];
        let row = TsvRow { line, ends: &ends };
        let mut output = Vec::new();
        let mut excluded = HashSet::new();
        excluded.insert(4); // Exclude d from rest

        write_with_rest(
            &mut output,
            &row,
            b'\t',
            &[2],
            Some(&excluded),
            RestMode::Last,
        )
        .unwrap();
        // Selected: 2 -> b
        // Rest (all - selected - excluded): {1, 2, 3, 4} - {2} - {4} = {1, 3} -> a, c
        // Output: b, a, c
        assert_eq!(output, b"b\ta\tc\n");
    }

    #[test]
    fn test_write_with_rest_error() {
        let line = b"a\tb\tc";
        let ends = vec![1, 3, 5];
        let row = TsvRow { line, ends: &ends };
        let mut output = Vec::new();
        let err = write_with_rest(
            &mut output,
            &row,
            b'\t',
            &[4], // Out of bounds
            None,
            RestMode::None,
        )
        .unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
        assert!(err.to_string().contains("Field index 4 out of range"));
    }
}
