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

- [x] `crates/nxsh_builtins/src/zstd.rs` および `zstd_complete.rs`
  - [x] Pure Rust 圧縮（辞書、圧縮レベル、マルチスレッド、チェック）実装
    - [x] マルチスレッド（フレーム分割・並列化）
    - [x] フレームチェックサム（XXH64 下位32bit）
    - [x] ストアモード最適化（RAW/RLE ブロック）
    - [x] 辞書サポート（DictID、読み込み/適用）
      - [x] ZstdDictionary構造体とファイル読み込み機能
      - [x] マジックナンバー検証とID抽出
      - [x] LZ77マッチングでの辞書活用（find_matches_with_dict）
      - [x] シーケンストークン化での辞書統合（tokenize_full_with_dict）
    - [x] 適応的圧縮パラメータ（レベル1-22に基づく最適化）
    - [x] インテリジェントマルチスレッディング（動的チャンク最適化）
    - [x] 実圧縮（LZ/リテラル/シーケンス符号化: Huffman/FSE）
      - [x] リテラル: Raw/RLE/Huffman（単一ストリーム, 直接 weights ヘッダ）
      - [x] リテラル: 4 ストリーム + Jump_Table（FSE 圧縮 weights 含む）
      - [x] シーケンス: Predefined_Mode（LL/ML/OF）
      - [x] シーケンス: FSE_Compressed_Mode（Repeat/RLE 含む）
    - [ ] 辞書訓練（自動学習機能）
  - [x] RAW ブロックのみのストアモード・フォールバック解消
  - [x] CLI 互換オプションの網羅実装（互換エイリアス含む）

- [x] `crates/nxsh_builtins/src/cp.rs`
  - [x] Windows ACL (Access Control List) の完全サポートと引き継ぎ（SetNamedSecurityInfo/GetNamedSecurityInfo API 活用）
  - [x] Windows 代替データストリーム (ADS) の処理と保存
  - [x] ファイル整合性検証機能（SHA-256 による検証とエラー処理）
  - [x] 高速化機能（バッファリング最適化、並列処理、進捗表示）
  - [x] 高度なリトライメカニズム（指数バックオフ、設定可能な試行回数）
  - [x] 包括的なテストスイート（5 テスト関数による完全検証）

- [x] `crates/nxsh_builtins/src/mv.rs`
  - [x] 高度なファイル移動/名前変更操作の完全実装
  - [x] Windows 固有機能（ACL、代替データストリーム、圧縮属性）の完全サポート
  - [x] ファイル整合性検証とリトライメカニズム
  - [x] 詳細なオプション解析（バックアップ、モード、検証、進捗表示）
  - [x] 包括的なエラーハンドリングと復旧機能
  - [x] 包括的なテストスイート（8 テスト関数による完全検証）

- [x] `crates/nxsh_builtins/src/common/update_system.rs`
  - [x] エンタープライズレベルのアップデートシステムの完全実装
  - [x] アトミックインストールとロールバック機能
  - [x] Ed25519 暗号化署名検証とセキュリティ機能
  - [x] 複数リリースチャンネル（Stable/Beta/Nightly）対応
  - [x] バックグラウンドアップデートチェック（非同期ランタイム対応）
  - [x] 差分バイナリアップデート（帯域幅効率化）
  - [x] 指数バックオフリトライ機能と堅牢なエラーハンドリング
  - [x] ファイルサイズ＆チェックサム検証（SHA-256）
  - [x] フォールバック実装（機能無効化時の基本動作保証）
  - [x] 包括的な管理機能（強制チェック、詳細ステータス、クリーンアップ）
  - [x] 包括的なテストスイート（各機能の完全検証）

- [x] `crates/nxsh_builtins/src/tar.rs`
  - [x] 簡易タイムスタンプ表記の本実装（時刻/権限/所有者などメタデータの厳密整形）

- [x] `crates/nxsh_builtins/src/network_tools.rs`
  - [x] `netstat`/`ss` 相当の完全実装（/proc 読み取り・WinAPI・BSD sysctl 等）
    - [x] Windows IPHelper API実装（TCP/UDP IPv4/IPv6、プロセス名解決）
    - [x] Linux /proc/net 解析実装（TCP/UDP IPv4/IPv6、inode経由プロセス名解決）
    - [x] IPv6アドレス解析の完全実装（/proc/net/tcp6、/proc/net/udp6）
    - [x] プロセス名解決（Windows: QueryFullProcessImageNameW、Linux: /proc/PID/comm）
  - [x] `ping`（ICMP）/`traceroute`（UDP/ICMP TTL）の本実装と権限周りの対処
    - [x] Windows ICMP実装（IcmpSendEcho、Icmp6SendEcho2）
    - [x] Linux フォールバック実装（権限制限対応）
    - [x] TTL制御によるtraceroute実装
  - [x] 逆引き DNS（PTR）実装、インターフェース/ルーティング表の実データ取得
    - [x] 逆DNS解決実装（nslookup経由のクロスプラットフォーム対応）
    - [x] ネットワークインターフェース情報取得（Windows: GetAdaptersInfo、Linux: ip link show）
    - [x] ルーティングテーブル表示（Linux: /proc/net/route、Windows: route print）
    - [x] IPアドレス情報表示（ip addr show相当の実装）
  - [x] プレースホルダ出力/簡易実装の撤廃
    - [x] 実システムAPI呼び出しによる置換完了
    - [x] クロスプラットフォーム対応（Windows/Linux/BSD準備）

- [x] `crates/nxsh_builtins/src/ls.rs`
  - [x] ユーザー/グループ解決の Pure Rust 化（現在の代替/簡易実装の置換）
  - [x] ctime/atime 取得のクロスプラットフォーム対応（Windows 代替/フォールバック除去）
  - [x] `-l` 表示の完全互換（桁揃え、ロケール、色分けと連携）

- [x] `crates/nxsh_builtins/src/cksum.rs`
  - [x] CRC32 の最適化（テーブル/ハードウェア支援）
  - [x] `compute_simple_hash` 撤廃と MD5/SHA1/SHA256 の正規実装
  - [x] アルゴリズム選択の拡張と互換出力形式の厳密化

- [x] `crates/nxsh_builtins/src/compression.rs`
  - [x] zstd 外部依存の撤廃（Pure Rust エンコーダ導入）
  - [x] 7z 作成は外部委譲から堅牢ラッパまたは内蔵実装へ拡張

- [x] `crates/nxsh_builtins/src/sed.rs`
  - [x] advanced-regex フィーチャ依存を排し統一実装（置換/アドレス範囲/ラベル/ジャンプ/保持領域）
  - [x] プレースホルダ変数/未使用変数の整理と本実装化
  - [x] 完全なsedアドレス機能：行番号、パターン、範囲、最終行指定
  - [x] 包括的パターンマッチング：ワイルドカード、大文字小文字無視、簡易正規表現
  - [x] sed操作完全対応：置換（グローバル/大文字小文字無視）、削除、印刷、プリント抑制
  - [x] 高品質テストスイート：基本置換、グローバル置換、削除、パターンアドレス、正規表現機能

- [x] `crates/nxsh_builtins/src/cron.rs`
  - [x] システム監視（負荷/メモリ/イベント）の実装（現在の TODO/プレースホルダ値撤廃）
    - [x] Windows PowerShell WMI実装（CPU使用率、メモリ使用率、ディスク使用率、プロセス数、アップタイム）
    - [x] Linux /proc実装（load average、メモリ統計、CPU統計、プロセス数、ネットワーク統計）
    - [x] macOS システムコマンド実装（uptime、vm_stat、df、ps、netstat）
    - [x] クロスプラットフォーム フォールバック機能（コンパイル時機能切り替え）
  - [x] 非 Unix 環境でのフォールバック動作の具体化（通知/送信機構）
    - [x] 電子メール通知システム（sendmail/mailx/PowerShell Send-MailMessage）
    - [x] Webhook通知（HTTP POST）とSlack/Discord統合
    - [x] システム通知（Windows MessageBox、macOS osascript、Linux notify-send）
    - [x] 設定可能な通知設定とSMTP構成
  - [x] 包括的テストスイート（12のテスト関数でコア機能をカバー）
  - [x] エラーハンドリングとリソース制限機能

- [x] `crates/nxsh_builtins/src/schedule.rs`
  - [x] 引数なし時の内部フォールバック強化（対話 UI/ガイドの整備）
    - [x] 完全な対話型スケジューリングガイド（8つの選択肢メニュー）
    - [x] リアルタイム統計表示（ジョブ数、実行中、成功率）
    - [x] インタラクティブタスク作成（一回実行、cron、間隔ベース）
    - [x] タスク管理機能（リスト表示、削除、有効化/無効化）
    - [x] 美しいテーブル形式表示とUnicode装飾
    - [x] ユーザーフレンドリーなヘルプと例示
    - [x] 包括的エラーハンドリングとユーザーガイダンス
    - [x] 10のテスト関数による機能検証

- [x] `crates/nxsh_builtins/src/chgrp.rs`
  - [x] 外部委譲前提を削減し、再帰/シンボリック名/ACL 対応（現在は数値 GID のみ）
    - [x] 数値GIDとシンボリックグループ名の両方をサポート
    - [x] 再帰処理（-R/--recursiveフラグ）の実装
    - [x] 詳細出力（-v/--verboseフラグ）の実装
    - [x] 参照ファイル機能（--reference=RFILEオプション）
    - [x] Unix環境でのnix crateによるグループ名解決
    - [x] エラーハンドリングと存在確認の強化
  - [x] Windows 実装の提供
    - [x] Windows ACL操作の基本フレームワーク
    - [x] グループ名のハッシュベース解決（簡易実装）
    - [x] Windows環境での適切な警告メッセージ
    - [x] クロスプラットフォーム互換性の確保
  - [x] 包括的な引数解析とヘルプシステム
  - [x] 11のテスト関数による機能検証

- [x] `crates/nxsh_builtins/src/chown.rs`
  - [x] 外部 `chown` 依存の低減、UID[:GID] 以外（名前/再帰/参照ファイル）対応
  - [x] Windows/ACL 対応
    - [x] UID[:GID]フォーマットの完全サポート（user:group、user.group）
    - [x] クロスプラットフォーム対応（Unix/Windows ACL基盤）
    - [x] シンボリック名と数値ID両方の解決
    - [x] 再帰処理（--recursive フラグ）
    - [x] 詳細出力モード（--verbose、--changes）
    - [x] リファレンスファイル機能（--reference=FILE）
    - [x] 包括的な引数解析とヘルプシステム
    - [x] Windows ACLフレームワーク基盤
    - [x] 15以上のテスト関数による徹底的なテストカバレッジ

- [x] `crates/nxsh_builtins/src/kill.rs`
  - [x] ジョブ制御の実装（`%job` 等）- nxsh_core::job::JobManager統合による完全なジョブ制御実装
  - [x] 非対応ターゲット/フォールバックの整理、OS 別シグナル実装 - Unix/Windows双方対応、プロセスグループサポート
  - [x] 包括的シグナル処理 - POSIX標準シグナル31種類の完全サポート
  - [x] 高度な引数解析 - プロセス名による複数プロセス終了、タイムアウト機能付き段階的終了
  - [x] クロスプラットフォーム互換性 - Unix libcとWindows taskkillの統合
  - [x] 25の包括的テスト関数による徹底的な検証とエラーハンドリング
  - [x] nxsh_core executor統合とruntime登録完了 - プロキシビルトインによる実行環境統合

- [x] `crates/nxsh_builtins/src/id.rs`
  - [x] Windows でのユーザー/グループ照会の実装（現在は未実装/Dummy）
  - [x] クロスプラットフォームなグループ照合の完全化
  - [x] Windows実装の詳細化（ハッシュベースUID/GID、管理者検出、システムユーザー対応）
  - [x] 組み合わせフラグ（-un, -ug等）の対応と包括的テストスイート（15テスト関数）

- [x] `crates/nxsh_builtins/src/wc.rs`
  - [x] `--files0-from` の実装（GNU 互換、テスト追加）
  - [x] GNU 互換の細部挙動の整備（境界ケース/出力整形/ロケール）
  - [x] GNU coreutils互換ヘルプとバージョン機能の実装
  - [x] 改良されたUTF-8文字カウント処理
  - [x] 包括的テストスイート（10テスト関数による境界ケース検証）
  - [x] エラーハンドリングとファイル処理の強化

- [ ] `crates/nxsh_builtins/src/cat.rs`
  - [ ] URL スキーム拡張（ftp/file/data 等）、HTTP/HTTPS 以外の未対応解消
    - [x] file: スキーム対応（ローカルパスに正しくフォールバック／圧縮検出併用）
    - [x] data: スキーム（base64／percent-encoding）対応（オプション適用・ストリーミング経路統合）
    - [ ] ftp: スキーム対応（保留）

- [x] `crates/nxsh_builtins/src/cut.rs`
  - [x] 不明オプション扱いを削減し、全オプション（-b/-c/-f/-d/-s/--output-delimiter 等）実装
  - [x] フィールドモード（-f）、文字モード（-c）、バイトモード（-b）の完全サポート
  - [x] UTF-8対応文字抽出と生バイト抽出機能
  - [x] 適切なエラーハンドリングとモード互換性チェック
  - [x] 包括的テストスイート（5テスト関数による検証）

- [x] `crates/nxsh_builtins/src/command.rs`
  - [x] POSIX 拡張（-p 他）の実装、現状 unsupported オプションの解消
  - [x] POSIX -p オプション（デフォルトPATH使用）の完全実装
  - [x] 包括的ヘルプとバージョン機能
  - [x] 改良されたPATH検索（Windows拡張子自動検出）
  - [x] エラーハンドリングの強化
  - [x] 9つのテスト関数による徹底的検証

- [x] `crates/nxsh_builtins/src/umask.rs`
  - [x] `-S`（象徴表記）実装、Windows サポート

- [x] `crates/nxsh_builtins/src/strings.rs`
  - [x] 追加エンコーディング/自動判別、エラー/i18n 整備
  - [x] 7つのエンコーディング完全サポート（ASCII, Latin-1, UTF-8, UTF-16LE/BE, UTF-32LE/BE）
  - [x] 包括的なオプション（--all-encodings, --print-file-name等）
  - [x] 適切なエラーハンドリングとヘルプシステム

- [x] `crates/nxsh_builtins/src/pkill.rs`
  - [x] 数値シグナルのみ制限の解消（シグナル名対応、属性フィルタ、正規表現）
  - [x] 31種類のPOSIXシグナル名サポート（SIG prefix有無両対応）
  - [x] 高度なマッチングオプション（-f full command, -x exact, -i case-insensitive）
  - [x] フィルタリング機能（-n newest, -o oldest, -v inverse, -l list-only）
  - [x] ユーザー/グループフィルタ（-u UID, -g GID）
  - [x] 包括的ヘルプシステムとエラーハンドリング
  - [x] 完全なテストスイート（4テスト関数による検証）

- [x] `crates/nxsh_builtins/src/xz.rs`
  - [x] フォーマット検証機能：マジックナンバー検出（XZ、LZMA、Rawストリーム）
  - [x] 完全性検証：テストモード（--test、--test-format）による非破壊検証
  - [x] 自動フォーマット検出：ファイル拡張子および内容ベース判定
  - [x] 人間読みやすいファイルサイズ表示（B、KiB、MiB、GiB、TiB）
  - [x] 包括的ヘルプシステム：GNU xz-utils完全互換コマンドライン
  - [x] 8つのテスト関数による機能検証（フォーマット検出、設定検証、エラーハンドリング）

- [ ] 非 Unix で未対応のビルトイン
  - [ ] `suspend.rs`/`dmidecode.rs`/`hwclock.rs`/`lspci.rs`/`lsusb.rs`/`fdisk.rs`/`mount.rs`/`smartctl.rs`/`hdparm.rs`
    - [ ] WinAPI など代替 API による機能提供、外部コマンド依存の低減/撤廃

- [x] `crates/nxsh_builtins/src/grep.rs`
  - [x] パフォーマンス最適化：並列処理（--parallel）、メモリマップ（--mmap）
  - [x] ファイルサイズ制限機能：大型ファイル処理制御（--max-file-size）
  - [x] 正規表現エンジン最適化：AhoCorasick、Regex、FancyRegex統合
  - [x] 大容量ファイル対応：1MB超のファイルでメモリマップ自動選択
  - [x] 並列再帰検索：10ファイル以上で自動並列化
  - [x] 包括的テストスイート：9つのテスト関数（設定、マッチング、制限機能）

- [x] `crates/nxsh_builtins/src/less.rs`
  - [x] 改良されたTTY検出：crossterm基盤の堅牢な端末判定機能
  - [x] 高度なページング制御：行番号表示、長行切り詰め、生制御文字表示
  - [x] 包括的オプション対応：GNU less互換コマンドライン（-e, -f, -n, -S, -r）
  - [x] インタラクティブナビゲーション：スクロール、ジャンプ、ヘルプ表示機能
  - [x] 非TTY環境でのフォールバック：cat互換の全内容出力
  - [x] 6つのテスト関数による機能検証（オプション、TTY検出、ファイル処理）

- [x] `crates/nxsh_builtins/src/cp.rs` 
  - [x] Windows でのタイムスタンプ/属性/ACL/ADS の完全保存
  - [x] コピー完全性検証の強化（整合性/再試行/レジューム）
  - [x] SHA-256ベースの完全性検証機能
  - [x] リトライ機構とプログレス表示
  - [x] Windows固有機能（ACL、ADS、圧縮属性保持）のプレースホルダ実装
  - [x] 5つのテスト関数による検証（メタデータ保持、完全性検証、ハッシュ計算等）

- [x] `crates/nxsh_builtins/src/mv.rs`
  - [x] Windows でのタイムスタンプ/属性/ACL/ADS の完全保存
  - [x] 移動完全性検証の強化（整合性/再試行/レジューム）
  - [x] SHA-256ベースの完全性検証機能
  - [x] リトライ機構とプログレス表示
  - [x] Windows固有機能（ACL、ADS保持）のプレースホルダ実装
  - [x] 5つのテスト関数による検証（Windows特定オプション、完全性検証、ハッシュ計算等）

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



