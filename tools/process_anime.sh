#!/bin/bash
set -e
SOURCE="${1:-_sources/merged}"
MIN_FIXED="${2:-auto}"
for dir in "$SOURCE"/*/; do
    [ -d "$dir" ] || continue
    name=$(basename "$dir")
    files=$(find "$dir" -maxdepth 1 -type l | wc -l | tr -d ' ')
    if [ "$MIN_FIXED" = "auto" ]; then
        if   [ "$files" -lt 200 ]; then min=5
        elif [ "$files" -lt 500 ]; then min=7
        else                            min=10
        fi
    else
        min="$MIN_FIXED"
    fi
    echo "=== $name ($files files, min-count=$min) ==="
    cargo run --bin extract-anime-freq -- "$dir" --min-count "$min" 2>&1
done
echo "=== ALL DONE ==="
