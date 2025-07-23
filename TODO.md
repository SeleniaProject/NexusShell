# NexusShell TODO 一覧（Unix/Bash コマンド版）

> **目的**: Unix/Bash の伝統的なコマンドのみに焦点を当てた NexusShell 実装タスクリスト

---

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
- [✓] 17.15 join — relational join
- [✓] 17.16 comm — three-way compare
- [✓] 17.17 diff — file difference
- [✓] 17.18 patch — apply patch
- [✓] 17.19 rev — reverse lines

## 18. システム & プロセス管理コマンド
- [✓] 18.1 ps — process list
- [✓] 18.2 top — dynamic system monitor
- [✓] 18.3 htop — enhanced top
- [✓] 18.4 kill — send signal
- [✓] 18.5 pkill — kill by name
- [✓] 18.6 pgrep — process search
- [✓] 18.7 nice — set priority
- [✓] 18.8 renice — change running priority
- [✓] 18.9 uptime — system uptime
- [✓] 18.10 free — memory usage
- [✓] 18.11 vmstat — virtual memory stats
- [✓] 18.12 lsof — list open files
- [✓] 18.13 uname — kernel information
- [✓] 18.14 hostname — host name operations
- [✓] 18.15 env — environment variable list
- [✓] 18.16 printenv — print single variable
- [✓] 18.17 id — user/group ids
- [✓] 18.18 groups — group membership
- [✓] 18.19 who — logged-in users
- [✓] 18.20 time — execution time measurement

## 19. ネットワークツール（基本的なもののみ）
- [✓] 19.1 ping — ICMP reachability
- [✓] 19.2 traceroute — route tracing
- [✓] 19.3 nslookup — DNS lookup
- [✓] 19.14 arp — ARP table
- [✓] 19.15 telnet — debugging client
- [✓] 19.16 ftp — FTP client
- [✓] 19.18 nc — netcat utility

## 20. 圧縮・アーカイブコマンド（伝統的なもののみ）
- [✓] 20.1 gzip — compression
- [✓] 20.2 gunzip — decompression
- [✓] 20.3 bzip2 — compression
- [✓] 20.4 bunzip2 — decompression
- [✓] 20.9 cpio — archive tool
- [✓] 20.10 ar — static library archiver

## 21. パーミッション & 所有権
- [✓] 21.1 chmod — permission change
- [✓] 21.2 chown — ownership change
- [✓] 21.3 chgrp — group change
- [✓] 21.5 su — switch user

## 22. 基本的なファイルシステムコマンド
- [✓] 22.12 sync — buffer flush
- [✓] 22.13 mount — filesystem mount
- [✓] 22.14 umount — filesystem unmount

## 23. 時刻 & スケジューリング（基本的なもののみ）
- [✓] 23.1 date — current date/time
- [✓] 23.2 cal — calendar
- [✓] 23.3 sleep — wait seconds

## 25. 雑多ユーティリティ
- [ ] 25.1 yes — infinite output
- [ ] 25.2 printf — formatted output
- [ ] 25.3 seq — number sequence
- [ ] 25.4 bc — arbitrary precision calc
- [ ] 25.5 expr — simple expressions

---

> **備考**: Unix/Bash の伝統的なコマンドに焦点を当て、外部ツール依存や現代的な機能は除外 