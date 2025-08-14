# Changelog

All notable changes to this project are documented in this file.
The project follows a date-based release scheme (YY.MM.DD[-patch]).

## [Unreleased]

### Added
- Initial changelog and automated version tracking pipeline.
- zstd 圧縮（Pure Rust ストアモード: RAW ブロックの zstd フレーム生成）。解凍は ruzstd。
- `zstd`/`unzstd`/`tar --zstd` の往復テスト、BATS/Pester 統合テストを追加。
- `tar --zstd` を Pure Rust ストアモードで作成/展開対応。
- ベンチゲート (`scripts/check_jit_speedup.py`) を追加し、`just bench-gate`/`full-ci-with-bench` に統合。
- Fuzz 雛形（`fuzz/`）：`ruzstd` ストリームと `nxsh_parser` 入力のターゲットを追加。
- Nightly ベンチゲート CI を追加（`.github/workflows/nightly_bench_gate.yml`）。毎日UTC 03:00 に `nxsh_core` の `jit_vs_interp` ベンチを実行し、`scripts/check_jit_speedup.py` により 2.0x 以上の JIT/MIR 速度向上を自動検証。Criterion レポートをアーティファクトとして保存。