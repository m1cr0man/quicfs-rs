#!/usr/bin/env bash

if [ -z "$1" ]; then
    echo "Usage: $(basename $0) extensions_folder"
    exit 1
fi

set -euo pipefail
extdir="$1"

cd "$(dirname $0)"
nix-build vscode-lldb.nix -o vscode-lldb &

# Need to clean up/delete any existing installations
find "$extdir" -maxdepth 1 -name 'vadimcn.vscode-lldb*' -type d -exec chmod -R +w '{}' \;
find "$extdir" -maxdepth 1 -name 'vadimcn.vscode-lldb*' -exec rm -rf '{}' \;

wait
ln -s "$(pwd)"/vscode-lldb/share/vscode/extensions/vadimcn.vscode-lldb "$extdir"/vadimcn.vscode-lldb-1.6.99

echo "CodeLLDB configured. Restart VS Code."
