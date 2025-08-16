#!/bin/bash

# Case statement examples
# This demonstrates the bash case statement syntax and common usage patterns

echo "=== Basic Case Statement Example ==="

# Example 1: Basic case statement with simple patterns
case "$1" in
    "start")
        echo "Starting the service..."
        ;;
    "stop")
        echo "Stopping the service..."
        ;;
    "restart")
        echo "Restarting the service..."
        ;;
    *)
        echo "Usage: $0 {start|stop|restart}"
        exit 1
        ;;
esac

echo "=== Case Statement with Pattern Matching ==="

# Example 2: Case statement with pattern matching
filename="$2"
case "$filename" in
    *.txt)
        echo "Processing text file: $filename"
        ;;
    *.sh)
        echo "Processing shell script: $filename"
        ;;
    *.py)
        echo "Processing Python file: $filename"
        ;;
    *)
        echo "Unknown file type: $filename"
        ;;
esac

echo "=== Case Statement with Multiple Patterns ==="

# Example 3: Case statement with multiple patterns per case
case "$3" in
    "help"|"h"|"-h"|"--help")
        echo "Help information:"
        echo "  start  - Start the service"
        echo "  stop   - Stop the service"
        echo "  status - Show service status"
        ;;
    "status"|"s"|"-s"|"--status")
        echo "Service status: Running"
        ;;
    *)
        echo "Unknown option: $3"
        ;;
esac

echo "=== Case Statement with Character Classes ==="

# Example 4: Case statement with character classes
case "$4" in
    [0-9])
        echo "Single digit: $4"
        ;;
    [a-z])
        echo "Lowercase letter: $4"
        ;;
    [A-Z])
        echo "Uppercase letter: $4"
        ;;
    [0-9][0-9])
        echo "Two digit number: $4"
        ;;
    *)
        echo "Other character: $4"
        ;;
esac

echo "=== Case Statement with Default Action ==="

# Example 5: Case statement with default action
case "$5" in
    "red")
        echo "Color is red"
        ;;
    "green")
        echo "Color is green"
        ;;
    "blue")
        echo "Color is blue"
        ;;
esac

echo "=== Case Statement with Commands ==="

# Example 6: Case statement with command execution
case "$6" in
    "ls")
        ls -la
        ;;
    "date")
        date
        ;;
    "pwd")
        pwd
        ;;
    "whoami")
        whoami
        ;;
    *)
        echo "Available commands: ls, date, pwd, whoami"
        ;;
esac
