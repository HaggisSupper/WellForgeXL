//! Directed-flow continuity bookkeeping for reusable network models.

/// One directed flow edge.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FlowEdge {
    /// Source node index.
    pub from: usize,
    /// Destination node index.
    pub to: usize,
    /// Signed flow magnitude; positive values follow `from -> to`.
    pub flow: f64,
}

impl FlowEdge {
    /// Creates one directed flow edge.
    #[must_use]
    pub const fn new(from: usize, to: usize, flow: f64) -> Self {
        Self { from, to, flow }
    }
}

/// Flow-network validation failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FlowError {
    /// Node count is zero.
    Empty,
    /// An edge references a node outside the network.
    IndexOutOfBounds,
    /// An edge flow is NaN or infinite.
    NonFinite,
}

/// Computes node continuity residuals as `inflow - outflow`.
///
/// # Errors
/// Returns [`FlowError`] for an empty network, invalid node index, or non-finite flow.
pub fn continuity_residuals(
    node_count: usize,
    edges: &[FlowEdge],
) -> Result<Vec<f64>, FlowError> {
    if node_count == 0 {
        return Err(FlowError::Empty);
    }
    if edges
        .iter()
        .any(|edge| edge.from >= node_count || edge.to >= node_count)
    {
        return Err(FlowError::IndexOutOfBounds);
    }
    if edges.iter().any(|edge| !edge.flow.is_finite()) {
        return Err(FlowError::NonFinite);
    }

    let mut residuals = vec![0.0; node_count];
    for edge in edges {
        residuals[edge.from] -= edge.flow;
        residuals[edge.to] += edge.flow;
    }
    Ok(residuals)
}
