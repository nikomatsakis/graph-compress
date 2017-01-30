#![feature(conservative_impl_trait)]
#![feature(field_init_shorthand)]
#![feature(pub_restricted)]
#![feature(rustc_private)]

extern crate rustc_data_structures;

use rustc_data_structures::graph::{Graph, NodeIndex};
use rustc_data_structures::unify::UnificationTable;
use std::fmt::Debug;

#[cfg(test)]
#[macro_use]
mod test_macro;

mod construct;

mod classify;
use self::classify::Classify;

mod dag_id;
use self::dag_id::DagId;

#[cfg(test)]
mod test;

pub struct GraphReduce<'g, N> where N: 'g + Debug {
    in_graph: &'g Graph<N, ()>,
    start_nodes: &'g [NodeIndex],
    unify: UnificationTable<DagId>,
}

struct Dag {
    // The "parent" of a node is the node which reached it during the
    // initial DFS. To encode the case of "no parent" (i.e., for the
    // roots of the walk), we make `parents[i] == i` to start, which
    // turns out be convenient.
    parents: Vec<NodeIndex>,

    // Additional edges beyond the parents.
    cross_edges: Vec<(NodeIndex, NodeIndex)>,

    // Nodes with no successors.
    leaf_nodes: Vec<NodeIndex>,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
struct DagNode {
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

    pub fn compute(mut self) -> Graph<&'g N, ()> {
        let dag = Classify::new(&mut self).walk();
        construct::construct_graph(&mut self, dag)
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
    fn cycle_head(&mut self, in_node: NodeIndex) -> NodeIndex {
        let i = DagId::from_in_index(in_node);
        self.unify.find(i).as_in_index()
    }

    #[cfg(test)]
    fn in_cycle(&mut self, ni1: NodeIndex, ni2: NodeIndex) -> bool {
        self.cycle_head(ni1) == self.cycle_head(ni2)
    }
}
