## Summary

Describe the change concisely.

## Changes

- What was changed and why
- Breaking changes (if any)

## Testing

- How you tested this change (commands, platforms)
- CI status should be green

## Checklist

- [ ] Code builds locally: `cargo check --workspace --all-features`
- [ ] Tests pass: `cargo test --workspace --all-features`
- [ ] Format: `cargo fmt --all -- --check`
- [ ] Clippy review where relevant
- [ ] Docs/README updated if needed
- [ ] No secrets or credentials in changes
## 概要
- 目的 / 背景:

## 変更点
- 

## 影響範囲
- 機能フラグ / 互換性:

## 動作確認
- ビルド: cargo build --workspace
- テスト: cargo test -q (必要なら)

## セキュリティ / パフォーマンス
- FFI 依存の除去（純 Rust のみ）
- 圧縮: gzip, xz(圧縮/解凍), bzip2/zstd(解凍のみ)

## 注意事項
- 警告は既知。エラーなしを確認。
