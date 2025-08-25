#!/bin/bash

# 7. Complex array operations with associative arrays
declare -A config
config["user"]="admin"
config["host"]="localhost"
config["port"]="8080"

echo "Config: ${config[@]}"
