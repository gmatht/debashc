#!/bin/bash

# 18. Complex here-string with nested expansions
tr '[:upper:]' '[:lower:]' <<< "$(echo "UPPER: ${var^^}")"
