//! Deterministic piecewise interval partitioning.

/// Closed-open numerical interval except that the final endpoint may be queried exactly.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Interval {
    /// Interval start coordinate.
    pub start: f64,
    /// Interval end coordinate.
    pub end: f64,
}

/// Interval construction failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IntervalError {
    /// A boundary value is NaN or infinite.
    NonFiniteBoundary,
    /// A supplied boundary set decreases.
    DecreasingBoundarySet,
    /// Fewer than two unique boundaries remain after merging.
    InsufficientBoundaries,
}

fn canonical_zero(value: f64) -> f64 {
    if value == 0.0 { 0.0 } else { value }
}

/// Merges ordered boundary sets and returns strictly positive-length intervals.
///
/// Duplicate boundaries are collapsed deterministically. Each individual boundary set may contain
/// duplicates but must otherwise be non-decreasing.
///
/// # Errors
/// Returns [`IntervalError::NonFiniteBoundary`] for NaN/infinite values,
/// [`IntervalError::DecreasingBoundarySet`] when an input set decreases, and
/// [`IntervalError::InsufficientBoundaries`] when fewer than two unique boundaries remain.
pub fn partition_boundaries(boundary_sets: &[&[f64]]) -> Result<Vec<Interval>, IntervalError> {
    let mut boundaries = Vec::new();
    for set in boundary_sets {
        if set.iter().any(|value| !value.is_finite()) {
            return Err(IntervalError::NonFiniteBoundary);
        }
        if set.windows(2).any(|pair| pair[1] < pair[0]) {
            return Err(IntervalError::DecreasingBoundarySet);
        }
        boundaries.extend(set.iter().copied().map(canonical_zero));
    }

    boundaries.sort_by(f64::total_cmp);
    boundaries.dedup_by(|left, right| *left == *right);
    if boundaries.len() < 2 {
        return Err(IntervalError::InsufficientBoundaries);
    }

    Ok(boundaries
        .windows(2)
        .filter_map(|pair| {
            (pair[1] > pair[0]).then_some(Interval {
                start: pair[0],
                end: pair[1],
            })
        })
        .collect())
}
