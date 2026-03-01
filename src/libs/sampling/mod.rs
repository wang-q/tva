//! Sampling algorithms for the `tva sample` command.
//!
//! This module implements various sampling strategies:
//! - **Bernoulli Sampling**: Selects each record independently with probability `p`.
//! - **Reservoir Sampling**: Selects a fixed number of records `k` from a stream of unknown length.
//! - **Weighted Reservoir Sampling**: Selects `k` records where inclusion probability is proportional to a weight.
//! - **Distinct Sampling**: Selects records based on the hash of a key, ensuring consistent selection across runs for the same key.
//! - **Other**: Random shuffle, sampling with replacement, etc.
//!
//! All samplers implement the [`Sampler`] trait.

pub mod bernoulli;
pub mod other;
pub mod reservoir;
pub mod traits;

// Re-export common types
pub use bernoulli::{BernoulliSampler, DistinctBernoulliSampler};
pub use other::{
    CompatRandomSampler, InorderSampler, ReplacementSampler, ShuffleSampler,
};
pub use reservoir::{ReservoirSampler, WeightedReservoirSampler};
pub use traits::{Sampler, WeightedItem, INV_U64_MAX_PLUS_1};
