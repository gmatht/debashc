#!/usr/bin/env bash

# Here-string examples
set -euo pipefail

echo "== Here-string with grep -o =="
grep -o pattern <<< "some pattern here"
