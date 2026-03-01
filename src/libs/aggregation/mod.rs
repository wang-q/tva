//! Aggregation module for tva
//! 
//! This module implements the core aggregation logic using a Data-Oriented (SoA) approach
//! for performance, combined with a modular `Calculator` trait for maintainability.
//! 
//! # Architecture
//! 
//! - **StatsProcessor**: The schema manager. It analyzes the requested operations (`Operation`),
//!   calculates the memory layout, and creates `Calculator` instances.
//! - **Aggregator**: The state container. It holds flattened vectors (SoA) for all aggregation states.
//!   This ensures minimal memory overhead and CPU cache friendliness.
//! - **Calculator**: A trait implemented by each aggregation operator (Sum, Mean, Max, etc.).
//!   It encapsulates the logic of how to update the `Aggregator` state from a row.
//! 
//! # Usage
//! 
//! ```rust,ignore
//! use tva::libs::aggregation::{StatsProcessor, Operation, OpKind};
//! 
//! let ops = vec![
//!     Operation { kind: OpKind::Sum, field_idx: Some(0) },
//!     Operation { kind: OpKind::Count, field_idx: None },
//! ];
//! 
//! let processor = StatsProcessor::new(ops);
//! let mut agg = processor.create_aggregator();
//! 
//! // processor.update(&mut agg, &row);
//! // let results = processor.format_results(&agg);
//! ```

pub mod aggregator;
pub mod math;
pub mod ops;
pub mod processor;
pub mod traits;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OpKind {
    Count,
    Sum,
    Mean,
    Min,
    Max,
    Range,
    Stdev,
    Variance,
    CV,
    GeoMean,
    HarmMean,
    Median,
    Q1,
    Q3,
    IQR,
    Mad,
    First,
    Last,
    NUnique,
    Mode,
    Unique,
    Collapse,
    Rand,
}

#[derive(Debug, Clone)]
pub struct Operation {
    pub kind: OpKind,
    pub field_idx: Option<usize>,
}

// Re-export core types for convenience
pub use aggregator::Aggregator;
pub use processor::StatsProcessor;
pub use traits::Calculator;
