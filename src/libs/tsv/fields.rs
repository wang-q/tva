//! Common field list parsing utilities shared across tva commands.
//!
//! Field lists are used to refer to columns by index or by name. The shared
//! syntax is documented in [`crate::libs::cli::FIELD_SYNTAX_HELP`].
//!
//! Basic numeric-only parsing:
//!
//! ```
//! use tva::libs::tsv::fields::parse_numeric_field_list;
//!
//! let indices = parse_numeric_field_list("1,3-5").unwrap();
//! assert_eq!(indices, vec![1, 3, 4, 5]);
//! ```
//!
//! Header-aware parsing (mixing indices and names):
//!
//! ```
//! use tva::libs::tsv::fields::{Header, parse_field_list_with_header};
//!
//! let header = Header::from_line("run\tuser_time\tsystem_time", '\t');
//! let indices = parse_field_list_with_header("run,system_time", Some(&header), '\t').unwrap();
//! assert_eq!(indices, vec![1, 3]);
//! ```

use intspan::IntSpan;
use std::collections::HashMap;

pub fn fields_to_ints(s: &str) -> IntSpan {
    let mut ints = IntSpan::new();
    for p in tokenize_field_spec(s) {
        ints.add_runlist(&p);
    }

    ints
}

pub fn parse_numeric_field_list(spec: &str) -> Result<Vec<usize>, String> {
    if spec.trim().is_empty() {
        return Ok(Vec::new());
    }

    let mut ints: Vec<i32> = Vec::new();
    for part in tokenize_field_spec(spec) {
        let mut part = part.trim().to_string();
        if part.is_empty() {
            return Err(format!("empty field list element in `{}`", spec));
        }

        // Handle reverse ranges like "6-4" by swapping them to "4-6"
        if let Some((s, e)) = part.split_once('-') {
            if let (Ok(start), Ok(end)) = (s.parse::<usize>(), e.parse::<usize>()) {
                if start > end {
                    part = format!("{}-{}", end, start);
                }
            }
        }

        let intspan = IntSpan::from(&part);
        for e in intspan.elements() {
            if e <= 0 {
                return Err(format!("field index must be >= 1 in `{}`", spec));
            }
            ints.push(e);
        }
    }

    ints.sort_unstable();
    ints.dedup();

    Ok(ints.iter().map(|e| *e as usize).collect())
}

pub fn parse_numeric_field_list_preserve_order(
    spec: &str,
) -> Result<Vec<usize>, String> {
    if spec.trim().is_empty() {
        return Ok(Vec::new());
    }

    let mut ints: Vec<i32> = Vec::new();
    for part in tokenize_field_spec(spec) {
        let part = part.trim().to_string();
        if part.is_empty() {
            return Err(format!("empty field list element in `{}`", spec));
        }

        // Handle reverse ranges like "6-4" by swapping them to "4-6" for expansion,
        // but we might want to support "6,5,4".
        // Current IntSpan doesn't support order preserving or reverse ranges directly.
        // If it's a range "start-end":
        if let Some((s, e)) = part.split_once('-') {
            if let (Ok(start), Ok(end)) = (s.parse::<i32>(), e.parse::<i32>()) {
                if start <= 0 || end <= 0 {
                    return Err(format!("field index must be >= 1 in `{}`", spec));
                }
                if start <= end {
                    for i in start..=end {
                        ints.push(i);
                    }
                } else {
                    // Reverse range
                    let mut i = start;
                    while i >= end {
                        ints.push(i);
                        i -= 1;
                    }
                }
                continue;
            }
        }

        // Single number?
        if let Ok(n) = part.parse::<i32>() {
            if n <= 0 {
                return Err(format!("field index must be >= 1 in `{}`", spec));
            }
            if n > 1048576 {
                // tsv-select limit check
                return Err(format!(
                    "Maximum allowed '--e|exclude' field number is 1048576."
                ));
            }
            ints.push(n);
            continue;
        }

        return Err(format!("invalid numeric field spec: `{}`", part));
    }

    Ok(ints.iter().map(|e| *e as usize).collect())
}

pub fn fields_to_idx(spec: &str) -> Vec<usize> {
    parse_numeric_field_list(spec).unwrap()
}

pub fn tokenize_field_spec(spec: &str) -> Vec<String> {
    // Split by comma, but respect escaped commas
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut escaped = false;

    for c in spec.chars() {
        if escaped {
            current.push(c);
            escaped = false;
        } else if c == '\\' {
            current.push(c);
            escaped = true;
        } else if c == ',' {
            tokens.push(current);
            current = String::new();
        } else {
            current.push(c);
        }
    }
    // Always push last token, even if empty?
    // "1," -> ["1", ""]
    // tsv-select "1," error: "empty field list element"
    tokens.push(current);
    tokens
}

fn split_name_range_token(token: &str) -> Option<(String, String)> {
    let mut split_idx = None;
    let mut escaped = false;
    for (i, c) in token.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if c == '\\' {
            escaped = true;
            continue;
        }
        if c == '-' {
            split_idx = Some(i);
            break;
        }
    }

    if let Some(idx) = split_idx {
        let start = token[..idx].to_string();
        let end = token[idx + 1..].to_string();

        if start.is_empty() || end.is_empty() {
            return None;
        }
        Some((start, end))
    } else {
        None
    }
}

fn name_matches_pattern(name: &str, pattern: &str) -> bool {
    let name_bytes = name.as_bytes();
    let pat_bytes = pattern.as_bytes();
    let mut i = 0;
    let mut j = 0;
    let mut star_i: Option<usize> = None;
    let mut star_j: Option<usize> = None;

    while i < name_bytes.len() {
        if j < pat_bytes.len() && pat_bytes[j] == b'*' {
            star_i = Some(i);
            star_j = Some(j + 1);
            j += 1;
        } else if j < pat_bytes.len() && pat_bytes[j] == name_bytes[i] {
            i += 1;
            j += 1;
        } else if let (Some(si), Some(sj)) = (star_i, star_j) {
            i = si + 1;
            star_i = Some(i);
            j = sj;
        } else {
            return false;
        }
    }

    while j < pat_bytes.len() && pat_bytes[j] == b'*' {
        j += 1;
    }

    j == pat_bytes.len()
}

fn unescape_name_pattern(token: &str) -> (String, bool) {
    let mut out = String::new();
    let mut has_unescaped_star = false;
    let mut chars = token.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(next) = chars.next() {
                out.push(next);
            } else {
                out.push('\\');
            }
        } else {
            if c == '*' {
                has_unescaped_star = true;
            }
            out.push(c);
        }
    }

    (out, has_unescaped_star)
}

pub fn parse_field_list_with_header(
    spec: &str,
    header: Option<&Header>,
    _delimiter: char,
) -> Result<Vec<usize>, String> {
    let trimmed = spec.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    let mut indices: Vec<usize> = Vec::new();

    for part in tokenize_field_spec(trimmed) {
        // tokenize_field_spec handles splitting by comma and respects escaping.
        let token = part.trim();
        if token.is_empty() {
            return Err(format!("empty field list element in `{}`", spec));
        }

        let is_numeric_like = !token.starts_with('\\')
            && token.chars().all(|c| c.is_ascii_digit() || c == '-');

        if is_numeric_like {
            // Handle reverse ranges like "6-4" by swapping them to "4-6"
            let mut token_str = token.to_string();
            if let Some((s, e)) = token_str.split_once('-') {
                if let (Ok(start), Ok(end)) = (s.parse::<usize>(), e.parse::<usize>()) {
                    if start > end {
                        token_str = format!("{}-{}", end, start);
                    }
                }
            }

            let intspan = IntSpan::from(&token_str);
            for e in intspan.elements() {
                if e <= 0 {
                    return Err(format!("field index must be >= 1 in `{}`", spec));
                }
                indices.push(e as usize);
            }
        } else {
            match header {
                Some(h) => {
                    let (pattern, has_unescaped_star) = unescape_name_pattern(token);
                    if has_unescaped_star {
                        let mut matched = false;
                        for (idx0, name) in h.fields.iter().enumerate() {
                            if name_matches_pattern(name, &pattern) {
                                indices.push(idx0 + 1);
                                matched = true;
                            }
                        }
                        if !matched {
                            return Err(format!(
                                "unknown field name `{}` in `{}`",
                                token, spec
                            ));
                        }
                    } else if let Some((start_name, end_name)) =
                        split_name_range_token(token)
                    {
                        let start_idx0 = h.get_index(&start_name).ok_or_else(|| {
                            format!("unknown field name `{}` in `{}`", start_name, spec)
                        })?;
                        let end_idx0 = h.get_index(&end_name).ok_or_else(|| {
                            format!("unknown field name `{}` in `{}`", end_name, spec)
                        })?;
                        let (lo, hi) = if start_idx0 <= end_idx0 {
                            (start_idx0, end_idx0)
                        } else {
                            (end_idx0, start_idx0)
                        };
                        for idx0 in lo..=hi {
                            indices.push(idx0 + 1);
                        }
                    } else if let Some(idx0) = h.get_index(&pattern) {
                        indices.push(idx0 + 1);
                    } else {
                        return Err(format!(
                            "unknown field name `{}` in `{}`",
                            token, spec
                        ));
                    }
                }
                None => {
                    return Err(format!(
                        "field name `{}` requires header in `{}`",
                        token, spec
                    ));
                }
            }
        }
    }

    indices.sort_unstable();
    indices.dedup();

    Ok(indices)
}

pub fn parse_field_list_with_header_preserve_order(
    spec: &str,
    header: Option<&Header>,
    _delimiter: char,
) -> Result<Vec<usize>, String> {
    if header.is_none() {
        // If header is missing, we can only parse numeric indices.
        // If there are non-numeric tokens, we should error out with "requires header" or similar.
        // parse_numeric_field_list_preserve_order already handles numeric parsing.
        // If it encounters non-numeric, it returns "invalid numeric field spec".
        // We need to catch that and see if it looks like a name.
        // Or better: iterate tokens here.
        let mut indices = Vec::new();
        for token in tokenize_field_spec(spec) {
            if token.is_empty() {
                return Err(format!("empty field list element in `{}`", spec));
            }
            if let Ok(idx) = token.parse::<usize>() {
                if idx == 0 {
                    return Err(format!("field index must be >= 1 in `{}`", spec));
                }
                if idx > 1048576 {
                    return Err(format!(
                        "Maximum allowed '--e|exclude' field number is 1048576."
                    ));
                }
                indices.push(idx);
            } else if let Some((start_str, end_str)) = split_name_range_token(&token) {
                // Range logic for numeric ranges (e.g. 1-3) is handled here too?
                // split_name_range_token splits on '-'.
                // "1-3" -> start="1", end="3".
                if let (Ok(start), Ok(end)) =
                    (start_str.parse::<usize>(), end_str.parse::<usize>())
                {
                    if start == 0 || end == 0 {
                        return Err(format!("field index must be >= 1 in `{}`", spec));
                    }
                    if start <= end {
                        for i in start..=end {
                            indices.push(i);
                        }
                    } else {
                        let mut i = start;
                        while i >= end {
                            indices.push(i);
                            i -= 1;
                        }
                    }
                } else {
                    return Err(format!(
                        "field name `{}` requires header in `{}`",
                        token, spec
                    ));
                }
            } else if token.ends_with('-') {
                return Err(format!(
                    "Incomplete ranges are not supported: '{}'.",
                    token
                ));
            } else if token.starts_with('-') {
                // Might be negative number or range starting with empty?
                // tsv-select: "Field numbers must be greater than zero" or "Incomplete ranges"
                // If token is like "-2", parse::<usize> fails.
                // split_name_range_token("-2") -> start="", end="2". Returns None.
                // So we fall here.
                // Check if it is a number.
                if token.parse::<isize>().is_ok() {
                    return Err(format!("field index must be >= 1 in `{}`", spec));
                }
                return Err(format!(
                    "field name `{}` requires header in `{}`",
                    token, spec
                ));
            } else {
                return Err(format!(
                    "field name `{}` requires header in `{}`",
                    token, spec
                ));
            }
        }
        return Ok(indices);
    }
    let header = header.unwrap();
    let mut indices = Vec::new();

    for token in tokenize_field_spec(spec) {
        // tokenize_field_spec handles splitting by ',' and respects escaped commas.
        // However, it does not unescape other characters.

        // Try parsing as usize first (simple index)
        if let Ok(idx) = token.parse::<usize>() {
            if idx == 0 {
                return Err(format!("field index must be >= 1 in `{}`", spec));
            }
            if idx > 1048576 {
                return Err(format!(
                    "Maximum allowed '--e|exclude' field number is 1048576."
                ));
            }
            indices.push(idx);
        } else if let Some((start_str, end_str)) = split_name_range_token(&token) {
            let (start_unescaped, _) = unescape_name_pattern(&start_str);
            let (end_unescaped, _) = unescape_name_pattern(&end_str);

            let start_idx = if let Ok(idx) = start_unescaped.parse::<usize>() {
                if idx == 0 {
                    return Err(format!("field index must be >= 1 in `{}`", spec));
                }
                idx
            } else {
                header.get_index(&start_unescaped).map(|i| i + 1).ok_or_else(|| {
                    format!("First field in range not found in file header. Range: '{}'. Not specifying a range? Backslash escape any hyphens in the field name.", token)
                })?
            };

            let end_idx = if let Ok(idx) = end_unescaped.parse::<usize>() {
                if idx == 0 {
                    return Err(format!("field index must be >= 1 in `{}`", spec));
                }
                idx
            } else {
                // Check if end_str matches header fields.
                // Currently only supports exact match for range end.
                header.get_index(&end_unescaped).map(|i| i + 1).ok_or_else(|| {
                    format!("Second field in range not found in file header. Range: '{}'. Not specifying a range? Backslash escape any hyphens in the field name.", token)
                })?
            };

            if start_idx <= end_idx {
                for i in start_idx..=end_idx {
                    indices.push(i);
                }
            } else {
                let mut i = start_idx;
                while i >= end_idx {
                    indices.push(i);
                    i -= 1;
                }
            }
        } else if token.ends_with('-') {
            return Err(format!("Incomplete ranges are not supported: '{}'.", token));
        } else {
            // Name or Wildcard
            let (pattern, has_wildcard) = unescape_name_pattern(&token);
            match header.get_index(&pattern) {
                Some(idx) => {
                    indices.push(idx + 1);
                }
                None => {
                    if has_wildcard {
                        let mut found = false;
                        for (i, field) in header.fields.iter().enumerate() {
                            if name_matches_pattern(field, &pattern) {
                                indices.push(i + 1);
                                found = true;
                            }
                        }
                        if !found {
                            return Err(format!(
                                "Field not found in file header: '{}'",
                                token
                            ));
                        }
                    } else {
                        // Check if it looks like a number <= 0
                        if token.parse::<isize>().is_ok() {
                            return Err(format!(
                                "field name `{}` requires header in `{}`",
                                token, spec
                            ));
                        }

                        // Handle special case for incomplete ranges starting with -
                        if token.starts_with('-') {
                            return Err(format!(
                                "Incomplete ranges are not supported: '{}'.",
                                token
                            ));
                        }

                        // If not found, report Field not found in file header
                        return Err(format!(
                            "Field not found in file header: '{}'",
                            token
                        ));
                    }
                }
            }
        }
    }

    Ok(indices)
}

#[derive(Clone)]
pub struct Header {
    pub fields: Vec<String>,
    pub index_by_name: HashMap<String, usize>,
}

impl Header {
    pub fn from_fields(fields: Vec<String>) -> Header {
        let mut index_by_name = HashMap::new();
        for (idx, name) in fields.iter().enumerate() {
            index_by_name.entry(name.clone()).or_insert(idx);
        }
        Header {
            fields,
            index_by_name,
        }
    }

    pub fn from_line(line: &str, delimiter: char) -> Header {
        let fields: Vec<String> = if line.is_empty() {
            Vec::new()
        } else {
            line.split(delimiter).map(|s| s.to_string()).collect()
        };
        Header::from_fields(fields)
    }

    pub fn get_index(&self, name: &str) -> Option<usize> {
        self.index_by_name.get(name).copied()
    }
}

/// Resolves field specifications using column names from raw bytes.
/// This is a convenience function that combines Header creation and field parsing.
/// Returns 1-based indices (suitable for use with TsvRow::get_bytes).
pub fn resolve_fields_from_header(
    spec: &str,
    column_names_bytes: &[u8],
    delimiter: char,
) -> Result<Vec<usize>, String> {
    let header_str = std::str::from_utf8(column_names_bytes)
        .map_err(|e| format!("invalid UTF-8 in header: {}", e))?;
    let header = Header::from_line(header_str, delimiter);
    parse_field_list_with_header_preserve_order(spec, Some(&header), delimiter)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_numeric_field_list_basic() {
        let v = parse_numeric_field_list("1,3-5").unwrap();
        assert_eq!(v, vec![1, 3, 4, 5]);
    }

    #[test]
    fn test_parse_numeric_field_list_whitespace_and_dup() {
        let v = parse_numeric_field_list(" 2 , 2 , 4-5 ").unwrap();
        assert_eq!(v, vec![2, 4, 5]);
    }

    #[test]
    fn test_parse_numeric_field_list_empty() {
        let v = parse_numeric_field_list("   ").unwrap();
        assert!(v.is_empty());
    }

    #[test]
    fn test_header_from_line_empty() {
        let h = Header::from_line("", '\t');
        assert!(h.fields.is_empty());
        assert!(h.index_by_name.is_empty());
    }

    #[test]
    fn test_parse_field_list_with_header_unknown_field_pattern() {
        let h = Header::from_line("f1\tf2", '\t');
        let res = parse_field_list_with_header("f3*", Some(&h), '\t');
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), "unknown field name `f3*` in `f3*`");
    }

    #[test]
    fn test_parse_field_list_with_header_unknown_start_range() {
        let h = Header::from_line("f1\tf2\tf3", '\t');
        let res = parse_field_list_with_header("x-f2", Some(&h), '\t');
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), "unknown field name `x` in `x-f2`");
    }

    #[test]
    fn test_parse_field_list_with_header_unknown_end_range() {
        let h = Header::from_line("f1\tf2\tf3", '\t');
        let res = parse_field_list_with_header("f1-x", Some(&h), '\t');
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), "unknown field name `x` in `f1-x`");
    }

    #[test]
    fn test_parse_field_list_with_header_preserve_order_numeric_zero_or_negative() {
        let res = parse_field_list_with_header_preserve_order("0", None, '\t');
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), "field index must be >= 1 in `0`");

        // The logic for negative numbers is:
        // token.parse::<usize>() fails.
        // split_name_range_token("-1") returns None (because start is empty).
        // token.starts_with('-') is true.
        // token.parse::<isize>() -> Ok(-1).
        let res = parse_field_list_with_header_preserve_order("-1", None, '\t');
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), "field index must be >= 1 in `-1`");
    }

    #[test]
    fn test_parse_field_list_with_header_preserve_order_no_header_name_error() {
        let res = parse_field_list_with_header_preserve_order("f1", None, '\t');
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), "field name `f1` requires header in `f1`");

        let res =
            parse_field_list_with_header_preserve_order("invalid-range", None, '\t');
        assert!(res.is_err());
        // This hits "field name requires header" in the no-header branch because "invalid-range" doesn't parse as usize range
        assert_eq!(
            res.unwrap_err(),
            "field name `invalid-range` requires header in `invalid-range`"
        );
    }

    #[test]
    fn test_parse_field_list_with_header_preserve_order_range_errors() {
        let h = Header::from_line("f1\tf2\tf3", '\t');

        // Start index 0
        let res = parse_field_list_with_header_preserve_order("0-f2", Some(&h), '\t');
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), "field index must be >= 1 in `0-f2`");

        // Start name not found
        let res = parse_field_list_with_header_preserve_order("x-f2", Some(&h), '\t');
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("First field in range not found"));

        // End index 0
        let res = parse_field_list_with_header_preserve_order("f1-0", Some(&h), '\t');
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), "field index must be >= 1 in `f1-0`");

        // End name not found (Wait, end_idx logic uses unescape_name_pattern too)
        // If end_unescaped parses as usize, it uses it.
        // Else it looks up in header.
        let res = parse_field_list_with_header_preserve_order("f1-x", Some(&h), '\t');
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("Second field in range not found"));
    }

    #[test]
    fn test_parse_field_list_with_header_preserve_order_name_requires_header_error() {
        // This covers the case where header is Some, but we encounter a name that looks like a number <= 0 (e.g. "-5")
        // But wait, if header is Some, we go to the second main block.
        // token "-5"
        // parse::<usize> fails.
        // split_name_range_token("-5") -> None.
        // ends_with('-') -> false.
        // Unescape name pattern "-5". has_wildcard=false.
        // get_index("-5") -> None.
        // parse::<isize>("-5") -> Ok.
        // Returns "field name `-5` requires header..." -> Wait, the error message in code is "field name `{}` requires header in `{}`".
        // But we HAVE a header. This error message seems misleading in the context of "header is present but token looks like a negative number".
        // Actually, looking at L518-522:
        // if let Ok(_) = token.parse::<isize>() { return Err(...) }
        // The message says "requires header". This implies it thinks it's a name, but maybe it should say "invalid field index"?
        // Or maybe this logic is for when we failed to find it as a name, and it looks like a number, so we assume the user meant a number but it's invalid.
        // But the error message is specific.

        let h = Header::from_line("f1", '\t');
        let res = parse_field_list_with_header_preserve_order("-5", Some(&h), '\t');
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), "field name `-5` requires header in `-5`");
    }

    #[test]
    fn test_name_matches_pattern_trailing_stars() {
        // Matches "abc" against "abc**"
        let h = Header::from_line("abc", '\t');
        let res = parse_field_list_with_header("abc**", Some(&h), '\t');
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![1]);
    }

    #[test]
    fn parse_field_list_with_header_numeric_and_names() {
        let header = Header::from_line("a\tb\tc", '\t');
        let v = parse_field_list_with_header("1,c", Some(&header), '\t').unwrap();
        assert_eq!(v, vec![1, 3]);
    }

    #[test]
    fn parse_field_list_with_header_name_range() {
        let header = Header::from_line(
            "run\telapsed_time\tuser_time\tsystem_time\tmax_memory",
            '\t',
        );
        let v =
            parse_field_list_with_header("run-user_time", Some(&header), '\t').unwrap();
        assert_eq!(v, vec![1, 2, 3]);
    }

    #[test]
    fn parse_field_list_with_header_wildcard_basic() {
        let header = Header::from_line(
            "run\telapsed_time\tuser_time\tsystem_time\tmax_memory",
            '\t',
        );
        let v = parse_field_list_with_header("*_time", Some(&header), '\t').unwrap();
        assert_eq!(v, vec![2, 3, 4]);
    }

    #[test]
    fn parse_field_list_with_header_wildcard_preserve_order() {
        let header = Header::from_line(
            "run\telapsed_time\tuser_time\tsystem_time\tmax_memory",
            '\t',
        );
        let v = parse_field_list_with_header_preserve_order(
            "*_time,run",
            Some(&header),
            '\t',
        )
        .unwrap();
        assert_eq!(v, vec![2, 3, 4, 1]);
    }

    #[test]
    fn parse_field_list_with_header_name_range_preserve_order() {
        let header = Header::from_line(
            "run\telapsed_time\tuser_time\tsystem_time\tmax_memory",
            '\t',
        );
        let v = parse_field_list_with_header_preserve_order(
            "run-user_time",
            Some(&header),
            '\t',
        )
        .unwrap();
        assert_eq!(v, vec![1, 2, 3]);
    }

    #[test]
    fn parse_field_list_with_header_preserve_order_and_duplicates() {
        let header = Header::from_line("a\tb\tc", '\t');
        let v =
            parse_field_list_with_header_preserve_order("c,1,c", Some(&header), '\t')
                .unwrap();
        assert_eq!(v, vec![3, 1, 3]);
    }

    #[test]
    fn parse_field_list_with_header_preserve_order_numeric_only() {
        let header = Header::from_line("a\tb\tc", '\t');
        let v =
            parse_field_list_with_header_preserve_order("1,3-4", Some(&header), '\t')
                .unwrap();
        assert_eq!(v, vec![1, 3, 4]);
    }

    #[test]
    fn parse_field_list_with_header_unknown_name() {
        let header = Header::from_line("a\tb\tc", '\t');
        let err = parse_field_list_with_header("d", Some(&header), '\t').unwrap_err();
        assert!(err.contains("unknown field name"));
    }

    #[test]
    fn parse_field_list_with_header_special_char_escapes() {
        let header = Header::from_line("test id\trun:id\ttime-stamp\t001\t100", '\t');
        let v = parse_field_list_with_header(
            r"test\ id,run\:id,time\-stamp,\001,\100",
            Some(&header),
            '\t',
        )
        .unwrap();
        assert_eq!(v, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn parse_field_list_with_header_requires_header_for_name() {
        let err = parse_field_list_with_header("name", None, '\t').unwrap_err();
        assert!(err.contains("requires header"));
    }

    #[test]
    fn header_from_line_basic() {
        let h = Header::from_line("a\tb\tc", '\t');
        assert_eq!(h.fields, vec!["a", "b", "c"]);
        assert_eq!(h.get_index("a"), Some(0));
        assert_eq!(h.get_index("b"), Some(1));
        assert_eq!(h.get_index("c"), Some(2));
        assert_eq!(h.get_index("d"), None);
    }

    #[test]
    fn header_from_line_empty() {
        let h = Header::from_line("", '\t');
        assert!(h.fields.is_empty());
        assert!(h.index_by_name.is_empty());
    }

    #[test]
    fn test_parse_numeric_field_list_error_empty_element() {
        let err = parse_numeric_field_list("1,,2").unwrap_err();
        assert_eq!(err, "empty field list element in `1,,2`");

        let err = parse_numeric_field_list(",").unwrap_err();
        assert_eq!(err, "empty field list element in `,`");
    }

    #[test]
    fn test_parse_numeric_field_list_error_zero_index() {
        let err = parse_numeric_field_list("0").unwrap_err();
        assert_eq!(err, "field index must be >= 1 in `0`");

        let err = parse_numeric_field_list("1,0,2").unwrap_err();
        assert_eq!(err, "field index must be >= 1 in `1,0,2`");
    }

    #[test]
    fn test_parse_field_list_with_header_error_empty_element() {
        let header = Header::from_line("a\tb\tc", '\t');
        let err = parse_field_list_with_header("1,,c", Some(&header), '\t').unwrap_err();
        assert_eq!(err, "empty field list element in `1,,c`");
    }

    #[test]
    fn test_parse_field_list_with_header_error_zero_index() {
        let header = Header::from_line("a\tb\tc", '\t');
        let err = parse_field_list_with_header("0,c", Some(&header), '\t').unwrap_err();
        assert_eq!(err, "field index must be >= 1 in `0,c`");
    }

    #[test]
    fn test_tokenize_field_spec_trailing_backslash() {
        let tokens = tokenize_field_spec(r"col1,col2\");
        assert_eq!(tokens, vec!["col1", r"col2\"]);
    }

    #[test]
    fn test_split_name_range_token_trailing_backslash() {
        let res = split_name_range_token(r"start-end\");
        assert_eq!(res, Some(("start".to_string(), r"end\".to_string())));
    }

    #[test]
    fn test_split_name_range_token_escaped_dash() {
        // "start\-end" should be treated as a single name "start-end", not a range
        let res = split_name_range_token(r"start\-end");
        assert_eq!(res, None);
    }

    #[test]
    fn test_split_name_range_token_trailing_backslash_in_start() {
        // "start\\-end" -> The dash is not escaped (it is preceded by escaped backslash),
        // so it splits into "start\\" and "end".
        let res = split_name_range_token(r"start\\-end");
        assert_eq!(res, Some((r"start\\".to_string(), "end".to_string())));
    }

    #[test]
    fn test_split_name_range_token_empty_parts() {
        assert_eq!(split_name_range_token("-"), None);
        assert_eq!(split_name_range_token("start-"), None);
        assert_eq!(split_name_range_token("-end"), None);
    }

    #[test]
    fn test_unescape_name_pattern_trailing_backslash() {
        let (res, has_star) = unescape_name_pattern(r"abc\");
        assert_eq!(res, r"abc\");
        assert!(!has_star);
    }

    #[test]
    fn test_name_matches_pattern_complex() {
        assert!(name_matches_pattern("foobar", "*bar"));
        assert!(name_matches_pattern("foobar", "foo*"));
        assert!(name_matches_pattern("foobar", "*ooba*"));
        assert!(name_matches_pattern("foobar", "f*b*r"));
        assert!(!name_matches_pattern("foobar", "f*b*z"));

        // Backtracking test
        assert!(name_matches_pattern("mississippi", "*sip*"));
        assert!(name_matches_pattern("abacadae", "*a*e"));
    }

    #[test]
    fn test_parse_numeric_field_list_reverse_range() {
        let v = parse_numeric_field_list("5-3").unwrap();
        assert_eq!(v, vec![3, 4, 5]);
    }

    #[test]
    fn test_parse_field_list_with_header_reverse_name_range() {
        let header = Header::from_line("a\tb\tc", '\t');
        let v = parse_field_list_with_header("c-a", Some(&header), '\t').unwrap();
        assert_eq!(v, vec![1, 2, 3]);
    }

    #[test]
    fn test_parse_numeric_field_list_preserve_order_basic() {
        let v = parse_numeric_field_list_preserve_order("1,3-5").unwrap();
        assert_eq!(v, vec![1, 3, 4, 5]);
    }

    #[test]
    fn test_parse_numeric_field_list_preserve_order_reverse() {
        let v = parse_numeric_field_list_preserve_order("5-3").unwrap();
        assert_eq!(v, vec![5, 4, 3]);
    }

    #[test]
    fn test_parse_numeric_field_list_preserve_order_mixed() {
        let v = parse_numeric_field_list_preserve_order("2,4-5,1").unwrap();
        assert_eq!(v, vec![2, 4, 5, 1]);
    }

    #[test]
    fn test_parse_numeric_field_list_preserve_order_error_zero() {
        let err = parse_numeric_field_list_preserve_order("0").unwrap_err();
        assert!(err.contains("field index must be >= 1"));
    }

    #[test]
    fn test_parse_numeric_field_list_preserve_order_error_empty() {
        let err = parse_numeric_field_list_preserve_order(",").unwrap_err();
        assert!(err.contains("empty field list element"));
    }

    #[test]
    fn test_parse_numeric_field_list_preserve_order_error_invalid() {
        let err = parse_numeric_field_list_preserve_order("a").unwrap_err();
        assert!(err.contains("invalid numeric field spec"));
    }

    #[test]
    fn test_fields_to_ints() {
        let span = fields_to_ints("1,3-5");
        assert_eq!(span.to_string(), "1,3-5");
    }

    #[test]
    fn test_fields_to_idx() {
        let v = fields_to_idx("1,3-5");
        assert_eq!(v, vec![1, 3, 4, 5]);
    }

    #[test]
    fn test_parse_field_list_with_header_preserve_order_no_header() {
        let v =
            parse_field_list_with_header_preserve_order("3,1-2", None, '\t').unwrap();
        assert_eq!(v, vec![3, 1, 2]);
    }

    #[test]
    fn test_parse_field_list_with_header_preserve_order_no_header_reverse() {
        let v = parse_field_list_with_header_preserve_order("3-1", None, '\t').unwrap();
        assert_eq!(v, vec![3, 2, 1]);
    }

    #[test]
    fn test_parse_field_list_with_header_preserve_order_no_header_error() {
        let err =
            parse_field_list_with_header_preserve_order("a", None, '\t').unwrap_err();
        assert!(err.contains("requires header"));
    }

    #[test]
    fn test_parse_numeric_field_list_preserve_order_limit() {
        let err = parse_numeric_field_list_preserve_order("1048577").unwrap_err();
        assert!(err.contains("Maximum allowed '--e|exclude' field number is 1048576"));
    }

    #[test]
    fn test_parse_numeric_field_list_preserve_order_negative() {
        let err = parse_numeric_field_list_preserve_order("-1").unwrap_err();
        // The current implementation might parse this as a range "-1" with no second part
        // But since split_once('-') splits on first dash, "-1" -> ("", "1").
        // parse::<i32>("") fails.
        // It falls through to parse::<i32>("-1"), which is Ok(-1).
        // Then checks n <= 0.
        assert!(err.contains("field index must be >= 1"));
    }

    #[test]
    fn test_parse_numeric_field_list_preserve_order_range_negative() {
        let err = parse_numeric_field_list_preserve_order("1--1").unwrap_err();
        // "1--1" -> "1" and "-1". parse::<i32>("-1") is -1.
        // Checks end <= 0.
        assert!(err.contains("field index must be >= 1"));
    }

    #[test]
    fn test_parse_field_list_with_header_preserve_order_reverse_name_range() {
        let header = Header::from_line("a\tb\tc", '\t');
        let v = parse_field_list_with_header_preserve_order("c-a", Some(&header), '\t')
            .unwrap();
        assert_eq!(v, vec![3, 2, 1]);
    }

    #[test]
    fn test_parse_field_list_with_header_preserve_order_incomplete_range() {
        let header = Header::from_line("a\tb\tc", '\t');
        let err = parse_field_list_with_header_preserve_order("a-", Some(&header), '\t')
            .unwrap_err();
        assert!(err.contains("Incomplete ranges are not supported"));
    }

    #[test]
    fn test_parse_field_list_with_header_preserve_order_incomplete_range_start() {
        let header = Header::from_line("a\tb\tc", '\t');
        let err = parse_field_list_with_header_preserve_order("-a", Some(&header), '\t')
            .unwrap_err();
        assert!(err.contains("Incomplete ranges are not supported"));
    }

    #[test]
    fn test_parse_field_list_with_header_preserve_order_wildcard_not_found() {
        let header = Header::from_line("a\tb\tc", '\t');
        let err = parse_field_list_with_header_preserve_order("d*", Some(&header), '\t')
            .unwrap_err();
        assert!(err.contains("Field not found in file header"));
    }

    #[test]
    fn test_parse_field_list_with_header_preserve_order_limit() {
        let header = Header::from_line("a", '\t');
        let err =
            parse_field_list_with_header_preserve_order("1048577", Some(&header), '\t')
                .unwrap_err();
        assert!(err.contains("Maximum allowed '--e|exclude' field number is 1048576"));
    }
}
