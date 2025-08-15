#!/usr/bin/env bash

# Pattern matching and regex examples
# Demonstrates [[ ]] test operator with patterns and regex

set -euo pipefail

echo "== [[ pattern and regex ]]"
s="file.txt"
[[ $s == *.txt ]] && echo pattern-match
[[ $s =~ ^file\.[a-z]+$ ]] && echo regex-match

echo "== extglob =="
shopt -s extglob
f1="file.js"; f2="thing.min.js"
[[ $f1 == !(*.min).js ]] && echo f1-ok
[[ $f2 == !(*.min).js ]] || echo f2-filtered

echo "== nocasematch =="
shopt -s nocasematch
word="Foo"; [[ $word == foo ]] && echo ci-match
