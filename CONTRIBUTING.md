# Contributing

Use `just` as the shared command surface. Contributors should run the same
repository recipes, while each platform supplies those tools through its native
environment boundary.

## Linux Readiness

Linux uses Nix as the environment boundary. Enter the development shell before
running day-to-day repo commands:

```sh
nix develop
just check
```

For local PR readiness, run the full Linux gate through Nix:

```sh
nix develop -c just ci-local
```

## macOS Readiness

macOS uses native Xcode, Mise, and Homebrew commands. Nix is not required for
macOS onboarding or readiness checks.

Install or refresh the native macOS tools:

```sh
just macos-bootstrap
```

Then run the macOS readiness gate:

```sh
just macos-check
```

Pull requests should keep generated files out of version control unless the generator is unavailable in CI.
New Rust crates must inherit the workspace license, lints, edition, and Rust version.
