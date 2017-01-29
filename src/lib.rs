#![feature(conservative_impl_trait)]
#![feature(field_init_shorthand)]
#![feature(rustc_private)]

extern crate rustc_data_structures;

use rustc_data_structures::graph::{Graph, NodeIndex};
use rustc_data_structures::unify::UnificationTable;
use std::fmt::Debug;

#[cfg(test)]
#[macro_use]
mod test_macro;

mod classify;
use self::classify::Classify;

mod dag_id;
use self::dag_id::DagId;

pub struct GraphReduce<'g, N> where N: 'g + Debug {
    in_graph: &'g Graph<N, ()>,
    start_nodes: &'g [NodeIndex],
    unify: UnificationTable<DagId>,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct DagNode {
    in_index: NodeIndex
}

impl<'g, N> GraphReduce<'g, N>
    where N: Debug
{
    pub fn new(in_graph: &'g Graph<N, ()>, start_nodes: &'g [NodeIndex]) -> Self {
        let mut unify = UnificationTable::new();

        // create a set of unification keys whose indices
        // correspond to the indices from the input graph
        for i in 0..in_graph.len_nodes() {
            let k = unify.new_key(());
            assert!(k == DagId::from_in_index(NodeIndex(i)));
        }

        GraphReduce { in_graph, unify, start_nodes }
    }

    pub fn compute(&mut self) {
        let _cross_targets = Classify::new(self).walk();
    }

    fn inputs(&self, in_node: NodeIndex) -> impl Iterator<Item = NodeIndex> + 'g {
        self.in_graph.predecessor_nodes(in_node)
    }

    fn mark_cycle(&mut self, in_node1: NodeIndex, in_node2: NodeIndex) {
        let dag_id1 = DagId::from_in_index(in_node1);
        let dag_id2 = DagId::from_in_index(in_node2);
        self.unify.union(dag_id1, dag_id2);
    }

    /// Convert a dag-id into its cycle head representative. This will
    /// be a no-op unless `in_node` participates in a cycle, in which
    /// case a distinct node *may* be returned.
    fn cycle_head(&mut self, in_node: NodeIndex) -> DagId {
        let i = DagId::from_in_index(in_node);
        self.unify.find(i)
    }

    #[cfg(test)]
    fn in_cycle(&mut self, ni1: NodeIndex, ni2: NodeIndex) -> bool {
        self.cycle_head(ni1) == self.cycle_head(ni2)
    }
}
