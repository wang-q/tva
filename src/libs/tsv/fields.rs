//! Common field list parsing utilities shared across tva commands.
//!
//! Field lists are used to refer to columns by index or by name. The shared
//! syntax is documented in [`crate::libs::cli::FIELD_SYNTAX_HELP`].
//!
//! The primary API is [`FieldResolver`], which handles both numeric and header-based
//! field specifications while preserving order and duplicates.
//!
//! Example:
//!
//! ```
//! use tva::libs::tsv::fields::FieldResolver;
//!
//! // Without header - numeric indices only
//! let resolver = FieldResolver::new(None, '\t');
//! let indices = resolver.resolve("1,3-5").unwrap();
//! assert_eq!(indices, vec![1, 3, 4, 5]);
//!
//! // With header - supports field names
//! let resolver = FieldResolver::new(Some(b"run\tuser_time\tsystem_time".to_vec()), '\t');
//! let indices = resolver.resolve("run,system_time").unwrap();
//! assert_eq!(indices, vec![1, 3]);
//! ```

#[cfg(test)]
use intspan::IntSpan;

/// Converts field spec to IntSpan for range operations.
/// Internal helper used by tests.
#[cfg(test)]
fn fields_to_ints(s: &str) -> IntSpan {
    let mut ints = IntSpan::new();
    for p in tokenize_field_spec(s) {
        ints.add_runlist(&p);
    }

    ints
}

/// Parses a numeric field list, sorting and deduplicating the results.
///
/// Internal tool function for cases where sorted, unique indices are needed.
/// For general field resolution, use [`FieldResolver`] instead.
#[cfg(test)]
fn parse_numeric_field_list(spec: &str) -> Result<Vec<usize>, String> {
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

/// Parses numeric field list preserving order and duplicates.
///
/// Internal implementation detail - only used by tests.
#[cfg(test)]
fn parse_numeric_field_list_preserve_order(spec: &str) -> Result<Vec<usize>, String> {
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
                return Err(format!("Maximum allowed field number is 1048576."));
            }
            ints.push(n);
            continue;
        }

        return Err(format!("invalid numeric field spec: `{}`", part));
    }

    Ok(ints.iter().map(|e| *e as usize).collect())
}

/// Convenience function for tests to parse field spec and unwrap.
#[cfg(test)]
fn fields_to_idx(spec: &str) -> Vec<usize> {
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

/// Parses a field list specification with optional header context.
///
/// Internal tool function for cases where sorted, unique indices are needed.
/// For general field resolution, use [`FieldResolver`] instead.
#[cfg(test)]
fn parse_field_list_with_header(
    spec: &str,
    header: Option<&crate::libs::tsv::header::Header>,
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
                        let column_names = h.column_names_list().unwrap_or_default();
                        for (idx0, name) in column_names.iter().enumerate() {
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

/// Parses a field list specification preserving order and duplicates.
///
/// Internal implementation detail used by [`FieldResolver`].
fn parse_field_list_with_header_preserve_order(
    spec: &str,
    header: Option<&crate::libs::tsv::header::Header>,
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
                    return Err(format!("Maximum allowed field number is 1048576."));
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
                return Err(format!("Maximum allowed field number is 1048576."));
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
                        let column_names =
                            header.column_names_list().unwrap_or_default();
                        for (i, field) in column_names.iter().enumerate() {
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

/// Resolves field specifications using column names from raw bytes.
/// This is a convenience function that combines Header creation and field parsing.
/// Returns 1-based indices (suitable for use with TsvRow::get_bytes).
///
/// Internal implementation detail used by [`FieldResolver`].
fn resolve_fields_from_header(
    spec: &str,
    column_names_bytes: &[u8],
    delimiter: char,
) -> Result<Vec<usize>, String> {
    use crate::libs::tsv::header::Header;
    let header = Header::from_column_names(column_names_bytes.to_vec(), delimiter);
    parse_field_list_with_header_preserve_order(spec, Some(&header), delimiter)
}

/// Unified field resolver that handles both numeric and header-based field specifications.
///
/// This struct encapsulates the logic for resolving field specifications, automatically
/// choosing between numeric parsing and header-aware parsing based on available header data.
///
/// See module-level documentation for usage examples.
pub struct FieldResolver {
    header_bytes: Option<Vec<u8>>,
    delimiter: char,
}

impl FieldResolver {
    /// Creates a new FieldResolver.
    ///
    /// # Arguments
    /// * `header_bytes` - Optional header line bytes containing column names
    /// * `delimiter` - Field delimiter character
    pub fn new(header_bytes: Option<Vec<u8>>, delimiter: char) -> Self {
        Self {
            header_bytes,
            delimiter,
        }
    }

    /// Resolves a field specification string into 1-based indices.
    ///
    /// If header_bytes is available, supports field names and patterns.
    /// Otherwise, only numeric specifications are allowed.
    ///
    /// # Arguments
    /// * `spec` - Field specification (e.g., "1,3-5", "name,age", "col*")
    ///
    /// # Returns
    /// * `Ok(Vec<usize>)` - 1-based field indices
    /// * `Err(String)` - Error message if parsing fails
    pub fn resolve(&self, spec: &str) -> Result<Vec<usize>, String> {
        match &self.header_bytes {
            Some(bytes) => resolve_fields_from_header(spec, bytes, self.delimiter),
            None => {
                parse_field_list_with_header_preserve_order(spec, None, self.delimiter)
            }
        }
    }

    /// Returns column names if header is available.
    ///
    /// # Returns
    /// * `Some(Vec<String>)` - Column names parsed from header
    /// * `None` - If no header was provided
    pub fn column_names(&self) -> Option<Vec<String>> {
        self.header_bytes.as_ref().and_then(|bytes| {
            let s = std::str::from_utf8(bytes).ok()?;
            Some(s.split(self.delimiter).map(|f| f.to_string()).collect())
        })
    }

    /// Returns true if header information is available.
    pub fn has_header(&self) -> bool {
        self.header_bytes.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::libs::tsv::header::Header;

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
    fn test_header_from_column_names_empty() {
        let h = Header::from_column_names(b"".to_vec(), '\t');
        assert!(h.column_names_list().map(|v| v.is_empty()).unwrap_or(true));
        assert!(h.get_index("any").is_none());
    }

    #[test]
    fn test_field_resolver_unknown_field_pattern() {
        let resolver = FieldResolver::new(Some(b"f1\tf2".to_vec()), '\t');
        let res = resolver.resolve("f3*");
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("Field not found in file header"));
    }

    #[test]
    fn test_field_resolver_unknown_start_range() {
        let resolver = FieldResolver::new(Some(b"f1\tf2\tf3".to_vec()), '\t');
        let res = resolver.resolve("x-f2");
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("First field in range not found"));
    }

    #[test]
    fn test_field_resolver_unknown_end_range() {
        let resolver = FieldResolver::new(Some(b"f1\tf2\tf3".to_vec()), '\t');
        let res = resolver.resolve("f1-x");
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("Second field in range not found"));
    }

    #[test]
    fn test_field_resolver_numeric_zero_or_negative() {
        let resolver = FieldResolver::new(None, '\t');
        let res = resolver.resolve("0");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), "field index must be >= 1 in `0`");

        let res = resolver.resolve("-1");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), "field index must be >= 1 in `-1`");
    }

    #[test]
    fn test_field_resolver_no_header_name_error() {
        let resolver = FieldResolver::new(None, '\t');
        let res = resolver.resolve("f1");
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("requires header"));

        let res = resolver.resolve("invalid-range");
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("requires header"));
    }

    #[test]
    fn test_field_resolver_range_errors() {
        let resolver = FieldResolver::new(Some(b"f1\tf2\tf3".to_vec()), '\t');

        // Start index 0
        let res = resolver.resolve("0-f2");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), "field index must be >= 1 in `0-f2`");

        // Start name not found
        let res = resolver.resolve("x-f2");
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("First field in range not found"));

        // End index 0
        let res = resolver.resolve("f1-0");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), "field index must be >= 1 in `f1-0`");

        // End name not found
        let res = resolver.resolve("f1-x");
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("Second field in range not found"));
    }

    #[test]
    fn test_field_resolver_negative_number_with_header() {
        let resolver = FieldResolver::new(Some(b"f1".to_vec()), '\t');
        let res = resolver.resolve("-5");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), "field name `-5` requires header in `-5`");
    }

    #[test]
    fn test_field_resolver_trailing_stars() {
        // Matches "abc" against "abc**"
        let resolver = FieldResolver::new(Some(b"abc".to_vec()), '\t');
        let res = resolver.resolve("abc**");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![1]);
    }

    #[test]
    fn test_field_resolver_numeric_and_names() {
        let resolver = FieldResolver::new(Some(b"a\tb\tc".to_vec()), '\t');
        let v = resolver.resolve("1,c").unwrap();
        assert_eq!(v, vec![1, 3]);
    }

    #[test]
    fn test_field_resolver_name_range() {
        let resolver = FieldResolver::new(
            Some(b"run\telapsed_time\tuser_time\tsystem_time\tmax_memory".to_vec()),
            '\t',
        );
        let v = resolver.resolve("run-user_time").unwrap();
        assert_eq!(v, vec![1, 2, 3]);
    }

    #[test]
    fn test_field_resolver_wildcard_basic() {
        let resolver = FieldResolver::new(
            Some(b"run\telapsed_time\tuser_time\tsystem_time\tmax_memory".to_vec()),
            '\t',
        );
        let v = resolver.resolve("*_time").unwrap();
        assert_eq!(v, vec![2, 3, 4]);
    }

    #[test]
    fn test_field_resolver_wildcard_preserve_order() {
        let resolver = FieldResolver::new(
            Some(b"run\telapsed_time\tuser_time\tsystem_time\tmax_memory".to_vec()),
            '\t',
        );
        let v = resolver.resolve("*_time,run").unwrap();
        assert_eq!(v, vec![2, 3, 4, 1]);
    }

    #[test]
    fn test_field_resolver_name_range_preserve_order() {
        let resolver = FieldResolver::new(
            Some(b"run\telapsed_time\tuser_time\tsystem_time\tmax_memory".to_vec()),
            '\t',
        );
        let v = resolver.resolve("run-user_time").unwrap();
        assert_eq!(v, vec![1, 2, 3]);
    }

    #[test]
    fn test_field_resolver_preserve_order_and_duplicates() {
        let resolver = FieldResolver::new(Some(b"a\tb\tc".to_vec()), '\t');
        let v = resolver.resolve("c,1,c").unwrap();
        assert_eq!(v, vec![3, 1, 3]);
    }

    #[test]
    fn test_field_resolver_numeric_only_with_header() {
        let resolver = FieldResolver::new(Some(b"a\tb\tc".to_vec()), '\t');
        let v = resolver.resolve("1,3-4").unwrap();
        assert_eq!(v, vec![1, 3, 4]);
    }

    #[test]
    fn test_field_resolver_unknown_name() {
        let resolver = FieldResolver::new(Some(b"a\tb\tc".to_vec()), '\t');
        let err = resolver.resolve("d").unwrap_err();
        assert!(err.contains("Field not found in file header"));
    }

    #[test]
    fn test_field_resolver_special_char_escapes() {
        let resolver = FieldResolver::new(
            Some(b"test id\trun:id\ttime-stamp\t001\t100".to_vec()),
            '\t',
        );
        let v = resolver
            .resolve(r"test\ id,run\:id,time\-stamp,\001,\100")
            .unwrap();
        assert_eq!(v, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_field_resolver_requires_header_for_name() {
        let resolver = FieldResolver::new(None, '\t');
        let err = resolver.resolve("name").unwrap_err();
        assert!(err.contains("requires header"));
    }

    #[test]
    fn header_from_column_names_basic() {
        let h = Header::from_column_names(b"a\tb\tc".to_vec(), '\t');
        assert_eq!(h.column_names_list().unwrap(), vec!["a", "b", "c"]);
        assert_eq!(h.get_index("a"), Some(0));
        assert_eq!(h.get_index("b"), Some(1));
        assert_eq!(h.get_index("c"), Some(2));
        assert_eq!(h.get_index("d"), None);
    }

    #[test]
    fn header_from_column_names_empty() {
        let h = Header::from_column_names(b"".to_vec(), '\t');
        assert!(h.column_names_list().map(|v| v.is_empty()).unwrap_or(true));
        assert!(h.get_index("any").is_none());
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
    fn test_field_resolver_error_empty_element() {
        let resolver = FieldResolver::new(Some(b"a\tb\tc".to_vec()), '\t');
        let err = resolver.resolve("1,,c").unwrap_err();
        // FieldResolver uses resolve_fields_from_header which has different error message
        assert!(
            err.contains("Field not found in file header")
                || err.contains("empty field list element")
        );
    }

    #[test]
    fn test_field_resolver_error_zero_index() {
        let resolver = FieldResolver::new(Some(b"a\tb\tc".to_vec()), '\t');
        let err = resolver.resolve("0,c").unwrap_err();
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
    fn test_field_resolver_reverse_name_range_sorted() {
        let resolver = FieldResolver::new(Some(b"a\tb\tc".to_vec()), '\t');
        let v = resolver.resolve("c-a").unwrap();
        // FieldResolver uses resolve_fields_from_header which preserves order
        // So reverse range c-a becomes [3, 2, 1] (preserved order)
        assert_eq!(v, vec![3, 2, 1]);
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

    // FieldResolver tests - migrated from deprecated parse_field_list_with_header_preserve_order tests
    #[test]
    fn test_field_resolver_preserve_order_no_header() {
        let resolver = FieldResolver::new(None, '\t');
        let v = resolver.resolve("3,1-2").unwrap();
        assert_eq!(v, vec![3, 1, 2]);
    }

    #[test]
    fn test_field_resolver_preserve_order_no_header_reverse() {
        let resolver = FieldResolver::new(None, '\t');
        let v = resolver.resolve("3-1").unwrap();
        assert_eq!(v, vec![3, 2, 1]);
    }

    #[test]
    fn test_field_resolver_preserve_order_no_header_error() {
        let resolver = FieldResolver::new(None, '\t');
        let err = resolver.resolve("a").unwrap_err();
        // Without header, field names require header
        assert!(err.contains("requires header"));
    }

    #[test]
    fn test_field_resolver_preserve_order_limit() {
        let resolver = FieldResolver::new(None, '\t');
        let err = resolver.resolve("1048577").unwrap_err();
        assert!(err.contains("Maximum allowed field number is 1048576"));
    }

    #[test]
    fn test_field_resolver_preserve_order_negative() {
        let resolver = FieldResolver::new(None, '\t');
        let err = resolver.resolve("-1").unwrap_err();
        assert!(err.contains("field index must be >= 1"));
    }

    #[test]
    fn test_field_resolver_preserve_order_range_negative() {
        let resolver = FieldResolver::new(None, '\t');
        let err = resolver.resolve("1--1").unwrap_err();
        // With parse_field_list_with_header_preserve_order, "1--1" is parsed as range "1-" with trailing "-1"
        // or as incomplete range. The error message depends on the parsing logic.
        assert!(
            err.contains("field index must be >= 1") || err.contains("requires header")
        );
    }

    #[test]
    fn test_field_resolver_preserve_order_reverse_name_range() {
        let resolver = FieldResolver::new(Some(b"a\tb\tc".to_vec()), '\t');
        let v = resolver.resolve("c-a").unwrap();
        assert_eq!(v, vec![3, 2, 1]);
    }

    #[test]
    fn test_field_resolver_preserve_order_name_range_three_fields() {
        // Test case for stats.rs tsv_utils_test_50_group_by_names
        // Header: color pattern length width height
        // length-height should resolve to length(3), width(4), height(5)
        let resolver = FieldResolver::new(
            Some(b"color\tpattern\tlength\twidth\theight".to_vec()),
            '\t',
        );
        let v = resolver.resolve("length-height").unwrap();
        assert_eq!(v, vec![3, 4, 5]);
    }

    #[test]
    fn test_field_resolver_preserve_order_incomplete_range() {
        let resolver = FieldResolver::new(Some(b"a\tb\tc".to_vec()), '\t');
        let err = resolver.resolve("a-").unwrap_err();
        assert!(err.contains("Incomplete ranges are not supported"));
    }

    #[test]
    fn test_field_resolver_preserve_order_incomplete_range_start() {
        let resolver = FieldResolver::new(Some(b"a\tb\tc".to_vec()), '\t');
        let err = resolver.resolve("-a").unwrap_err();
        assert!(err.contains("Incomplete ranges are not supported"));
    }

    #[test]
    fn test_field_resolver_preserve_order_wildcard_not_found() {
        let resolver = FieldResolver::new(Some(b"a\tb\tc".to_vec()), '\t');
        let err = resolver.resolve("d*").unwrap_err();
        assert!(err.contains("Field not found in file header"));
    }

    #[test]
    fn test_field_resolver_preserve_order_with_header_limit() {
        let resolver = FieldResolver::new(Some(b"a".to_vec()), '\t');
        let err = resolver.resolve("1048577").unwrap_err();
        assert!(err.contains("Maximum allowed field number is 1048576"));
    }

    // FieldResolver tests
    #[test]
    fn test_field_resolver_with_header() {
        let header_bytes = b"name\tage\tcity".to_vec();
        let resolver = FieldResolver::new(Some(header_bytes), '\t');

        // Test field name resolution
        let indices = resolver.resolve("name,city").unwrap();
        assert_eq!(indices, vec![1, 3]);

        // Test numeric resolution with header
        let indices = resolver.resolve("1,3").unwrap();
        assert_eq!(indices, vec![1, 3]);

        // Test range
        let indices = resolver.resolve("1-3").unwrap();
        assert_eq!(indices, vec![1, 2, 3]);
    }

    #[test]
    fn test_field_resolver_without_header() {
        let resolver = FieldResolver::new(None, '\t');

        // Test numeric resolution without header
        let indices = resolver.resolve("1,3").unwrap();
        assert_eq!(indices, vec![1, 3]);

        // Test range
        let indices = resolver.resolve("1-3").unwrap();
        assert_eq!(indices, vec![1, 2, 3]);

        // Field names should fail without header
        let result = resolver.resolve("name");
        assert!(result.is_err());
    }

    #[test]
    fn test_field_resolver_column_names() {
        let header_bytes = b"name\tage\tcity".to_vec();
        let resolver = FieldResolver::new(Some(header_bytes), '\t');

        let names = resolver.column_names().unwrap();
        assert_eq!(names, vec!["name", "age", "city"]);
    }

    #[test]
    fn test_field_resolver_no_column_names() {
        let resolver = FieldResolver::new(None, '\t');
        assert!(resolver.column_names().is_none());
    }

    #[test]
    fn test_field_resolver_has_header() {
        let with_header = FieldResolver::new(Some(b"a\tb".to_vec()), '\t');
        assert!(with_header.has_header());

        let without_header = FieldResolver::new(None, '\t');
        assert!(!without_header.has_header());
    }

    #[test]
    fn test_field_resolver_wildcard() {
        let header_bytes = b"run\tuser_time\tsystem_time".to_vec();
        let resolver = FieldResolver::new(Some(header_bytes), '\t');

        let indices = resolver.resolve("*_time").unwrap();
        assert_eq!(indices, vec![2, 3]);
    }
}
