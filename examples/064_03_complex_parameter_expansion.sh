#!/bin/bash

# 3. Complex parameter expansion with nested substitutions
name="John Doe"
echo "Hello ${name// /_}"  # Replace spaces with underscores
echo "Length: ${#name}"    # String length
echo "First: ${name:0:4}"  # Substring
echo "Last: ${name: -3}"   # Last 3 characters
