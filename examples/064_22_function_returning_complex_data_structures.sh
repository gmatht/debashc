#!/bin/bash

# 22. Function returning complex data structures
get_system_info() {
    local -A info
    info["os"]="$(uname -s)"
    info["arch"]="$(uname -m)"
    info["hostname"]="$(hostname)"
    info["user"]="$USER"
    
    # Return as associative array (bash 4+)
    declare -p info
}

# Test the function
get_system_info
