use cuckoo::cuckoo::Graph;

fn main() {
    let key = [1, 2, 3, 4];
    let graph = Graph::new(key, 8);

    println!("{:?}", graph);
}