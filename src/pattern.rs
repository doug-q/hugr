//! Patterns for matching Hugr graphs.

use crate::{hugr::circuit_hugr::CircuitHugr, ops::LeafOp};

use portmatching::{
    constraint::{Address, WeightedAdjConstraint},
    pattern::{self, Edge},
    Pattern, TrieMatcher, WeightedPattern,
};

/// Type alias for Hugr trie matcher
pub type HugrMatcher = TrieMatcher<WeightedAdjConstraint<Option<LeafOp>>, Address, HugrPattern>;

/// A pattern for matching Hugr graphs.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct HugrPattern(WeightedPattern<Option<LeafOp>>);

impl Pattern for HugrPattern {
    type Constraint = <WeightedPattern<Option<LeafOp>> as Pattern>::Constraint;

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
    pub fn new(pattern: WeightedPattern<Option<LeafOp>>) -> Self {
        Self(pattern)
    }

    /// Create a new pattern from a [`CircuitHugr`].
    pub fn from_circuit(hugr: CircuitHugr) -> Result<Self, pattern::InvalidPattern> {
        hugr.into_pattern()
    }
}
