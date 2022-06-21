/// Cuckoo module
///
/// Written in 2022 by:
///     nlanson <nlanson@pm.me>
///
/// This module implements the graph solving and verifying 
/// logic of the cuckoo cycle PoW algorithm. 

use crate::sip::SipHash;
use std::collections::{
    HashSet,
    HashMap
};
use std::cell::RefCell;

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
///     edges - The edges of the graph which
///             connect nodes together. Each edge
///             consists of the nodes the it incidents
///             on.
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
        self.edges.iter().position(|x| x == edge)
    }

    /// Solve for a cycle with the given number of edges.
    /// The result of this function is either a vector of edge indicies
    /// or nothing in the case that no cycle exists on the graph.
    pub fn solve(&self, cycle_len: usize) -> Option<Vec<usize>> {
        // Run a few rounds of edge trimming to remove unecessary edges
        let mut adjmatrix = self.adjacency_matrix();
        Self::edge_trim(&mut adjmatrix, 3);

        Self::graph_mine(&mut adjmatrix, cycle_len)
    } 

    /// Given a adjacency matrix, trim edges that cannot be part of a cycle.
    /// This is done by removing edges that incident on nodes with a degree < 2.
    /// Running edge trimming a few times can drastically reduce the time it takes
    /// to solve for a cycle in the graph.
    pub fn edge_trim(adjmatrix: &mut AdjacencyMatrix, count: usize) {        
        for _ in 0..count {
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
    /// Along the way, visited nodes and used edges are kept track of
    /// so that edges and nodes are not repeated.
    ///
    /// This method uses 256 bits per edge (an edge is 128 bits but they
    /// are replicated in both directions in the adjacency matrix).
    fn graph_mine(adjmatrix: &mut AdjacencyMatrix, cycle_len: usize) -> Option<Vec<usize>> {
        // From here, we can select an arbitrary node in either the u set or
        // the v set which has 2 or more edges that incident on it. From there,
        // we iterate over each edge and get to the next node, and so and and so
        // forth, until we either get stuck at a node without any usable edges or
        // the path we have taken is the length of the target and we check if the 
        // path is a cycle (start == end).
        //
        // While traversing, we keep track of nodes we have visited and edges we have
        // used so the path does not repeat edges or nodes.
        
        // placeholder...
        None
    }

    // Create an adjacency matrix representation of the graph.
    // The matrix is made of two HashMaps, each holding adjacency values of nodes in either
    // partition of the node set.
    pub fn adjacency_matrix(&self) -> AdjacencyMatrix {
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
    ///     - Verify that the given cycle is not multiple disjoint cycles.
    ///       This can be done by verifying that by following the cycle from
    ///       an arbitrary starting point, we use all the edges in the cycle.
    /// 
    ///     - Add and return a enum for returning verification results. This
    ///       can help identify the reason why verification fails.
    pub fn verify(&self, cycle_len: usize, edges: &[usize]) -> bool { 
        // Early exit upon cycle length mismatch
        if edges.len() != cycle_len {
            return false
        }
        
        // Initialise node and edge tracker
        let mut counter: HashMap<Node, usize> = HashMap::new();
        let mut edgeset: HashSet<usize> = HashSet::new();
        
        for index in edges {
            // If edge is used before, fail verification,
            if edgeset.contains(index) {
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
        }

        // The cycle is verified if every involved vertice is invidented on twice.
        !counter.iter().any(|(_, i)| *i != 2)
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