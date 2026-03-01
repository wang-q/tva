//! Field-based filter engine shared by `tva filter` and other commands.
//!
//! This module turns `<field-list>:value` specifications into test operators
//! and evaluates them on a single row of fields.
//!
//! Basic example: split a `<field-list>:value` specification:
//!
//! ```
//! use tva::libs::filter::split_spec;
//!
//! let (fields, value) = split_spec("2-3:10").unwrap();
//! assert_eq!(fields, "2-3");
//! assert_eq!(value, "10");
//! ```
//!
//! Numeric filtering on a single row:
//!
//! ```
//! use tva::libs::filter::{TestKind, NumericOp};
//! use tva::libs::tsv::record::StrSliceRow;
//!
//! let row = StrSliceRow { fields: &["id", "10.5"] };
//! let test = TestKind::NumericCmp {
//!     fields: vec![2],
//!     op: NumericOp::Gt,
//!     value: 10.0,
//! };
//!
//! assert!(test.eval_row(&row));
//! ```
//!
//! Substring matching on a single row:
//!
//! ```
//! use tva::libs::filter::TestKind;
//! use tva::libs::tsv::record::StrSliceRow;
//!
//! let row = StrSliceRow { fields: &["foo", "barbaz"] };
//! let test = TestKind::StrIn {
//!     fields: vec![2],
//!     value: "bar".to_string(),
//!     case_insensitive: false,
//!     negated: false,
//! };
//!
//! assert!(test.eval_row(&row));
//! ```

pub mod builder;
pub mod config;
pub mod engine;
pub mod runner;

// Re-export core types for convenience
pub use builder::{build_tests, split_spec};
pub use config::{FilterConfig, FilterSpecConfig, NumericOp, NumericProp};
pub use config::{
    PendingByteLen, PendingCharLen, PendingFieldFieldAbsDiff, PendingFieldFieldNumeric,
    PendingFieldFieldRelDiff, PendingFieldFieldStr, PendingNumeric, PendingNumericProp,
    PendingRegex, PendingStrCmp, PendingStrEq, PendingSubstr,
};
pub use engine::TestKind;
pub use runner::run_filter;
