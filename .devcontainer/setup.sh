#!/usr/bin/env bash

set -ex

git config pull.rebase true

mkdir -p /home/vscode/.config/fish
cat <<EOF > /home/vscode/.config/fish/config.fish
set fish_greeting ""
set -gx FORCE_COLOR true
fish_vi_key_bindings
EOF
