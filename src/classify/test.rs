use super::*;

#[test]
fn detect_cycles() {
    let (graph, nodes) = graph! {
        A -> C0,
        A -> C1,
        B -> C1,
        C0 -> C1,
        C1 -> C0,
        C0 -> D,
        C1 -> E,
    };
    let outputs = [nodes("D"), nodes("E")];
    let mut reduce = GraphReduce::new(&graph, &outputs);
    Classify::new(&mut reduce).walk();

    assert!(!reduce.in_cycle(nodes("A"), nodes("C0")));
    assert!(!reduce.in_cycle(nodes("B"), nodes("C0")));
    assert!(reduce.in_cycle(nodes("C0"), nodes("C1")));
    assert!(!reduce.in_cycle(nodes("D"), nodes("C0")));
    assert!(!reduce.in_cycle(nodes("E"), nodes("C0")));
    assert!(!reduce.in_cycle(nodes("E"), nodes("A")));
}
