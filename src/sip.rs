/// SipHash module
/// 
/// Written in 2022 by:
///     nlanson <nlanson@pm.me>
/// 
/// This module implements the modified
/// siphash-2-4 hash function which is
/// used in cuckoo cycle to generate a graph.
/// The modification is specified in:
///    https://github.com/tromp/cuckoo/blob/master/doc/mathspec


/// Modified SipHash-2-4 compute which:
///   - Uses a 256bit key instead of a 128bit key.
///   - Initialises the internal state without XORing
///     constants.
pub struct SipHash {
    key: [u64; 4]
}

impl SipHash {
    /// Create a new siphash-2-4 computer with the given keys
    pub fn new(key: [u64; 4]) -> Self {
        Self { key }
    }

    /// Run siphash-2-4 on a given a single word input
    pub fn hash(&self, data: u64) -> u64 {
        // Initialisation
        let mut v0 = self.key[0];// ^ 0x736f6d6570736575;
        let mut v1 = self.key[1];// ^ 0x646f72616e646f6d;
        let mut v2 = self.key[2];// ^ 0x6c7967656e657261;
        let mut v3 = self.key[3];// ^ 0x7465646279746573;
        
        // Compression
        v3 ^= data;
        Self::sip_round(&mut v0, &mut v1, &mut v2, &mut v3);
        Self::sip_round(&mut v0, &mut v1, &mut v2, &mut v3);
        v0 ^= data;
        
        // Finalisation
        v2 ^= 0xff;
        Self::sip_round(&mut v0, &mut v1, &mut v2, &mut v3);
        Self::sip_round(&mut v0, &mut v1, &mut v2, &mut v3);
        Self::sip_round(&mut v0, &mut v1, &mut v2, &mut v3);
        Self::sip_round(&mut v0, &mut v1, &mut v2, &mut v3);
        
        // Return
        v0 ^ v1 ^ v2 ^ v3
    }

    /// One round of sip compression as specified in
    /// figure 2.1 of http://cr.yp.to/siphash/siphash-20120918.pdf.
    fn sip_round(v0: &mut u64, v1: &mut u64, v2: &mut u64, v3: &mut u64) {
        *v0 = v0.wrapping_add(*v1); *v2 = v2.wrapping_add(*v3);
        *v1 = v1.rotate_left(13);   *v3 = v3.rotate_left(16);
        *v1 ^= *v0;                 *v3 ^= *v2;
        *v0 = v0.rotate_left(32);   
        *v2 = v2.wrapping_add(*v1); *v0 = v0.wrapping_add(*v3);
        *v1 = v1.rotate_left(17);   *v3 = v3.rotate_left(21);
        *v1 ^= *v2;                 *v3 ^= *v0;
        *v2 = v2.rotate_left(32);
    }
}