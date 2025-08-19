#!/usr/bin/env bash

# Case-insensitive matching examples
set -euo pipefail

echo "== nocasematch =="
shopt -s nocasematch
word="Foo"; [[ $word == foo ]] && echo ci-match
