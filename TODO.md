# NexusShell TODO 一覧（超詳細版）

> **目的**: 本ファイルは NexusShell 完成までのあらゆる作業を粒度「ファイル / 関数」レベルまで分解した決定版タスクリストである。各項目は `[ ]` 未着手 → `[~]` 進行中 → `[✓]` 完了の 3 状態で管理する。全タスクの総数は 500 以上。

---


## 12. CI/CD & 配布詳細
- [✓] 12.1 GitHub Actions Self-hosted ARM64 Runner
- [✓] 12.2 SBOM CycloneDX 生成
- [✓] 12.3 Notary V2 Container 署名
- [✓] 12.4 Homebrew Tap 自動 PR
- [✓] 12.5 Scoop Manifest JSON 生成

## 13. ガバナンス & 運用詳細
- [✓] 13.1 ドキュメントバージョン管理 (`docs/CHANGELOG.md`)
- [✓] 13.2 インシデント対応 Runbook 作成
- [✓] 13.3 SLA モニタリング Dashboards (Grafana)
- [✓] 13.4 Secrets Rotation Policy 自動化
- [✓] 13.5 License Compatibility 内部監査 (monthly)

## 14. リリースマイルストーン詳細
- [✓] 14.1 Preview QA checklist (50 items)
- [✓] 14.2 Beta External Pilot (社内 30User)
- [✓] 14.3 Stable Rollout Plan (Phased 10%→100%)

## 15. Built-in コマンド実装
- [✓] 15.1 bg — implement `bg` builtin (resume stopped job in background)
- [✓] 15.2 bind — implement `bind` builtin (key binding configuration)
- [✓] 15.3 break — implement `break` builtin (exit loops)
- [✓] 15.4 builtin — implement `builtin` builtin (force built-in execution)
- [✓] 15.5 command — implement `command` builtin (path search & type query)
- [✓] 15.6 complete — implement `complete` builtin (completion script registration)
- [✓] 15.7 continue — implement `continue` builtin (skip to next loop iteration)
- [✓] 15.8 declare — implement `declare` builtin (variable declaration)
- [✓] 15.9 dirs — implement `dirs` builtin (directory stack display)
- [✓] 15.10 disown — implement `disown` builtin (detach job from shell)
- [✓] 15.11 echo — implement `echo` builtin with -n/-e/-E support
- [✓] 15.12 eval — implement `eval` builtin (re-evaluate argument string)
- [✓] 15.13 exec — implement `exec` builtin (process replacement)
- [✓] 15.14 exit — implement `exit` builtin (terminate shell with status)
- [✓] 15.15 fg — implement `fg` builtin (bring job to foreground)
- [✓] 15.16 getopts — implement `getopts` builtin (argument parsing helper)
- [✓] 15.17 hash — implement `hash` builtin (command cache management)
- [✓] 15.18 let — implement `let` builtin (arithmetic evaluation)
- [✓] 15.19 local — implement `local` builtin (function-scope variables)
- [✓] 15.20 popd — implement `popd` builtin (pop directory stack)
- [✓] 15.21 pushd — implement `pushd` builtin (push directory stack)
- [✓] 15.22 pwd — implement `pwd` builtin (print current directory)
- [✓] 15.23 read — implement `read` builtin (stdin→var)
- [✓] 15.24 readonly — implement `readonly` builtin (immutable variables)
- [✓] 15.25 return — implement `return` builtin (function return)
- [✓] 15.26 shift — implement `shift` builtin (positional parameter shift)
- [✓] 15.27 source — implement `source` builtin (load script)
- [✓] 15.28 suspend — implement `suspend` builtin (SIGSTOP self)
- [✓] 15.29 times — implement `times` builtin (show cumulative CPU time)
- [✓] 15.30 trap — implement `trap` builtin (signal handler setup)
- [✓] 15.31 type — implement `type` builtin (determine command kind)
- [✓] 15.32 ulimit — implement `ulimit` builtin (resource limits)
- [✓] 15.33 umask — implement `umask` builtin (default permission mask)
- [✓] 15.34 unalias — implement `unalias` builtin (remove alias)
- [✓] 15.35 unset — implement `unset` builtin (remove variable/function)
- [✓] 15.36 wait — implement `wait` builtin (wait on job)

## 16. ファイル & ディレクトリ操作コマンド
- [✓] 16.1 cp — implement file copy
- [✓] 16.2 mv — implement move/rename
- [✓] 16.3 rm — implement file deletion
- [✓] 16.4 mkdir — implement directory creation
- [✓] 16.5 rmdir — implement empty directory removal
- [✓] 16.6 ln — implement hard/symbolic link creation
- [✓] 16.7 stat — implement detailed file status
- [✓] 16.8 touch — implement timestamp update
- [✓] 16.9 tree — implement directory tree listing
- [✓] 16.10 du — implement disk usage reporting
- [✓] 16.11 df — implement filesystem usage reporting
- [✓] 16.12 sync — implement buffer flush
- [✓] 16.13 mount — implement filesystem mount
- [✓] 16.14 umount — implement filesystem unmount
- [✓] 16.15 shred — implement secure file deletion
- [✓] 16.16 split — implement file splitter
- [✓] 16.17 cat — implement file concatenation
- [✓] 16.18 more — implement basic pager
- [✓] 16.19 less — implement advanced pager

## 17. テキスト処理コマンド
- [✓] 17.1 egrep — extended regex search
- [✓] 17.2 fgrep — fixed-string grep
- [✓] 17.3 awk — pattern scanning & processing
- [✓] 17.4 sed — stream editor
- [✓] 17.5 tr — translate characters
- [✓] 17.6 cut — column extraction
- [✓] 17.7 paste — horizontal merge
- [✓] 17.8 sort — sort lines
- [✓] 17.9 uniq — remove duplicates
- [✓] 17.10 head — output first lines
- [✓] 17.11 tail — output last lines / follow
- [✓] 17.12 wc — word/line/byte count
- [✓] 17.13 fmt — text formatter
- [✓] 17.14 fold — line wrap
- [ ] 17.15 join — relational join
- [ ] 17.16 comm — three-way compare
- [ ] 17.17 diff — file difference
- [ ] 17.18 patch — apply patch
- [ ] 17.19 rev — reverse lines

## 18. システム & プロセス管理コマンド
- [ ] 18.1 ps — process list
- [ ] 18.2 top — dynamic system monitor
- [ ] 18.3 htop — enhanced top
- [ ] 18.4 kill — send signal
- [ ] 18.5 pkill — kill by name
- [ ] 18.6 pgrep — process search
- [ ] 18.7 nice — set priority
- [ ] 18.8 renice — change running priority
- [ ] 18.9 uptime — system uptime
- [ ] 18.10 free — memory usage
- [ ] 18.11 vmstat — virtual memory stats
- [ ] 18.12 lsof — list open files
- [ ] 18.13 uname — kernel information
- [ ] 18.14 hostname — host name operations
- [ ] 18.15 env — environment variable list
- [ ] 18.16 printenv — print single variable
- [ ] 18.17 id — user/group ids
- [ ] 18.18 groups — group membership
- [ ] 18.19 who — logged-in users
- [ ] 18.20 time — execution time measurement

## 19. ネットワークツール
- [ ] 19.1 ping — ICMP reachability
- [ ] 19.2 traceroute — route tracing
- [ ] 19.3 nslookup — DNS lookup
- [ ] 19.4 dig — detailed DNS query
- [ ] 19.5 curl — HTTP client
- [ ] 19.6 wget — file downloader
- [ ] 19.7 ssh — secure shell client
- [ ] 19.8 scp — secure copy
- [ ] 19.9 netstat — socket status
- [ ] 19.10 ss — socket statistics
- [ ] 19.11 ip — network configuration
- [ ] 19.12 ifconfig — legacy network config
- [ ] 19.13 route — routing table
- [ ] 19.14 arp — ARP table
- [ ] 19.15 telnet — debugging client
- [ ] 19.16 ftp — FTP client
- [ ] 19.17 rsync — file synchronizer
- [ ] 19.18 nc — netcat utility
- [ ] 19.19 curlftpfs — FTP mount via FUSE

## 20. 圧縮・アーカイブコマンド
- [ ] 20.1 gzip — compression
- [ ] 20.2 gunzip — decompression
- [ ] 20.3 bzip2 — compression
- [ ] 20.4 bunzip2 — decompression
- [ ] 20.5 xz — compression
- [ ] 20.6 unxz — decompression
- [ ] 20.7 zip — zip archive
- [ ] 20.8 unzip — unzip archive
- [ ] 20.9 cpio — archive tool
- [ ] 20.10 ar — static library archiver
- [ ] 20.11 zstd — high-speed compression
- [ ] 20.12 unzstd — decompression
- [ ] 20.13 7z — multi-format archiver

## 21. パーミッション & 所有権
- [ ] 21.1 chmod — permission change
- [ ] 21.2 chown — ownership change
- [ ] 21.3 chgrp — group change
- [ ] 21.4 sudo — privilege escalation wrapper
- [ ] 21.5 su — switch user
- [ ] 21.6 setfacl — ACL set
- [ ] 21.7 getfacl — ACL get
- [ ] 21.8 passwd — password change
- [ ] 21.9 visudo — sudoers editor

## 22. デバイス & ファイルシステム
- [ ] 22.1 lsblk — block device list
- [ ] 22.2 blkid — uuid retrieval
- [ ] 22.3 fdisk — partition editor
- [ ] 22.4 mkfs — filesystem creator
- [ ] 22.5 fsck — filesystem checker
- [ ] 22.6 hdparm — disk benchmarking
- [ ] 22.7 smartctl — SMART status
- [ ] 22.8 lsusb — USB device list
- [ ] 22.9 lspci — PCI device list
- [ ] 22.10 dmidecode — BIOS info dump

## 23. 時刻 & スケジューリング
- [ ] 23.1 date — current date/time
- [ ] 23.2 cal — calendar
- [ ] 23.3 sleep — wait seconds
- [ ] 23.4 at — one-shot scheduler
- [ ] 23.5 cron — periodic scheduler interface
- [ ] 23.6 watch — repeat execution viewer
- [ ] 23.7 tzselect — timezone selector
- [ ] 23.8 hwclock — hardware clock
- [ ] 23.9 timedatectl — time service controller

## 24. 開発者ツール連携
- [ ] 24.1 git — VCS integration
- [ ] 24.2 make — build automation
- [ ] 24.3 gcc — C compiler
- [ ] 24.4 clang — LLVM compiler
- [ ] 24.5 cargo — Rust build tool
- [ ] 24.6 rustc — Rust compiler
- [ ] 24.7 go — Go compiler
- [ ] 24.8 python — Python interpreter
- [ ] 24.9 node — Node.js runtime
- [ ] 24.10 javac — Java compiler
- [ ] 24.11 gdb — debugger
- [ ] 24.12 strace — syscall tracer

## 25. 雑多ユーティリティ
- [ ] 25.1 yes — infinite output
- [ ] 25.2 printf — formatted output
- [ ] 25.3 seq — number sequence
- [ ] 25.4 bc — arbitrary precision calc
- [ ] 25.5 expr — simple expressions

## 26. コアランタイム & エンジン
- [ ] 26.1 MIR/SSA IR generation
- [ ] 26.2 Constant folding optimization
- [ ] 26.3 Dead pipe removal
- [ ] 26.4 JIT backend with Cranelift
- [ ] 26.5 Pipe manager (byte/object/mixed)
- [ ] 26.6 Redirect handling implementation
- [ ] 26.7 Work-stealing job scheduler
- [ ] 26.8 Nice value scheduling
- [ ] 26.9 Capability sandbox (seccomp)
- [ ] 26.10 HAL process abstraction layer
- [ ] 26.11 HAL filesystem abstraction layer
- [ ] 26.12 HAL network abstraction layer
- [ ] 26.13 History encryption (Argon2id + AES-GCM)
- [ ] 26.14 Metrics subsystem implementation
- [ ] 26.15 Structured error categorization

## 27. UI/TUI コンポーネント
- [ ] 27.1 Splash screen implementation
- [ ] 27.2 Real-time status bar
- [ ] 27.3 Advanced prompt widget
- [ ] 27.4 Scrollable output pane
- [ ] 27.5 Generic table renderer
- [ ] 27.6 Toast notification overlay
- [ ] 27.7 Sidebar completion panel
- [ ] 27.8 Progress bar integration
- [ ] 27.9 Theme engine framework
- [ ] 27.10 Accessibility modes (screen reader, high contrast)

## 28. プラグインシステム & ストア
- [ ] 28.1 WASI runtime integration
- [ ] 28.2 Plugin registrar API refinement
- [ ] 28.3 Capability manifest validation
- [ ] 28.4 Signature verification (Ed25519)
- [ ] 28.5 Plugin search & install commands

## 29. 国際化 / ローカライズ
- [ ] 29.1 gettext catalog extraction & tooling
- [ ] 29.2 Locale-aware formatting utilities
- [ ] 29.3 Multilingual command aliases framework

## 30. セキュリティ & コンプライアンス
- [ ] 30.1 Automated CVE monitoring
- [ ] 30.2 Cargo audit enforcement in CI
- [ ] 30.3 Reproducible build flags & checks
- [ ] 30.4 Secrets management integration
- [ ] 30.5 Policy DSL enforcement engine

## 31. オブザーバビリティ & メトリクス
- [ ] 31.1 Structured logging via tracing
- [ ] 31.2 Prometheus metrics exporter
- [ ] 31.3 Crash dump generation & storage
- [ ] 31.4 Telemetry opt-in workflow

## 32. テスト & ファジング
- [ ] 32.1 Unit test coverage ≥ 95%
- [ ] 32.2 Comprehensive integration test suite
- [ ] 32.3 Property testing with proptest
- [ ] 32.4 Fuzzing harness with cargo-fuzz
- [ ] 32.5 Performance benchmark scripts

---

> **備考**: 本 TODO は随時 Pull Request で更新し、番号は変更不可 (参照用) とする。 