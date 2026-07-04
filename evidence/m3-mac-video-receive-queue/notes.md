# m3-mac-video-receive-queue

This node adds a Swift client-side receive queue for synthetic encoded video
samples.

It validates:

- payload byte count against `EncodedVideoFrameMetadata.payloadBytes`
- SHA-256 payload hash against `payloadHash.value`
- queue depth of one ready frame
- deterministic stale-frame drop events
- Codable receive/dequeue timeline output

It does not claim live network transport, Linux capture, VideoToolbox decode,
Metal render, or cross-device latency behavior.
