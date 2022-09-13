{ pkgs ? import (builtins.getFlake "nixpkgs") {} }: pkgs.vscode-extensions.vadimcn.vscode-lldb
