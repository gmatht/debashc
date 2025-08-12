#!/usr/bin/env bash

# Practical brace expansion examples
set -euo pipefail

echo "== Practical examples =="
# Create numbered files
touch file_{001..005}.txt
ls file_*.txt
rm file_*.txt
