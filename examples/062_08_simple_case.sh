#!/bin/bash

# 8. Simple case statement to avoid parser issues
echo "Testing simple case patterns..."
case "$1" in
    "test")
        echo "Matched test"
        ;;
    *)
        echo "Default case"
        ;;
esac
