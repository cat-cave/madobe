# m4-product-quic-result-artifact-check

This node adds a local `jq` checker for product QUIC `result.json` artifacts.

The checker validates the checked-in golden fixture by default and includes negative self-tests for:

- obsolete flat product QUIC fields such as `framesSent` and `validated`
- unsupported downstream decode claims without a `decode_evidence` artifact

No live product QUIC or cross-device behavior is claimed by this node.
