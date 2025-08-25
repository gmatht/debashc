#!/bin/bash

# 24. Advanced parameter expansion with default values and transformations
input="${1:-default_value}"
sanitized="${input//[^a-zA-Z0-9]/_}"
uppercase="${sanitized^^}"
echo "Input: '$input' -> Sanitized: '$sanitized' -> Uppercase: '$uppercase'"
