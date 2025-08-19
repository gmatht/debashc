#!/usr/bin/env bash

# Default values in parameter expansion
set -euo pipefail

echo "== Default values =="
unset maybe
echo "${maybe:-default}"  # default
echo "${maybe:=default}"  # default (and sets maybe)
echo "${maybe:?error}"    # error if unset
