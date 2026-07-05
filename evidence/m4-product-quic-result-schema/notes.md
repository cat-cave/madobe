# m4-product-quic-result-schema

This node adds a checked Rust model and golden fixture for the product QUIC smoke result contract.

The contract covers:

- `transport=quic`
- `productQuic=true`
- sender and receiver roles
- payload byte count
- SHA-256 payload validation
- receiver acknowledgement
- optional certificate fingerprint SHA-256
- explicit non-claims for decode, render, presentation, and latency unless matching downstream evidence is attached

This is schema and fixture evidence only. It does not claim a live product QUIC run, cross-device success, decode,
render, presentation, or latency behavior.
