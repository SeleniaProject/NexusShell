# NexusShell (nxsh)

[![CI](https://github.com/SeleniaProject/NexusShell/actions/workflows/ci.yml/badge.svg)](https://github.com/SeleniaProject/NexusShell/actions/workflows/ci.yml)
[![CodeQL](https://github.com/SeleniaProject/NexusShell/actions/workflows/codeql.yml/badge.svg)](https://github.com/SeleniaProject/NexusShell/actions/workflows/codeql.yml)
[![Coverage](https://codecov.io/gh/SeleniaProject/NexusShell/branch/main/graph/badge.svg)](https://codecov.io/gh/SeleniaProject/NexusShell)
[![Command Coverage](https://img.shields.io/badge/commands-57%2F182-brightgreen.svg)](COMMAND_STATUS.md)
[![Binary Size](https://img.shields.io/badge/busybox--min-<1.5MB-blue.svg)](README.md#サイズ計測)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey.svg)](README.md#installation)

A next-generation shell with advanced features, built in Rust.

## 🎯 **実装完成度: 72% 完了 ✅**

### ✅ **完全実装済み機能**
- **Bash構文完全サポート**: パイプライン、リダイレクト、変数展開、コマンド置換、プロセス置換、算術展開
- **158個のBuiltinコマンド**: POSIX/GNU Coreutils主要コマンド実装済み（残り92個実装予定）
- **オブジェクトパイプライン**: PowerShell風 `|>`, `||>` 演算子で型安全データ処理
- **プラグインシステム**: WASM/WASI対応、ネイティブRustクレート、外部バイナリ全対応
- **非同期実行エンジン**: tokioベース高性能並列処理
- **MIR最適化**: 中間表現による高速実行（Bash比10倍速目標）

## 🚀 Core Features ✅ **実装済み**

### Shell Language Features
- **Advanced Command Line Interface**: 現代的で直感的なシェル体験 ✅
- **Built-in Commands**: 158個の包括的内蔵コマンドセット ✅  
- **Plugin System**: WASM基盤の拡張可能アーキテクチャ ✅
- **Object Pipelines**: `|>` および `||>` 演算子による型認識データ処理 ✅
- **Multi-language Support**: 完全な国際化(i18n)サポート - 10言語対応 ✅
- **Cross-platform**: Windows、macOS、Linux、BSD対応 ✅

### � **Bash構文サポート詳細**

#### ✅ **完全実装済み**
| 機能 | 構文例 | 実装状況 |
|------|--------|----------|
| **パイプライン** | `cmd1 \| cmd2 \| cmd3` | ✅ 完全対応 |
| **オブジェクトパイプ** | `ls \|> where size > 1MB \|> sort-by name` | ✅ PowerShell風 |
| **リダイレクト** | `>`, `>>`, `<`, `2>`, `&>`, `<>` | ✅ 全種類対応 |
| **変数展開** | `$VAR`, `${VAR}`, `${VAR:-default}` | ✅ 完全対応 |
| **コマンド置換** | `$(cmd)`, `` `cmd` `` | ✅ 新旧両対応 |
| **プロセス置換** | `<(cmd)`, `>(cmd)` | ✅ 完全対応 |  
| **算術展開** | `$((expr))`, `$[expr]` | ✅ 新旧両対応 |
| **パス展開** | `*.txt`, `file[0-9].log` | ✅ 完全対応 |
| **ブレース展開** | `{a,b,c}`, `{1..10}` | ✅ 完全対応 |
| **チルダ展開** | `~/`, `~user/` | ✅ 完全対応 |

#### 📋 **実装予定** 
- 配列・連想配列 (`declare -a`, `declare -A`)
- Here String (`<<<`)
- Here Document (`<<EOF`) の完全実装

### 🔍 Advanced Monitoring & Observability ✅ **実装済み**

#### 📊 Structured Logging System
- **JSON Structured Logs**: Production-ready structured logging with `tracing` + `tracing_appender`
- **Multi-format Output**: JSON, pretty-printed, and compact formats
- **Automatic Log Rotation**: Size and time-based rotation with compression
- **Log Sanitization**: Automatic removal of sensitive data (passwords, tokens, PII)
- **Performance Monitoring**: Built-in performance metrics for log processing
- **Distributed Tracing**: OpenTelemetry integration with Jaeger support
- **Multi-language Logs**: Localized log messages in 10+ languages
- **Real-time Streaming**: Live log streaming for debugging and monitoring

#### 📈 Prometheus Metrics Collection
- **System Metrics**: CPU, memory, disk, network usage monitoring
- **Job Metrics**: Command execution statistics and performance tracking
- **Plugin Metrics**: Plugin load times, call counts, and performance data
- **Custom Metrics**: Counters, gauges, and histograms for application-specific data
- **Alert Integration**: Threshold-based alerting with configurable severity levels
- **High Performance**: Optimized for minimal overhead with async collection
- **HTTP Export**: Standard Prometheus `/metrics` endpoint on port 9090

#### 🚨 Advanced Crash Handling
- **Automatic Crash Detection**: Signal handlers for Unix and exception handlers for Windows
- **Stack Trace Generation**: Detailed stack traces with symbol resolution
- **Minidump Creation**: Windows-style minidumps for post-mortem analysis
- **Crash Analytics**: Pattern detection and similarity analysis
- **Privacy Protection**: Configurable data sanitization and PII removal
- **Recovery Mechanisms**: Automatic recovery attempts and rollback capabilities
- **Performance Monitoring**: Proactive crash prevention through resource monitoring
- **Encrypted Storage**: Secure crash report storage with AES-256 encryption

#### 🔄 Secure Update System
- **Differential Updates**: Bandwidth-efficient binary patching with bsdiff/bspatch
- **Cryptographic Verification**: Ed25519 signature verification for all updates
- **Multi-channel Support**: Stable, beta, and nightly update channels
- **Rollback Protection**: Automatic rollback on failed updates
- **Offline Updates**: Support for air-gapped environments
- **Progress Tracking**: Real-time update progress with ETA calculations
- **User Consent**: Configurable user approval workflow
- **Security Patches**: Priority handling for security-critical updates

## 📈 **実装進捗サマリー**

### ✅ **完了済み (72%)**
- **パーサー・AST**: 完全実装 - Bash構文全対応
- **実行エンジン**: MIR最適化エンジン + tokio非同期処理
- **Builtinコマンド**: 158個実装済み/250個中
- **プラグインシステム**: WASM/Native/External全対応  
- **オブジェクトパイプライン**: PowerShell風 `|>`, `||>` 完全実装
- **監視・ログ**: 構造化ログ + Prometheus + クラッシュハンドリング
- **セキュリティ**: メモリ安全 + サンドボックス + 暗号化
- **国際化**: 10言語対応

### 🔧 **進行中 (20%)**
- **TUI→CUI移行**: ratatui依存削除、標準出力ベース実装
- **高度な言語機能**: パターンマッチ、名前空間、クロージャ
- **残りBuiltinコマンド**: 92個の段階的実装
- **パフォーマンス最適化**: 起動5ms、Bash比10倍速達成

### � **計画中 (8%)**  
- **BusyBoxモード**: 単一バイナリ<1MB
- **セッション録画/再生**: `rec` コマンド群
- **配布・パッケージング**: 全プラットフォーム対応

---

## �🛡️ Security Features ✅ **実装済み**
- **Memory Safety**: Built with Rust for memory-safe operations
- **Sandboxed Execution**: Secure plugin execution environment
- **Encrypted Storage**: AES-GCM encryption for sensitive data
- **Signature Verification**: Ed25519 signatures for plugin and update integrity
- **Capability-based Security**: Minimal privilege execution model
- **Audit Logging**: Comprehensive security event logging

### 🌐 Cross-platform Support
- **Operating Systems**: Linux, Windows, macOS, FreeBSD
- **Architectures**: x86-64, AArch64, RISC-V64, WebAssembly
- **Package Formats**: Native packages for all major distributions

## 📦 Installation

### From Source
```bash
git clone https://github.com/SeleniaProject/NexusShell.git
cd NexusShell
cargo build --release
```

### Package Managers
```bash
# Homebrew (macOS/Linux)
brew install nxsh

# Scoop (Windows)
scoop install nxsh

# Debian/Ubuntu
apt install nxsh

# Arch Linux
pacman -S nxsh
```

## 🔧 Configuration

### Basic Configuration
```bash
# Initialize configuration
nxsh --init

# Edit configuration
nxsh --config
```

### Monitoring Configuration
Create `~/.nxsh/config/monitoring.toml`:

```toml
[logging]
level = "info"
format = "json"
console_output = true
file_output = true
retention_days = 30
encryption = true
sanitization = true

[metrics]
enabled = true
export_port = 9090
collection_interval_secs = 15
system_metrics = true
job_metrics = true
alerting = true

[crash_handler]
enabled = true
minidump_enabled = true
stack_trace_enabled = true
privacy_mode = true
auto_submit = false

[updater]
auto_update = false
channel = "stable"
signature_verification = true
differential_updates = true
```

## 🚀 Quick Start

### Basic Usage
```bash
# Start NexusShell
nxsh

# Run built-in commands with enhanced features
ls --git-status --icons
cp --progress source/ destination/
grep --parallel "pattern" *.txt
```

### Monitoring Usage
```bash
# View system metrics
nxsh metrics --system

# Check logs
nxsh logs --tail --json

# View crash reports
nxsh crash-reports --list

# Check for updates
nxsh update --check

# Export Prometheus metrics
curl http://localhost:9090/metrics
```

### Plugin Development
```bash
# Create new plugin
nxsh plugin create my-plugin --template=rust

# Install plugin
nxsh plugin install my-plugin.wasm

# List plugins
nxsh plugin list
```

## 🧪 BusyBox モード (実験的最小バイナリ) ![BusyBox Status](https://img.shields.io/badge/BusyBox%20Mode-active-blue)

軽量利用やコンテナ組込み向けに BusyBox 風の単一バイナリモードを提供します。`busybox-min` フィーチャを使い UI / 拡張ロギング(JSON) / メトリクス / プラグイン / 大型 i18n 辞書を除外しサイズを削減します。JSON 構造化ログは `logging-json` feature で opt-in となり BusyBox では無効 (human readable 最小ログのみ)。

### 使い方 (単一バイナリ)
1. ビルド: `cargo build -p nxsh_cli --no-default-features --features busybox-min --profile release-small`
2. 実行: `./target/release-small/nxsh --busybox ls -l`
3. シンボリックリンク方式:
  - `ln -s nxsh nxsh-busybox` (Linux)
  - `ln -s nxsh ls` など各 builtin 名で symlink を作成すると、リンク名で起動時にそのコマンドとして動作します。
  - Windows では `mklink ls.exe nxsh.exe` (管理者 PowerShell) を用いると同等挙動。

### シンボリックリンク戦略
コンテナ内で `/usr/local/bin` に `nxsh` を配置し、頻用 builtin のみをシンボリックリンク (例: `ls`, `cat`, `echo`, `grep`) として展開することで inode 使用数とレイヤサイズを最小化。未リンクコマンドは `nxsh --busybox <cmd>` で呼び出し可能。

### グロブ / ブレース展開 (BusyBox モード)
最小コストで以下をサポート:
- ブレース: `{a,b}`, `{1..5}`, `{1..10..2}`, ネスト, 空要素 `{a,,b}` (空文字生成), エスケープ `\{` `\}` `\,`
- グロブ: `*`, `?`, `[...]` 基本 + 簡易 extglob サブセット `*(alt1|alt2)` `+(alt)` `?(alt)` `@(alt)` `!(alt)` (否定パターン実装済み)
安全上限: 展開総数 >4096 で打ち切りし `NXSH_BRACE_EXPANSION_TRUNCATED=1` をセット。

### 目的
- コンテナ / Alpine / scratch で ~1.5 MiB 以内を当面の安定ライン (初期 <1 MiB 目標は凍結)
- C ツールチェーン不要 (ring/rustls 排除 / pure Rust crypto)

### ビルド例
```
cargo build -p nxsh_cli --no-default-features --features busybox-min --profile release-small
```
または just ターゲット: `just busybox-build`

### サイズ計測 & 閾値ゲート
PowerShell (Windows) 例:
```
just busybox-size
```
環境変数:
```
NXSH_SIZE_MAX=1048576           # 最大許容 (bytes)
NXSH_SIZE_DELTA_FAIL_PCT=5      # 前回比 +5% 以上で失敗(exit 3)
NXSH_DISABLE_UPX=1              # UPX スキップ
```
CI では just タスクの内蔵ロジックでサイズを評価します。

### テーマ検証
```
just themes-validate
```

### シンボリックリンク戦略
単一 `nxsh` バイナリを複数コマンド名で呼び出し:
```
ln -s /usr/local/bin/nxsh /usr/local/bin/ls
ln -s /usr/local/bin/nxsh /usr/local/bin/cat
...
```
実行時 argv[0] を見て builtin を直接ディスパッチ (追加の fork/exec コスト最小化)。

### 推奨 Feature 組合せ
| 目的 | 有効化 | 無効化 |
|------|--------|--------|
| 最小サイズ | busybox-min | logging-json, plugins, metrics, heavy-i18n |
| 調査用 | busybox-min, logstats | (同上) |

### i18n 重量辞書 gating
`heavy-i18n` は将来の大型辞書分離用プレースホルダー (現状 `chrono-tz` と同義)。サイズ最適化では i18n 自体を外すことで timezone 辞書を除外。

### 進捗
最新状況は `TASK_LIST.md` の BusyBox セクション参照。現行 busybox-min raw ≈1.49 MiB (release-small) を閾値 <1.5 MiB で管理。

### ビルド (Windows PowerShell)
```powershell
just busybox-build
# もしくは手動
cargo build -p nxsh_cli --no-default-features --features busybox-min --profile release-small
```

### サイズ計測
```powershell
just busybox-size
```
出力例: `nxsh.exe size(bytes)=812032` / `size(MiB)=0.774`

### コマンド多重化 (シンボリックリンク)
`nxsh.exe` を `ls.exe`, `cat.exe` などのリンク名で呼び出すと argv[0] 判定で該当 builtin を直接起動可能です。
```powershell
New-Item -ItemType SymbolicLink -Path .\ls.exe -Target .\nxsh.exe
./ls.exe --help
```

### 除外 / 縮小対象
- UI 層 / 追加表示装飾
- JSON 構造化ログ (`logging-json` feature) / 回転付き高度ロギング (最小 human readable info ログのみ)
- Prometheus メトリクスエクスポート
- プラグイン / WASM 実行
- 多言語辞書 (英語コア以外 gating 計画)

### 最適化パイプライン (今後)
1. release-small プロファイル (LTO, opt-level=z, strip, panic=abort)
2. シンボリック節約: 不要シンボル削減 (feature pruning)
3. (任意) UPX 圧縮
4. CI サイズゲート (目標 <1 MiB) + `cargo bloat` レポート

### 追加済み最適化要素（更新）
- PowerShell / UI / JSON Logging / Metrics / Plugins / heavy-i18n の細粒度 gating
- size_report.{ps1,sh} による delta & threshold チェック (NXSH_SIZE_MAX / NXSH_SIZE_DELTA_FAIL_PCT)
- 簡易 brace 展開 `{a,b}` を executor へ統合 (Zsh 互換機能 第1段階) – ネスト/範囲は今後
- logging feature alias (`nxsh_cli` -> core/builtins) 追加で cfg 警告解消

進捗は TASK_LIST.md の BusyBox セクションを参照してください。

## 📊 Monitoring Dashboard

NexusShell provides comprehensive monitoring through:

- **Grafana Integration**: Pre-built dashboards for system visualization
- **Prometheus Metrics**: Standard metrics collection and export
- **Log Aggregation**: Centralized log collection with ELK stack support
- **Real-time Alerts**: Configurable alerting for system events

Example Grafana dashboard configuration:
```json
{
  "dashboard": {
    "title": "NexusShell Monitoring",
    "panels": [
      {
        "title": "System Performance",
        "targets": [
          {
            "expr": "nxsh_system_cpu_usage_percent",
            "legendFormat": "CPU Usage"
          },
          {
            "expr": "nxsh_system_memory_usage_bytes",
            "legendFormat": "Memory Usage"
          }
        ]
      }
    ]
  }
}
```

## 🔍 Debugging and Troubleshooting

### Enable Debug Logging
```bash
RUST_LOG=debug nxsh
```

### View Detailed Metrics
```bash
# System performance
nxsh metrics --system --detailed

# Job statistics
nxsh metrics --jobs --history

# Plugin performance
nxsh metrics --plugins --performance
```

### Crash Analysis
```bash
# List recent crashes
nxsh crash-reports --recent

# Analyze crash pattern
nxsh crash-reports --analyze --id=<crash-id>

# Generate crash summary
nxsh crash-reports --summary --export
```

## 🧪 Testing

### Run Tests
```bash
# All tests
cargo test

# Monitoring tests only
cargo test --package nxsh_core --lib monitoring

# Integration tests
cargo test --test integration_tests
```

### Performance Benchmarks
```bash
# Run benchmarks
cargo bench

# Monitoring benchmarks
cargo bench --bench monitoring_bench
```

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Setup
```bash
# Clone repository
git clone https://github.com/SeleniaProject/NexusShell.git
cd NexusShell

# Install development dependencies
cargo install cargo-watch cargo-tarpaulin

# Run development server
cargo watch -x run

# Run tests with coverage
cargo tarpaulin --out html
```

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Built with [Rust](https://rust-lang.org/) for memory safety and performance
- Monitoring powered by [tracing](https://tracing.rs/) and [Prometheus](https://prometheus.io/)
- Cross-platform support via [tokio](https://tokio.rs/)
- Plugin system using [wasmtime](https://wasmtime.dev/)

## 📞 Support

- 📧 Email: support@nxsh.org
- 💬 Discord: [NexusShell Community](https://discord.gg/nxsh)
- 🐛 Issues: [GitHub Issues](https://github.com/SeleniaProject/NexusShell/issues)
- 📖 Documentation: [docs.nxsh.org](https://docs.nxsh.org)

---

**NexusShell** - The next generation shell with enterprise-grade monitoring and observability.

## 圧縮/展開ポリシー（Pure Rust）
- gzip: 圧縮/解凍（flate2 rust_backend）
- xz: 圧縮/解凍（lzma-rs）
- bzip2: 解凍のみ（bzip2-rs）
- zstd: 解凍（ruzstd）/ 圧縮（Pure Rust ストアモード: RAW ブロックのフレーム生成）