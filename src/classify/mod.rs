//! First phase. Detect cycles and cross-edges.

use rustc_data_structures::fx::FxHashSet;

use super::*;

#[cfg(test)]
mod test;

pub struct Classify<'a, 'g: 'a, N: 'g>
    where N: Debug
{
    r: &'a mut GraphReduce<'g, N>,
    stack: Vec<NodeIndex>,
    colors: Vec<Color>,
    cross_targets: FxHashSet<NodeIndex>,
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
            cross_targets: FxHashSet(),
        }
    }

    pub fn walk(mut self) -> FxHashSet<DagId> {
        for &start_node in self.r.start_nodes {
            match self.colors[start_node.0] {
                Color::White => self.walk_white(start_node),
                Color::Grey(_) => panic!("grey node but have not yet started a walk"),
                Color::Black => (), // already visited, skip
            }
        }

        // convert cross-edges to the canonical dag-id and return
        let Classify { r, cross_targets, .. } = self;
        cross_targets.iter()
                     .map(|&n| r.cycle_head(n))
                     .collect()
    }

    fn walk_white(&mut self, node: NodeIndex) {
        let index = self.stack.len();
        self.stack.push(node);
        self.colors[node.0] = Color::Grey(index);
        for child in self.r.inputs(node) {
            self.walk_edge(node, child);
        }
        self.colors[node.0] = Color::Black;
    }

    fn walk_edge(&mut self, parent: NodeIndex, child: NodeIndex) {
        println!("walk_edge: {:?} -> {:?}, {:?}",
                 self.r.in_graph.node_data(parent),
                 self.r.in_graph.node_data(child),
                 self.colors[child.0]);

        match self.colors[child.0] {
            Color::White => {
                // Not yet visited this node; start walking it.
                self.walk_white(child);
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
                self.cross_targets.insert(child);
            }
        }
    }
}
