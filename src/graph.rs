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
    ///     - Add and return a enum for returning verification results. This
    ///       can help identify the reason why verification fails.
    pub fn verify(&self, cycle_len: usize, edges: &[usize]) -> bool { 
        // Initialise node and edge tracking sets.
        let mut u: HashMap<u64, usize> = HashMap::new();
        let mut v: HashMap<u64, usize> = HashMap::new();
        let mut edgeset = HashSet::new();

        // Process each edge in the cycle...
        for index in edges {
            // Check if the working edge has been used before.
            if !edgeset.contains(index) {
                edgeset.insert(index);
            } else {
                return false
            }
            
            // Check if the edge exists in the graph, and if it does,
            // keep a track of the vertices that the edge incidents on.
            if let Some((a, b)) = self.edge_at(*index) {
                if !u.contains_key(&a) {
                    u.insert(a, 1);
                } else {
                    *u.get_mut(&a).expect("Value not found") += 1;
                }

                if !v.contains_key(&b) {
                    v.insert(b, 1);
                } else {
                    *v.get_mut(&b).expect("Value not found") += 1;
                }
            } else {
                return false
            }
        }

        // If any of the vertices have been visited more than once in the cycle, then return false.
        if u.iter().any(|(_, i)| *i != 2) || v.iter().any(|(_, i)| *i != 2) {
            return false
        }
        
        edges.len() == cycle_len
    }
}

impl From<Vec<(u64, u64)>> for Graph {
    fn from(edges: Vec<(u64, u64)>) -> Self {
        Self { edges }
    }
}