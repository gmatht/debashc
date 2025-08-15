#!/usr/bin/env bash

# Associative array examples
set -euo pipefail

echo "== Associative arrays =="
declare -A map
map[foo]=bar
map[answer]=42
map[two]="1 + 1"
echo "${map[foo]}"      # bar
echo "${map[answer]}"   # 42

# Show all keys and values
for k in "${!map[@]}"; do echo "$k => ${map[$k]}"; done | sort
