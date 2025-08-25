#!/bin/bash

# 11. Complex test expressions with extended operators
if [[ "$1" =~ ^[0-9]+$ ]] && [[ "$2" == "test" || "$2" == "debug" ]]; then
    echo "Valid input"
fi
