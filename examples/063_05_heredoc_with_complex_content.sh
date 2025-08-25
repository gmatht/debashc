#!/bin/bash

# 5. Heredoc with complex content and nested expansions
cat << 'EOF' | grep -v "^#" | sed 's/^/  /'
# This is a comment
$(echo "Command substitution")
${var:-default}
$(( 1 + 2 * 3 ))
EOF
