#!/bin/sh

# Check argument count
if [ "$#" -ne 2 ]; then
  echo "Usage: $0 <input_file> <output_directory>"
  exit 1
fi

INPUT="$1"
OUTPUT_DIR="$2"

mkdir -p "$OUTPUT_DIR"  
echo "sorting $INPUT to $OUTPUT_DIR"
while IFS= read -r line; do
  # Skip empty lines
  [ -z "$line" ] && continue

  # Sort words in the line
  sorted_line=$(echo "$line" | tr ' ' '\n' | sort | tr '\n' ' ' | sed 's/ *$//')

  # Extract prefix (first 3 letters of sorted line)
  prefix=$(echo "$sorted_line" | sed 's/^ *//' | cut -c1-3 | tr '[:upper:]' '[:lower:]')

  # Skip if prefix is empty or only whitespace
  if [ -z "$prefix" ] || echo "$prefix" | grep -q '^[[:space:]]*$'; then
    continue
  fi

  # Write to appropriate file
  echo "$sorted_line" >> "$OUTPUT_DIR/${prefix}.txt"

done < "$INPUT"
