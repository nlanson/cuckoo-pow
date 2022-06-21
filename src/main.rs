use cuckoo::graph::Graph;

fn main() {    
    // Print adjacency matrix of a graph
    let graph = Graph::new([0,6, 2, 7], 8);
    let mut adjmatrix = graph.adjacency_matrix();
    println!("{:?}", graph);
    println!("\n");
    println!("Adjacency Matrix \n{:?}", adjmatrix);
    println!("\n");
    Graph::edge_trim(&mut adjmatrix, 3);
    println!("Adjacency Matrix U after trimming\n{:?}", adjmatrix);
}