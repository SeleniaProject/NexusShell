#!/bin/bash
# NexusShell vs Bash ベンチマーク
set -e

function bench() {
  local shell="$1"
  local cmd="$2"
  local n=1000
  local start end
  start=$(date +%s%N)
  for i in $(seq 1 $n); do
    $shell -c "$cmd" > /dev/null
  done
  end=$(date +%s%N)
  echo $(( (end-start)/1000000 ))
}

echo "Bash benchmark:"
bash_time=$(bench bash "echo hello")
echo "bash: $bash_time ms"

echo "NexusShell benchmark:"
nxsh_time=$(bench ./target/debug/nxsh "echo hello")
echo "nxsh: $nxsh_time ms"

ratio=$(echo "scale=2; $bash_time/$nxsh_time" | bc)
echo "速度比: $ratio"
