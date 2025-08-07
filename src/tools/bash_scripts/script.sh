#!/bin/bash

if [ "$#" -ne 1 ]; then
  echo "Usage: $0 <input_file>"
  exit 1
fi

INPUT="$1"

if [[ "$INPUT" == *.tar.gz ]]; then
	mkdir -p Combo
	tar -xzf "$INPUT" -C Combo
#	/root/PassDB/src/folder Combo /root/PassDB/import
else
	echo"File does not end with .tar.gz"
fi
