#!/bin/bash

input_file="../aggregator/time-recorder.txt"
output_file="../aggregator/time-recorder-corrected.txt"

line_length=19

while IFS= read -r line; do
  if [ -z "$line" ]; then
    continue
  fi

  while [ ${#line} -ge $line_length ]; do
    echo "${line:0:$line_length}" >> "$output_file"
    line="${line:$line_length}"
  done

  if [ -n "$line" ]; then
    read -r next_line
    line="${line}${next_line}"
    echo "$line" >> "$output_file"
  fi
done < "$input_file"

echo "Processed file saved as $output_file"