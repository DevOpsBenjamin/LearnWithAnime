#!/bin/bash
set -euo pipefail

SOURCE_DIR="_sources/kitsunekko/subtitles/anime_tv"
MERGED_DIR="_sources/merged"
CONFIG="merge_groups.toml"

if [ ! -f "$CONFIG" ]; then
    echo "$CONFIG not found"
    exit 1
fi

echo "=== Preparing $MERGED_DIR ==="
rm -rf "$MERGED_DIR" 2>/dev/null; mkdir -p "$MERGED_DIR"

# --- Parse merge groups ---
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
        acc=""
        for entry in "${entries[@]}"; do
            entry="$(echo "$entry" | sed 's/^[[:space:]]*//;s/[[:space:]]*$//')"
            entry="${entry#\"}"
            entry="${entry%\"}"
            if [ -n "$entry" ]; then
                if [ -z "$acc" ]; then acc="$entry"; else acc="$acc|$entry"; fi
            fi
        done
        GROUP_DIRS[${#GROUP_DIRS[@]}-1]="$acc"
    fi
done < "$CONFIG"

# Collect member dirs
group_members=""
for dirlist in "${GROUP_DIRS[@]}"; do
    IFS='|' read -ra dirs <<< "$dirlist"
    for d in "${dirs[@]}"; do
        group_members="$group_members|$d"
    done
done

# --- Merge groups ---
echo ""
echo "--- Merge groups ---"
for ((i = 0; i < ${#GROUP_NAMES[@]}; i++)); do
    group="${GROUP_NAMES[$i]}"
    mkdir -p "$MERGED_DIR/$group"
    IFS='|' read -ra dirs <<< "${GROUP_DIRS[$i]}"
    for d in "${dirs[@]}"; do
        src="$SOURCE_DIR/$d"
        [ -d "$src" ] && cp -rs "$(realpath "$src")" "$MERGED_DIR/$group/" 2>/dev/null || true
    done
    total=$(find "$MERGED_DIR/$group" -type l \( -name '*.srt' -o -name '*.ass' \) 2>/dev/null | wc -l | tr -d ' ')
    echo "   $group → $total files"
done

# --- Precompute 50+ anime ---
echo ""
echo "--- Individual 50+ ---"
eligible=""
for d in "$SOURCE_DIR"/*/; do
    name="$(basename "$d")"
    files=$(find "$d" -maxdepth 3 \( -name '*.srt' -o -name '*.ass' \) 2>/dev/null | wc -l | tr -d ' ')
    [ "$files" -ge 50 ] || continue
    eligible="$eligible|$name"
done

count=0
IFS='|' read -ra names <<< "$eligible"
for name in "${names[@]}"; do
    [ -z "$name" ] && continue
    if echo "$group_members" | grep -F -q "|$name|"; then
        continue
    fi
    cp -rs "$(realpath "$SOURCE_DIR/$name")" "$MERGED_DIR/" 2>/dev/null || true
    count=$((count + 1))
done

echo ""
echo "=== Summary ==="
echo "   Merge groups:  ${#GROUP_NAMES[@]}"
echo "   Individual 50+: $count"
echo "   Total: $(ls -d "$MERGED_DIR"/*/ 2>/dev/null | wc -l | tr -d ' ') anime"
