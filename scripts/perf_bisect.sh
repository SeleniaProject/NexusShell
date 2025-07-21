#!/usr/bin/env bash
set -e
if [ $# -lt 1 ]; then echo "usage: perf_bisect.sh <benchmark_cmd>"; exit 1; fi
BENCH="$1"
REV_LIST=$(git rev-list --first-parent --reverse $(git merge-base HEAD origin/main)..HEAD)
BEST_TIME=1000000
BEST_REV=""
for rev in $REV_LIST; do
  git checkout $rev --quiet
  T=$(hyperfine --warmup 2 -n current "$BENCH" | grep "time:" | awk '{print $2}')
  echo "$rev $T"
  if (( $(echo "$T < $BEST_TIME" | bc -l) )); then
    BEST_TIME=$T; BEST_REV=$rev
  fi
done
echo "Best revision: $BEST_REV $BEST_TIME" 