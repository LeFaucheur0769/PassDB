#!/bin/sh

if [ "$#" -ne 2 ]; then
  echo "Usage: $0 <input_dir> <output_dir>"
  exit 1
fi

INPUT="$1"
OUTPUT="$2"

echo "Sorting files from '$INPUT' into '$OUTPUT'..."

# Create the output directory if it doesn't exist
mkdir -p "$OUTPUT"

# Traverse all files in the input directory
find "$INPUT" -type f | while IFS= read -r src; do
    # Calculate destination path
    dst="$OUTPUT/${src#$INPUT/}"

    # Create destination directory if needed
    mkdir -p "$(dirname "$dst")"

    # Append content and delete the original file if successful
    cat "$src" >> "$dst" && rm "$src"
done
