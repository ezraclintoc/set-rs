{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };
        rust = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
          targets = [ "wasm32-unknown-unknown" ];
        };
      in {
        devShells.default = pkgs.mkShell {
          packages = [
            rust
            pkgs.wrangler        # nixpkgs build has a patched workerd that runs on NixOS
            pkgs.cargo-generate
            pkgs.trunk           # for the Leptos frontend
            pkgs.nodejs
          ];
        };
      });
}
