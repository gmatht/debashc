#!/bin/bash

# 3. Here-documents with complex delimiters and nested structures
echo "Testing complex here-documents..."
cat <<'EOF'
This is a test line
This is not a test line
This is another test line
EOF
