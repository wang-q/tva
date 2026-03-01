use super::Aggregator;
use crate::libs::tsv::record::Row;

/// A calculator handles the logic for a specific aggregation operation.
/// It separates the "what to do" from the "where to store it".
pub trait Calculator: Send + Sync {
    /// Update the aggregator state with a new row
    fn update(&self, agg: &mut Aggregator, row: &dyn Row);
    
    /// Format the final result as a string
    fn format(&self, agg: &Aggregator) -> String;
}
