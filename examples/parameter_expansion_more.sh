#!/usr/bin/env bash

# More parameter expansion examples
set -euo pipefail

echo "== More parameter expansion =="
var="hello world"
echo "${var#hello}"      #  world
echo "${var%world}"      # hello 
echo "${var//o/0}"       # hell0 w0rld
