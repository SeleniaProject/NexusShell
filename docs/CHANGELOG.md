# Changelog

All notable changes to this project are documented in this file.
The project follows a date-based release scheme (YY.MM.DD[-patch]).

## [Unreleased]

### Added
- Initial changelog and automated version tracking pipeline.
- zstd 圧縮（Pure Rust ストアモード: RAW ブロックの zstd フレーム生成）。解凍は ruzstd。
- `zstd`/`unzstd`/`tar --zstd` の往復テスト、BATS/Pester 統合テストを追加。
- `tar --zstd` を Pure Rust ストアモードで作成/展開対応。
- ベンチゲート（自動速度判定スクリプトは削除）。`just bench-gate` でベンチ実行のみを提供。
- Fuzz 雛形（`fuzz/`）：`ruzstd` ストリームと `nxsh_parser` 入力のターゲットを追加。
- Nightly ベンチゲート CI を追加（`.github/workflows/nightly_bench_gate.yml`）。毎日UTC 03:00 に `nxsh_core` の `jit_vs_interp` ベンチを実行（自動速度判定は停止）。Criterion レポートをアーティファクトとして保存。
- find: 並列探索オプション `--parallel`/`-P` を追加。`parallel` フィーチャ無効時は警告の上で逐次にフォールバック。
- ドキュメント: `docs/COMMANDS.md` に find の並列オプション説明を追記。`docs/man/find.md` を新規追加。