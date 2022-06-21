# Cuckoo Cycle Rust Implementation
This implementation of thbe Cuckoo Cycle proof of work algorithm aims to be simple and easy to follow for beginners in the cryptography and computer science space. Hence, it is far from optimised.

### What is Cuckoo Cycle?
Cuckoo cycle is a graph theory based proof of work algorithm that works by finding a cycle on a pseudorandomly generated graph. The cycle is easy to verify once found, but finding a cycle is a much harder task and serves as proof that computational resources have been put into finding the cycle.

For a full specification and optimised implementations, see https://github.com/tromp/cuckoo