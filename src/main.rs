use cuckoo::graph::Graph;

fn main() {    
    let mut i = 0;
    loop {
        println!("Starting graph #{}", i);
        let graph = Graph::new([2*i, i+2, i+1, i], 2u64.pow(15));
        let c = graph.solve(42);
        if c.is_some() {
            println!("Cycle {:?} was found on graph #{}", c.unwrap(), i);
        } else {
            println!("No solution was found on graph #{}", i);
        }

        i += 1
    }
}