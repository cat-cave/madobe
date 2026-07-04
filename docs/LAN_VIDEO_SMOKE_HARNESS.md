# LAN Video Smoke Harness

`m3-lan-video-smoke-harness` adds a dependency-free TCP harness for the
checked-in AV1 IVF sample:

```text
evidence/m2-nvenc-encode-sample/sample-av1.ivf
```

The helper sends the IVF file as opaque payload bytes with deterministic frame
metadata. The receiver validates:

- frame metadata: AV1, 160x90, frame id 1, keyframe, SHA-256 metadata.
- payload byte count: 84 bytes.
- payload SHA-256:
  `51945e4cd903e28019fbbfbe74572b5d836f6ef1184cb782b142aba1d5201875`.

## CLI

Start one receiver:

```sh
nix develop -c cargo run -q -p madobectl -- \
  video-smoke receive \
  --bind 127.0.0.1:47031 \
  --evidence-dir evidence/m3-lan-video-smoke-harness
```

Then run one sender:

```sh
nix develop -c cargo run -q -p madobectl -- \
  video-smoke send \
  --addr 127.0.0.1:47031 \
  --sample evidence/m2-nvenc-encode-sample/sample-av1.ivf \
  --evidence-dir evidence/m3-lan-video-smoke-harness
```

The receiver accepts one connection and exits after validation.

## Artifacts

When `--evidence-dir` is provided, the helper writes:

- `sender.log`
- `sender-timeline.json`
- `receiver-listening.log`
- `receiver.log`
- `receiver-timeline.json`
- `result.json`

Timelines use sequence numbers only. They do not use wall-clock timing and do
not support latency claims.

## Scope Exclusions

This harness is only cross-device smoke preparation. It is explicitly:

- not QUIC.
- not the product transport.
- not VideoToolbox decode.
- not Metal render.
- not presentation evidence.
- not latency proof.
