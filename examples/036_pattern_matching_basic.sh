#!/usr/bin/env bash

# Basic pattern matching examples
set -euo pipefail

echo "== [[ pattern and regex ]]"
s="file.txt"
[[ $s == *.txt ]] && echo pattern-match
[[ $s =~ ^file\.[a-z]+$ ]] && echo regex-match
