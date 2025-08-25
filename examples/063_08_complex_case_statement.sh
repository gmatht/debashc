#!/bin/bash

# 8. Complex case statement with patterns and command substitution
case "$(echo "$var" | tr '[:upper:]' '[:lower:]')" in
    *[0-9]*)
        case "${var,,}" in
            *pattern*)
                echo "Double nested pattern"
                ;;
            *)
                echo "Single nested pattern"
                ;;
        esac
        ;;
    *)
        echo "No numbers"
        ;;
esac
