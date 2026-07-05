# Pre-Fix Failures

Before this node, the local nightly deep checks had these known failures:

- `nix develop -c lychee .` failed on dead links in `docs/PRD.md`:
  - `https://wiki.hypr.land/Configuring/Using-hyprctl/`
  - `https://wiki.hypr.land/Configuring/Monitors/`
  - `https://docs.pipewire.org/page_dma_buf.html`
  - `https://developer.apple.com/newsroom/2023/10/apple-unveils-m3-m3-pro-and-m3-max-the-most-advanced-chips-for-a-personal-computer/`
- `nix develop -c zizmor .github/workflows` failed because external GitHub Action refs were not pinned to full commit SHAs and warned that `actions/checkout` did not set `persist-credentials: false`.
- `nix develop -c deadnix --fail .` passed.
- `nix develop -c statix check .` failed on `nix/flake.nix:6:3`, suggesting `inherit (pkgs) lib` instead of `lib = pkgs.lib;`.
