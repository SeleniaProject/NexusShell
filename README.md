# NexusShell

[![CI Linux](https://github.com/SeleniaProject/NexusShell/actions/workflows/linux.yml/badge.svg)](https://github.com/SeleniaProject/NexusShell/actions/workflows/linux.yml)
[![CI Windows](https://github.com/SeleniaProject/NexusShell/actions/workflows/windows.yml/badge.svg)](https://github.com/SeleniaProject/NexusShell/actions/workflows/windows.yml)
[![CI macOS](https://github.com/SeleniaProject/NexusShell/actions/workflows/macos.yml/badge.svg)](https://github.com/SeleniaProject/NexusShell/actions/workflows/macos.yml)
[![codecov](https://codecov.io/gh/SeleniaProject/NexusShell/branch/master/graph/badge.svg)](https://codecov.io/gh/SeleniaProject/NexusShell)
[![Crates.io](https://img.shields.io/crates/v/nexusshell)](https://crates.io/crates/nexusshell)

---

## 概要 (Japanese)

NexusShell は **高速・高機能・クロスプラットフォーム** を目指して開発されている次世代シェルです。豊富なビルトインコマンドとプラグインシステム、tui ベースの UI を備え、従来のテキストストリームに加えてオブジェクトパイプラインをサポートします。

## Overview (English)

NexusShell is a **high-performance, cross-platform next-generation shell** featuring a rich set of built-ins, a plugin ecosystem, and a modern TUI interface. It supports both traditional text streams and advanced object pipelines.

---

## Quick Start

```bash
# Clone repository
$ git clone https://github.com/SeleniaProject/NexusShell.git
$ cd NexusShell

# Build all workspace crates in release mode
$ just build

# Run interactive REPL (defaults to nxsh_cli once implemented)
$ cargo run --workspace
```

---

## Screenshot

![NexusShell Demo](docs/assets/demo.gif) 