/// Cuckoo module
///
/// Written in 2022 by:
///     nlanson <nlanson@pm.me>
///
/// This module implements the graph solving and verifying 
/// logic of the cuckoo cycle PoW algorithm. 

use crate::sip::SipHash;
use std::cell::RefCell;
use std::collections::{
    HashSet,
    HashMap
};


/// Graph structure
/// 
/// The graph for cuckoo cycle has N edges and N+N nodes.
/// It is bipartite, meaning the nodes can be divided into
/// two disjoint sets, U and V, where nodes in U only have
/// edges to nodes in V and vice versa.
/// 
/// The edges of the graph are generated pseudorandomly using
/// the siphash-2-4 hash function.
/// 
/// Struct Fields:
///     edges - Edges are stored as a list of Edges instead of
///             of an adjacency matrix to preserve the indexing.
#[derive(Debug)]
pub struct Graph {
    edges: Vec<Edge>,
}

#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
pub enum Node {
    U(u64),
    V(u64)
}

type Edge = (Node, Node);
type AdjacencyMatrix = HashMap<Node, RefCell<HashSet<Node>>>;

impl Graph {
    
    /// Construct a new graph with n edges and n+n nodes.
    pub fn new(key: [u64; 4], n: u64) -> Self {
        let mut edges = Vec::with_capacity(n as usize);
        let hasher = SipHash::new(key);

        // Construct the edges of the graph G_K
        let mut i: u64 = 0;
        while i < n {
            let u: u64 = hasher.hash(2*i)   % n;
            let v: u64 = hasher.hash(2*i+1) % n;
            edges.push((Node::U(u), Node::V(v)));
            
            i += 1;
        }

        // From the cuckoo cycle mathspec:
        //      > From G_K we obtain the graph G'_K by identifying nodes that differ only in the last bit:
        //      >     for 0 <= i < N, E'_i = (V_i_0 >> 1, V_i_1 >> 1)
        //
        // This is implemented in the code commented out below, but not sure if this is doing the right
        // thing. It certain helps save memory but it seems to be screwing with the edges and adjacency.
        //
        // let mut i = 0;
        // while i < edges.len() {
        //     edges[i] = (edges[i].0 >> 1, edges[i].1 >> 1);
        //     i += 1;
        // }


        Self { edges }
    }

    /// Return the number of nodes in self
    pub fn node_count(&self) -> usize {
        self.edges.len() * 2
    }

    /// Return the number of edges in self
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Get the edge at the given index
    fn edge_at(&self, index: usize) -> Option<Edge> {
        if index > self.edges.len() {
            return None
        }

        Some(self.edges[index])
    }

    // Given an edge, return the index of the edge if it exists.
    fn index_of(&self, edge: &Edge) -> Option<usize> {
        self.edges.iter().position(|(u, v)| (*u, *v) == *edge || (*v, *u) == *edge)
    }

    /// Solve for a cycle with the given number of edges.
    /// The result of this function is either a vector of edge indicies
    /// or nothing in the case that no cycle exists on the graph.
    pub fn solve(&self, cycle_len: usize) -> Option<Vec<usize>> {
        // Run a few rounds of edge trimming to remove unecessary edges
        let mut adjmatrix = self.adjacency_matrix();
        Self::edge_trim(&mut adjmatrix, 100);

        self.graph_mine(&adjmatrix, cycle_len)
    } 

    /// Given a adjacency matrix, trim edges that cannot be part of a cycle.
    /// This is done by removing edges that incident on nodes with a degree < 2.
    /// Running edge trimming a few times can drastically reduce the time it takes
    /// to solve for a cycle in the graph.
    fn edge_trim(adjmatrix: &mut AdjacencyMatrix, count: usize) {        
        for _ in 0..count {
            if adjmatrix.is_empty() {
                break;
            }
            
            for node in adjmatrix.keys() {
                let mut neighbours = adjmatrix
                                        .get(node)
                                        .expect("Node not found")
                                        .borrow_mut();
                
                if neighbours.len() >= 2 {
                    continue
                }
                
                for neighbour in neighbours.iter() {
                    adjmatrix
                        .get(neighbour)
                        .expect("Node not found")
                        .borrow_mut()
                        .remove(node);
                }

                neighbours.clear();
            }
        }
        
        adjmatrix.retain(|_, v| v.borrow().len() > 0);
    }

    /// Graph mining technique to solve for a cycle on the graph.
    /// This solving method uses brute force to traverse every path
    /// that is at most the specified solution length and checks 
    /// if the start and end of the path are equal, in which case the
    /// path is a cycle.
    /// 
    /// TODO: Add graph mining tests.
    fn graph_mine(&self, adjmatrix: &AdjacencyMatrix, cycle_len: usize) -> Option<Vec<usize>> {
        // For each node, 
        for node in adjmatrix.keys() {
            let neighbours = adjmatrix.get(node).expect("Node missing").borrow();
            
            // If it has less than 2 neighbours, skip it.
            if neighbours.len() < 2 {
                continue
            }
            
            // Otherwise, try find a cycle using depth first search.
            match self.dfs(adjmatrix, node, Vec::new(), cycle_len) {
                None => continue,
                Some(x) => return self.edges_to_indexes(&x)
            }
        }

        None
    }

    /// Find a cycle using depth first search.
    /// Modifying the adjacency matrix by removing used edges can make the algorithm more efficient.
    fn dfs(&self, adjmatrix: &AdjacencyMatrix, start: &Node, path: Vec<Edge>, limit: usize)-> Option<Vec<Edge>> {        
        // Base case where the length limit has been reached. Return the path if it is a cycle.
        if limit == 0 {
            // If the path is trivial, return None
            if path.len() == 0 {
                return None
            }
            
            let first = path.first().expect("Path is empty");
            let last = path.last().expect("Path is empty");
            let indexes = self.edges_to_indexes(&path)?;

            // If the path starts and ends on the same node and is a verified cycle, return it.
            if first.0 == last.1 && self.verify(path.len(), &indexes[..]) {
                return Some(path)
            }

            return None
        }

        // Recursive case, iterate each edge on the current node.
        if let Some(refc) = adjmatrix.get(start) {
            let neighbours = refc.borrow();
            let mut paths: Vec<Option<Vec<Edge>>> = Vec::with_capacity(neighbours.len());
            let nodes = neighbours.iter().map(|n| *n).collect::<Vec<Node>>();

            for n in nodes {
                let nadjmatrix = adjmatrix.clone();
                let mut path_cont = Vec::from(&path[..]);

                nadjmatrix.get(&n).expect("Node missing").borrow_mut().remove(start);

                path_cont.push((*start, n));
                paths.push(self.dfs(&nadjmatrix, &n, path_cont, limit-1));
            }

            // Of all possible paths from the current node, only keep the cycles.
            paths.retain(|x| x.is_some());

            // If there are any paths remaining, return the first path that was found.
            if paths.len() > 0 {
                return paths.into_iter().nth(0).expect("Path missing")
            }

            return None
        }

        None
    }

    /// Given a graph (self) and a list of edges, reutrn a list of corresponding edge indexes.
    fn edges_to_indexes(&self, edges: &Vec<Edge>) -> Option<Vec<usize>> {
        let mut indexes = Vec::with_capacity(edges.len());
        for edge in edges {
            indexes.push(self.index_of(edge)?);
        }

        indexes.sort();
        Some(indexes)
    }

    /// Create an adjacency matrix representation of the graph.
    /// The matrix is made of two HashMaps, each holding adjacency values of nodes in either
    /// partition of the node set.
    ///
    fn adjacency_matrix(&self) -> AdjacencyMatrix {
        let mut adjmatrix: AdjacencyMatrix = HashMap::new();

        for (a, b) in &self.edges {
            if !adjmatrix.contains_key(&a) {
                let mut set = HashSet::new();
                set.insert(*b);
                adjmatrix.insert(*a, RefCell::new(set));
            } else {
                adjmatrix
                    .get_mut(a)
                    .expect("Node not found")
                    .borrow_mut()
                    .insert(*b);
            }

            if !adjmatrix.contains_key(&b) {
                let mut set = HashSet::new();
                set.insert(*a);
                adjmatrix.insert(*b, RefCell::new(set));
            } else {
                adjmatrix
                    .get_mut(b)
                    .expect("Node not found")
                    .borrow_mut()
                    .insert(*a);
            }
        }

        adjmatrix
    }


    /// Verify a cycle and check if it is a cycle on self.
    /// This is done by storing each visited node in a list, 
    /// and making sure the edges of the provided cycle enter
    /// and leave the each node that is part of the cycle.
    /// 
    /// TODO:
    ///     - Add and return a enum for returning verification results. This
    ///       can help identify the reason why verification fails.
    pub fn verify(&self, cycle_len: usize, edges: &[usize]) -> bool { 
        // Early fail conditions
        //  - Provided edges or cycle len is odd.
        //  - Edge len does not equal cycle len.
        //  - Cycle len is zero.
        if edges.len()%2 == 1 || cycle_len%2 == 1 || edges.len() != cycle_len || cycle_len == 0 {
            return false
        }
        
        // Initialise node and edge tracker
        let mut counter: HashMap<Node, usize> = HashMap::new();
        let mut edgeset: HashSet<usize> = HashSet::new();
        let mut prev = edges[0];
        
        for index in edges {
            // If edge is used before, fail verification,
            if edgeset.contains(index) {
                return false;
            }

            // If edge indexes are not sorted, fail verification.
            if *index < prev {
                return false;
            }

            // Track the edge as used
            edgeset.insert(*index);

            // Track how the degree of each node in the given cycle.
            if let Some((u, v)) = self.edge_at(*index) {
                if counter.contains_key(&u) {
                    *counter.get_mut(&u).expect("Node not found") += 1;
                } else {
                    counter.insert(u, 1);
                }

                if counter.contains_key(&v) {
                    *counter.get_mut(&v).expect("Node not found") += 1;
                } else {
                    counter.insert(v, 1);
                }
            } else {
                return false
            }

            prev = *index;
        }

        // Fail if every involved vertice is not incidented on twice.
        if counter.iter().any(|(_, i)| *i != 2) {
            return false
        }

        // Follow cycle
        let cmatrix = Graph::from(
            edges.iter().map(|x| self.edge_at(*x).expect("Edge not found")).collect::<Vec<Edge>>()
        ).adjacency_matrix();
        let start = cmatrix.keys().next().expect("Node missing");
        let mut pos = *start;
        let mut n = 0;

        loop {
            let mut adjs = cmatrix.get(&pos).expect("Node missing").borrow_mut();
            
            // End of cycle
            if adjs.len() == 0 && pos == *start {
                return n == cycle_len;
            }

            match adjs.iter().next() {
                Some(node) => {
                    cmatrix.get(node).expect("Node missing").borrow_mut().remove(&pos);
                    pos = node.clone();
                },
                _ => return false // Dead end (should not occur...)
            }

            adjs.remove(&pos);
            n += 1;
        }
    }
}

impl From<Vec<(u64, u64)>> for Graph {
    fn from(edges: Vec<(u64, u64)>) -> Self {
        Self { 
            edges: edges
                    .iter()
                    .map(|(a, b)| (Node::U(*a), Node::V(*b)))
                    .collect() 
        }
    }
}

impl From<Vec<Edge>> for Graph {
    fn from(edges: Vec<Edge>) -> Self {
        let mut g = Vec::new();
        
        for edge in edges {
            match edge {
                (Node::U(_), Node::V(_)) => g.push(edge),
                (Node::V(v), Node::U(u)) => g.push((Node::U(u), Node::V(v))),
                _                        => continue
            }
        }

        Self { edges: g }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_cycle() {
        let edges = vec![(0, 0), (1, 0), (1, 2), (3, 2), (3, 3), (0, 3)];
        let graph = Graph::from(edges);
        let cycle = [0, 1, 2, 3, 4, 5];
        assert!(graph.verify(6, &cycle));
    }

    #[test]
    fn fail_verify_cycle() {
        let edges = vec![(0, 0), (0, 1), (1, 0), (1, 1), (6, 6), (6, 7), (7, 6), (7, 7)];
        let graph = Graph::from(edges);
        let cycle = [0, 1, 2, 3, 4, 5, 6, 7];
        assert!(!graph.verify(8, &cycle));
    }
}