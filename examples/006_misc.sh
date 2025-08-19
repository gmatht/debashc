#!/usr/bin/env bash

echo "== Subshell =="
( echo inside-subshell )

echo "== Simple pipeline =="
echo "alpha beta" | grep beta


