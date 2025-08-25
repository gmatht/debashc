#!/bin/bash

# 8. Here-documents with variable interpolation
cat <<'EOF' > config.txt
User: $USER
Host: ${HOSTNAME:-localhost}
Path: $PWD
EOF

echo "Config file created"
