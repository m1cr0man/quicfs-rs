let
    rustVersion = "1.57.0";
    rustOverlayRev = "1efeb891b85c70ded412eb78a04bccb9badb14c6";
    rustOverlaySrc = builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/${rustOverlayRev}.tar.gz";
in { pkgs ? import <nixpkgs> {
    overlays = [ (import rustOverlaySrc) ];
} }:
with pkgs;
mkShell {
  buildInputs = [
    (rust-bin.stable."${rustVersion}".default.override {
        extensions = [ "rust-src" "rustfmt" ];
    })
    lldb
    rust-analyzer
  ];
}
