use cuckoo::graph::Graph;

fn main() {
    let edges = vec![(0, 0), (1, 0), (1, 2), (3, 2), (3, 3), (0, 3)];
    let graph = Graph::from(edges);
    let cycle = [0, 1, 2, 3, 4, 5];

    assert!(graph.verify(6, &cycle));
}