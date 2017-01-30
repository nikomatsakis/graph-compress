//! Second phase. Construct new graph. The previous phase has
//! converted the input graph into a DAG by detecting and unifying
//! cycles. It provides us with the following (which is a
//! representation of the DAG):
//!
//! - SCCs, in the form of a union-find repr that can convert each node to
//!   its *cycle head* (an arbitrarly chosen representative from the cycle)
//! - a vector of *leaf nodes*, just a convenience
//! - a vector of *parents* for each node (in some cases, nodes have no parents,
//!   or their parent is another member of same cycle; in that case, the vector
//!   will be stored `v[i] == i`, after canonicalization)
//! - a vector of *cross edges*, meaning add'l edges between graphs nodes beyond
//!   the parents.

use rustc_data_structures::fx::FxHashMap;

use super::*;

pub(super) fn construct_graph<'g, N>(r: &mut GraphReduce<'g, N>, dag: Dag) -> Graph<&'g N, ()>
    where N: Debug
{
    let Dag { parents: old_parents, leaf_nodes, cross_edges } = dag;

    println!("construct_graph");

    // Create a canonical list of edges; this includes both parent and
    // cross-edges. We store this in `(target -> Vec<source>)` form.
    // We call the first edge to any given target its "parent".
    let mut edges = FxHashMap();
    let old_parent_edges = old_parents.iter().cloned().zip((0..).map(NodeIndex));
    for (source, target) in old_parent_edges.chain(cross_edges) {
        let source = r.cycle_head(source);
        let target = r.cycle_head(target);
        if source != target {
            let v = edges.entry(target).or_insert(vec![]);
            if !v.contains(&source) {
                v.push(source);
            }
        }
    }
    let parent = |ni: NodeIndex| -> NodeIndex {
        edges[&ni][0]
    };

    // `retain`: a set of those nodes that we will want to *retain* in
    // the ultimate graph. These are nodes in the following categories:
    //
    // - inputs
    // - work-products
    // - targets of a cross-edge
    //
    // The first two categories hopefully make sense. We want the
    // inputs so we can compare hashes later. We want the
    // work-products so we can tell precisely when a given
    // work-product is invalidated. But the last one isn't strictly
    // needed; we keep cross-target edges so as to minimize the total
    // graph size.
    //
    // Consider a graph like:
    //
    //     WP0 -> Y
    //     WP1 -> Y
    //     Y -> INPUT0
    //     Y -> INPUT1
    //     Y -> INPUT2
    //     Y -> INPUT3
    //
    // Now if we were to remove Y, we would have a total of 8 edges: both WP0 and WP1
    // depend on INPUT0...INPUT3. As it is, we have 6 edges.
    //
    // In theory, if a cross-target depends on *exactly one* input, we
    // should remove it, but we don't bother to detect that, as this
    // scenario is believed to arise rarely in practice. Perhaps it is
    // worth checking for, however? Would require tracking a bit more
    // data.

    // Start by adding start-nodes and inputs. These should not
    // participate in cycles (the former by fiat, the latter by
    // definition), so check that they are always their own
    // cycle-head.
    let retained_nodes = r.start_nodes.iter().chain(&leaf_nodes).cloned();

    // Next add in targets of cross-edges. Due to the caonicalization,
    // some of these may be self-edges or may may duplicate the parent
    // edges, so ignore those.
    let retained_nodes = retained_nodes.chain(
        edges.iter()
             .filter(|&(_, ref sources)| sources.len() > 1)
             .map(|(&target, _)| target));

    // Now create the new graph, adding in the entries from the map.
    let mut retain_map = FxHashMap();
    let mut new_graph = Graph::new();
    for n in retained_nodes {
        let n = r.cycle_head(n);
        let data = r.in_graph.node_data(n);
        let index = new_graph.add_node(data);
        retain_map.insert(n, index);
    }

    // Given a cycle-head `ni`, converts it to the closest parent that has
    // been retained in the output graph.
    let retained_parent = |mut ni: NodeIndex| -> NodeIndex {
        loop {
            println!("retained_parent({:?})", r.in_graph.node_data(ni));
            match retain_map.get(&ni) {
                Some(&v) => return v,
                None => ni = parent(ni),
            }
        }
    };

    // Now add in the edges into the graph.
    for (target, sources) in &edges {
        if let Some(&r_target) = retain_map.get(&target) {
            for &source in sources {
                let r_source = retained_parent(source);
                new_graph.add_edge(r_target, r_source, ());
            }
        } else {
            assert_eq!(sources.len(), 1);
        }
    }

    new_graph
}

