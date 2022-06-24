use cuckoo::graph::Graph;

fn main() {    
    let mut i = 0;
    loop {
        println!("Starting graph #{}", i);
        let graph = Graph::new([2*i, i+2, i+1, i], 2048);
        let c = graph.solve(16);
        if c.is_some() {
            println!("Cycle {:?} was found on graph #{}", c.unwrap(), i);
        }

        i += 1
    }
}