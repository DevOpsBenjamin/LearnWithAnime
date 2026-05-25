#!/bin/bash
set -euo pipefail

SOURCE_DIR="_sources/kitsunekko/subtitles/anime_tv"
MERGED_DIR="_sources/merged"
CONFIG="merge_groups.toml"

adaptive_min() {
    local files=$1
    if   [ "$files" -lt 50 ];  then echo 3
    elif [ "$files" -lt 200 ]; then echo 5
    elif [ "$files" -lt 500 ]; then echo 7
    else                            echo 10
    fi
}

if [ ! -f "$CONFIG" ]; then
    echo "$CONFIG not found"
    exit 1
fi

echo "=== Parsing $CONFIG ==="

declare -a GROUP_NAMES=()
declare -a GROUP_DIRS=()

while IFS= read -r line; do
    line="${line%%#*}"
    line="$(echo "$line" | sed 's/^[[:space:]]*//;s/[[:space:]]*$//')"
    [ -z "$line" ] && continue

    prefix="[mappings."
    if [[ "$line" == "$prefix"* ]] && [[ "$line" == *"]" ]]; then
        inner="${line#$prefix}"
        inner="${inner%]}"
        GROUP_NAMES+=("$inner")
        GROUP_DIRS+=("")
    elif [[ "$line" == "dirs ="* ]] && [ "${#GROUP_NAMES[@]}" -gt 0 ]; then
        raw="${line#dirs = }"
        raw="${raw#[}"
        raw="${raw%]}"
        IFS=',' read -ra entries <<< "$raw"
        accumulated=""
        for entry in "${entries[@]}"; do
            entry="$(echo "$entry" | sed 's/^[[:space:]]*//;s/[[:space:]]*$//')"
            entry="${entry#\"}"
            entry="${entry%\"}"
            if [ -n "$entry" ]; then
                if [ -z "$accumulated" ]; then
                    accumulated="$entry"
                else
                    accumulated="$accumulated|$entry"
                fi
            fi
        done
        GROUP_DIRS[${#GROUP_DIRS[@]}-1]="$accumulated"
    fi
done < "$CONFIG"

echo "   Found ${#GROUP_NAMES[@]} merge groups"
for ((i = 0; i < ${#GROUP_NAMES[@]}; i++)); do
    echo "     ${GROUP_NAMES[$i]}"
done

for ((i = 0; i < ${#GROUP_NAMES[@]}; i++)); do
    group="${GROUP_NAMES[$i]}"
    echo ""
    echo "=== $group ==="

    merged_path="$MERGED_DIR/$group"
    rm -rf "$merged_path"
    mkdir -p "$merged_path"

    IFS='|' read -ra dirs <<< "${GROUP_DIRS[$i]}"
    total_files=0

    for d in "${dirs[@]}"; do
        src="$SOURCE_DIR/$d"
        if [ ! -d "$src" ]; then
            echo "   Directory not found: $d"
            continue
        fi
        slug="$(echo "$d" | tr '[:upper:]' '[:lower:]' | sed 's/[^a-z0-9]/-/g' | sed 's/--*/-/g; s/^-//; s/-$//')"
        find "$src" -maxdepth 3 \( -name '*.srt' -o -name '*.ass' \) -print0 | while IFS= read -r -d '' f; do
            fname="$(basename "$f")"
            target="$merged_path/${slug}__${fname}"
            ln -sf "$(realpath "$f")" "$target"
        done
    done

    total_files_actual="$(find "$merged_path" -type l | wc -l | tr -d ' ')"
    if [ "$total_files_actual" -eq 0 ]; then
        echo "   No files found for $group, skipping"
        rm -rf "$merged_path"
        continue
    fi

    min="$(adaptive_min "$total_files_actual")"
    echo "   $total_files_actual files, min-count=$min"

    cargo run --bin extract-anime-freq -- "$merged_path" --min-count "$min" 2>&1

    rm -rf "$merged_path"
    echo "   Cleaned up merged dir"
done

echo ""
echo "=== ALL DONE ==="
