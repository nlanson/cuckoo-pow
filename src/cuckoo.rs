/// Cuckoo module
///
/// Written in 2022 by:
///     nlanson <nlanson@pm.me>
///
/// This module implements the graph solving and verifying 
/// logic of the cuckoo cycle PoW algorithm. 

use crate::sip::SipHash;

/// Graph structure
/// Fields:
///     edges - The edges of the graph which
///             connect nodes together. Each edge
///             consists of the nodes the it incidents
///             on.
#[derive(Debug)]
pub struct Graph {
    edges: Vec<(u64, u64)>
}

impl Graph {
    /// Construct a new graph with the given hash function key and edge count.
    /// The graph will be a bipartite graph with edge_count * 2 nodes.
    pub fn new(key: [u64; 4], n: u64) -> Self {
        let mut edges = Vec::with_capacity(n as usize);
        let hasher = SipHash::new(key);

        // Construct graph G_K
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
    pub fn solve(&self, cycle_len: usize) -> Option<Vec<u64>> {
        None
    }

    /// Verify a cycle and check if it is a cycle on self.
    pub fn verify(&self, cycle_len: usize, edges: &[usize]) -> bool {
        false
    }
}