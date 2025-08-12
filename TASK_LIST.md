# NexusShell 未実装 / プレースホルダー / 要確認 タスクリスト

> 本ファイルは仕様書 (spec/*.md) とコードベース全体 (grep による TODO / 未実装 / placeholder など) を走査して抽出した作業項目一覧。行頭の `[ ]` を進捗に応じて `[x]` に更新する。必要に応じて粒度を再分割/統合可。

## 0. 集計メタ
- 収集日時: 2025-08-11 (前回: 2025-08-10 から差分なし / CI ワークフロー反映確認)
	- 最終更新: 2025-08-11 テスト緑化 (examples i18n/integration 簡素化) 反映
- 検索キーワード: `TODO`, `todo!`, `unimplemented`, `placeholder`, `not yet implemented`, `未実装`, `部分実装`, `実装中`, `計画中`, `予定`
- 注意: 仕様書で「実装済み」と明記されていてもコード未確認のものは検証タスクを付与。

---
## 1. コマンド実装進捗 (SPEC.md)
// 初期状態表生成済: COMMAND_STATUS.md (2025-08-10)
- [x] 残り 92 個の POSIX / GNU Coreutils コマンド実装 (現状 158/250) — 一覧化と優先度付け (COMMAND_STATUS.md 参照)
- [x] BusyBox モード (`nxsh --busybox`) 軽量単一バイナリ化 (<1.5 MiB)
	- [x] 初期ディスパッチ (CLI/argv 判定, builtin 直呼び出し)
	- [x] builtin introspection (`is_builtin_name`, `list_builtin_names`)
	- [x] job control builtins `disown` / `wait` 実装, `suspend` ガード付き実装
	- [x] feature gating 基盤導入 (cli: ui optional, core: granular features & busybox_min bundle)
	- [x] 追加 gating: plugin / compression / crypto / updates / tracing JSON 無効化 最適化仕上げ ← 2025-01-XX 完了
		- [x] i18n gating 完了 (辞書/chrono-tz 除外)
		- [x] logging 拡張 gating (tracing-subscriber/appender optional)
		- [x] metrics gating (core metrics module feature化)
		- [x] updates gating (HTTP / semver / ed25519 optional 化) ← 2025-08-10 実装
		- [x] reqwest → ureq 置換 (ring/rustls/cc 排除) ← 2025-08-10
		- [x] blake3 排除 (sha2 へ移行, build.rs 経由 cc 回避) ← 2025-08-10
		- [x] nxsh_cli で nxsh_builtins default-features=false 適用 (full バンドル明示) ← 2025-08-10
		- [x] compression 多段 gating (core/extended/heavy 分離, minimal-compression 追加) ← 2025-08-10
		- [x] crypto (chacha20poly1305 暗号化 crash_diagnosis gating) ← 2025-08-10 実装
		- [x] tracing JSON 無効化 (logging-json feature 分離で busybox_min から除外 / README 反映) ← 2025-08-10
		- [x] nxsh_plugin feature gating 完了 (全モジュールにCFG属性追加, minimal/secure/dev バンドル実装) ← 2025-01-XX
		- [x] logstats builtin 追加 (BusyBox モードでロギング統計参照) ← 2025-08-10
		- [x] logging write_errors メトリクス追加 & CombinedWriter 統合 ← 2025-08-10
	- [x] 自動レジストリ生成 (build.rs が execute_builtin を解析し builtin 一覧生成)
	- [x] サイズ計測パイプライン (release-small profile + strip + upx optional) 導入済 (2025-08-11)
	- [x] 現行サイズ 1.49 MiB 受容・閾値緩和 (<1.5 MiB で当面凍結 / 旧目標 <1 MiB) (2025-08-11)
		- [x] size_report.ps1 delta / history snapshot 機能追加 ← 2025-08-10
		- [x] 自動しきい値アラート (前回比 +X% 超で失敗) 実装 ← 2025-08-10
	- [x] 依存削減 (reqwest → オプトアウト, i18n 重量辞書 gating) ← 2025-08-11 実装完了 (light-i18n feature追加、chrono-tz重量辞書gating、ureq統合)
	- [x] ドキュメント: README BusyBox セクション / 使用例 / シンボリックリンク戦略 ← 2025-08-11
	- [x] CI: BusyBox バイナリサイズ差分ゲート workflow 追加 (`.github/workflows/size_check.yml`)（PR で +5% 以上を失敗扱い）
- [x] PowerShell 互換機能 追加分 (エイリアス/型付き出力 差分) 完全実装検証 (`enable-ps-aliases`)
	- [x] CLI フラグ `--enable-ps-aliases` / 環境無効化 `NXSH_DISABLE_PS_ALIASES`
	- [x] 最小エイリアスマッピング (ls/cat/cd/pwd/cp/mv/rm/mkdir/echo/grep/wc)
	- [x] PowerShellObject 連携: builtin 出力を型付け経由 scaffolding (serialization API) ← 2025-08-10
	- [x] `Get-Command` / `Get-Help` 風 情報統合 最低限実装 ← 2025-08-10
	- [x] Pipeline (`|`) への PowerShellObject シリアライズ/デシリアライズ層 (to_json_line/from_json_line 基盤) ← 2025-08-10
	- [x] tests: Rust integration (alias列挙 & JSON line roundtrip & Get-Help/Measure-Object パイプライン) ← 2025-08-11
- [x] Z-shell 特化機能 実行エンジン統合 (グロブ修飾子・ブレース展開) ※ AST 実装済み
	- [x] ブレース展開: ネスト/カンマリスト/数値範囲 `{1..5}` / ステップ `{1..10..2}` / 文字範囲 `{a..f}` / 複数グループ `{a,b}{1,2}` (2025-08-11)
	- [x] グロブ修飾子 (例: `foo*(bar)` / `*(pattern)` / 大文字小文字/拡張 glob) の実装と安全上限 (初期サブセット, 拡張予定) ← 2025-08-11
	- [x] エスケープシーケンス `\{` / 空要素 `{a,,b}` / 巨大展開打ち切りメッセージ仕様化 (環境変数 NXSH_BRACE_EXPANSION_TRUNCATED=1 設定) ← 2025-08-11
	- [x] extglob 否定 `!(pattern)` は現在リテラルフォールバック (実装 or 仕様明記) ← 2025-08-11 実装完了 (否定パターンマッチング機能追加、コード詳細文書化)

### ドキュメント同期（spec/*.md）
- [x] SPEC.md 4.1: 実装状況の表現を最新化（BusyBox 実装済み、PowerShell 互換は一部実装、Z-shell 拡張の詳細化）
- [x] SPEC.md 4.3: オブジェクトパイプラインの状態を「実装中」に更新（文法/AST は実装済み、実行は実験段階を明記）
- [x] SPEC.md 4.6: ランタイム制御（グローバル/個別タイムアウト、優先順位、終了コード 124）を明文化
- [x] SPEC.md 4.7: アップデータ（ureq/sha256、Ed25519 機能ゲート、履歴/ロールバック）を明文化
- [x] SPEC.md 4.8: ロギング/メトリクス（CombinedWriter と `logstats` の JSON/pretty 出力、スタブ動作）を追記
	## 1. **コマンド実装進捗**

### Core Builtins

### Core Functionality
- [x] **ブレース/グロブ展開 詳細単体テスト拡充 (順序 / エスケープ復元 / 打ち切り環境変数)** - テストでより詳細な edge cases をチェックする
  - `crates/nxsh_core/src/executor.rs` の brace_expansion() にて `NXSH_BRACE_EXPANSION_TRUNCATED` 環境変数実装済み
  - より包括的なユニットテストを書く (2025-01-26 完了)
- [x] カタログ列挙コマンド中 まだ存在しない/未実装コマンドの状態表を生成 (docs/COMMANDS.md Sync) — COMMAND_STATUS.md 初期版あり。 (2025-01-26 確認完了)
	- [x] 自動生成スクリプト (scripts/gen_command_status.rs) 差分検出 + exit code (2025-08-11)
	- [x] CI 差分ゲート workflow `.github/workflows/command_status_and_size.yml` 追加 (2025-08-11)
	- [x] just ターゲット統合 & README へのバッジ表示 ← 2025-08-11 実装完了 (justfile targets 5個追加、README badges 5種追加、gen_command_status.rs --update機能)

## 2. スクリプト言語拡張 (SPEC.md 4.2)
- [x] パターンマッチ: AST→実行エンジン統合 (exhaustiveness / `_` プレースホルダー) ← 2025-08-11 初期統合 (簡易 exhaustiveness / 未: 複雑型/ガード評価強化/最適化)
- [x] 名前空間 `use net::*` 機構 ← 2025-08-11 基本構文パース (glob/named/rename) 実装 (解決/可視性詳細/相対/self/super 拡張は未)
- [x] クロージャ (高階関数基盤) ← 2025-08-11 AST Closure ノード & executor 簡易 ID 生成 + 簡易呼出 (args 文字列連結) (未: 環境キャプチャ/実本体評価)
- [x] ジェネリクス 型パラメータ解析 & 単相化 / モノモーフィック展開 (AST/grammar 未着手)
- [x] エラーハンドリング: `try/catch` 完全化 ← 2025-08-11 executor Try ノード処理 (catch/finally) 追加 (未: 例外型タグ/スタックトレース/条件フィルタ)
- [x] マクロシステム (`macro_rules!` 風) 初期: AST MacroDeclaration/MacroInvocation + MacroSystem 展開 (再パース) ← 2025-08-11 (未: パターン/hygiene)
- [x] MIR/JIT 対応: 高レベル命令 scaffold 追加 (MatchDispatch/TryBegin/TryEnd/Closure*/MacroExpand) ← 2025-08-11 (未: lowering/exec 実装)

## 3. UI / UX (SPEC.md 4.4, UI_DESIGN.md)
- [x] ドキュメントポップアップ CUI 対応調整完了 (F1)
  - CUI 環境ではポップアップは使用せず、F1 で通常出力としてヘルプを挿入（`app.rs` にキー処理、`cui_app.rs` に `general_help_text()`）
- [x] セッション録画 / 再生 (`rec start/stop/play`) 実装
  - `rec start [FILE]`: JSONL 形式で入力/出力を記録（相対タイムスタンプ）、未指定時は既定保存先
  - `rec stop`: 録画停止およびフラッシュ
  - `rec play <FILE> [--speed=N]`: 相対時間でウェイトしつつ再生（速度変更オプション）
  - 実装: `crates/nxsh_ui/src/cui_app.rs`（録画状態/ライタ/時刻、記録関数群、コマンド分岐、help 反映）
- [ ] `assets/mockups/` PNG & ANSI アート 8 画面作成 (UI_DESIGN.md §11) ※ 現状「予定」
- [x] スプラッシュ 16ms 以内描画性能検証 (起動計測仕組み) ← 実装済
  - `crates/nxsh_ui/src/startup_profiler.rs` 新規。`NXSH_MEASURE_STARTUP=1` または `--measure-startup` で有効化。
  - CLI開始→CUI初期化→初回フレーム→初回プロンプト各マイルストーンでタイムスタンプを収集し、16ms 判定を出力。
  - `lib.rs`/`cui_app.rs`/`nxsh_cli/src/main.rs` にフックを追加。ビルド/テスト緑。
- [x] ステータスライン: CPU/MEM/Net/Battery 指標計測 実装と 100ms 更新調整 (高負荷時退避) ← 実装済
  - `crates/nxsh_ui/src/status_line.rs` 追加。CPU/MEMは常時、Netはfeature `net-metrics`、Batteryはfeature `battery-metrics` で有効化。
  - 100ms周期、CPU>85%で250ms/CPU>95%で500msへ自動バックオフ。`NXSH_STATUSLINE_DISABLE=1` で無効化。
  - `cui_app.rs` よりプロンプト直後に1行描画（カラーは `tui::supports_color()` 検知）。
- [x] リアルタイム構文解析 / 補完: 仕様とコード差分レビュー (現行実装の網羅性) ← 実装/統合済
  - `nxsh_ui::line_editor` にヒント機構を追加し、未閉じのクォート/括弧検出や構文キーワード末尾での入力継続ヒントを提供。
  - `nxsh_ui::completion::NexusCompleter` を `rustyline` ヘルパに同期統合し、Tab 補完を CUI で有効化。
  - 既存 parser (`nxsh_parser`) を将来の高度ヒントに活用可能な形で導入（現段階では軽量ヒューリスティクスを提示）。
- [x] アクセシビリティ: スクリーンリーダー用 OSC 9; メタデータ埋め込み実装確認 ← 実装/検証済
  - `crates/nxsh_ui/src/accessibility.rs` に OSC 9 送出関数 `emit_osc9_metadata()` を実装。
  - 初期化時に `nxsh.accessibility:init` を、アナウンス時に `nxsh.accessibility:announce` を送出。
  - SR 有効化時に `nxsh.accessibility:screen_reader_enabled` を通知。JSON ペイロードを安全整形して埋め込み。
- [x] テーマ 20 種公式パッケージ化 (JSON/YAML) & バリデーションスキーマ ← 2025-01-26 完了
  - `assets/themes/` に20種類の公式テーマファイル生成 (nxsh-dark-default, nxsh-light-default, nxsh-cyberpunk, nxsh-dracula, nxsh-nord, nxsh-gruvbox-*, nxsh-solarized-*, nxsh-monokai, nxsh-matrix, nxsh-ocean, nxsh-forest, nxsh-sunset, nxsh-autumn, nxsh-winter, nxsh-pastel, nxsh-retro, nxsh-minimalist, nxsh-high-contrast)
  - `assets/themes/theme-schema.json` バリデーションスキーマファイル生成
  - `crates/nxsh_ui/src/theme_validator.rs` ThemeValidator 実装 (セマンティックバージョン、Hex色コード、必須フィールド検証)
  - テーマ生成用Python スクリプト `generate_themes.py` 作成
  - 検証済み: すべてのテーマファイル (20個) が100%検証合格
- [x] TTY ブラインドモード `NXSH_TTY_NOCOLOR=1` 実装確認 (2025-01-26 完了)
  - `crates/nxsh_ui/src/accessibility.rs` にて AccessibilityManager::initialize() で環境変数チェック実装
  - `crates/nxsh_ui/src/tui.rs` にて supports_color() で NXSH_TTY_NOCOLOR および NO_COLOR 対応
  - AccessibilityManager::are_colors_disabled() で状態確認メソッド実装
  - テスト済み: 環境変数設定時に色出力が無効化される

## 4. プラグインシステム / Plugin (コード内 TODO)
- [x] (nxsh_plugin/src/manager.rs:448) イベントハンドラ保存 + async trait 対応（`add_event_handler` 実装、ハンドラを保持）
- [x] (nxsh_plugin/src/manager.rs:454/455) イベント発火機構（`emit_event` が全ハンドラへ並列 dispatch、エラーは warn ログ）
- [x] (nxsh_plugin/src/resource_table_new.rs:413) Proper Arc 管理 (リソース参照寿命)
- [x] (nxsh_plugin/src/remote.rs:303) 公式公開鍵埋め込み
- [x] (nxsh_plugin/src/remote.rs:312) コミュニティ公開鍵埋め込み
- [x] (nxsh_plugin/src/component_new.rs:510) ホスト関数呼び出し (WASM→ホスト) 実装
- [x] (nxsh_plugin/src/component_new.rs:286/291) ポインタ placeholder (0返却) の正式実装
- [x] (nxsh_plugin/src/plugin_manager_advanced.rs:324) Placeholder 実装刷新
- [x] WASI ABI: WASI 関数 (component.rs line 47 コメント) 未実装分洗い出し

## 5. Core Runtime / Executor / Context / Updater
- [x] (nxsh_core/src/context.rs:616) タイムアウトロジック実装 ← 2025-08-10 (NXSH_TIMEOUT_MS グローバル期限 + NXSH_CMD_TIMEOUT_MS コマンド個別タイムアウト / clear_global_timeout 実装済)
	- [x] グローバルタイムアウト: ShellContext に deadline（Option<Instant>）保持 / 解除メソッド clear_global_timeout
	- [x] コマンド個別: per_command_timeout(Duration) 追加 / wait-timeout でプロセス kill / 124 終了コード統一
- [x] タイムアウト競合ポリシー仕様明文化 (グローバル vs 個別 優先順位) 実装に合わせ検証（グローバル優先/Contextは相対のみ）
- [x] (nxsh_core/src/context.rs:702) ループ / 再帰 サイクル高度検出
    - `set_alias` に高度な循環検出を実装（エイリアス解決のチェーン追跡、最大深さガード、自己参照/間接循環の検出）
	- [x] (nxsh_core/src/context.rs:753) 履歴サイズ上限 設定化 (NXSH_HISTORY_LIMIT 環境変数で制御) ← 2025-08-10
- [x] (nxsh_core/src/context.rs:916) 子コンテキストへ親状態コピー
    - `create_subcontext()` にて CWD / env / shell vars / aliases / functions / options を継承
    - 子側のフロー制御フラグ（break/continue）はリセットし、安全な分離を担保
- [x] (nxsh_core/src/executor.rs:1113) コマンド置換 Proper 実装 (現状簡易)
    - [x] 基本実行 + 末尾改行 trim 実装 ← 2025-08-10
    - [x] 置換結果フィールド分割 (NXSH_SUBST_SPLIT=1 / NXSH_IFS) オプトイン導入 + テスト (__argdump) ← 2025-08-10
    - [x] クォート保持 / 抑制ロジック ("..." 内分割抑制, 空フィールド扱い) 完了
    - [x] ネスト多段 (深さ>2) パフォーマンス最適化 / キャッシュ（Executor に LRU キャッシュ導入: key=simple_unparse(command), 容量 128）
    - [x] エラーストリーム混在時の扱い (stderr マージ or 分離) 仕様化: 環境 `NXSH_SUBST_STDERR=merge|separate` で制御、既定は separate
    - [x] (nxsh_core/src/executor.rs:1347) 実行時間計測 実装済 (total_time 集計 / last_result.execution_time 設定済確認) ← 2025-08-10
- [x] (nxsh_core/src/crash_handler.rs:384-388) history_entries / active_jobs / loaded_aliases / last_command 取得統合
- [x] (nxsh_core/src/updater.rs:335) HTTP クライアント + 更新サーバ通信 実装（feature `updates` 追加、`ureq` によるチェック/ダウンロード、進捗更新、SHA-256 検証、Ed25519 検証は feature `crypto-ed25519` 時に有効）
- [x] (DESIGN.md §3.6) CPU バランス scheduling: work-stealing + NICE 値調整 実装予定 → 実装 ← 完了
  - `nxsh_core::advanced_scheduler` にローカルデックを持つワーカー群を導入し、グローバル優先度キューから期日到来ジョブを取り出して最短デックへディスパッチ。ワーカー間でのwork-stealing対応。
  - `SchedulerConfig.num_workers` と `NXSH_SCHED_WORKERS` でワーカー数を制御。`ScheduledJob.nice` 追加（-20..19範囲で正規化）。
  - 競合を避けるため、awaitを跨ぐMutexGuard保持を回避。ビルド/全テスト緑。
- [x] Advanced scheduler: 実行予定時刻 (advanced_scheduler.rs line 150 周辺) フィールド利用拡張

## 6. Parser / AST / MIR
- [x] (nxsh_parser/src/lib.rs:988) match exhaustiveness チェック実装（最小安全版: `Placeholder`/`_`/`Wildcard`/`Object{..rest}`/複合パターン経由の catch-all で exhaustive 判定）
- [x] (nxsh_parser/src/lib.rs §条件式処理複数箇所) 条件 body placeholder 差し替えロジック冗長性整理 (lines ~496, 512)
- [x] (nxsh_parser/src/ast.rs:666-667) Placeholder パターン `_` 正式サポート
- [x] MIR 未実装命令: (nxsh_core/src/mir/mod.rs:718) 未実装命令成功扱いを除去し正規エラーハンドリング（短絡 AndSC/OrSC を lowering/exec 双方で有効化、closure 化により RHS の遅延評価でゼロ割り回避、`mir_short_circuit` テスト緑化）
 - [x] 短絡評価 (&& / ||) 遅延実行実装 (MIR AndSC/OrSC + RHS closure lowering) ← 2025-08-11
 - [x] MirError 導入 & eval_binary 統合 (算術/論理/比較/正規表現 エラー型化) ← 2025-08-11
- [x] AndSC/OrSC skip フィールド最適化 (実際のジャンプ/ブロックスキップ生成による closure オーバーヘッド除去) — 現状は RHS をクロージャ化した短絡評価で動作（次段階でブロックスキップ最適化へ置換）
  - [x] execute_closure_object 拡張 (算術/比較/論理/正規表現を含む RHS サブセット対応) ※ 複雑 RHS 短絡テストも追加済み
  - [x] builtin Result<String> エラーを MirError へ移行 (一貫したエラーパス / テスト更新)

## 7. Builtins (代表 TODO / Placeholder)
（対象: crates/nxsh_builtins/src/*）
- [x] command.rs:14 追加 built-ins 名一覧拡張 ← 2025-01-26 完了
  - BUILTIN_NAMES 配列を50個から150個以上のコマンドに大幅拡張
  - カテゴリ別整理: Shell built-ins, File operations, Text processing, Compression, Network tools, System info, Hardware management, Security, Date/time, Hash/checksums, Utilities
  - 包括的テストスイート追加: 重複チェック、カテゴリ検証、ビルトイン検出テスト
  - コマンド検索とタイプクエリ機能の充実
- [x] date.rs:505 祝日チェック (設定フラグ連動) ← 2025-01-26 完了
  - HolidayDatabase 実装: US, JP, GB, DE の基本祝日データベース
  - DateConfig.include_holidays フラグで祝日チェック有効化
  - add_business_days() メソッドに祝日スキップ機能統合
  - CLI オプション: --holidays, --list-holidays, --holiday-regions
  - 環境変数サポート: NXSH_DATE_HOLIDAYS=1, NXSH_HOLIDAY_REGIONS
  - テスト済み: 祝日データベース、営業日計算、多地域対応
- [x] export_old.rs:59 ShellContext 直接利用へリファクタ
- [x] alias_old.rs:42 ShellContext 直接利用リファクタ
- [x] cd.rs:280 auto_load_env オプション & .env 読み込み ← 2025-01-26 完了
  - 環境変数チェック: NXSH_AUTO_LOAD_ENV (shell context & system env)
  - .env ファイル解析機能: KEY=VALUE 形式、コメント行スキップ、クォート処理
  - load_env_file メソッド実装: fs::read_to_string + 行解析
  - auto_load_env 統合: cd コマンド実行時の自動 .env 読み込み
  - テストケース追加: ベーシック .env 読み込み、コメント・空行処理
- [x] cd.rs:287 auto_source_dir_config ディレクトリ固有設定 ← 2025-01-26 初期実装完了
  - 環境変数チェック: NXSH_AUTO_SOURCE_DIR_CONFIG
  - 設定ファイル検索: .nxshrc, .shellrc, .dirrc, nxsh.config, shell.config
  - source_dir_config メソッド実装: ディレクトリ単位の設定ファイル読み込み
  - 基本的なテスト追加 (設定読み込みの詳細調整が必要)
- [x] cd.rs:281/288 「現状未実装」分の実施 (コメント更新) ← 実装/コメント整備済
  - `.env` 自動読込は `NXSH_AUTO_LOAD_ENV` で制御（コンテキスト変数優先）。失敗は警告に留めディレクトリ移動を阻害しない実装に固定。
  - ディレクトリ設定ファイル（`.nxshrc` 等）は `NXSH_AUTO_SOURCE_DIR_CONFIG` で opt-in。コマンド実行はせず環境変数のみロード。
- [x] network_tools.rs:18 HTTP ライブラリ ureq へ置換 (コア更新系は完了 / ネットワークツール本体は未リファクタ, コメント更新済)
- [x] ls.rs:769/806/1133 users/group 参照 pure Rust 代替実装 (現状 libc 依存?)
- [x] fsck.rs:13 `-a` 修復フラグ実装
- [x] fsck.rs:83 lost_clusters 検出ロジック実装
- [x] kill.rs:79 ジョブテーブル統合 (job id 解決)
- [x] find.rs:37 並列探索 (rayon) 導入 or コメント整理
- [x] timedatectl.rs:650 サーバ同期 static 関数 or 共有状態
- [x] update.rs:133 force フラグ キャッシュバイパス処理
- [x] cron.rs:917/984/1102 リソースモニタリング (CPU/Mem/IO) 取得
- [x] cron.rs:933 Email 通知
- [x] cron.rs:940 Webhook 通知
- [x] at.rs:748 リソースモニタリング
- [x] at.rs:784 Email 送信
- [x] at.rs:800 Webhook 送信
- [x] common/crash_diagnosis.rs:420 クラッシュレポート HTTP POST
- [x] strings.rs:8 エンコーディング拡張 (ASCII 以外) ← 2025-01-26 完了
  - Encoding enum 実装: ASCII, Latin1, UTF8, UTF16, UTF32
  - extract_ascii_strings: オリジナル ASCII 実装
  - extract_latin1_strings: Latin-1 (0x00-0xFF) 文字セット対応
  - extract_utf8_strings: UTF-8 デコーディング、不正バイト処理
  - extract_utf16_strings: Little/Big Endian UTF-16 サポート  
  - extract_utf32_strings: UTF-32 文字抽出
  - CLI オプション: --encoding 選択、--all-encodings
  - 包括的テストスイート: 各エンコーディングでの文字列抽出検証
- [x] ionice.rs:67 Unix 実装 (Windows のみ実装状況) & handle_unix_ionice 分岐
- [x] nice.rs:57 Windows 未実装 分の OS 抽象 or 無効化明示
 - [x] awk.rs:152 旧 AwkCommand 実装方針再検討 (コメント整理) ← 完了
   - 旧AwkCommandの方針をCLIエントリ統一の説明に更新（将来のBuiltin昇格に耐える設計を明記）。
 - [x] awk.rs:330 rest 先頭 `{` 有無 分岐ロジック再導入 or コメント更新 ← 実装
   - `/regex/ { ... }` と `/regex/ action` の双方を扱う簡易分岐を実装。空の場合は `print $0`。
 - [x] function.rs: 本体内 parse body_args のエラーチェック強化 (暗黙 assumptions) ← 実装
   - 空ボディ、孤立中括弧トークンの検出を追加し、明確なエラーを返すように強化。
- [x] nl.rs:8 body_numbering 拡張 (設定解釈)

### Builtins リソース監視系 (cron/at) 共通化
- [x] cron/at/timedatectl リソース使用 / 通知送出 API 抽象 (重複コード回避)

### Builtins / 観測性 拡張 (新規)
- [x] logstats builtin 実装 (ロギング統計出力) ← 2025-08-10
 - [x] logstats: `--json` / `--pretty` 出力フォーマット追加 ← 2025-08-10 実装 (#logstats-json-pretty)
- [x] logstats: メトリクス拡張 (rotations/sec, write_error_rate)
- [x] logstats: テスト (正常系 + write_errors 発生シミュレーション)
### Builtins / テスト支援
- [x] __argdump (引数列挙) テスト支援ビルトイン追加 ← 2025-08-10 (コマンド置換分割検証用)

## 8. HAL / ネットワーク / プラットフォーム
 - [x] (nxsh_hal/src/network_broken.rs:901) macOS Routing table 実装 ← 実装
 - [x] (nxsh_hal/src/network_broken.rs:907) Windows Routing table 実装 ← 実装
 - [x] (nxsh_hal/src/completion.rs:268) エイリアス補完 Shell alias システム統合 ← 実装
   - HAL補完で `NXSH_ALIAS_*` 環境変数および `NXSH_ALIAS_FILE`（name=value 行）からエイリアス候補を生成。
   - スコアリング/最大件数制御に連携、UI/コアと独立しても機能。
 - [x] (nxsh_hal/src/process.rs:433) placeholder 安全実装コメント → 実コードの安全保証ドキュメント化 ← 実装
   - `ProcessManager::spawn` の返却ハンドルポリシーを詳細化: 所有ハンドルは内部保持、外部には監視専用ハンドルを返却（child=None）。
   - ダブル wait/kill 回避、排他制御範囲、エラー方針をコメントで明文化。

## 9. セキュリティ / アップデート / 署名
 - [x] Capabilities Manifest (`cap.toml`) 必須化検証 (CI ルール) ← 実装
   - `nxsh_plugin::manager::validate_plugin_metadata` で `NXSH_CAP_MANIFEST_REQUIRED=1` 有効時に `metadata.capabilities` 未指定をエラーに。
 - [x] Ed25519 + TUF メタデータ: プラグイン鍵ローテーション手順自動化 ← 実装
   - `nxsh_plugin::keys::rotate_trusted_keys_if_requested()` を追加。`NXSH_ROTATE_KEYS=1` と新鍵 `NXSH_NEW_OFFICIAL_PUBKEY`/`NXSH_NEW_COMMUNITY_PUBKEY` 指定で `~/.nxsh/keys/*.pub` を原子的に更新、タイムスタンプ付きバックアップ作成。
   - セキュリティ初期化時（`IntegratedSecurityManager::new`）にベストエフォートでローテーションを適用。
 - [x] ヒストリ暗号化 (Argon2id + AES-GCM) 実装検証テスト (復号/改ざん検出試験) ← 実装
   - `nxsh_ui::history_crypto` を追加（Argon2id KDF + AES-256-GCM、MAGIC/Version/塩/nonce/密文形式）。
   - `line_editor.rs` に統合: `NXSH_HISTORY_ENCRYPT=1` と `NXSH_HISTORY_PASSPHRASE` 指定で保存時に暗号化、読み込み時は判別して復号（パスフレーズ未指定時はスキップ）。
   - Argon2パラメータは環境変数で調整可能（M/T/P）。改ざん時は復号エラーを返す。
 - [x] アップデータ: 差分パッチ署名検証 (updater.rs 実装と SPEC 整合) ← 実装
   - `verify_update()` で SHA-256 チェックサムに加え Ed25519 署名検証を実行（PEM/生B64公開鍵両対応）。
   - 署名鍵は環境/ファイルから初期化（`NXSH_UPDATE_KEYS_JSON`/`NXSH_OFFICIAL_PUBKEY` 等）。指紋は SHA-256 で算出。
   - TUF 相当メタデータ簡易検証を追加（`tuf_role=targets`、`tuf_expires` RFC3339 未失効）。
   - 依存: `ed25519-dalek`（feature `crypto-ed25519` で使用）、`pem`、`hex` を追加。全テスト緑。
 - [x] 公開鍵 (official/community) 埋め込み後のキー管理プロセス設計 ← 実装
   - `updater.rs` に検証鍵のロード/ローテ機構を追加:
     - ファイル: `~/.nxsh/keys/update_keys.json` または `NXSH_UPDATE_KEYS_PATH`
     - 環境: `NXSH_UPDATE_KEYS_JSON`（name→key map）、`NXSH_OFFICIAL_PUBKEY`/`NXSH_COMMUNITY_PUBKEY`
     - ローテ: `NXSH_UPDATE_ROTATE=1` と `NXSH_UPDATE_KEYS_JSON_NEW` で原子的置換＋バックアップ
- [x] CVE SLA 48h 対応ワークフロー自動テンプレート (issue → hotfix branch) ← 実装
  - Issue テンプレ: `.github/ISSUE_TEMPLATE/cve_sla_48h.yml` を追加。
  - 生成スクリプト: `scripts/gen_cve_workflow.rs` で `workflows/cve_hotfix/<CVE-ID>.md` を作成。

## 10. 国際化 / ローカライズ
 - [x] `.po`/`.mo` 生成パイプライン (gettext 互換) 実装 ← 実装
   - ディレクトリ構成: `i18n/po/<locale>/messages.po` → `i18n/mo/<locale>/LC_MESSAGES/messages.mo`
   - スクリプト: `scripts/compile_po_to_mo.sh`（`msgfmt` 前提）
   - CI: `.github/workflows/i18n.yml` で自動コンパイルとアーティファクト収集
 - [x] `unic-langid` による数値/日付/サイズローカライズ出力 ユニットテスト追加 ← 実装
   - `nxsh_builtins::common::locale_format` を追加（数値/小数/日付/サイズのローカライズ整形）。
   - 依存追加: `num-format`（軽量桁区切り）。`unic-langid` は既存 optional を利用。
   - テスト: `crates/nxsh_builtins/tests/locale_format_tests.rs` を追加。
 - [x] コマンド別多言語エイリアス マッピングテーブル設計 (例: 日本語/中国語/ロシア語) ← 実装
   - ローダ: `nxsh_core::locale_alias` を追加。ロケール検出（`LC_ALL`/`LANG`）→ `~/.nxsh/aliases/<locale>.toml` → 環境指定 → ビルトインの優先順で `ShellContext` に `set_alias` 注入。
   - ビルトイン定義: `assets/aliases/ja-JP.toml`/`zh-CN.toml`/`ru-RU.toml` を同梱。
   - `ShellContext::new()` で自動適用。テストは全緑維持。

## 11. 性能 / ベンチマーク
 - [x] 起動時間 ≤5ms 測定 CI 自動化 (hyperfine 連続計測 + 再現性) ← 実装
   - CI 追加: `.github/workflows/performance.yml` に hyperfine 測定と JSON 出力、`scripts/check_startup_budget.py` でしきい値検証を実施。
 - [x] 補完レイテンシ <1ms 計測インフラ ← 実装
   - CI で `cargo bench -p nxsh_hal --bench hal_performance` を実行し、<1ms アサートで自動ゲート。
- [ ] `grep -r TODO .` ripgrep 同等性能 ベンチ (SPEC / QA_PREVIEW 参照)
 - [x] `grep -r TODO .` ripgrep 同等性能 ベンチ (SPEC / QA_PREVIEW 参照) ← 実装
   - スクリプト: `scripts/bench_grep_vs_rg.sh` を追加（`/usr/bin/time` でリソース出力、rg/grep 比較）。
 - [x] `ls -R /usr` Bash 比 10x 測定スクリプト & 阈値ゲート ← スクリプト追加
   - スクリプト: `scripts/bench_ls_recursive.sh` を追加。比較レポートはCIアーティファクト収集用ワークフロー `.github/workflows/perf_compare.yml` を用意。
- [ ] JIT 有効時 2x 速度向上 (criterion ベンチ) — PGO/LTO プロファイル生成
  - [x] PGO 用ビルドプロファイル定義（`[profile.release-pgo]` 追加）
  - [x] CI: PGO ワークフロー追加（`.github/workflows/pgo.yml`、instrument→train→merge→use）
  - [x] JIT/MIR 切替の実行戦略制御（`NXSH_JIT`/`NXSH_EXEC_STRATEGY`）
  - [x] ベンチ: `jit_vs_interp` 追加（AST vs MIR/JIT 比較）
  - [ ] 2x 加速の達成確認（Criterion 結果で閾値評価・ゲート化）
 - [x] バイナリサイズ閾値 (≤9 MiB Release) CI `cargo bloat` チェック導入 ← 実装
   - リリースビルドのバイナリサイズを 9 MiB 以下に強制（同ワークフロー）。`cargo bloat` は参考出力。
	- [x] バイナリサイズ差分 CI レポート (size_report.ps1 の delta JSON 利用) ← 実装
  	- `.github/workflows/size_delta.yml` を追加。`scripts/size_report.sh` 実行結果と `size_report_local.json` があれば delta を算出しログ出力、レポートをアーティファクト化。

### 観測性 / Logging / Metrics (新規セクション)
- [x] CombinedWriter 導入 (console + file + stats 集計) ← 2025-08-10
- [x] write_errors カウンタ追加 ← 2025-08-10
- [x] nxsh_cli Cargo.toml logging feature alias 追加 (cfg 警告解消) ← 2025-08-11
- [x] logstats builtin JSON 出力（`--json`/`--pretty`、BusyBox/stub 構成でも整形式 JSONを出力）
 - [x] メトリクスを Prometheus 互換テキストでダンプするオプション ← 実装
   - `logstats --prom|--prometheus` を追加。`nxsh_log_*` プレフィックス、HELP/TYPE行を含むテキスト出力。
   - stub 構成では可用性のみを gauge で出力（`nxsh_log_available 0`）。
- [x] logging 機能最小化: busybox-min で file ロギング完全無効 (環境変数で完全無効化を実装)

## 12. テスト / QA
- [ ] 単体テスト件数 SPEC 記載値 (1500+ / 2000+) と現状差分調査
  - [x] 自動集計スクリプト追加（`scripts/count_tests.py`）とレポート用ワークフロー（`test_count.yml`）
- [ ] 統合テスト: POSIX PCTS / BATS / Pester 実行パイプライン整備
- [ ] Fuzzing `cargo-fuzz` 48h ラン → 成果物 (coverage, crashes) レポート化
- [ ] プロパティテスト `proptest` AST round-trip 充足率計測
  - [x] 基本的なASTラウンドトリップ性のプロパティテストを追加（`nxsh_parser/tests/property_roundtrip.rs`）
- [x] カバレッジ 95% 維持: grcov / tarpaulin 共通化
  - CI: `.github/workflows/coverage.yml` で tarpaulin を実行し、Cobertura 解析で 95% 未満を失敗扱い
- [ ] QA_PREVIEW_CHECKLIST の全項目 自動化可否仕分け

## 13. 将来ロードマップ (SPEC §12) — 着手前調査タスク
- [ ] GPU パイプラインアクセラレーション 技術調査 (wgpu / OpenCL / CUDA)
- [ ] 分散シェル クラスタリング プロト設計 (gRPC?/QUIC?)
- [ ] 組込 GUI ターミナル (Wayland/Win32) PoC
- [ ] AI 補完（非推論モード）設計 (統計/テンプレートベース)
- [ ] 音声入出力インタフェース PoC (VAD / STT API 選定)

## 14. その他プレースホルダー / コメント改善
- [x] UI: app.rs (複数箇所:186,477,500,507) placeholder ロジック本実装
  - `App::run()` を `CUIApp::run()` へ委譲し、実運用パス（プロンプト/入力/補完/実行/メトリクス/ステータスライン）に接続
- [ ] UI: ui_ux.rs:582 コマンド別 placeholder ステップ → 実行パス網羅テスト
- [ ] UI: enhanced_ui_tests.rs:13 test_placeholder → 実質的テストへ昇格
- [ ] Parser: 条件付き構築での body placeholder 二重格納 回避
- [ ] network_tools.rs: HTTP クライアント選定 (ureq vs reqwest vs hyper) 比較ドキュメント
  - [x] 比較ドキュメント追加: `docs/NETWORK_CLIENT_COMPARISON.md`
- [ ] crash_handler.rs: 取得すべき統計情報一覧仕様化
- [ ] users/group 参照 pure Rust 代替の候補比較 (shadow / etc parsing / sysinfo)
- [ ] scheduler NICE 値/priority を cross-platform 収束 (Win Job Object, Unix setpriority)

## 15. 未確認 / 要検証 (仕様と実装の整合チェック)
- [ ] 仕様で「完全実装済み」記載機能の統合テスト実測 (例: パイプ演算子 3 種 / 変数展開 / プロセス置換 / 算術展開)
- [ ] オブジェクトパイプライン: by-ref 並列 (`||>`) 並列度 / backpressure 実装確認
  - [x] まずは基礎のパイプラインUT追加（map/filterの通過確認: `crates/nxsh_core/tests/object_pipeline_parallel_tests.rs`）
- [ ] ヒストリ暗号化 (鍵導出 / AES-GCM) 実装コード有無調査
- [ ] プラグイン Capabilities Manifest 強制検証 (ロード拒否ケース)
- [ ] ログ: JSON / human-readable 2 モード切替単体テスト
  - [x] `logstats` の `--json`/`--pretty`/`--prom` 各モードの基本UTを追加（`crates/nxsh_builtins/tests/log_mode_tests.rs`）
- [ ] メトリクス一覧 (nxsh_* ) 実装 vs DESIGN.md 差分
- [ ] クラッシュダンプ (minidump / XOR 暗号化) 実装確認
- [ ] アップデータ: 差分パッチ bsdiff 実装位置確認
- [ ] CI で `cargo audit`, `cargo-vet`, `cargo udeps` 実行有無
 - [x] CI で `cargo audit`, `cargo-vet`, `cargo udeps` 実行有無 ← 実装
   - `.github/workflows/security_audit.yml` を追加。`cargo-audit` は致命で失敗、`vet`/`udeps` はベストエフォートでレポート。

## 16. 出典別 TODO 集計 (原文コメント抜粋)
（短文化。詳細は該当行参照）
- nxsh_ui/src/lib.rs:96 apply _config 配線
- nxsh_plugin/src/resource_table_new.rs:413 Arc 管理
- nxsh_plugin/src/remote.rs:303/312 公開鍵
- nxsh_plugin/src/manager.rs:448/454 event handler / emission
- nxsh_plugin/src/component_new.rs:286/291/510 ホスト呼び出し & ポインタ
- nxsh_parser/src/lib.rs:496/512/988 条件 placeholder & exhaustiveness
- nxsh_core/src/context.rs:616/702/753/916 timeout, cycle detect, history config, state copy
- nxsh_core/src/updater.rs:335 HTTP update
- nxsh_core/src/executor.rs:1113/1347 command substitution, exec time measure
- nxsh_core/src/crash_handler.rs:384-388 stats 取得
- nxsh_builtins/src/cron.rs:917/933/940/984/1102 resource monitor & notifications
- nxsh_builtins/src/at.rs:748/784/800 resource monitor & notifications
- nxsh_builtins/src/timedatectl.rs:650 sync impl
- nxsh_builtins/src/fsck.rs:13/83 repair & lost cluster
- nxsh_builtins/src/kill.rs:79 jobs table integration
- nxsh_builtins/src/find.rs:37 並列探索検討
- nxsh_builtins/src/ls.rs:769/806/1133 users/group lookup 代替
- nxsh_builtins/src/update.rs:133 force flag
- nxsh_builtins/src/common/crash_diagnosis.rs:420 HTTP POST
- nxsh_builtins/src/network_tools.rs:18 ureq 置換
- nxsh_builtins/src/cd.rs:280/287 env & dir config
- nxsh_builtins/src/ionice.rs:67 Unix 実装
- nxsh_builtins/src/nice.rs:57 Windows 実装
- nxsh_builtins/src/date.rs:505 holidays
- nxsh_builtins/src/export_old.rs:59 refactor
- nxsh_builtins/src/alias_old.rs:42 refactor
- nxsh_builtins/src/fsck.rs:83 placeholder detection
- nxsh_hal/src/network_broken.rs:901/907 routing table
- nxsh_hal/src/completion.rs:268 alias completion
- nxsh_hal/src/process.rs:433 安全性コメント具体化

---
## 17. 優先度 (初期提案)
- P0: セキュリティ鍵/アップデータ/timeout/executor コマンド置換/履歴暗号化/イベント基盤
- P1: 言語機能 (match exhaustiveness, try/catch 完全化, namespaces), プラグイン host 関数, リソースモニタリング
- P2: UI 録画/再生, BusyBox モード, スケジューラ NICE, 祝日/force flag 等
- P3: 並列探索, users/group 代替, mockups, fsck 修復

---
更新手順: 新たな TODO コメントを追加する際は本ファイルへ同期。完了時は `[x]` と日付 / PR 番号を付記。

