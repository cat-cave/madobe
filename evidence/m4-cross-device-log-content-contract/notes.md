# m4-cross-device-log-content-contract

This node tightens explicit generic cross-device result validation only. It adds stable log-token checks for the
existing generic `linux_host_log` and `mac_client_log` artifact kinds when the result claims frames were sent or the
overall result passed.

Validation uses checked-in fixtures and generated local artifacts under `target/`; it does not require or claim Mac
hardware, a live LAN run, product QUIC smoke validation, workflow dispatch, credentials, or PR #49 changes.
