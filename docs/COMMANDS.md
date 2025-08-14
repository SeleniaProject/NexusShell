# NexusShell コマンドカタログ

> **目的**: 本書は NexusShell で実装されるすべてのコマンド／ビルトインを網羅的に示し、カテゴリ別に概要・シノプシスを定義する。各コマンドのフルマニュアルは `docs/man/` 以下に個別 Markdown で提供され、本書はトップレベル目録として機能する。

---

## カテゴリ一覧
1. シェル組込 (Built-ins)
2. ファイル・ディレクトリ操作
3. テキスト処理
4. システム & プロセス管理
5. ネットワークツール
6. 圧縮・アーカイブ
7. パーミッション & 所有権
8. デバイス & ファイルシステム
9. 時刻 & スケジューリング
10. 開発者ツール
11. 雑多ユーティリティ

---

### 1. シェル組込 (Built-ins)
| Command | Synopsis | 概要 | 備考 |
|---------|----------|------|------|
| alias | `alias [NAME[=VALUE] ...]` | コマンドエイリアスを定義／表示 | 永続化は `~/.nxshrc` |
| bg | `bg [JOB]` | 停止中ジョブをバックグラウンド実行 | | 
| bind | `bind KEYSEQ:COMMAND` | キーバインド設定 | | 
| break | `break [N]` | ループを N レベル終了 | | 
| builtin | `builtin CMD [ARGS...]` | 外部同名より組込版を強制実行 | | 
| cd | `cd [DIR]` | カレントディレクトリ変更 | `cd -` 前回DIR |
| command | `command [-pVv] CMD` | PATH 検索 or 実行前確認 | | 
| complete | `complete OPTS NAME` | 補完スクリプト登録 | fish-style |
| continue | `continue [N]` | ループの残りをスキップ | | 
| declare | `declare [-aAfFgilnrtux] NAME[=VAL]` | 変数宣言 | | 
| dirs | `dirs [-clpv] [+N]` | ディレクトリスタック表示 | pushd/popd と併用 |
| disown | `disown [-a] [JOB]` | ジョブをシェル管理から除外 | shutdown 影響なし |
| echo | `echo [-neE] ARG...` | 文字列出力 | ビルトイン版 |
| eval | `eval [STRING]` | 引数を再解釈して実行 | | 
| exec | `exec [-cl] CMD` | プロセス置換 | PID 変化 |
| exit | `exit [N]` | シェル終了 | N= exit status |
| export | `export NAME[=VAL]` | 環境変数登録 | | 
| fg | `fg [JOB]` | バックグラウンドジョブを前面化 | | 
| getopts | `getopts OPTSTRING NAME [ARGS]` | 引数解析 | Bash 互換 |
| hash | `hash [-lr] [CMD]` | コマンド検索キャッシュ | | 
| help | `help [CMD]` | 組込ヘルプ | 表形式カラー |
| history | `history [-c] [N]` | コマンド履歴 | 暗号化保存 |
| jobs | `jobs [-lps]` | ジョブ一覧 | CPU%, MEM% 付加 |
| let | `let EXPR` | 算術評価 | | 
| local | `local NAME=VAL` | 関数ローカル変数 | | 
| popd | `popd [+N]` | ディレクトリスタック POP | | 
| pushd | `pushd [DIR]` | ディレクトリスタック PUSH | | 
| pwd | `pwd` | 現在ディレクトリ表示 | | 
| read | `read [-r] VAR` | 標準入力読み込み | | 
| readonly | `readonly NAME[=VAL]` | 変更不可変数 | | 
| return | `return [N]` | 関数からリターン | | 
| set | `set [OPTS]` | シェル設定変更 | `-e`, `-x` 等 |
| shift | `shift [N]` | 位置パラメータを左シフト | | 
| source | `source FILE` | スクリプト読み込み | `.` 同義 |
| suspend | `suspend` | シェルを SIGSTOP | | 
| times | `times` | 累積 CPU 時間 | | 
| trap | `trap CMD SIGNALS` | シグナルハンドラ設定 | | 
| type | `type CMD` | コマンド種別判定 | | 
| ulimit | `ulimit [-SaH] [LIMIT]` | 資源制限表示／設定 | | 
| umask | `umask [MASK]` | デフォルト権限マスク | | 
| unalias | `unalias NAME` | エイリアス削除 | | 
| unset | `unset [-fv] NAME` | 変数・関数削除 | | 
| wait | `wait [JOB]` | ジョブ終了待機 | | 

---

### 2. ファイル・ディレクトリ操作
| Command | Synopsis | 概要 | カテゴリ |
|---------|----------|------|----------|
| ls | `ls [OPTS] [PATH]...` | ファイル一覧をカラー表示 | File |
| cp | `cp [OPTS] SRC... DST` | ファイル／ディレクトリコピー | File |
| mv | `mv [OPTS] SRC... DST` | 移動・改名 | File |
| rm | `rm [OPTS] FILE...` | 削除 | File |
| mkdir | `mkdir [-p] DIR...` | ディレクトリ作成 | File |
| rmdir | `rmdir DIR...` | 空ディレクトリ削除 | File |
| ln | `ln [-sfr] SRC DST` | ハード／シンボリックリンク | File |
| stat | `stat FILE...` | ファイル詳細情報 | File |
| touch | `touch [-a] [-m] FILE...` | タイムスタンプ更新 | File |
| tree | `tree [DIR]` | ディレクトリ階層表示 | File |
| du | `du [-h] [PATH]` | ディスク使用量 | FS |
| df | `df [-h] [PATH]` | ファイルシステム使用率 | FS |
| sync | `sync` | バッファフラッシュ | FS |
| mount | `mount DEV DIR` | ファイルシステムマウント | FS |
| umount | `umount DIR` | アンマウント | FS |
| shred | `shred FILE` | 復元困難な削除 | Security |
| split | `split [-b N] FILE [PREFIX]` | ファイル分割 | File |
| cat | `cat [FILE...]` | 連結表示 | Text |
| more | `more FILE` | ページャ | Text |
| less | `less FILE` | 高機能ページャ | Text |

---

### 3. テキスト処理
| Command | Synopsis | 概要 | |
|---------|----------|------|--|
| grep | `grep [OPTS] PATTERN FILE...` | 正規表現検索 (PCRE2) | |
| egrep | `egrep PATTERN FILE` | POSIX ERE 検索 | リンク先同上 |
| fgrep | `fgrep PATTERN FILE` | 固定文字列検索 | |
| awk | `awk 'PROGRAM' FILE` | パターン処理言語 | |
| sed | `sed -e 's/REGEX/REPL/g' FILE` | ストリームエディタ（`-n/-e/-f/-i/-r/-z` 等をサポート） | |
| tr | `tr SET1 SET2` | 文字集合変換 | |
| cut | `cut -f LIST FILE` | 列抽出 | |
| paste | `paste FILE1 FILE2` | 行横結合 | |
| sort | `sort [OPTS]` | 並び替え | |
| uniq | `uniq [OPTS]` | 重複行削除 | |
| find | `find [PATH...] [EXPR]` | 階層検索（`-name`/`-type`/`-size`/`-mtime`/`-exec` 等をサポート） | |
| head | `head [-n N] FILE` | 先頭表示 | |
| tail | `tail [-f] [-n N] FILE` | 末尾表示・追跡 | |
| wc | `wc [-lwmc] FILE` | 行・単語数 | |
| fmt | `fmt FILE` | テキスト整形 | |
| fold | `fold [-w N] FILE` | 文字幅折返し | |
| join | `join FILE1 FILE2` | 共通フィールド結合 | |
| comm | `comm FILE1 FILE2` | 3 列比較 | |
| diff | `diff FILE1 FILE2` | 差分 | |
| patch | `patch < DIFF` | パッチ適用 | |
| rev | `rev FILE` | 行反転 | |

---

### 4. システム & プロセス管理
| Command | Synopsis | 概要 |
|---------|----------|------|
| ps | `ps aux` | プロセス一覧 | |
| top | `top` | 動的システムモニタ | ANSI UI |
| htop | `htop` | 強化版トップ | 内蔵 tui |
| kill | `kill [-SIG] PID` | シグナル送信 | |
| pkill | `pkill NAME` | 名前で kill | |
| pgrep | `pgrep PATTERN` | プロセス検索 | |
| nice | `nice -n N CMD` | 優先度変更 | |
| renice | `renice -n N -p PID` | 実行中優先度変更 | |
| uptime | `uptime` | 稼働時間 | |
| free | `free -h` | メモリ | |
| vmstat | `vmstat` | 仮想メモリ統計 | |
| lsof | `lsof -i` | 開放ファイル | |
| uname | `uname -a` | カーネル情報 | |
| hostname | `hostname [-s] [NAME]` | ホスト名 | |
| env | `env` | 環境変数一覧 | |
| printenv | `printenv VAR` | 単一変数 | |
| id | `id` | UID/GID | |
| groups | `groups USER` | 所属グループ | |
| who | `who` | ログインユーザ | |
| time | `time CMD` | 実行時間計測 | 外部版

---

### 5. ネットワークツール
| Command | Synopsis | 概要 |
|---------|----------|------|
| ping | `ping HOST` | ICMP 到達確認 |
| traceroute | `traceroute HOST` | 経路調査 |
| nslookup | `nslookup HOST` | DNS 解析 |
| dig | `dig HOST` | DNS 詳細 |
| curl | `curl URL` | HTTP クライアント |
| wget | `wget URL` | ファイル取得 |
| ssh | `ssh USER@HOST` | Secure Shell |
| scp | `scp SRC DST` | SSH コピー |
| netstat | `netstat -tulnp` | ソケット状態 |
| ss | `ss -lntu` | netstat 代替 |
| ip | `ip addr` | ネット設定 |
| ifconfig | `ifconfig` | (互換) |
| route | `route -n` | ルーティング |
| arp | `arp -a` | ARP テーブル |
| telnet | `telnet HOST` | デバッグ |
| ftp | `ftp HOST` | FTP クライアント |
| rsync | `rsync SRC DST` | 同期 |
| dig | `dig @SERVER NAME` | DNS |
| nc | `nc HOST PORT` | ネットキャット |
| curlftpfs | `curlftpfs URL MOUNT` | FUSE 圧縮 |

---

### 6. 圧縮・アーカイブ
| Command | Synopsis | 概要 |
|---------|----------|------|
| gzip | `gzip FILE` | 圧縮 |
| gunzip | `gunzip FILE.gz` | 解凍 |
| bzip2 | `bzip2 FILE` | 圧縮 |
| bunzip2 | `bunzip2 FILE.bz2` | 解凍 |
| xz | `xz FILE` | LZMA 圧縮 |
| unxz | `unxz FILE.xz` | 解凍 |
| zip | `zip ARCHIVE.zip FILE...` | Zip 作成 |
| unzip | `unzip ARCHIVE.zip` | 解凍 |
| tar | `tar -czf ARCHIVE.tar.gz DIR` | TAR 一括（`--zstd` は Pure Rust ストアモードで作成/展開対応。`--exclude=PATTERN`, `--strip-components=N`, `-p/--preserve-permissions`, `--no-same-permissions`, `--overwrite`, `-W/--verify`, `--owner/--group/--numeric-owner/--mtime` をサポート） |
| cpio | `cpio -o < FILES` | アーカイブ |
| ar | `ar rcs LIB.a OBJ...` | 静的ライブラリ |
| zstd | `zstd FILE` | zstd 圧縮（Pure Rust ストアモード: RAW ブロックでフレーム生成。`-T/--threads`, `-M/--memory` は互換情報用） |
| unzstd | `unzstd FILE.zst` | zstd 解凍（Pure Rust） |
| 7z | `7z a ARCHIVE.7z FILE...` | マルチフォーマット（外部 7z に委譲） |

---

### 7. パーミッション & 所有権
| Command | Synopsis | 概要 |
|---------|----------|------|
| chmod | `chmod MODE FILE` | パーミッション変更 |
| chown | `chown USER FILE` | 所有者変更 |
| chgrp | `chgrp GROUP FILE` | グループ変更 |
| umask | `umask 022` | マスク設定 |
| sudo | `sudo CMD` | 権限昇格 |
| su | `su [USER]` | ユーザ切替 |
| setfacl | `setfacl -m u:USER:r FILE` | ACL 設定 |
| getfacl | `getfacl FILE` | ACL 取得 |
| passwd | `passwd USER` | パスワード変更 |
| visudo | `visudo` | sudoers 編集 |

---

### 8. デバイス & ファイルシステム
| Command | Synopsis | 概要 |
|---------|----------|------|
| lsblk | `lsblk` | ブロックデバイス一覧 |
| blkid | `blkid` | UUID 取得 |
| fdisk | `fdisk /dev/sda` | パーティション編集 |
| mkfs | `mkfs.ext4 /dev/sda1` | FS 作成 |
| fsck | `fsck /dev/sda1` | FS チェック |
| mount | `mount DEV DIR` | マウント |
| umount | `umount DIR` | アンマウント |
| df | `df -h` | 使用率 |
| du | `du -sh DIR` | 使用量 |
| sync | `sync` | 書込フラッシュ |
| hdparm | `hdparm -Tt /dev/sda` | ディスク性能 |
| smartctl | `smartctl -a /dev/sda` | SMART 状態 |
| lsusb | `lsusb` | USB デバイス |
| lspci | `lspci` | PCI デバイス |
| dmidecode | `dmidecode` | BIOS 情報 |

---

### 9. 時刻 & スケジューリング
| Command | Synopsis | 概要 |
|---------|----------|------|
| date | `date` | 現在日時 |
| cal | `cal` | カレンダー |
| sleep | `sleep N` | 秒待機 |
| at | `at TIME` | 1回ジョブ |
| cron | `crontab -e` | 定期ジョブ |
| watch | `watch CMD` | 定期実行表示 |
| time | `time CMD` | 実行時間計測 |
| tzselect | `tzselect` | TZ 選択 |
| hwclock | `hwclock -r` | ハードウェアクロック |
| timedatectl | `timedatectl` | 時刻サービス |

---

### 10. 開発者ツール
| Command | Synopsis | 概要 |
|---------|----------|------|
| git | `git ...` | バージョン管理 |
| make | `make` | ビルド自動化 |
| gcc | `gcc SRC.c -o BIN` | C コンパイラ |
| clang | `clang SRC.c` | LLVM コンパイラ |
| cargo | `cargo build` | Rust ビルド |
| rustc | `rustc SRC.rs` | Rust コンパイラ |
| go | `go build` | Go |
| python | `python SCRIPT.py` | Python |
| node | `node SCRIPT.js` | Node.js |
| javac | `javac SRC.java` | Java |
| gdb | `gdb BIN` | デバッガ |
| strace | `strace CMD` | Syscall トレース |

---

### 11. 雑多ユーティリティ
| Command | Synopsis | 概要 |
|---------|----------|------|
| yes | `yes STRING` | 無限出力 |
| echo | `echo STRING` | 文字列表示 |
| printf | `printf FORMAT ARGS` | 書式出力 |
| seq | `seq 1 10` | 整数列出力 |
| sleep | `sleep SECONDS` | 待機 |
| uname | `uname -a` | システム名 |
| bc | `bc` | 任意精度計算 |
| expr | `expr 1 + 2` | 計算式 |

---

> **備考**: 上記コマンドは NexusShell でネイティブ実装される予定です。カテゴリは内部モジュール構成・ビルドフラグにも対応しており、`nxsh --features "minimal"` で Built-in のみの軽量構成に、`--features "full"` で全コマンドを含むバイナリを生成できます。今後追加される新コマンドは本書に随時追記されます。 