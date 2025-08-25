#!/bin/bash

# 23. Complex error handling with traps
trap 'echo "Error on line $LINENO"; exit 1' ERR
trap 'echo "Cleaning up..."; rm -f /tmp/temp_*' EXIT

echo "Traps set up"
