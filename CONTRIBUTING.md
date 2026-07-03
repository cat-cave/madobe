# Contributing

Use Nix as the command boundary:

```sh
nix develop
just verify
```

Pull requests should keep generated files out of version control unless the generator is unavailable in CI.
New Rust crates must inherit the workspace license, lints, edition, and Rust version.
