use super::*;
use std::fmt::{Debug, Display};

fn reduce<D: Debug + Display>(graph: &Graph<D, ()>,
                              outputs: &[NodeIndex],
                              expected: &[&'static str])
{
    let reduce = GraphReduce::new(&graph, &outputs);
    let result = reduce.compute();
    let mut edges: Vec<String> =
        result.all_edges()
              .iter()
              .map(|edge| format!("{} -> {}",
                                  result.node_data(edge.source()),
                                  result.node_data(edge.target())))
              .collect();
    edges.sort();
    println!("{:#?}", edges);
    assert_eq!(edges.len(), expected.len());
    for (expected, actual) in expected.iter().zip(&edges) {
        assert_eq!(expected, actual);
    }
}

#[test]
fn test1() {
    let (graph, nodes) = graph! {
        A -> C0,
        A -> C1,
        B -> C1,
        C0 -> C1,
        C1 -> C0,
        C0 -> D,
        C1 -> E,
    };
    reduce(&graph, &[nodes("D"), nodes("E")], &[
        "A -> C1",
        "B -> C1",
        "C1 -> D",
        "C1 -> E",
    ]);
}

#[test]
fn test2() {
    let (graph, nodes) = graph! {
        A -> C0,
        A -> C1,
        B -> C1,
        C0 -> C1,
        C1 -> C0,
        C0 -> D,
        D -> E,
    };
    reduce(&graph, &[nodes("D"), nodes("E")], &[
        "A -> D",
        "B -> D",
        "D -> E",
    ]);
}
