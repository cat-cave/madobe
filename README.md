# madobe

madobe is a Nix-primary monorepo for the host, client, protocol, and tooling pieces of the
project.

M0 is intentionally small: it proves the workspace, command layer, CI shape, Rust link graph,
and Tuist-generated macOS shell before product protocol or capture work begins.

## Bootstrap

```sh
nix develop
just direct-capture-preflight
just verify
```

For Linux PR readiness, run the local CI parity gate:

```sh
nix develop -c just ci-local
```

That gate runs the direct-capture helper preflight before the broader
`verify` and `coverage` checks. macOS validation stays native with
`just macos-check`; it does not own Linux/Nix direct-capture dependencies.

Without Nix, the dependency-free Rust proof can still be checked with:

```sh
cargo test --workspace --all-features
cargo run -p hostd
cargo run -p madobectl -- hello
```

Both binaries should print:

```text
madobe 0.1.0 protocol=1 event=madobe.bootstrap ts=0 status=ok
```
