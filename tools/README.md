# Development tools

Scripts used to set up development environments should live in this folder,
with the exception of shell.nix since it is a Nix standard for it to
reside in the root.

## vscode-lldb.sh

This script will build the patched CodeLLDB plugin for use on NixOS
and install it into the given extensions folder. Example invocation:

```bash
$ ./vscode-lldb.sh ~/.vscode-server/extensions/
```
