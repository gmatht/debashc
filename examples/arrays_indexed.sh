#!/usr/bin/env bash

# Indexed array examples
set -euo pipefail

echo "== Indexed arrays =="
arr=(one two three )
echo "${arr[1]}"        # two
echo "${#arr[@]}"       # 3
for x in "${arr[@]}"; do printf "%s " "$x"; done; echo
