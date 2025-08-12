#!/usr/bin/env python3
import json, argparse, sys

def main():
    p = argparse.ArgumentParser()
    p.add_argument('--file', required=True)
    p.add_argument('--budget-ms', type=float, required=True)
    p.add_argument('--enforce', type=int, default=0, help='1 to fail if budget exceeded')
    args = p.parse_args()

    with open(args.file, 'r', encoding='utf-8') as f:
        data = json.load(f)
    # hyperfine JSON structure: top-level has "results" with timings in seconds
    results = data.get('results', [])
    if not results:
        print('No results in hyperfine JSON')
        return 1
    best_s = min(r.get('times', [r.get('mean', 0)]) and [min(r.get('times', [r.get('mean', 0)]))][0] for r in results)
    best_ms = best_s * 1000.0
    print(f'Best startup: {best_ms:.3f} ms (budget {args.budget_ms:.3f} ms)')
    if args.enforce and best_ms > args.budget_ms:
        print('Budget exceeded')
        return 2
    return 0

if __name__ == '__main__':
    sys.exit(main())

