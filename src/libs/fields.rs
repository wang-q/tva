//! Common field list parsing utilities shared across tva commands.
//!
//! Field lists are used to refer to columns by index or by name. The shared
//! syntax is documented in [`FIELD_SYNTAX_HELP`].
//!
//! Basic numeric-only parsing:
//!
//! ```
//! use tva::libs::fields::parse_numeric_field_list;
//!
//! let indices = parse_numeric_field_list("1,3-5").unwrap();
//! assert_eq!(indices, vec![1, 3, 4, 5]);
//! ```
//!
//! Header-aware parsing (mixing indices and names):
//!
//! ```
//! use tva::libs::fields::{Header, parse_field_list_with_header};
//!
//! let header = Header::from_line("run\tuser_time\tsystem_time", '\t');
//! let indices = parse_field_list_with_header("run,system_time", Some(&header), '\t').unwrap();
//! assert_eq!(indices, vec![1, 3]);
//! ```

use intspan::IntSpan;
use std::collections::HashMap;

pub const FIELD_SYNTAX_HELP: &str = r#"Field syntax

Field lists are used across tva commands (select, join, uniq, split, etc.) to
identify columns. They share a common syntax:

- 1-based numeric indices:
  1,3,5

- Ranges of indices:
  1-3,5-7

- Mixed lists and ranges:
  1,3-5,10

- Header-aware names (with --header):
  run,time

- Name ranges (inclusive, by header position):
  run-user_time

- Wildcards on names:
  *_time

- Escaping special characters in names (space, comma, dash, colon, star, digits):
  test\ id,run\:id,time\-stamp,\001,\100

Special rules:
- Index 0 or negative indices are not allowed.
- In header mode, numeric indices and names can be mixed.
- Without --header, only numeric indices are allowed; using names is an error.
"#;

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

pub fn fields_to_idx(spec: &str) -> Vec<usize> {
    parse_numeric_field_list(spec).unwrap()
}

fn tokenize_field_spec(spec: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut chars = spec.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(next) = chars.next() {
                current.push('\\');
                current.push(next);
            } else {
                current.push('\\');
            }
        } else if c == ',' {
            tokens.push(current.clone());
            current.clear();
        } else {
            current.push(c);
        }
    }

    tokens.push(current);
    tokens
}

fn split_name_range_token(token: &str) -> Option<(String, String)> {
    let mut start = String::new();
    let mut end = String::new();
    let mut in_end = false;
    let mut chars = token.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(next) = chars.next() {
                if in_end {
                    end.push(next);
                } else {
                    start.push(next);
                }
            } else {
                if in_end {
                    end.push('\\');
                } else {
                    start.push('\\');
                }
            }
        } else if c == '-' && !in_end {
            in_end = true;
        } else if in_end {
            end.push(c);
        } else {
            start.push(c);
        }
    }

    if !in_end {
        return None;
    }

    let start_trimmed = start.trim();
    let end_trimmed = end.trim();
    if start_trimmed.is_empty() || end_trimmed.is_empty() {
        return None;
    }

    Some((start_trimmed.to_string(), end_trimmed.to_string()))
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
    let trimmed = spec.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    let mut indices: Vec<usize> = Vec::new();

    for part in tokenize_field_spec(trimmed) {
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

    Ok(indices)
}

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
}
