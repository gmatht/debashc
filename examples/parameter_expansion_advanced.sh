#!/usr/bin/env bash

# Advanced parameter expansion examples
set -euo pipefail

echo "== Advanced parameter expansion =="
path="/tmp/file.txt"
echo "${path##*/}"       # file.txt
echo "${path%/*}"        # /tmp
s2="abba"; echo "${s2//b/X}"  # aXXa
