# m0-ci-green Evidence

## Linux Local Checks

- `nix flake check`: passed.
- `nix develop -c just check`: passed.
- `nix develop -c just test`: passed.
- `qd doctor --strict`: passed.

## CI Context

- Initial hosted CI run failed on macOS because the workflow attempted to install and use Nix on macOS.
- This branch changes macOS CI to use native macOS tooling through `jdx/mise-action@v4.2.0`, Homebrew
  SwiftFormat/SwiftLint, and `just macos-check`.
- [PR CI run](https://github.com/cat-cave/madobe/actions/runs/28633801053)
- [Linux job passed](https://github.com/cat-cave/madobe/actions/runs/28633801053/job/84915937778)
- [macOS job passed](https://github.com/cat-cave/madobe/actions/runs/28633801053/job/84915937781)
