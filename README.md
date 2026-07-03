# madobe

madobe is a Nix-primary monorepo for the host, client, protocol, and tooling pieces of the
project.

M0 is intentionally small: it proves the workspace, command layer, CI shape, Rust link graph,
and Tuist-generated macOS shell before product protocol or capture work begins.

## Bootstrap

```sh
nix develop
just verify
```

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
