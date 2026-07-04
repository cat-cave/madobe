# m3-audit notes

M3 is complete only under a conservative endpoint-foundation boundary:

- Rust protocol fixtures and Swift mirror compatibility are in place.
- Local Mac VideoToolbox AV1 capability and checked-in sample decode are proven.
- Local Metal renderer skeleton/timing evidence is proven, but onscreen presentation from a received network sample is not.
- Mac receive queue validation is synthetic/in-memory and correctly bounded.
- Linux-to-Mac TCP LAN sample transfer is proven by sender and receiver evidence.

M3 does not claim product QUIC, network-driven VideoToolbox decode, Metal render or presentation of a received sample, or cross-device latency. Those are follow-up DAG candidates for the next milestone rather than M3 completion blockers.
