/// Cuckoo module
///
/// Written in 2022 by:
///     nlanson <nlanson@pm.me>
///
/// This module implements the graph solving and verifying 
/// logic of the cuckoo cycle PoW algorithm. 

use crate::sip::SipHash;
use std::collections::HashSet;

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
    edges: Vec<(u64, u64)>
}

impl Graph {
    /// Construct a new graph
    pub fn new(key: [u64; 4], n: u64) -> Self {
        let mut edges = Vec::with_capacity(n as usize);
        let hasher = SipHash::new(key);

        // Construct the edges of the graph G_K
        let mut i: u64 = 0;
        while i < n{
            let u: u64 = hasher.hash(2*i)   % n;
            let v: u64 = hasher.hash(2*i+1) % n;
            edges.push((u, v));
            
            i += 1;
        }

        // Not sure if this series of right shifts is necessary apart from
        // saving memory...
        let mut i = 0;
        while i < edges.len() {
            edges[i] = (edges[i].0 >> 1, edges[i].1 >> 1);
            i += 1;
        }


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
    fn edge_at(&self, index: usize) -> Option<(u64, u64)> {
        if index > self.edges.len() {
            return None
        }

        Some(self.edges[index])
    }

    /// Solve for a cycle with the given number of edges.
    /// The result of this function is either a vector of edge indicies
    /// or nothing in the case that no cycle exists on the graph.
    pub fn solve(&self, cycle_len: usize) -> Option<Vec<usize>> {
        None
    }

    /// Verify a cycle and check if it is a cycle on self.
    /// TODO:
    ///     - Remove edges that have been used so cycles that repeat an edge
    ///       cannot be verified.
    ///     - Add and return a enum for returning verification results. This
    ///       can help identify the reason why verification fails.
    ///     - Cycles cannot repeat any nodes. Need to keep track of visited nodes
    ///       on both sides of the graph to detect any repeated vertices.
    pub fn verify(&self, cycle_len: usize, edges: &[usize]) -> bool { 
        // Initialise node tracking sets.
        let mut u = Vec::new();
        let mut v = Vec::new();
        let mut edgeset = HashSet::new();

        for index in edges {
            // Check if the working edge is a duplicate.
            if edgeset.contains(index) {
                return false
            } else {
                edgeset.insert(index);
            }
            
            // Extract the edge from the edge set using the index.
            let (a, b) = match self.edge_at(*index) {
                Some(edge) => edge,
                None       => return false
            };

            
            // Insert and track visited edges, removing them if it is already visited.
            if u.contains(&a) {
                let index = u.iter().position(|x| *x == a).expect("Node not found");
                u.remove(index);
            } else {
                u.push(a);
            }

            if v.contains(&b) {
                let index = v.iter().position(|x| *x == b).expect("Node not found");
                v.remove(index);
            } else {
                v.push(b);
            }
        }

        // Once all edges are processed, each vertice should have been visited twice,
        // meaning that the tracking vecs should be empty.
        u.len() == 0 && v.len() == 0 && edges.len() == cycle_len
    }
}

impl From<Vec<(u64, u64)>> for Graph {
    fn from(edges: Vec<(u64, u64)>) -> Self {
        Self { edges }
    }
}