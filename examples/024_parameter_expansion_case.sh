#!/usr/bin/env bash

# Case modification in parameter expansion
set -euo pipefail

echo "== Case modification in parameter expansion =="
name="world"
echo "${name^^}"        # WORLD
echo "${name,,}"        # world
echo "${name^}"         # World
