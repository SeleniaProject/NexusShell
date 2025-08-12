# 圧縮/展開ポリシー（純 Rust）

- gzip: 圧縮/解凍（flate2 rust_backend）
- xz: 圧縮/解凍（lzma-rs）
- bzip2: 解凍のみ（bzip2-rs）
- zstd: 解凍のみ（ruzstd）

注意:
- C 連携が必要な xz2/zstd/bzip2 のエンコーダは未採用
- tar の xz 圧縮は一度 .tar を作成してから lzma-rs で xz 化
