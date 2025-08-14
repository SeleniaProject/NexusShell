# NexusShell 未実装タスクリスト（コードベース準拠）

> 本ファイルは仕様書とコードベース（ソース内のコメント/エラーメッセージ）から抽出した「未実装/プレースホルダー」のみを列挙する。実装完了時は当該行を `[x]` に更新し、関連PR/コミットを付記する。

## 0. 集計メタ
- 収集方法: キーワード検索と手動確認
  - 検索語: `TODO`, `unimplemented`, `not yet implemented`, `not implemented`, `未実装`, `placeholder`
- 注意: 仕様上の将来構想（研究項目）はここに含めない

---
## 5. Builtins（未実装/部分実装）
  - [ ] `crates/nxsh_builtins/src/zstd_complete.rs`: 圧縮機能の実装（Pure Rust 圧縮ライブラリ統合）
  - [x] `crates/nxsh_builtins/src/fsck.rs`: 修復モード/デバイス処理（FAT12/16/32クラスタ解放、FATミラー整合性検証、ジャーナル署名/検証 追加）
  - [x] `crates/nxsh_builtins/src/timedatectl.rs`: ドリフト計算/統計/イベント送出の実装（監視ループの実体化）＋ `status`/`timesync-status`/`statistics` の JSON 出力対応（i18n ヘルプ更新含む）
  - [x] `crates/nxsh_builtins/src/update.rs`: アップデートキャッシュ実体（固定パスの解消とキャッシュ検証）
  - [x] `crates/nxsh_builtins/src/awk.rs`: 簡易版で未対応のアクション/式の拡充（if/while/for/printf/範囲パターン/next/exit）
  - [x] `crates/nxsh_hal`: 高速/通常補完に `timedatectl` と主要圧縮系（zstd/unzstd/zip/unzip/bzip2/xz/unxz）を追加＋ `timedatectl`/`zstd`/`unzstd` の代表的フラグ補完を追加
  - [x] `crates/nxsh_builtins/src/mkfs.rs`: FAT12/16/32 のフォーマット生成に対応（fatfs を利用）

## 6. テスト / QA / CI
  - [ ] 単体テスト件数の充足（SPEC 記載値との差分把握と拡充）
  - [x] fsck: FAT12/16/32 クラスタ編集およびハッシュ/署名検証の単体テストを追加（FAT12/16/32 のクラスタ解放、FAT12/16/32 のミラー同期、ジャーナル署名/検証を追加）
  - [x] timedatectl: `--json/-J` のスモーク/サンプル往復テストを追加（構造シリアライズ整合性の検証）
  - [x] nxsh_core: ログインシェル検出の安定化（`SHLVL` 未設定時でもログイン環境が揃う場合は true と判定）
  - [x] nxsh_core: グローバルタイムアウトの安定化（ENVロックと `ensure_global_timeout_from_env` 導入で `124` 返却を安定化）
  - [x] zstd/unzstd: CLI テスト拡充（`--help`/`-l`/標準出力モード、外部 zstd 有無に応じた往復、無効 .zst のエラーパス、unzstd の純Rust解凍テスト）
 - [ ] 統合テスト（POSIX PCTS / BATS / Pester）パイプライン拡充
 - [ ] Fuzzing（`cargo-fuzz` 長時間実行）とレポート化
 - [ ] QA_PREVIEW_CHECKLIST 自動化可否の仕分け
 - [ ] JIT/MIR ベンチでの 2x 目標達成（Criterion による閾値ゲート）

---
更新手順: 未実装が解消されたら当該行を `[x]` に変更し、関連 PR/コミットを付記する。
