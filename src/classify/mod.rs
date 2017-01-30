//! First phase. Detect cycles and cross-edges.

use super::*;

#[cfg(test)]
mod test;

pub struct Classify<'a, 'g: 'a, N: 'g>
    where N: Debug
{
    r: &'a mut GraphReduce<'g, N>,
    stack: Vec<NodeIndex>,
    colors: Vec<Color>,
    dag: Dag,
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum Color {
    // not yet visited
    White,

    // visiting; usize is index on stack
    Grey(usize),

    // finished visiting
    Black,
}

impl<'a, 'g, N> Classify<'a, 'g, N>
    where N: Debug
{
    pub fn new(r: &'a mut GraphReduce<'g, N>) -> Self {
        Classify {
            r: r,
            colors: vec![Color::White; r.in_graph.len_nodes()],
            stack: vec![],
            dag: Dag {
                parents: (0..r.in_graph.len_nodes()).map(|i| NodeIndex(i)).collect(),
                cross_edges: vec![],
                leaf_nodes: vec![],
            },
        }
    }

    pub(super) fn walk(mut self) -> Dag {
        for &start_node in self.r.start_nodes {
            match self.colors[start_node.0] {
                Color::White => self.open(start_node),
                Color::Grey(_) => panic!("grey node but have not yet started a walk"),
                Color::Black => (), // already visited, skip
            }
        }

        // At this point we've identifed all the cycles, and we've
        // constructed a spanning tree over the original graph
        // (encoded in `self.parents`) as well as a list of
        // cross-edges that reflect additional edges from the DAG.
        //
        // If we converted each node to its `cycle-head` (a
        // representative choice from each SCC, basically) and then
        // take the union of `self.parents` and `self.cross_edges`
        // (after canonicalization), that is basically our DAG.
        //
        // Note that both of those may well contain trivial `X -> X`
        // cycle edges after canonicalization, though. e.g., if you
        // have a graph `{A -> B, B -> A}`, we will have unioned A and
        // B, but A will also be B's parent (or vice versa), and hence
        // when we canonicalize the parent edge it would become `A ->
        // A` (or `B -> B`).
        self.dag
    }

    fn open(&mut self, node: NodeIndex) {
        let index = self.stack.len();
        self.stack.push(node);
        self.colors[node.0] = Color::Grey(index);
        let mut any_children = false;
        for child in self.r.inputs(node) {
            self.walk_edge(node, child);
            any_children = true;
        }

        if !any_children {
            self.dag.leaf_nodes.push(node);
        }

        self.colors[node.0] = Color::Black;
    }

    fn walk_edge(&mut self, parent: NodeIndex, child: NodeIndex) {
        println!("walk_edge: {:?} -> {:?}, {:?}",
                 self.r.in_graph.node_data(parent),
                 self.r.in_graph.node_data(child),
                 self.colors[child.0]);

        // Ignore self-edges, just in case they exist.
        if child == parent {
            return;
        }

        match self.colors[child.0] {
            Color::White => {
                // Not yet visited this node; start walking it.
                assert_eq!(self.dag.parents[child.0], child);
                self.dag.parents[child.0] = parent;
                self.open(child);
            }

            Color::Grey(stack_index) => {
                // Back-edge; unify everything on stack between here and `stack_index`
                // since we are all participating in a cycle
                assert!(self.stack[stack_index] == child);
                assert!(stack_index > 0,
                        "start node `{:?}` participating in cycle",
                        self.r.in_graph.node_data(child));

                for &n in &self.stack[stack_index..] {
                    self.r.mark_cycle(n, parent);
                }
            }

            Color::Black => {
                // Cross-edge, record and ignore
                self.dag.cross_edges.push((parent, child));
            }
        }
    }
}
