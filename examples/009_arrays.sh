#!/usr/bin/env bash

# Array examples - indexed and associative arrays
# Demonstrates basic array operations in Bash

set -euo pipefail

echo "== Indexed arrays =="
arr=(one two three )
echo "${arr[1]}"        # two
echo "${#arr[@]}"       # 3
for x in "${arr[@]}"; do printf "%s " "$x"; done; echo

echo "== Associative arrays =="
declare -A map
map[foo]=bar
map[answer]=42
map[two]="1 + 1"
echo "${map[foo]}"      # bar
echo "${map[answer]}"   # 42

# Show all keys and values
for k in "${!map[@]}"; do echo "$k => ${map[$k]}"; done | sort #Do not care about the order of the elements?
