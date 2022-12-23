# Quic FS

A network filesystem utilising QUIC.

## Development instructions

- Open the workspace in VS Code on your NixOS host
- Install the recommended extensions
- Run `tools/vscode-lldb.sh /path/to/vscode/extensions`
- Reload the window.
- Try debugging [main.rs](./src/main.rs) by opening it, adding a breakpoint, and hitting F5.

That should be it. Your default terminal in VS Code should be "Nix Shell", and
rust-analyzer should load just fine. If not, use ctrl+shift+p -> "Nix-env: Select Environment"
and make sure the shell.nix is selected.

### Generating code from the Thrift schema

Edit the IDL then you can run:

```bash
thrift -gen rs -out src -r -strict schema/quicfs.thrift
```

## Updating Rust

Just change the rustVersion in [shell.nix](./shell.nix). You may also have to update
the `rustOverlayRev` with the latest revision from [rust-overlay](https://github.com/oxalica/rust-overlay).
