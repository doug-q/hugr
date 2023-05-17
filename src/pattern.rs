//! Patterns for matching Hugr graphs.

use crate::{ops::LeafOp, Hugr};

use portmatching::{
    pattern::{self, Edge},
    Pattern, WeightedPattern,
};

/// A pattern for matching Hugr graphs.
pub struct HugrPattern(WeightedPattern<LeafOp>);

impl Pattern for HugrPattern {
    type Constraint = <WeightedPattern<LeafOp> as Pattern>::Constraint;

    fn graph(&self) -> &portgraph::PortGraph {
        self.0.graph()
    }

    fn root(&self) -> portgraph::NodeIndex {
        self.0.root()
    }

    fn to_constraint(&self, e: &Edge) -> Self::Constraint {
        self.0.to_constraint(e)
    }

    fn all_lines(&self) -> Vec<Vec<Edge>> {
        self.0.all_lines()
    }
}

impl HugrPattern {
    /// Create a new HugrPattern from a WeightedPattern.
    pub fn new(pattern: WeightedPattern<LeafOp>) -> Self {
        Self(pattern)
    }

    /// Create a new pattern from a [`Hugr`].
    pub fn from_hugr(hugr: Hugr) -> Result<Self, pattern::InvalidPattern> {
        hugr.into_pattern()
    }
}
