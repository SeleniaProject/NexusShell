#!/usr/bin/env python3
import os
import re
import json
from pathlib import Path


TEST_ATTR_PATTERNS = [
    re.compile(r"^\s*#\s*\[test\s*\]"),
    re.compile(r"^\s*#\s*\[tokio::test(\s*,.*)?\]"),
]


def is_rust_file(path: Path) -> bool:
    return path.suffix == ".rs"


def count_tests_in_file(path: Path) -> int:
    try:
        text = path.read_text(encoding="utf-8", errors="ignore")
    except Exception:
        return 0
    count = 0
    for line in text.splitlines():
        if any(p.search(line) for p in TEST_ATTR_PATTERNS):
            count += 1
    return count


def main() -> int:
    workspace = Path(__file__).resolve().parents[1]
    include_dirs = [
        workspace / "crates",
        workspace / "tests",
        workspace / "examples",
    ]
    total = 0
    per_file = {}
    for base in include_dirs:
        if not base.exists():
            continue
        for path in base.rglob("*.rs"):
            c = count_tests_in_file(path)
            if c:
                per_file[str(path.relative_to(workspace))] = c
                total += c

    result = {
        "total_test_attributes": total,
        "files": per_file,
    }
    print(json.dumps(result, ensure_ascii=False, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())


