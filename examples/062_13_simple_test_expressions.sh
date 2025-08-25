#!/bin/bash

# 13. Simple test expressions to avoid parser issues
echo "Testing simple test expressions..."
if [[ -f "file.txt" ]]; then
    echo "File exists"
else
    echo "File does not exist"
fi
