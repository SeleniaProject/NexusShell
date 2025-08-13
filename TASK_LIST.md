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
  - [x] `crates/nxsh_builtins/src/timedatectl.rs`: ドリフト計算/統計/イベント送出の実装（監視ループの実体化）
  - [x] `crates/nxsh_builtins/src/update.rs`: アップデートキャッシュ実体（固定パスの解消とキャッシュ検証）
  - [x] `crates/nxsh_builtins/src/awk.rs`: 簡易版で未対応のアクション/式の拡充（if/while/for/printf/範囲パターン/next/exit）

## 6. テスト / QA / CI
  - [ ] 単体テスト件数の充足（SPEC 記載値との差分把握と拡充）
  - [x] fsck: FAT12/16/32 クラスタ編集およびハッシュ/署名検証の単体テストを追加（FAT32のクラスタ解放およびFATミラー同期、ジャーナル署名/検証を追加済み。FAT12/16の追加テストは継続項目）
 - [ ] 統合テスト（POSIX PCTS / BATS / Pester）パイプライン拡充
 - [ ] Fuzzing（`cargo-fuzz` 長時間実行）とレポート化
 - [ ] QA_PREVIEW_CHECKLIST 自動化可否の仕分け
 - [ ] JIT/MIR ベンチでの 2x 目標達成（Criterion による閾値ゲート）

---
更新手順: 未実装が解消されたら当該行を `[x]` に変更し、関連 PR/コミットを付記する。
