# m2-capture-contract Notes

This node adds a dependency-free Rust contract crate for captured frame metadata:
`madobe-capture`.

Acceptance mapping:

- Capture metadata types compile under workspace lints: `evidence/m2-capture-contract/just-check.log`.
- Types represent format, modifier, size, damage, sync, and timestamps:
  `crates/capture/src/lib.rs`.
- Serialization fixtures or golden debug output exist:
  `crates/capture/fixtures/captured-frame-dmabuf.json` and the
  `madobe-capture::captured_frame` tests in `evidence/m2-capture-contract/just-test.log`.
- No platform API is claimed beyond documented evidence: DMA-BUF and sync file descriptors are stored as
  opaque backend-local integer evidence values only. The crate does not own, duplicate, close, wait on, or
  otherwise operate on descriptors, and it does not claim compositor, kernel, or graphics API availability.

Dependency note: no external Rust dependencies were added.
