#!/bin/bash
set -e
SOURCE="${1:-_sources/kitsunekko/subtitles/anime_tv}"
MIN_FIXED="${2:-auto}"
for dir in "$SOURCE"/*/; do
    name=$(basename "$dir")
    files=$(find "$dir" -maxdepth 3 \( -name '*.srt' -o -name '*.ass' \) | wc -l)
    if [ "$files" -eq 0 ]; then
        continue
    fi
    if [ "$MIN_FIXED" = "auto" ]; then
        if   [ "$files" -lt 50 ];  then min=3
        elif [ "$files" -lt 200 ]; then min=5
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
