#!/usr/bin/env bash

# Parameter expansion examples
# Demonstrates advanced parameter manipulation in Bash

set -euo pipefail

echo "== Case modification in parameter expansion =="
name="world"
echo "${name^^}"        # WORLD
echo "${name,,}"        # world
echo "${name^}"         # World

echo "== Advanced parameter expansion =="
path="/tmp/file.txt"
echo "${path##*/}"       # file.txt
echo "${path%/*}"        # /tmp
s2="abba"; echo "${s2//b/X}"  # aXXa

echo "== More parameter expansion =="
var="hello world"
echo "${var#hello}"      #  world
echo "${var%world}"      # hello 
echo "${var//o/0}"       # hell0 w0rld

echo "== Default values =="
unset maybe
echo "${maybe:-default}"  # default
echo "${maybe:=default}"  # default (and sets maybe)
echo "${maybe:?error}"    # error if unset
