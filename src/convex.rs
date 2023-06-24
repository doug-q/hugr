//! Convexity checking for portgraphs.
use std::collections::{BTreeMap, BTreeSet};

use bitvec::vec::BitVec;
use portgraph::{algorithms::toposort, Direction, NodeIndex, PortGraph};

#[derive(Default, Clone, Debug, PartialEq, Eq)]
enum Causal {
    #[default]
    P, // in the past
    F, // in the future
}

/// Checks whether a portgraph is convex.
///
/// Precomputes some data so that it is fast when reusing multiple times.
pub struct ConvexChecker<'g> {
    graph: &'g PortGraph,
    // The nodes in topological order
    topsort_nodes: Vec<NodeIndex>,
    // The index of a node in the topological order (the inverse of topsort_nodes)
    topsort_ind: BTreeMap<NodeIndex, usize>,
    // A temporary datastructure used during `is_convex`
    causal: Vec<Causal>,
}

impl<'g> ConvexChecker<'g> {
    /// Create a new ConvexChecker
    pub fn new(roots: impl IntoIterator<Item = NodeIndex>, graph: &'g PortGraph) -> Self {
        let topsort_nodes: Vec<_> = toposort::<BitVec>(graph, roots, Direction::Outgoing).collect();
        let flip = |(i, &n)| (n, i);
        let topsort_ind = topsort_nodes.iter().enumerate().map(flip).collect();
        let causal = vec![Causal::default(); topsort_nodes.len()];
        Self {
            graph,
            topsort_nodes,
            topsort_ind,
            causal,
        }
    }

    /// Whether the set of nodes are convex
    pub fn is_convex(&mut self, nodes: impl IntoIterator<Item = NodeIndex>) -> bool {
        let nodes: BTreeSet<_> = nodes.into_iter().map(|n| self.topsort_ind[&n]).collect();
        let min_ind = *nodes.first().unwrap();
        let max_ind = *nodes.last().unwrap();
        for ind in min_ind..=max_ind {
            let n = self.topsort_nodes[ind];
            let mut in_inds = {
                let in_neighs = self
                    .graph
                    .input_links(n)
                    .flatten()
                    .map(|p| self.graph.port_node(p).unwrap());
                in_neighs
                    .map(|n| self.topsort_ind[&n])
                    .filter(|&ind| ind >= min_ind)
            };
            if nodes.contains(&ind) {
                if in_inds.any(|ind| self.causal[ind] == Causal::F) {
                    // There is a node in the past that is also in the future!
                    return false;
                }
                self.causal[ind] = Causal::P;
            } else {
                self.causal[ind] = match in_inds
                    .any(|ind| nodes.contains(&ind) || self.causal[ind] == Causal::F)
                {
                    true => Causal::F,
                    false => Causal::P,
                };
            }
        }
        true
    }
}
