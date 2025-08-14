# 圧縮/展開ポリシー（純 Rust）

- gzip: 圧縮/解凍（flate2 rust_backend）
- xz: 圧縮/解凍（lzma-rs）
- bzip2: 解凍のみ（bzip2-rs）
- zstd: 解凍（ruzstd）/ 圧縮（Pure Rust ストアモード: RAW ブロックのフレーム生成）
 - zstd: 解凍（ruzstd）/ 圧縮（Pure Rust ストアモード: RAW ブロックのフレーム生成）
   - `--threads/-T` や `--memory/-M` は情報/互換用でストアモードの圧縮動作には影響しません

注意:
- C 連携が必要な xz2/zstd/bzip2 のエンコーダは未採用
- zstd の圧縮は非圧縮（RAW ブロック）フレームを生成します（サイズは元データと同等）。将来的に純 Rust エンコーダが安定した場合に差し替え予定。
- tar の xz 圧縮は一度 .tar を作成してから lzma-rs で xz 化
 - tar の zstd 圧縮（`--zstd`）は `.tar` をストリームでラップして RAW ブロックの zstd フレーム化（純 Rust）
