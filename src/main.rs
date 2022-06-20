use cuckoo::graph::Graph;

fn main() {
    // Verify a cycle
    let edges = vec![(0, 0), (1, 0), (1, 2), (3, 2), (3, 3), (0, 3)];
    let graph = Graph::from(edges);
    let cycle = [0, 1, 2, 3, 4, 5];

    println!("{}", graph.verify(6, &cycle));


    // Print adjacency matrix of a graph
    let graph = Graph::new([0,1,1,5], 1024);
    let mut adjmatrix = graph.adjacency_matrix();
    println!("{:?}", graph);
    println!("\n");
    println!("Adjacency Matrix U\n{:?}", adjmatrix.0);
    println!("\n");
    println!("Adjacency Matrix V\n{:?}", adjmatrix.1);
    println!("\n");
    Graph::edge_trim(&mut adjmatrix);
    Graph::edge_trim(&mut adjmatrix);
    Graph::edge_trim(&mut adjmatrix);
    println!("Adjacency Matrix U after trimming\n{:?}", adjmatrix.0);
    println!("\n");
    println!("Adjacency Matrix V after trimming\n{:?}", adjmatrix.1);
}