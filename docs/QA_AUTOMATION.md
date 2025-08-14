# QA 自動化の仕分け

本ドキュメントは `docs/QA_PREVIEW_CHECKLIST.md` の各項目について、現状の自動化可否と実行フローを整理します。

- 自動化済み（just タスクで一括実行可能）
  - ビルド/テスト: `just ci`（clippy, fmt, test）
  - テーマ検証: `just themes-validate` / `just themes-validate-json`
  - コマンド実装確認: `just command-status-check`
    - docs/spec 整合チェックを強化（`scripts/gen_command_status.rs` が `docs/COMMANDS.md` と `spec/COMMANDS.md` を突合。差分は `COMMAND_STATUS.diff.md` として出力しCIで失敗）
  - バイナリサイズゲート: `just busybox-size-gate`
  - JIT/MIR ベンチ・ゲート: `just bench-gate`（Criterion 出力から 2x 判定）
  - フル CI+ベンチ: `just full-ci-with-bench`
  - Nightly ベンチゲート（GitHub Actions）: `.github/workflows/nightly_bench_gate.yml`（毎日 UTC 03:00、自動／手動トリガー）

- 自動化対象外（手動検証推奨）
  - 署名・ノータライズ（macOS）
  - 署名付きコンテナ検証（Notary v2）
  - 実マシンでの FPS/描画系 UX（TUI/UI 未採用のため NA）
  - 国際化の実端末表示（フォント/端末依存の表示揺れ）

- 追加タスク（将来）
  - （オプション）GitHub Actions 上で `just full-ci-with-bench` を nightly でも実行（ベンチの環境差によるフレーク回避）。現状はベンチゲートのみを夜間実行。
  - `cargo audit`/SBOM 生成/署名検証はワークフロー化

実行例:

```
# Windows PowerShell でも同様（justfileはPS行を含む）
just full-ci-with-bench
```
