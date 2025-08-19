#!/usr/bin/env bash

# Extended glob examples
set -euo pipefail

echo "== extglob =="
shopt -s extglob
f1="file.js"; f2="thing.min.js"
[[ $f1 == !(*.min).js ]] && echo f1-ok
[[ $f2 == !(*.min).js ]] || echo f2-filtered
