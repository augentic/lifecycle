#!/usr/bin/env bash

# Restore the original augentic marketplace cache, removing local symlinks.
# Usage: ./scripts/revert-plugins.sh

set -euo pipefail

rm -rf "$HOME/.cursor/plugins/cache/augentic"
