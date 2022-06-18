/// A simple and easy to follow rust
/// implementation of Cuckoo Cycle.
/// 
/// Written in 2022 by:
///     nlanson <nlanson@pm.me>
/// 
/// Spec: https://github.com/tromp/cuckoo
/// 
/// This implementation does not aim to be
/// the fastest, but rather a more simple
/// and easy to follow algorithm to solve
/// for the cuckoo cycle proof of work
/// algorithm.

mod sip;
pub mod cuckoo;


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
