## ビルトイン簡略化/未実装チェックリスト

- [x] `crates/nxsh_builtins/src/awk.rs`
  - [x] 簡易パーサを完全対応へ拡張（BEGIN/END、正規表現、式/配列/連想配列、ユーザー関数、条件/ループ、フィールド分割、レコード分割）
  - [x] `printf`/`print` の完全互換（フォーマット指定子/幅/精度/エスケープ）
  - [x] 簡易式評価の撤廃と本格評価器の実装（数値/文字列/ブール/比較/正規表現一致）

- [x] `crates/nxsh_builtins/src/find.rs`
  - [x] 式ツリー評価の完全実装（AND/OR/NOT、括弧、優先順位、短絡評価）
  - [x] 正規表現エンジンの実装/選択（Basic/Extended/Perl 相当）
  - [x] `-printf`/`-fls` 出力の完全化（現在の簡易実装を置換）
  - [x] GID→グループ名解決の実装（現在は簡易名称）
  - [x] 並列探索（rayon）導入と `--parallel` オプション実装
  - [x] レガシーベクターフォールバック動作の撤廃
  - [x] 進捗UI（`progress-ui`）と統合した堅牢な進行表示

- [x] `crates/nxsh_builtins/src/timedatectl.rs`
  - [x] NTP クライアントの本実装（送受時刻スタンプ、オフセット/遅延/ジッタ/ストラタム計算）
  - [x] タイムゾーンDB/夏時間判定の実装（最小ビルドの UTC 限定スタブ排除）
  - [x] 外部 `ntpdate/chrony/date` フォールバック依存の削減/撤廃
  - [x] 簡略オフセット計算の精密化（NTP 固定小数点処理）
  - [x] 完全 i18n 対応（メッセージ/書式）

- [x] `crates/nxsh_builtins/src/fsck.rs`
  - [x] FAT チェイン解析/ロストクラスタ/クロスリンク厳密検出
  - [x] `-a`（自動修復）実装とジャーナリングによる安全な書き戻し
  - [x] 簡易マーキング/naive 比較の撤廃と完全検査の導入

- [ ] `crates/nxsh_builtins/src/zstd.rs` および `zstd_complete.rs`
  - [ ] Pure Rust 圧縮（辞書、圧縮レベル、マルチスレッド、チェック）実装
    - [x] マルチスレッド（フレーム分割・並列化）
    - [x] フレームチェックサム（XXH64 下位32bit）
    - [x] ストアモード最適化（RAW/RLE ブロック）
    - [ ] 実圧縮（LZ/リテラル/シーケンス符号化: Huffman/FSE）
      - [x] リテラル: Raw/RLE/Huffman（単一ストリーム, 直接 weights ヘッダ）
      - [ ] リテラル: 4 ストリーム + Jump_Table（FSE 圧縮 weights 含む）
      - [ ] シーケンス: Predefined_Mode（LL/ML/OF）
      - [ ] シーケンス: FSE_Compressed_Mode（Repeat/RLE 含む）
    - [ ] 辞書（DictID、読み込み/適用/訓練）
  - [x] RAW ブロックのみのストアモード・フォールバック解消
  - [x] CLI 互換オプションの網羅実装（互換エイリアス含む）

- [x] `crates/nxsh_builtins/src/tar.rs`
  - [x] 簡易タイムスタンプ表記の本実装（時刻/権限/所有者などメタデータの厳密整形）

- [ ] `crates/nxsh_builtins/src/network_tools.rs`
  - [ ] `netstat`/`ss` 相当の完全実装（/proc 読み取り・WinAPI・BSD sysctl 等）
  - [ ] `ping`（ICMP）/`traceroute`（UDP/ICMP TTL）の本実装と権限周りの対処
  - [ ] 逆引き DNS（PTR）実装、インターフェース/ルーティング表の実データ取得
  - [ ] プレースホルダ出力/簡易実装の撤廃

- [ ] `crates/nxsh_builtins/src/ls.rs`
  - [ ] ユーザー/グループ解決の Pure Rust 化（現在の代替/簡易実装の置換）
  - [ ] ctime/atime 取得のクロスプラットフォーム対応（Windows 代替/フォールバック除去）
  - [ ] `-l` 表示の完全互換（桁揃え、ロケール、色分けと連携）

- [ ] `crates/nxsh_builtins/src/cksum.rs`
  - [ ] CRC32 の最適化（テーブル/ハードウェア支援）
  - [ ] `compute_simple_hash` 撤廃と MD5/SHA1/SHA256 の正規実装
  - [ ] アルゴリズム選択の拡張と互換出力形式の厳密化

- [ ] `crates/nxsh_builtins/src/compression.rs`
  - [x] zstd 外部依存の撤廃（Pure Rust エンコーダ導入）
  - [ ] 7z 作成は外部委譲から堅牢ラッパまたは内蔵実装へ拡張

- [ ] `crates/nxsh_builtins/src/sed.rs`
  - [ ] advanced-regex フィーチャ依存を排し統一実装（置換/アドレス範囲/ラベル/ジャンプ/保持領域）
  - [ ] プレースホルダ変数/未使用変数の整理と本実装化

- [ ] `crates/nxsh_builtins/src/cron.rs`
  - [ ] システム監視（負荷/メモリ/イベント）の実装（現在の TODO/プレースホルダ値撤廃）
  - [ ] 非 Unix 環境でのフォールバック動作の具体化（通知/送信機構）

- [ ] `crates/nxsh_builtins/src/schedule.rs`
  - [ ] 引数なし時の内部フォールバック強化（対話 UI/ガイドの整備）

- [ ] `crates/nxsh_builtins/src/chgrp.rs`
  - [ ] 外部委譲前提を削減し、再帰/シンボリック名/ACL 対応（現在は数値 GID のみ）
  - [ ] Windows 実装の提供

- [ ] `crates/nxsh_builtins/src/chown.rs`
  - [ ] 外部 `chown` 依存の低減、UID[:GID] 以外（名前/再帰/参照ファイル）対応
  - [ ] Windows/ACL 対応

- [ ] `crates/nxsh_builtins/src/kill.rs`
  - [ ] ジョブ制御の実装（`%job` 等）
  - [ ] 非対応ターゲット/フォールバックの整理、OS 別シグナル実装

- [ ] `crates/nxsh_builtins/src/id.rs`
  - [ ] Windows でのユーザー/グループ照会の実装（現在は未実装/Dummy）
  - [ ] クロスプラットフォームなグループ照合の完全化

- [ ] `crates/nxsh_builtins/src/wc.rs`
  - [x] `--files0-from` の実装（GNU 互換、テスト追加）
  - [ ] GNU 互換の細部挙動の整備（境界ケース/出力整形/ロケール）

- [ ] `crates/nxsh_builtins/src/cat.rs`
  - [ ] URL スキーム拡張（ftp/file/data 等）、HTTP/HTTPS 以外の未対応解消
    - [x] file: スキーム対応（ローカルパスに正しくフォールバック／圧縮検出併用）
    - [x] data: スキーム（base64／percent-encoding）対応（オプション適用・ストリーミング経路統合）
    - [ ] ftp: スキーム対応（保留）

- [ ] `crates/nxsh_builtins/src/cut.rs`
  - [ ] 不明オプション扱いを削減し、全オプション（-b/-c/-f/-d/-s/--output-delimiter 等）実装

- [ ] `crates/nxsh_builtins/src/command.rs`
  - [ ] POSIX 拡張（-p 他）の実装、現状 unsupported オプションの解消

- [ ] `crates/nxsh_builtins/src/umask.rs`
  - [x] `-S`（象徴表記）実装、Windows サポート

- [ ] `crates/nxsh_builtins/src/strings.rs`
  - [ ] 追加エンコーディング/自動判別、エラー/i18n 整備

- [ ] `crates/nxsh_builtins/src/pkill.rs`
  - [ ] 数値シグナルのみ制限の解消（シグナル名対応、属性フィルタ、正規表現）

- [ ] `crates/nxsh_builtins/src/xz.rs`
  - [ ] 未対応 `format/check` の拡張、完全圧縮/解凍機能

- [ ] 非 Unix で未対応のビルトイン
  - [ ] `suspend.rs`/`dmidecode.rs`/`hwclock.rs`/`lspci.rs`/`lsusb.rs`/`fdisk.rs`/`mount.rs`/`smartctl.rs`/`hdparm.rs`
    - [ ] WinAPI など代替 API による機能提供、外部コマンド依存の低減/撤廃

- [ ] `crates/nxsh_builtins/src/grep.rs`
  - [ ] 正規表現エンジン不在時のリテラルフォールバック解消（BRE/ERE/PCRE 相当実装）
  - [ ] 性能最適化（並列/mmap/巨大ファイル）

- [ ] `crates/nxsh_builtins/src/less.rs`
  - [ ] 非 TTY 時のフォールバック改善（ページング模擬、幅制御、色抜き）

- [ ] `crates/nxsh_builtins/src/cp.rs` / `mv.rs`
  - [ ] Windows でのタイムスタンプ/属性/ACL/ADS の完全保存
  - [ ] コピー完全性検証の強化（整合性/再試行/レジューム）

- [ ] `crates/nxsh_builtins/src/common/update_system.rs`
  - [ ] `updates`/`async-runtime` 無効時のスタブ解消
  - [ ] 原子的インストール/ロールバックの実装とテスト
  - [ ] バックグラウンドチェッカーの no-op 解消

- [ ] `crates/nxsh_builtins/src/common/logging.rs`
  - [ ] 複数出力サブスクライバの合成（現在の stderr 単独フォールバック撤廃）

- [ ] `crates/nxsh_builtins/src/common/metrics.rs`
  - [ ] プレースホルダ更新の撤廃と実メトリクス集計/公開

- [ ] `crates/nxsh_builtins/src/lib.rs`
  - [ ] `super-min` 構成時の egrep 等スタブ整理と機能提供方針の確立

- [x] i18n スタブ解消
  - [x] `at.rs` の i18n/フォールバック実装
  - [x] `date.rs` の i18n/フォールバック実装
  - [x] `cat.rs` の i18n/フォールバック実装
  - [x] `timedatectl.rs` の i18n/フォールバック実装
  - [x] ロケール（ja/ko/pt/ru/zh/en/es/fr/it/de）に必要キー追加とフォールバック整備

- [ ] テスト関連/その他
  - [ ] `crates/nxsh_builtins/src/echo.rs` のテストで示される `stdout.collect()` 相当の検証パス実装
  - [x] `crates/nxsh_core/src/builtins/testutils.rs` の最小 `echo` を本番相当の実装/注入に置換
    - 実装済: -n/-e/-E と各種エスケープ（\a, \b, \c, \e, \f, \n, \r, \t, \v, \\, 8進, 16進）をサポート
    - 影響: 既存テストは全てパス（回帰なし）
  - [x] `timedatectl` の NTP タイムスタンプテストを修正（正確なエポック変換）

## グローバル品質・安定化（本スプリント）

- [x] workspace 全体の Clippy（-D warnings）をゼロに（crate 横断のlint修正）
- [x] cargo test 全緑化とフレーク抑止
  - [x] nxsh_core: 環境変数ロックの統一（OnceLock + Mutex）と login-shell 検出の堅牢化（SSH/SHLVL ヒューリスティクス強化）
  - [x] nxsh_plugin: capabilities manifest の必須化を「設定/環境変数」両対応に拡張（`capabilities_manifest_required` 追加、`NXSH_CAP_MANIFEST_REQUIRED` とOR）
  - [x] PluginConfig の後方互換性確保（`#[serde(default)]` 付与）



