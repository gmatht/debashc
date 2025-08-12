#!/usr/bin/env bash

# mapfile examples
set -euo pipefail

echo "== readarray/mapfile =="
mapfile -t lines < <(printf 'x\ny\n')
printf '%s ' "${lines[@]}"; echo
