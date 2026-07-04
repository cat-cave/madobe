{ self, nixpkgs, flake-utils }:

flake-utils.lib.eachDefaultSystem (system:
let
  pkgs = import nixpkgs { inherit system; };
  lib = pkgs.lib;
  optionalPackage = name: lib.optionals (lib.hasAttr name pkgs) [ (lib.getAttr name pkgs) ];
  rustTools = with pkgs; [
    cargo
    clippy
    rustc
    rustfmt
  ];
  repoTools = with pkgs; [
    bash
    coreutils
    findutils
    git
    libdrm
    libgbm
    gnugrep
    gnused
    just
    jq
    llvmPackages.llvm
    nixpkgs-fmt
    pkg-config
    shellcheck
    shfmt
    wayland
    wayland-protocols
    wayland-scanner
  ];
  optionalTools =
    optionalPackage "actionlint"
    ++ optionalPackage "cargo-audit"
    ++ optionalPackage "cargo-deny"
    ++ optionalPackage "cargo-llvm-cov"
    ++ optionalPackage "cargo-machete"
    ++ optionalPackage "cargo-mutants"
    ++ optionalPackage "cargo-nextest"
    ++ optionalPackage "cargo-semver-checks"
    ++ optionalPackage "cargo-vet"
    ++ optionalPackage "deadnix"
    ++ optionalPackage "lychee"
    ++ optionalPackage "markdownlint-cli2"
    ++ optionalPackage "mise"
    ++ optionalPackage "statix"
    ++ optionalPackage "taplo"
    ++ optionalPackage "typos"
    ++ optionalPackage "zizmor";
  darwinTools = lib.optionals pkgs.stdenv.isDarwin (
    optionalPackage "swiftformat"
    ++ optionalPackage "swiftlint"
  );
  devPackages = rustTools ++ repoTools ++ optionalTools ++ darwinTools;
in
{
  devShells.default = pkgs.mkShell {
    packages = devPackages;
    LLVM_COV = "${pkgs.llvmPackages.llvm}/bin/llvm-cov";
    LLVM_PROFDATA = "${pkgs.llvmPackages.llvm}/bin/llvm-profdata";
    shellHook = ''
      echo "madobe dev shell: use 'just --list' to see commands"
    '';
  };

  formatter = pkgs.nixpkgs-fmt;

  checks.rust = pkgs.runCommand "madobe-rust-check"
    {
      nativeBuildInputs = rustTools;
      src = self;
    } ''
    cp -R "$src" source
    chmod -R u+w source
    cd source
    cargo check --workspace --all-targets --all-features
    touch "$out"
  '';
})
