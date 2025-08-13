# NexusShell 未実装タスクリスト（コードベース準拠）

> 本ファイルは仕様書とコードベース（ソース内のコメント/エラーメッセージ）から抽出した「未実装/プレースホルダー」のみを列挙する。実装完了時は当該行を `[x]` に更新し、関連PR/コミットを付記する。

## 0. 集計メタ
- 収集方法: キーワード検索と手動確認
  - 検索語: `TODO`, `unimplemented`, `not yet implemented`, `not implemented`, `未実装`, `placeholder`
- 注意: 仕様上の将来構想（研究項目）はここに含めない

---
## 1. Core / Runtime / Executor / Updater
- [x] `crates/nxsh_core/src/updater.rs`: 差分アップデート統合（pure-Rust bspatch 連携、`apply_delta_patch()` 本実装、フル/デルタ切替、ロールバック/バックアップの実体化）
- [x] `crates/nxsh_core/src/updater.rs`: インストール/バックアップ/ロールバックのプレースホルダー除去（実ファイル操作・整合性検証・障害復帰の実装）
- [x] `crates/nxsh_core/src/executor.rs`: 外部コマンド実行結果の `execution_time` 計測を正しく反映（現状 `0` 固定箇所の解消）
 - [x] `crates/nxsh_core/src/mir/lower.rs`: 関数宣言の別登録機構（シンボル表/可視性）の実装
 - [x] `crates/nxsh_core/src/mir/lower.rs`: AndSC/OrSC 近辺の一時挿入/パッチ処理の最適化・明確化
 - [x] `crates/nxsh_core/src/namespace.rs`: `super` インポート対応（相対参照/可視性エラー整備）
 - [x] `crates/nxsh_core/src/job.rs`: Windows のプロセス一時停止/継続の実装（ジョブ制御のクロスプラットフォーム整備）
- [x] `crates/nxsh_core/src/closures.rs`: 未対応バイナリ演算子の実装（演算網羅）
- [x] `crates/nxsh_core/src/performance_profiler.rs`: 実システム API によるメモリ/スループット/CPU/稼働時間等メトリクスの実装
 - [x] `crates/nxsh_core/src/shell.rs`: `shell` モジュールのプレースホルダー解消（実体実装）

## 2. HAL / Platform
- [x] `crates/nxsh_hal/src/process_enhanced.rs`: Windows でのプロセス kill 実装（TerminateProcess 等の統合）
- [x] `crates/nxsh_hal/src/completion.rs`: 履歴補完の実装（シェル履歴システム統合）

## 3. UI / UX
- [x] `crates/nxsh_ui/src/lib.rs`: `run_cui_with_config()` の `_config` 適用配線
- [x] `crates/nxsh_ui/src/app.rs`: 現在入力の取得（ラインエディタ統合）
- [x] `crates/nxsh_ui/src/app.rs`: 実メモリ使用量の取得（OS別 API 統合）

## 4. Parser / Scheduler
- [x] `crates/nxsh_core/src/advanced_scheduler.rs`: `parse_cron_expression_static()` の本実装（cron 式評価）

## 5. Builtins（未実装/部分実装）
- [x] `crates/nxsh_builtins/src/ls.rs`: user/group 参照の pure Rust 代替（libc 非依存化）
- [x] `crates/nxsh_builtins/src/cron.rs`: システムリソース監視の実装（CPU/Mem/IO/Load の収集としきい値評価）
- [x] `crates/nxsh_builtins/src/find.rs`: 並列探索の導入と複雑式の評価（式パーサ実装/エラー解消）
- [x] `crates/nxsh_builtins/src/command.rs`: `command` によるエイリアス回避実行の完全実装（外部コマンドへ直接ディスパッチ）
- [x] `crates/nxsh_builtins/src/cat.rs`: URL 入力の対応（HTTP/HTTPS 読み取り）
- [x] `crates/nxsh_builtins/src/common/logging.rs`: 複数ロギング出力の同時利用（出力先の複合）
- [x] `crates/nxsh_builtins/src/mv.rs`: タイムスタンプ保存（プラットフォーム別対応）
 - [x] `crates/nxsh_builtins/src/mkdir.rs`: SELinux コンテキスト設定
- [x] `crates/nxsh_builtins/src/id.rs`: ユーザ情報の照会（プラットフォーム横断）
- [x] `crates/nxsh_builtins/src/umask.rs`: `-S`（象徴表現）の実装
 - [x] `crates/nxsh_builtins/src/chgrp.rs`: Unix グループ操作（純 Rust 代替）
 - [x] `crates/nxsh_builtins/src/chown.rs`: Unix 所有者変更（純 Rust 代替）
- [x] `crates/nxsh_builtins/src/nohup.rs`: Unix シグナル処理
- [x] `crates/nxsh_builtins/src/cut.rs`: 欠損フィールドのパディング動作（仕様準拠オプション）
- [x] `crates/nxsh_builtins/src/paste.rs`: シリアルモード（`-s`）の実装
- [x] `crates/nxsh_builtins/src/read_builtin.rs`: `-n` オプションの完全実装
- [x] `crates/nxsh_builtins/src/wc.rs`: 追加フラグ群の対応
 - [ ] `crates/nxsh_builtins/src/zstd_complete.rs`: 圧縮機能の実装（Pure Rust 圧縮ライブラリ統合）
 - [x] `crates/nxsh_builtins/src/export.rs`: 関数エクスポート（`-f`）
- [x] `crates/nxsh_builtins/src/schedule.rs`: 内部スケジューラ（ジョブ登録/削除）
 - [x] `crates/nxsh_builtins/src/kill.rs`: ジョブ ID 指定の kill（ジョブテーブル連携強化）
- [ ] `crates/nxsh_builtins/src/fsck.rs`: 修復モード/デバイス処理（安全な書き戻し戦略・入出力）
- [ ] `crates/nxsh_builtins/src/timedatectl.rs`: ドリフト計算/統計/イベント送出の実装（監視ループの実体化）
- [ ] `crates/nxsh_builtins/src/update.rs`: アップデートキャッシュ実体（固定パスの解消とキャッシュ検証）
- [ ] `crates/nxsh_builtins/src/awk.rs`: 簡易版で未対応のアクション/式の拡充

## 6. テスト / QA / CI
- [ ] 単体テスト件数の充足（SPEC 記載値との差分把握と拡充）
- [ ] 統合テスト（POSIX PCTS / BATS / Pester）パイプライン拡充
- [ ] Fuzzing（`cargo-fuzz` 長時間実行）とレポート化
- [ ] QA_PREVIEW_CHECKLIST 自動化可否の仕分け
- [ ] JIT/MIR ベンチでの 2x 目標達成（Criterion による閾値ゲート）

---
更新手順: 未実装が解消されたら当該行を `[x]` に変更し、関連 PR/コミットを付記する。
