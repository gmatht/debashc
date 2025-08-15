#!/usr/bin/env bash

# Practical ANSI-C quoting examples
set -euo pipefail

echo "== Practical examples =="
# Create a formatted table
printf $'%-10s %-10s %s\n' "Name" "Age" "City"
printf $'%-10s %-10s %s\n' "John" "25" "NYC"
printf $'%-10s %-10s %s\n' "Jane" "30" "LA"
