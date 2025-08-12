#!/usr/bin/env bash

# Advanced brace expansion examples
set -euo pipefail

echo "== Advanced brace expansion =="
echo {a,b,c}{1,2,3}
echo {1..10..2}
echo {a..z..3}
