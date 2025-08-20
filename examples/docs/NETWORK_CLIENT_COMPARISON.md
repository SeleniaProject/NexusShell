# HTTP クライアント選定: ureq / reqwest / hyper 比較

本比較は公式ドキュメント・既知のベンチ/実運用事例を踏まえ、NexusShell の要件（純Rust、依存縮小、バイナリサイズ、同期/非同期両適性）に沿って整理する。

## 要件整理
- 純Rust（C/C++ビルド依存なし）
- 小さなフットプリント（BusyBoxモードを阻害しない）
- シンプルなAPI（更新機構・通知・メトリクスの用途中心）
- TLS は rustls 系を優先（OS 依存を避ける）

## 概要
- ureq: 同期・軽量・依存少。`rustls` 構成で純Rust。簡素なAPIで実装/監視が容易。
- reqwest: 非同期/同期両対応、高機能（HTTP/2、redirect、cookie、proxy等）。依存とサイズは重め。
- hyper: 低レベルHTTPクライアント/サーバ。非同期のみ。上位ラッパが必要（機能構築コスト大）。

## 詳細比較
- 依存/サイズ: ureq < hyper < reqwest（機能を有効にするとさらに増）
- 実装容易性: ureq（最小）/ reqwest（中〜大）/ hyper（大）
- パフォーマンス: 高スループット用途は hyper/reqwest、単発/簡易用途は ureq で十分
- TLS: いずれも rustls 構成可能（OS依存回避）

## 結論
NexusShell のアップデータ/通知用途では `ureq` を第一選択とし、以下の方針を採用する：
- 既定は `ureq`（純Rust・小サイズ・十分な機能）
- 将来の高スループット要件発生時のみ `reqwest` を feature で opt-in
- `hyper` はサーバ/高度制御が必要な場合に限定


