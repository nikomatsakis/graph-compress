use super::*;

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
    let outputs = [nodes("D"), nodes("E")];
    let mut reduce = GraphReduce::new(&graph, &outputs);
    let cross_targets = Classify::new(&mut reduce).walk();

    assert!(!reduce.in_cycle(nodes("A"), nodes("C0")));
    assert!(!reduce.in_cycle(nodes("B"), nodes("C0")));
    assert!(reduce.in_cycle(nodes("C0"), nodes("C1")));
    assert!(!reduce.in_cycle(nodes("D"), nodes("C0")));
    assert!(!reduce.in_cycle(nodes("E"), nodes("C0")));
    assert!(!reduce.in_cycle(nodes("E"), nodes("A")));

    // FIXME -- nodes("A") is only present b/c we overapproximate cross-targets
    let ct = set!(reduce.cycle_head(nodes("C0")),
                  reduce.cycle_head(nodes("A")));
    assert_eq!(cross_targets, ct);
}
