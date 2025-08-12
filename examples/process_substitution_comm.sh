#!/usr/bin/env bash

# Process substitution with comm examples
set -euo pipefail

echo "== Process substitution with comm =="
comm -12 <(printf 'a\nb\n') <(printf 'b\nc\n')
