#!/usr/bin/env bash

# Demonstrates reading command-line arguments
# This example is intentionally simple so it parses cleanly

echo "== Argument count =="
echo "$#"

echo "== Arguments =="
for a in "$@"; do
  echo "Arg: $a"
done


