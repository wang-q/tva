//! Core TSV parsing and manipulation primitives.
//!
//! This module provides the low-level building blocks for reading, writing, and
//! manipulating TSV data. It includes:
//!
//! - **Reader**: Fast, zero-copy, buffered TSV reading.
//! - **Record**: Efficient row representation.
//! - **Fields**: Parsing of field selection specs (e.g. `1,3-5`).
//! - **Select**: High-performance field selection logic.
//! - **Split**: SIMD-accelerated line splitting.
//! - **Key**: Key extraction for grouping and joining.
//! - **Header**: Header detection and handling.
//! - **SIMD**: Hand-written SIMD parsers (SSE2 for x86_64, NEON for aarch64).

pub mod fields;
pub mod header;
pub mod key;
pub mod reader;
pub mod record;
pub mod select;
pub mod split;
pub mod simd;

// Re-export SIMD modules for backward compatibility
#[cfg(target_arch = "x86_64")]
pub use simd::sse2;

#[cfg(target_arch = "aarch64")]
pub use simd::neon;
