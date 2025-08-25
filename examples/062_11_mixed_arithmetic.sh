#!/bin/bash

# 11. Arithmetic with mixed bases and complex expressions
echo "Testing mixed arithmetic..."
hex=255
octal=511
binary=10
result=$(( hex + octal + binary ))
echo "Mixed base result: $result"
