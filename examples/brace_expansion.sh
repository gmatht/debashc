#!/usr/bin/env bash

# Brace expansion examples
# Demonstrates various brace expansion patterns in Bash

set -euo pipefail

echo "== Basic brace expansion =="
echo {1..5}
echo {a..c}
echo {00..04..2}

echo "== Advanced brace expansion =="
echo {a,b,c}{1,2,3}
echo {1..10..2}
echo {a..z..3}

echo "== Practical examples =="
# Create numbered files
touch file_{001..005}.txt
ls file_*.txt
rm file_*.txt
