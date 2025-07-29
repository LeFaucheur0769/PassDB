#!/bin/bash

SOURCE_DIR="/root/PassDB/sorted"
DEST_DIR="/root/PassDB/import"

mkdir -p "$DEST_DIR"

# Loop through files in the source directory
find "$SOURCE_DIR" -maxdepth 1 -type f | while read -r file; do
    filename=$(basename "$file")

    # If filename does NOT start with [a-zA-Z0-9+]
    if [[ ! "$filename" =~ ^[a-zA-Z0-9+] ]]; then
        echo "Moving: $filename"
        mv "$file" "$DEST_DIR/"
    fi
done