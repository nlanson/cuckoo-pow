/// Cuckoo module
///
/// Written in 2022 by:
///     nlanson <nlanson@pm.me>
///
/// This module implements the graph solving and verifying 
/// logic of the cuckoo cycle PoW algorithm. 

use crate::sip::SipHash;

/// Graph structure
/// Generic parameters:
///     N - The number of nodes in the graph.
///     M - The number of edges in the graph.
///         This should be N+N as per the
///         specification.
/// Fields:
///     nodes - The nodes (vertices) of the graph. 
///             Each node is given a unique* number
///             to identify it.
///     edges - The edges of the graph which
///             connect nodes together. Each edge
///             consists of the nodes the it incidents
///             on.
pub struct Graph<const N: usize, const M: usize> {
    nodes: [u64; N],
    edges: [(u64, u64); M]
}