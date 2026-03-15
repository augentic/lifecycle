#!/usr/bin/env bash

# Symlink local plugins into Cursor's cache so edits are picked up on restart.
# Usage: ./scripts/local-plugins.sh

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CACHE_DIR="$HOME/.cursor/plugins/cache/augentic"

# clear the augentic cache directory
rm -rf "$CACHE_DIR"
mkdir -p "$CACHE_DIR"

# symlink each plugin that has a `.cursor-plugin` manifest
for dir in "$REPO_ROOT"/plugins/*/; do
  plugin="$(basename "$dir")"
  [ -f "$dir/.cursor-plugin/plugin.json" ] || continue

  mkdir -p "$CACHE_DIR/$plugin"
  ln -sf "$dir" "$CACHE_DIR/$plugin/main"
done
