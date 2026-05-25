#!/bin/bash
set -e
SOURCE="${1:-_sources/JP-Subtitles}"
MIN="${2:-3}"
for dir in "$SOURCE"/*/; do
    name=$(basename "$dir")
    echo "=== $name ==="
    cargo run --bin extract-anime-freq -- "$dir" --min-count "$MIN" 2>&1
done
echo "=== ALL DONE ==="
