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
rm -rf "$MERGED_DIR"
mkdir -p "$MERGED_DIR"

# --- Step 1: Parse merge groups ---
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
            [ -n "$entry" ] && [ -z "$acc" ] && acc="$entry"
            [ -n "$entry" ] && [ -n "$acc" ] && acc="$acc|$entry"
        done
        GROUP_DIRS[${#GROUP_DIRS[@]}-1]="$acc"
    fi
done < "$CONFIG"
echo "   ${#GROUP_NAMES[@]} merge groups defined"

# Collect all source dir names that are part of merge groups
group_members=""
for dirlist in "${GROUP_DIRS[@]}"; do
    IFS='|' read -ra dirs <<< "$dirlist"
    for d in "${dirs[@]}"; do
        group_members="$group_members|$d"
    done
done

# --- Step 2: Process merge groups ---
for ((i = 0; i < ${#GROUP_NAMES[@]}; i++)); do
    group="${GROUP_NAMES[$i]}"
    mkdir -p "$MERGED_DIR/$group"

    IFS='|' read -ra dirs <<< "${GROUP_DIRS[$i]}"
    total=0
    for d in "${dirs[@]}"; do
        src="$SOURCE_DIR/$d"
        if [ ! -d "$src" ]; then
            echo "   SKIP (not found): $d"
            continue
        fi
        slug="$(echo "$d" | tr '[:upper:]' '[:lower:]' | sed 's/[^a-z0-9]/-/g; s/--*/-/g; s/^-//; s/-$//')"
        while IFS= read -r -d '' f; do
            fname="$(basename "$f")"
            ln -sf "$(realpath "$f")" "$MERGED_DIR/$group/${slug}__${fname}"
            total=$((total + 1))
        done < <(find "$src" -maxdepth 3 \( -name '*.srt' -o -name '*.ass' \) -print0)
    done
    echo "   $group → $total files"
done

# --- Step 3: Add individual anime (50+ files, not in any merge group) ---
count=0
skipped=0
for d in "$SOURCE_DIR"/*/; do
    name="$(basename "$d")"
    files="$(find "$d" -maxdepth 3 \( -name '*.srt' -o -name '*.ass' \) 2>/dev/null | wc -l | tr -d ' ')"

    [ "$files" -lt 50 ] && { skipped=$((skipped + 1)); continue; }

    # Skip if already part of a merge group
    if echo "$group_members" | grep -F -q "|$name|"; then
        skipped=$((skipped + 1))
        continue
    fi

    mkdir -p "$MERGED_DIR/$name"
    while IFS= read -r -d '' f; do
        fname="$(basename "$f")"
        ln -sf "$(realpath "$f")" "$MERGED_DIR/$name/${fname}"
    done < <(find "$d" -maxdepth 3 \( -name '*.srt' -o -name '*.ass' \) -print0)
    count=$((count + 1))
done

echo ""
echo "=== Summary ==="
echo "   Merged groups:  ${#GROUP_NAMES[@]}"
echo "   Individual 50+: $count"
echo "   Skipped (<50 or in merge): $skipped"
echo "   Total in $MERGED_DIR: $(ls -d "$MERGED_DIR"/*/ 2>/dev/null | wc -l | tr -d ' ') anime"
