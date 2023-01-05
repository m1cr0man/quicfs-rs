{
  description = "QuicFS Rust";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        devDependencies = with pkgs; [
          openssl
          pkgconfig
          (rust-bin.nightly."2022-12-24".default.override {
            extensions = [ "rust-src" "rustfmt" ];
          })
          lldb
          rust-analyzer
          cmake
          libev
          uthash
          protobuf
          protolint
        ];
      in
      rec {
        devShells.default = pkgs.mkShell {
          buildInputs = devDependencies;
        };

        packages.venv = pkgs.symlinkJoin {
          name = "quicfs-rs-venv";
          paths = devDependencies;
        };

        # Nix < 2.7 compatibility
        devShell = devShells.default;
      }
    );
}
