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
