#!/bin/bash

# 17. Complex while loop with nested conditionals
while IFS=: read -r user pass uid gid info home shell; do
    if [[ "$uid" -gt 1000 ]] && [[ "$shell" != "/bin/false" ]]; then
        if [[ "$home" =~ ^/home/ ]]; then
            echo "User: $user (UID: $uid) - $home"
        fi
    fi
done < /etc/passwd
