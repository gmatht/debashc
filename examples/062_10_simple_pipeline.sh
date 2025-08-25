#!/bin/bash

# 10. Simple pipeline without complex redirections
echo "Testing simple pipeline..."
ls -la | grep "^d" | head -5
