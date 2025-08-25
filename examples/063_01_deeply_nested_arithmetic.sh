#!/bin/bash

# 1. Deeply nested arithmetic expressions with mixed operators
result=$(( (a + b) * (c - d) / (e % f) + (g ** h) - (i << j) | (k & l) ^ (m | n) ))
echo "Deeply nested arithmetic result: $result"
