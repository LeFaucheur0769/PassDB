#!/bin/sh

if [ "$#" -ne 2 ]; then
  echo "Usage: $0 <input_file> <output_directory>"
  exit 1
fi

INPUT="$1"
OUTPUT="$2"
# OUTPUT_FILE="$OUTPUT_DIR/sorted.txt"

# mkdir -p "$OUTPUT_DIR"  
echo "Sorting $INPUT into $OUTPUT_FILE"

# awk -F: '{if (NF >= 3) print $2 ":" $3 ":" $1}' "$INPUT" > "$OUTPUT"
awk '
{
    # Match from start: capture URL (greedy up to last 2 colons), password, then email
    match($0, /^(https?:\/\/[^:]+):([^:]+):([^:]+)$/, groups)
    if (RSTART > 0) {
        url = groups[1]
        email = groups[2]
        pass = groups[3]
	      print email ":" pass   
    }
}
' "$OUTPUT" > "$OUTPUT"
sed -i 's/\r//' "$OUTPUT"