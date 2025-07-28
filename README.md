# NexusShell (nxsh)

A next-generation shell with advanced features, built in Rust.

## 🚀 Features

### Core Shell Features
- **Advanced Command Line Interface**: Modern, intuitive shell experience
- **Built-in Commands**: Comprehensive set of 100+ built-in commands
- **Plugin System**: Extensible architecture with WASM-based plugins
- **Object Pipelines**: Type-aware data processing with `|>` and `||>` operators
- **Multi-language Support**: Full internationalization (i18n) support
- **Cross-platform**: Windows, macOS, Linux, and BSD support

### 🔍 Advanced Monitoring & Observability

NexusShell includes a world-class monitoring and observability system:

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

### 🛡️ Security Features
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