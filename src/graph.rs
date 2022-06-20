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

type AdjacencyMatrix = (HashMap<u64, HashSet<u64>>, HashMap<u64, HashSet<u64>>);

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
    
    /// Construct a new graph with n edges and n+n nodes.
    pub fn new(key: [u64; 4], n: u64) -> Self {
        let mut edges = Vec::with_capacity(n as usize);
        let hasher = SipHash::new(key);

        // Construct the edges of the graph G_K
        let mut i: u64 = 0;
        while i < n {
            let u: u64 = hasher.hash(2*i)   % n;
            let v: u64 = hasher.hash(2*i+1) % n;
            edges.push((u, v));
            
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
    fn edge_at(&self, index: usize) -> Option<(u64, u64)> {
        if index > self.edges.len() {
            return None
        }

        Some(self.edges[index])
    }

    // Given an edge, return the index of the edge if it exists.
    fn index_of(&self, edge: &(u64, u64)) -> Option<usize> {
        self.edges.iter().position(|x| x == edge)
    }

    /// Solve for a cycle with the given number of edges.
    /// The result of this function is either a vector of edge indicies
    /// or nothing in the case that no cycle exists on the graph.
    pub fn solve(&self, cycle_len: usize) -> Option<Vec<usize>> {
        // Run a few rounds of edge trimming to remove unecessary edges
        let mut adjmatrix = self.adjacency_matrix();
        Self::edge_trim(&mut adjmatrix);

        Self::graph_mine(&mut adjmatrix, cycle_len)
    } 

    /// Given a adjacency matrix, trim edges that cannot be part of a cycle.
    /// This is done by removing edges that incident on nodes with a degree < 2.
    /// Running edge trimming a few times can drastically reduce the time it takes
    /// to solve for a cycle in the graph.
    pub fn edge_trim(adjmatrix: &mut AdjacencyMatrix) {
        let (u, v) = adjmatrix;

        for (node, neighbours) in u.iter_mut() {
            if neighbours.len() < 2 {
                for neighbour in neighbours.iter() {
                    if let Some(entry) = v.get_mut(neighbour) {
                        entry.remove(node);
                    }
                }
            }
        }

        u.retain(|_, v| v.len() >= 2);


        for (node, neighbours) in v.iter_mut() {
            if neighbours.len() < 2 {
                for neighbour in neighbours.iter() {
                    if let Some(entry) = u.get_mut(neighbour) {
                        entry.remove(node);
                    }
                }
            }
        }
        
        v.retain(|_, v| v.len() >= 2);
        u.retain(|_, v| v.len() >= 1);
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
        let (u, v) = adjmatrix;

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
        // Build an adjacency matrix from the edges
        let mut u: HashMap<u64, HashSet<u64>> = HashMap::new();
        let mut v: HashMap<u64, HashSet<u64>> = HashMap::new();

        for (a, b) in &self.edges {
            if u.contains_key(a) {
                u.get_mut(a).expect("Node not found").insert(*b);
            } else {
                let mut set = HashSet::new();
                set.insert(*b);
                u.insert(*a, set);
            }
            
            if v.contains_key(b) {
                v.get_mut(b).expect("Node not found").insert(*a);
            } else {
                let mut set = HashSet::new();
                set.insert(*a);
                v.insert(*b, set);
            }
        }

        (u, v)
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