# NexusShell TODO 一覧（超詳細版）

> **目的**: 本ファイルは NexusShell 完成までのあらゆる作業を粒度「ファイル / 関数」レベルまで分解した決定版タスクリストである。各項目は `[ ]` 未着手 → `[~]` 進行中 → `[✓]` 完了の 3 状態で管理する。全タスクの総数は 500 以上。

---

## 0. プロジェクト初期設定
- [ ] 0.1 Git 初期化
  - [ ] 0.1.1 `.gitignore` 生成 (`cargo`, `idea`, `vscode`)
  - [ ] 0.1.2 `main` ブランチ保護ルール設定
- [ ] 0.2 Cargo ワークスペース
  - [ ] 0.2.1 ルート `Cargo.toml` 作成 (`[workspace]` メンバ空)
  - [ ] 0.2.2 ターゲットディレクトリを `target/` に統一
- [ ] 0.3 CI 雛形
  - [ ] 0.3.1 `ci/linux.yml` (Ubuntu + Rust stable)
  - [ ] 0.3.2 `ci/windows.yml` (Windows Server 2022)
  - [ ] 0.3.3 `ci/macos.yml` (macOS 14)
- [ ] 0.4 形式設定
  - [ ] 0.4.1 `rustfmt.toml` (max_width = 100)
  - [ ] 0.4.2 `clippy.toml` (warns → denies)
- [ ] 0.5 ツールチェーン
  - [ ] 0.5.1 `rust-toolchain.toml` (channel = stable, components = rust-src, clippy, rustfmt)
  - [ ] 0.5.2 `justfile` に `build`, `test`, `ci`, `bench` タスク記述

## 1. ドキュメント復元
- [ ] 1.1 `docs/` ディレクトリ作成
- [ ] 1.2 仕様書群を再コミット
  - [ ] 1.2.1 `SPEC.md`
  - [ ] 1.2.2 `DESIGN.md`
  - [ ] 1.2.3 `COMMANDS.md`
  - [ ] 1.2.4 `UI_DESIGN.md`
- [ ] 1.3 `README.md`
  - [ ] 1.3.1 バッジ: CI, Coverage, Crates.io
  - [ ] 1.3.2 Quick Start セクション
  - [ ] 1.3.3 スクリーンショット GIF 追加

## 2. ワークスペースクレート生成
- [ ] 2.1 `nxsh_core`
  - [ ] 2.1.1 `src/lib.rs` (feat flags: `jit`, `object-pipe`)
  - [ ] 2.1.2 `src/context.rs`
  - [ ] 2.1.3 `src/executor/mod.rs`
- [ ] 2.2 `nxsh_parser`
  - [ ] 2.2.1 `src/lexer.rs`
  - [ ] 2.2.2 `src/grammar.pest`
  - [ ] 2.2.3 `src/ast.rs`
- [ ] 2.3 `nxsh_ui`
  - [ ] 2.3.1 `src/app.rs` (ratatui `AppState`)
  - [ ] 2.3.2 `src/widgets/` ディレクトリ
- [ ] 2.4 `nxsh_hal`
  - [ ] 2.4.1 `src/process.rs`
  - [ ] 2.4.2 `src/fs.rs`
- [ ] 2.5 `nxsh_plugin`
  - [ ] 2.5.1 `src/lib.rs` (WASI bindings)
- [ ] 2.6 `nxsh_builtins`
  - [ ] 2.6.1 `src/lib.rs` (re-export)
- [ ] 2.7 `nxsh_cli`
  - [ ] 2.7.1 `src/main.rs` (argparse)

## 3. パーサ & AST 詳細
### 3.1 Tokenizer
- [ ] 3.1.1 トークン種 15 個実装 (`Word`, `String`, ...)
- [ ] 3.1.2 ヒアドキュメント開始検知 (`<<` + delimiter)
- [ ] 3.1.3 ベンチ: 1MB スクリプト 2ms 以内 lex

### 3.2 PEG Grammar
- [ ] 3.2.1 文法ファイル `shell.pest` 書き起こし
- [ ] 3.2.2 単体文解析テスト 100 ケース
- [ ] 3.2.3 エラー位置ハイライト関数

### 3.3 AST & MIR
- [ ] 3.3.1 ノード定義 (35 種)
- [ ] 3.3.2 MIR SSA 生成パス
- [ ] 3.3.3 定数畳み込みアルゴリズム
- [ ] 3.3.4 IR → JIT (`cranelift`) トランスレータ

## 4. コアランタイム詳細
### 4.1 Context
- [ ] 4.1.1 リファレンスカウント付き `Stream` 型
- [ ] 4.1.2 環境変数ハッシュマップ (dashmap)

### 4.2 Executor
- [ ] 4.2.1 AST walker 実装
- [ ] 4.2.2 リダイレクトオープン (`O_CLOEXEC`)
- [ ] 4.2.3 パイプ生成 (`pipe2` with `O_NONBLOCK`)
- [ ] 4.2.4 エラー→スタックトレース付与

### 4.3 Job Scheduler
- [ ] 4.3.1 `JobTable` lock-free map
- [ ] 4.3.2 `SIGCHLD` ハンドラ / Windows JobObject
- [ ] 4.3.3 `fg`, `bg` コマンド実装 & テスト

## 5. Built-in 実装詳細
- [ ] 5.1 ユーティリティ `builtins/common/logging.rs`
- [ ] 5.2 `cd`
  - [ ] 5.2.1 `~`, `-` 解決
  - [ ] 5.2.2 `CDPATH` 対応
- [ ] 5.3 `history`
  - [ ] 5.3.1 AES-GCM 暗号化実装
  - [ ] 5.3.2 `history -s` 追加
- [ ] 5.4 `help`
  - [ ] 5.4.1 表生成 (tui_table)
  - [ ] 5.4.2 `help --lang ja` 
  - [ ] 5.4.3 `help <command>` 詳細モーダル表示
  - [ ] 5.4.4 Markdown manpage → ANSI 変換フィルタ
- [ ] 5.5 `alias`
  - [ ] 5.5.1 `alias NAME=VALUE` 登録
  - [ ] 5.5.2 `alias -p` 一覧フォーマッタ
  - [ ] 5.5.3 循環参照検出ユニットテスト
- [ ] 5.6 `export`
  - [ ] 5.6.1 `export NAME=VALUE` 実装
  - [ ] 5.6.2 `export -p` 環境変数一覧 (色分け)
- [ ] 5.7 `set`
  - [ ] 5.7.1 `-e`, `-x`, `-o pipefail` 解析
  - [ ] 5.7.2 ランタイムフラグ更新に伴う Executor 再設定
- [ ] 5.8 ジョブ制御系 (`bg`, `fg`, `jobs`, `wait`, `disown`)
  - [ ] 5.8.1 `jobs` 出力テーブル (PID, CPU%, MEM%)
  - [ ] 5.8.2 `wait %1` 成功/失敗コード伝播
  - [ ] 5.8.3 `disown -a` 全ジョブ切離し
- [ ] 5.9 変数 / 算術 (`let`, `declare`, `printf`)
  - [ ] 5.9.1 `let "a += 1"` パーサ
  - [ ] 5.9.2 `declare -A assoc_array` 実装
  - [ ] 5.9.3 `printf "%08x\n" 255` 出力整形テスト
- [ ] 5.10 雛形生成ツール
  - [ ] 5.10.1 `cargo install nxsh-gen`
  - [ ] 5.10.2 CLI: `nxsh-gen builtin ping`
  - [ ] 5.10.3 テンプレート `{{command}}.rs` にスケルトン関数

## 6. ファイル・テキストユーティリティ詳細
### 6.1 `ls`
- [ ] 6.1.1 ディレクトリエントリ読み込み (async)
- [ ] 6.1.2 Git ステータス連携 (libgit2)
- [ ] 6.1.3 アイコンマッピングテーブル (`icons.rs`)
- [ ] 6.1.4 テーブルビュー + カラー (ratatui)
### 6.2 `grep`
- [ ] 6.2.1 PCRE2 バインディング設定
- [ ] 6.2.2 並列チャンク検索 (rayon)
- [ ] 6.2.3 ハイライト ANSI 出力
- [ ] 6.2.4 `--json` オブジェクトパイプ出力
### 6.3 `tar`
- [ ] 6.3.1 `tar::Builder` ラッパ
- [ ] 6.3.2 進捗バー (indicatif)
- [ ] 6.3.3 圧縮 backend 切替 (gz, bzip2, zstd)

## 7. オブジェクトパイプライン詳細
- [ ] 7.1 `object::Stream` 型定義
- [ ] 7.2 JSON デシリアライザプラグイン
- [ ] 7.3 `select` コマンド: JMESPath エンジン統合
- [ ] 7.4 `group-by` ハッシュアルゴリズム最適化
- [ ] 7.5 表形式レンダラとの相互変換ユニットテスト

## 8. UI / UX 拡張
- [ ] 8.1 スクロールバッファ: 行数 100k まで O(1) スクロール
- [ ] 8.2 サイドパネル: `F2` で補完リスト固定表示
- [ ] 8.3 トースト通知: `nxsh notify "Build done"` API
- [ ] 8.4 テーマスイッチャ: `promptctl theme list|set`
- [ ] 8.5 アニメーションフレームレート上限 60fps

## 9. プラグインシステム詳細
- [ ] 9.1 `PluginRegistrar` FFI 安定 ABI
- [ ] 9.2 サンドボックスメモリ制限オプション (`--max-mem`)
- [ ] 9.3 `nxsh plugin sign` キーペア生成
- [ ] 9.4 ストア REST API `/plugins/v1/download/{id}`
- [ ] 9.5 自動更新機構 (SemVer range)

## 10. セキュリティ強化
- [ ] 10.1 `seccomp` フィルタ生成 (Linux)
- [ ] 10.2 Windows `JobObject` 限界設定
- [ ] 10.3 ヒストリ Salt ローテーションジョブ (cron)
- [ ] 10.4 `cargo deny` Policy ファイル
- [ ] 10.5 メモリ安全監査 (`miri`)

## 11. テスト & QA 追加
- [ ] 11.1 Golden File 互換テスト生成スクリプト
- [ ] 11.2 Fuzz Corpus 自動最適化
- [ ] 11.3 Coverage Gate CI step (`<95% fail`)
- [ ] 11.4 UI Visual Test (Puppeteer + terminal emulator)
- [ ] 11.5 性能リグレッション自動 Bisect

## 12. CI/CD & 配布詳細
- [ ] 12.1 GitHub Actions Self-hosted ARM64 Runner
- [ ] 12.2 SBOM CycloneDX 生成
- [ ] 12.3 Notary V2 Container 署名
- [ ] 12.4 Homebrew Tap 自動 PR
- [ ] 12.5 Scoop Manifest JSON 生成

## 13. ガバナンス & 運用詳細
- [ ] 13.1 ドキュメントバージョン管理 (`docs/CHANGELOG.md`)
- [ ] 13.2 インシデント対応 Runbook 作成
- [ ] 13.3 SLA モニタリング Dashboards (Grafana)
- [ ] 13.4 Secrets Rotation Policy 自動化
- [ ] 13.5 License Compatibility 内部監査 (monthly)

## 14. リリースマイルストーン詳細
- [ ] 14.1 Preview QA checklist (50 items)
- [ ] 14.2 Beta External Pilot (社内 30User)
- [ ] 14.3 Stable Rollout Plan (Phased 10%→100%)

---

> **備考**: 本 TODO は随時 Pull Request で更新し、番号は変更不可 (参照用) とする。 