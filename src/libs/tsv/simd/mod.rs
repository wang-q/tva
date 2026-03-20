//! SIMD-accelerated TSV parsing implementations.
//!
//! This module provides architecture-specific SIMD implementations for
//! high-performance TSV delimiter searching.
//!
//! # Available Implementations
//!
//! - **SSE2** (`sse2`): x86_64 128-bit SIMD (all x86_64 CPUs)
//! - **NEON** (`neon`): ARM aarch64 128-bit SIMD (all aarch64 CPUs)
//!
//! # Performance
//!
//! Benchmarks show these implementations achieve ~6.5 GiB/s throughput,
//! which is ~670% faster than the standard two-pass approach.

#[cfg(target_arch = "x86_64")]
pub mod sse2;

#[cfg(target_arch = "aarch64")]
pub mod neon;

/// Trait for SIMD delimiter searchers.
///
/// This trait abstracts over SSE2 and NEON implementations, allowing
/// platform-agnostic code to use SIMD acceleration.
///
/// Note: CR (`\r`) is not searched directly. Instead, when a newline is found,
/// the caller should check if the preceding byte is CR and handle it accordingly.
pub trait DelimiterSearcher {
    /// Creates a new searcher for the given delimiter characters.
    ///
    /// # Safety
    ///
    /// This function is safe on platforms where the SIMD implementation
    /// is supported (SSE2 on x86_64, NEON on aarch64).
    unsafe fn new(tab: u8, newline: u8) -> Self
    where
        Self: Sized;

    /// Returns an iterator over all delimiter positions in the haystack.
    fn search<'a>(&'a self, haystack: &'a [u8]) -> impl Iterator<Item = usize>;
}

/// Re-export the appropriate SIMD searcher for the current architecture.
#[cfg(target_arch = "x86_64")]
pub use sse2::Sse2Searcher as SimdSearcher;

#[cfg(target_arch = "aarch64")]
pub use neon::NeonSearcher as SimdSearcher;

/// Check if SIMD acceleration is available on this platform.
#[inline]
pub fn is_simd_available() -> bool {
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    {
        true
    }
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    {
        false
    }
}
