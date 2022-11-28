{
  description = "QuicFS Rust";

  inputs = {
    nixpkgs.url = "nixpkgs";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        rustVersion = "1.63.0";
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
      in
      with pkgs;
      rec {
        devShell = devShells.default;
        devShells.default = mkShell {
          buildInputs = [
            openssl
            pkgconfig
            (rust-bin.stable."${rustVersion}".default.override {
              extensions = [ "rust-src" "rustfmt" ];
            })
            lldb
            rust-analyzer
            cmake
            libev
            uthash
            thrift
          ];
        };
      }
    );
}
