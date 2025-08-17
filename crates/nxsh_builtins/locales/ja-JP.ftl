# NexusShell ビルトインコマンド - 日本語ローカライゼーション

# 共通メッセージ
error-file-not-found = ファイルが見つかりません: {$filename}
error-permission-denied = アクセスが拒否されました: {$filename}
error-invalid-option = 無効なオプション: {$option}
error-missing-argument = オプションの引数が不足しています: {$option}
error-invalid-argument = 無効な引数: {$argument}
error-directory-not-found = ディレクトリが見つかりません: {$dirname}
error-not-a-directory = ディレクトリではありません: {$path}
error-not-a-file = ファイルではありません: {$path}
error-operation-failed = 操作が失敗しました: {$operation}
error-io-error = I/Oエラー: {$message}

# cat コマンド
cat-help-usage = 使用法: cat [オプション]... [ファイル]...
cat-help-description = ファイルを標準出力に連結して表示します。
cat-help-no-file = ファイルが指定されないか、ファイルが - の場合、標準入力から読み取ります。
cat-help-option-show-all = -vET と同等
cat-help-option-number-nonblank = 空でない出力行に行番号を付ける（-n を上書き）
cat-help-option-show-ends = 各行の末尾に $ を表示
cat-help-option-number = すべての出力行に行番号を付ける
cat-help-option-squeeze-blank = 連続する空行を抑制
cat-help-option-show-tabs = TAB文字を ^I として表示
cat-help-option-show-nonprinting = LFDとTAB以外に ^ と M- 記法を使用
cat-help-examples = 例:
cat-help-example1 = cat f - g  f の内容、標準入力、g の内容の順で出力
cat-help-example2 = cat        標準入力を標準出力にコピー
cat-version = cat (NexusShell) 1.0.0

# cat 統計と拡張ヘルプ
cat-stats-header = === 統計: { $filename } ===
cat-stats-total-header = === 合計の統計 ===
cat-stats-bytes-read = 読み取りバイト数
cat-stats-lines-processed = 処理した行数
cat-stats-processing-time = 処理時間
cat-stats-encoding-detected = 検出したエンコーディング
cat-stats-file-type = ファイル種別
cat-stats-compression = 圧縮
cat-stats-throughput = スループット
cat-binary-skipped = cat: { $filename }: バイナリファイルのためスキップ
cat-error-file = cat: { $filename }: { $error }
cat-warn-bzip2-missing = 警告: bzip2 解凍は利用できないため、そのまま読み込みます
cat-warn-xz-missing = 警告: XZ 解凍は利用できないため、そのまま読み込みます
cat-warn-zstd-missing = 警告: zstd 解凍は利用できないため、そのまま読み込みます
cat-help-advanced-title = 追加オプション:
cat-help-advanced-options =       --progress           大きなファイルで進捗バーを表示\n      --parallel           複数ファイルを並列処理\n      --threads N          並列処理のスレッド数\n      --encoding ENC       特定のエンコーディングを強制 (utf-8, utf-16le など)\n      --binary             すべてのファイルをバイナリとして扱う\n      --text               すべてのファイルをテキストとして扱う\n      --skip-binary        バイナリファイルをスキップ\n      --format FMT         出力形式 (raw, hex, base64, json)\n      --color WHEN         出力の色付け (always, never, auto)\n      --statistics         処理統計を表示\n      --buffer-size N      I/O のバッファサイズ\n      --no-mmap            大きなファイルのメモリマップを無効化\n      --no-decompress      自動展開を無効化\n      --no-follow-symlinks シンボリックリンクを辿らない\n      --timeout N          ネットワークのタイムアウト秒数\n      --help               このヘルプを表示して終了\n      --version            バージョン情報を表示して終了
cat-help-advanced-examples-title = 応用例:
cat-help-advanced-example1 =   cat --parallel --progress *.log    ログを進捗表示付きで並列処理
cat-help-advanced-example2 =   cat --format hex data.bin          バイナリを16進で出力
cat-help-advanced-example3 =   cat --statistics --encoding utf-16le file.txt  特定エンコーディングで統計を表示
cat-help-report-bugs = cat のバグ報告先: <bug-reports@nexusshell.org>

# cat 進捗と URL/HTTP エラー
cat-progress-complete = 完了
cat-error-invalid-file-url = 無効な file URL です
cat-error-invalid-base64 = data URL の base64 が不正です: {$error}
cat-error-malformed-data-url = 不正な data URL です
cat-error-unsupported-url-scheme = 未対応の URL スキーム: {$scheme}
cat-error-http-request-failed = HTTP リクエストに失敗しました: {$error}
cat-error-http-feature-missing = URL サポートには 'net-http' 機能が必要です

# cat 短いヘルプ説明
cat-help-option-e-short-desc = -vE と同等
cat-help-option-t-short-desc = -vT と同等
cat-help-option-u-ignored = （無視されます）

# ls コマンド
ls-help-usage = 使用法: ls [オプション]... [ファイル]...
ls-help-description = ファイルの情報を一覧表示します（デフォルトは現在のディレクトリ）。
ls-help-option-all = . で始まるエントリを無視しない
ls-help-option-almost-all = . と .. を除いて一覧表示
ls-help-option-long = 詳細形式で一覧表示
ls-help-option-human-readable = -l と組み合わせて、人間が読みやすいサイズで表示
ls-help-option-reverse = ソート順を逆にする
ls-help-option-recursive = サブディレクトリを再帰的に一覧表示
ls-help-option-directory = ディレクトリの内容ではなく、ディレクトリ自体を一覧表示
ls-help-option-one-per-line = 1行に1ファイルずつ表示
ls-help-option-color = 出力を色付け; WHEN は 'always'、'auto'、'never' のいずれか
ls-help-option-classify = エントリに識別子（*/=>@| のいずれか）を追加
ls-help-option-inode = 各ファイルのインデックス番号を表示
ls-help-option-size = 各ファイルの割り当てサイズをブロック単位で表示
ls-permission-read = 読み取り
ls-permission-write = 書き込み
ls-permission-execute = 実行
ls-type-directory = ディレクトリ
ls-type-file = 通常ファイル
ls-type-symlink = シンボリックリンク
ls-type-block = ブロックデバイス
ls-type-char = キャラクターデバイス
ls-type-fifo = FIFO
ls-type-socket = ソケット

# grep コマンド
grep-help-usage = 使用法: grep [オプション]... パターン [ファイル]...
grep-help-description = 各ファイルでパターンを検索します。
grep-help-option-extended-regexp = パターンを拡張正規表現として解釈
grep-help-option-fixed-strings = パターンを改行で区切られた固定文字列の集合として解釈
grep-help-option-ignore-case = 大文字小文字を区別しない
grep-help-option-invert-match = マッチしない行を選択
grep-help-option-word-regexp = パターンを単語全体のみにマッチさせる
grep-help-option-line-regexp = パターンを行全体のみにマッチさせる
grep-help-option-count = ファイルごとのマッチした行数のみを表示
grep-help-option-files-with-matches = マッチを含むファイル名のみを表示
grep-help-option-line-number = 出力行と一緒に行番号を表示
grep-help-option-no-filename = 出力でファイル名プレフィックスを抑制
grep-help-option-with-filename = 各マッチについてファイル名を表示
grep-help-option-quiet = すべての通常出力を抑制
grep-help-option-recursive = ディレクトリを再帰的に検索
grep-help-option-include = GLOB にマッチするファイルのみを検索
grep-help-option-exclude = GLOB にマッチするファイルとディレクトリをスキップ
grep-matches-found = {$count} 件のマッチが見つかりました
grep-no-matches = マッチが見つかりません
grep-binary-file-matches = バイナリファイル {$filename} がマッチしました

# ps コマンド
ps-help-usage = 使用法: ps [オプション]...
ps-help-description = 実行中のプロセスの情報を表示します。
ps-help-option-all = すべてのユーザーのプロセスを表示
ps-help-option-full = 完全形式で一覧表示
ps-help-option-long = 長い形式
ps-help-option-user = 指定されたユーザーのプロセスを表示
ps-help-option-pid = 指定されたPIDのプロセスを表示
ps-help-option-command = 指定されたコマンド名のプロセスを表示
ps-help-option-forest = プロセスツリーを表示
ps-help-option-sort = 指定されたフィールドでソート
ps-header-pid = PID
ps-header-ppid = PPID
ps-header-user = ユーザー
ps-header-cpu = CPU%
ps-header-mem = MEM%
ps-header-vsz = VSZ
ps-header-rss = RSS
ps-header-tty = TTY
ps-header-stat = STAT
ps-header-start = 開始
ps-header-time = 時間
ps-header-command = コマンド

# ping コマンド
ping-help-usage = 使用法: ping [オプション]... ホスト
ping-help-description = ネットワークホストにICMP ECHO_REQUESTを送信します。
ping-help-option-count = COUNT個のパケットを送信後に停止
ping-help-option-interval = 各パケット送信間隔を秒単位で指定
ping-help-option-size = 送信するデータバイト数としてSIZEを使用
ping-help-option-ttl = 生存時間を定義
ping-help-option-timeout = pingが終了するまでのタイムアウトを秒単位で指定
ping-help-option-flood = フラッドping
ping-help-option-quiet = 静かな出力
ping-help-option-verbose = 詳細出力
ping-help-option-ipv4 = IPv4のみを使用
ping-help-option-ipv6 = IPv6のみを使用
ping-help-option-numeric = ホストアドレスのシンボル名の検索を試行しない
ping-statistics = --- {$host} ping統計 ---
ping-packets-transmitted = {$transmitted} パケット送信
ping-packets-received = {$received} 受信
ping-packet-loss = {$loss}% パケットロス
ping-time-total = 時間 {$time}ms
ping-rtt-stats = rtt 最小/平均/最大/偏差 = {$min}/{$avg}/{$max}/{$mdev} ms
ping-reply-from = {$host} ({$ip}) から {$bytes} バイト: icmp_seq={$seq} ttl={$ttl} time={$time} ms
ping-destination-unreachable = 宛先ホスト到達不可
ping-request-timeout = icmp_seq {$seq} のリクエストタイムアウト

# rm コマンド
rm-help-usage = 使用法: rm [オプション]... [ファイル]...
rm-help-description = ファイルを削除（リンク解除）します。
rm-help-option-force = 存在しないファイルと引数を無視、プロンプトを表示しない
rm-help-option-interactive = 削除前に毎回プロンプトを表示
rm-help-option-recursive = ディレクトリとその内容を再帰的に削除
rm-help-option-verbose = 実行内容を説明
rm-confirm-delete = {$filename} を削除しますか？ (y/n): 
rm-removing = {$filename} を削除中
rm-removed = '{$filename}' を削除しました
rm-cannot-remove = '{$filename}' を削除できません: {$reason}

# mkdir コマンド
mkdir-help-usage = 使用法: mkdir [オプション]... ディレクトリ...
mkdir-help-description = 存在しない場合、ディレクトリを作成します。
mkdir-help-option-parents = 既存の場合はエラーなし、必要に応じて親ディレクトリを作成
mkdir-help-option-verbose = 作成された各ディレクトリのメッセージを表示
mkdir-help-option-mode = ファイルモードを設定（chmodのように）、a=rwx - umask ではない
mkdir-created = ディレクトリ '{$dirname}' を作成しました
mkdir-cannot-create = ディレクトリ '{$dirname}' を作成できません: {$reason}

# mv コマンド
mv-help-usage = 使用法: mv [オプション]... ソース... ディレクトリ
mv-help-description = ソースを宛先に名前変更、またはソースをディレクトリに移動します。
mv-help-option-force = 上書き前にプロンプトを表示しない
mv-help-option-interactive = 上書き前にプロンプトを表示
mv-help-option-no-clobber = 既存のファイルを上書きしない
mv-help-option-verbose = 実行内容を説明
mv-moving = '{$source}' -> '{$dest}'
mv-cannot-move = '{$source}' を '{$dest}' に移動できません: {$reason}
mv-overwrite-confirm = '{$dest}' を上書きしますか？ (y/n): 

# cp コマンド
cp-help-usage = 使用法: cp [オプション]... ソース 宛先
cp-help-description = ソースを宛先にコピー、または複数のソースをディレクトリにコピーします。
cp-help-option-recursive = ディレクトリを再帰的にコピー
cp-help-option-force = 既存の宛先ファイルを開けない場合、削除して再試行
cp-help-option-interactive = 上書き前にプロンプトを表示
cp-help-option-preserve = 指定された属性を保持
cp-help-option-verbose = 実行内容を説明
cp-copying = '{$source}' -> '{$dest}'
cp-cannot-copy = '{$source}' を '{$dest}' にコピーできません: {$reason}
cp-overwrite-confirm = '{$dest}' を上書きしますか？ (y/n): 

# ln コマンド
ln-help-usage = 使用法: ln [オプション]... ターゲット リンク名
ln-help-description = ファイル間のリンクを作成します。
ln-help-option-symbolic = ハードリンクの代わりにシンボリックリンクを作成
ln-help-option-force = 既存の宛先ファイルを削除
ln-help-option-interactive = 宛先を削除するかプロンプトを表示
ln-help-option-verbose = リンクされた各ファイルの名前を表示
ln-creating = リンク '{$link}' -> '{$target}' を作成中
ln-cannot-create = リンク '{$link}' を作成できません: {$reason}

# touch コマンド
touch-help-usage = 使用法: touch [オプション]... ファイル...
touch-help-description = 各ファイルのアクセス時刻と変更時刻を現在時刻に更新します。
touch-help-option-access = アクセス時刻のみを変更
touch-help-option-modify = 変更時刻のみを変更
touch-help-option-no-create = ファイルを作成しない
touch-help-option-reference = 現在時刻の代わりにこのファイルの時刻を使用
touch-help-option-time = 指定された時刻を変更
touch-cannot-touch = '{$filename}' をtouchできません: {$reason}

# stat コマンド
stat-help-usage = 使用法: stat [オプション]... ファイル...
stat-help-description = ファイルまたはファイルシステムの状態を表示します。
stat-help-option-format = デフォルトの代わりに指定されたフォーマットを使用
stat-help-option-filesystem = ファイル状態の代わりにファイルシステム状態を表示
stat-help-option-terse = 簡潔な形式で情報を表示
stat-file-info = ファイル: {$filename}
stat-size = サイズ: {$size}
stat-blocks = ブロック: {$blocks}
stat-device = デバイス: {$device}
stat-inode = Inode: {$inode}
stat-links = リンク: {$links}
stat-access-perms = アクセス: ({$octal}/{$symbolic})
stat-uid-gid = Uid: ({$uid}/{$user})   Gid: ({$gid}/{$group})
stat-access-time = アクセス: {$atime}
stat-modify-time = 変更: {$mtime}
stat-change-time = 変更: {$ctime}
stat-birth-time = 作成: {$btime}

# 共通ファイル操作
file-exists = ファイルが存在します: {$filename}
file-not-exists = ファイルが存在しません: {$filename}
directory-exists = ディレクトリが存在します: {$dirname}
directory-not-exists = ディレクトリが存在しません: {$dirname}
operation-cancelled = 操作がキャンセルされました
operation-completed = 操作が正常に完了しました
bytes-processed = {$bytes} バイト処理済み
files-processed = {$count} ファイル処理済み
progress-complete = 進行状況: {$percent}% 完了

# エラーメッセージ
error-out-of-memory = メモリ不足
error-disk-full = デバイスに空き容量がありません
error-read-only = 読み取り専用ファイルシステム
error-file-too-large = ファイルが大きすぎます
error-network-unreachable = ネットワークに到達できません
error-connection-refused = 接続が拒否されました
error-timeout = 操作がタイムアウトしました
error-interrupted = 操作が中断されました
error-broken-pipe = パイプが破損しています
error-invalid-utf8 = 無効なUTF-8シーケンス 

# schedule コマンド
schedule-help-title = schedule: 簡易タスクスケジューラ
schedule-help-usage = 使い方: schedule [オプション] 時刻/式 コマンド
schedule-help-options-title = オプション:
schedule-help-option-list =   -l, --list     予約済みタスクを一覧表示
schedule-help-option-delete =   -d, --delete   タスクを削除
schedule-help-option-help =   -h, --help     このヘルプを表示
schedule-help-examples-title = 例:
schedule-help-example-1 =   schedule 15:30 'echo Hello'
schedule-help-example-2 =   schedule tomorrow 'backup.sh'
schedule-help-example-3 =   schedule '2000-01-01 09:00' 'echo Happy New Year'
schedule-no-tasks = 予約済みタスクはありません
schedule-delete-missing-id = schedule: 削除対象のタスクIDが不足しています
schedule-deleted = 削除しました
schedule-job-not-found = schedule: タスクが見つかりません
schedule-stats-total = 総タスク数:
schedule-stats-running = 実行中:
schedule-stats-queued = 待機中:
schedule-stats-success-rate = 成功率:
schedule-stats-avg-exec-ms = 平均実行時間 (ms):
schedule-help-option-list-extended =   -l, --list       予約済みタスクを一覧表示
schedule-help-option-delete-extended =   -d, --delete ID  タスクを削除
schedule-help-option-stats =       --stats      スケジューラ統計を表示
schedule-help-option-enable =       --enable ID  無効なジョブを有効化
schedule-help-option-disable =       --disable ID ジョブを無効化
schedule-help-option-interval =       --interval 秒 コマンド  間隔実行ジョブを登録
schedule-help-option-at =       --at EPOCH秒 コマンド  1回限りジョブを登録
schedule-help-option-help =   -h, --help     このヘルプを表示
schedule-missing-command = schedule: コマンドが不足しています
schedule-usage-time-cmd = 使い方: schedule 時刻/式 コマンド
schedule-scheduled-as = schedule: 登録しました ID:
# at コマンド
at.help.title = at: 一回限りのジョブスケジューラ
at.help.usage = 使い方:
at.help.time_formats = 受け付ける時刻の形式:
at.help.options = オプション:
at.help.examples = 例:
at.help.inline-usage = at: 使い方: at [オプション] 時刻 [コマンド...]
at.help.usage-line =     at [オプション] 時刻 [コマンド...]
at.help.time_formats.details =     HH:MM [AM/PM] [日付]    - 特定の時刻（例: '14:30', '明日の 2:30 PM'）\n    HHMM [AM/PM] [日付]     - 数値形式（例: '1430', '230 PM'）\n    noon/midnight [日付]    - 名前付き時刻\n    now + N 単位            - 相対時刻（例: 'now + 2 hours'）\n    in N 単位               - もう一つの相対指定（例: 'in 30 minutes'）\n    tomorrow at 時刻        - 翌日にスケジュール\n    next 曜日 [at 時刻]     - 次の該当曜日\n    ISO-8601 形式           - 完全なタイムスタンプ\n    @timestamp              - Unix タイムスタンプ
at.help.options.list =     -h, --help              このヘルプを表示\n    -l, --list              予約済みジョブを一覧表示\n    -r, --remove ID         指定IDのジョブを削除\n    -q, --queue QUEUE       ジョブキューを指定（既定: 'a'）\n    -m, --mail              完了時にメール送信\n    -M, --no-mail           メールを送信しない\n    -f, --file FILE         ファイルからコマンドを読み込み\n    -t, --time TIME         時刻指定\n    --priority LEVEL        優先度を設定（low, normal, high, critical）\n    --output FILE           stdout をファイルにリダイレクト\n    --error FILE            stderr をファイルにリダイレクト\n    --max-runtime SECS      最大実行時間（秒）\n    --retry COUNT           失敗時のリトライ回数\n    --tag TAG               ジョブにタグを付与
at.help.examples.list =     at 14:30 tomorrow       # 明日の 14:30 にスケジュール\n    at 'now + 1 hour'       # 1時間後にスケジュール\n    at 'next friday at 9am' # 次の金曜 9:00 にスケジュール\n    at --queue b --priority high 16:00 # キュー b で高優先度ジョブ\n    echo 'backup.sh' | at midnight # 深夜にバックアップを実行\n    at -l -q a               # キュー 'a' のジョブを一覧表示\n    at -r at_123             # ID 'at_123' のジョブを削除
at.error.unable-parse-time = 時刻指定を解析できません: { $input }
at.error.invalid-time = 無効な時刻: { $hour }:{ $minute }
at.error.invalid-date-time-combo = 無効な日付/時刻の組み合わせ
at.error.ambiguous-local-time = あいまいなローカル時刻です
at.error.invalid-numeric-time = 無効な数値時刻形式です
at.error.unknown-named-time = 不明な名前付き時刻: { $name }
at.error.unknown-time-unit = 不明な時間単位: { $unit }
at.error.unknown-day = 不明な日: { $day }
at.error.unknown-weekday = 不明な曜日: { $weekday }
at.error.parse-iso = ISO 形式の解析に失敗しました
at.error.invalid-unix-timestamp = 無効な Unix タイムスタンプ: { $timestamp }
at.error.unable-parse-date = 日付を解析できません: { $date }
at.error.in-future = 予定時刻は未来である必要があります
at.error.job-not-found = ジョブが見つかりません: { $id }
at.error.user-not-allowed = ユーザー { $user } は at の使用を許可されていません
at.error.user-denied = ユーザー { $user } は at へのアクセスを拒否されています
at.error.missing-id-for-remove = -r にはジョブ ID が必要です
at.error.missing-queue-name = -q にはキュー名が必要です
at.error.read-file = ファイルの読み取りに失敗しました: { $filename }
at.error.missing-filename = -f にはファイル名が必要です
at.error.missing-time-spec = -t には時刻指定が必要です
at.error.invalid-priority = 無効な優先度: { $value }
at.error.missing-priority = --priority には優先度を指定してください
at.error.missing-output-filename = --output にはファイル名を指定してください
at.error.missing-error-filename = --error にはファイル名を指定してください
at.error.invalid-max-runtime = 無効な最大実行時間です
at.error.missing-max-runtime = --max-runtime には秒数を指定してください
at.error.invalid-retry-count = 無効なリトライ回数です
at.error.missing-retry-count = --retry には回数を指定してください
at.error.missing-tag-name = --tag にはタグ名を指定してください
at.error.unknown-option = 不明なオプション: { $option }
at.list.no-jobs = 予定されたジョブはありません
at.list.header.job-id = ジョブ ID
at.list.header.scheduled-time = 予定時刻
at.list.header.status = 状態
at.list.header.queue = キュー
at.list.header.command = コマンド
at.remove.removed = ジョブ { $id } を削除しました
at.remove.failed = ジョブ { $id } の削除に失敗しました: { $error }
at.error.time-spec-required = 時刻指定が必要です
at.error.read-stdin = 標準入力の読み取りに失敗しました: { $error }
at.error.no-command = コマンドが指定されていません
at.schedule.scheduled = job { $id } at { $time }
schedule-delegating-at = schedule: 絶対時刻は外部 'at' があれば委譲します

# cron デーモン
cron-daemon-started = Cron デーモンを開始しました
cron-daemon-stopped = Cron デーモンを停止しました
cron.log.cancelled_running_job = 実行中のジョブをキャンセルしました: { $job_id }
cron.log.added_job = ジョブを追加しました: { $job_id } ({ $name })
cron.log.removed_job = ジョブを削除しました: { $job_id } ({ $name })
cron.log.modified_job = ジョブを更新しました: { $job_id } ({ $name })
cron.log.enabled_job = ジョブを有効化しました: { $job_id } ({ $name })
cron.log.disabled_job = ジョブを無効化しました: { $job_id } ({ $name })
cron.log.manual_executed = 手動実行を開始しました: { $job_id } ({ $name })

# timedatectl コマンド
timedatectl.help.title = timedatectl: 時刻と日付の管理
timedatectl.help.usage = 使い方:
timedatectl.help.commands = コマンド一覧:
timedatectl.help.options = オプション:
timedatectl.help.time_formats = 受け付ける時刻フォーマット:
timedatectl.help.examples = 例:
timedatectl.help.timesync_options = timesync-status のオプション:
timedatectl.help.timesync_json_option =   -J, --json            ステータスと要約をJSONで出力
timedatectl.help.global_json_option =   グローバル: 一部のコマンドは -J/--json でJSON出力に対応

# timedatectl status/common/time-sync labels
timedatectl.common.yes = はい
timedatectl.common.no = いいえ
timedatectl.common.enabled = 有効
timedatectl.common.disabled = 無効
timedatectl.common.reachable = 到達可能
timedatectl.common.unreachable = 到達不可

timedatectl.msg.time_set_to = 時刻を設定しました:
timedatectl.msg.timezone_set_to = タイムゾーンを設定しました:
timedatectl.msg.rtc_in_local_tz = RTCのローカルタイムゾーン:
timedatectl.msg.ntp_sync = NTP同期:
timedatectl.msg.added_ntp_server = NTPサーバーを追加:
timedatectl.msg.removed_ntp_server = NTPサーバーを削除:

timedatectl.timesync.title = 時刻同期ステータス:
timedatectl.timesync.enabled = 有効:
timedatectl.timesync.synchronized = 同期済み:
timedatectl.timesync.last_sync = 最終同期:
timedatectl.timesync.sync_accuracy = 同期精度:
timedatectl.timesync.drift_rate = ドリフト率:
timedatectl.timesync.poll_interval = ポーリング間隔:
timedatectl.timesync.leap_status = うるう秒ステータス:
timedatectl.timesync.ntp_servers = NTPサーバー:
timedatectl.timesync.stratum = ストラタム:
timedatectl.timesync.delay = 遅延:
timedatectl.timesync.offset = オフセット:
timedatectl.timesync.summary = 要約:
timedatectl.timesync.servers_total_reachable = サーバー (総数/到達可能):
timedatectl.timesync.best_stratum = 最良ストラタム:
timedatectl.timesync.preferred_server = 優先サーバー:
timedatectl.timesync.avg_delay = 平均遅延:
timedatectl.timesync.min_delay = 最小遅延:
timedatectl.timesync.max_delay = 最大遅延:
timedatectl.timesync.avg_offset = 平均オフセット:
timedatectl.timesync.min_offset = 最小オフセット:
timedatectl.timesync.max_offset = 最大オフセット:
timedatectl.timesync.avg_jitter = 平均ジッタ:

# timedatectl ステータス表示ラベル
timedatectl.status.local_time = ローカル時刻
timedatectl.status.universal_time = 協定世界時 (UTC)
timedatectl.status.rtc_time = RTC 時刻
timedatectl.status.time_zone = タイムゾーン
timedatectl.status.system_clock_synchronized = システム時計の同期
timedatectl.status.ntp_service = NTP サービス
timedatectl.status.rtc_in_local_tz = RTC のローカルTZ
timedatectl.status.sync_accuracy = 同期精度
timedatectl.status.drift_rate = ドリフト率
timedatectl.status.last_sync = 最終同期
timedatectl.status.leap_second = うるう秒
timedatectl.status.pending = 保留中

# timedatectl ヘルプ各コマンド説明
timedatectl.help.cmd.status = 現在の時刻ステータスを表示
timedatectl.help.cmd.show = ステータスをJSONで表示
timedatectl.help.cmd.set_time = システム時刻を設定
timedatectl.help.cmd.set_timezone = システムのタイムゾーンを設定
timedatectl.help.cmd.list_timezones = 利用可能なタイムゾーンを一覧表示
timedatectl.help.cmd.set_local_rtc = RTC をローカル時刻に設定 (true/false)
timedatectl.help.cmd.set_ntp = NTP 同期を有効/無効にする (true/false)
timedatectl.help.cmd.timesync_status = 時刻同期ステータスを表示
timedatectl.help.cmd.show_timesync = 時刻同期ステータスをJSONで表示
timedatectl.help.cmd.add_ntp_server = NTP サーバーを追加
timedatectl.help.cmd.remove_ntp_server = NTP サーバーを削除
timedatectl.help.cmd.statistics = 時刻関連統計を表示
timedatectl.help.cmd.history = 時刻調整履歴を表示

# timedatectl ヘルプオプション項目
timedatectl.help.opt.help = このヘルプを表示して終了
timedatectl.help.opt.monitor = リアルタイム監視モードを実行
timedatectl.help.opt.all = すべてのプロパティを表示
timedatectl.help.opt.json = JSON 形式で出力

# 受け付ける時刻フォーマットの説明
timedatectl.help.fmt.full_datetime = 日付と時刻（秒を含む）
timedatectl.help.fmt.datetime_no_sec = 日付と時刻（秒なし）
timedatectl.help.fmt.time_only = 時刻のみ
timedatectl.help.fmt.time_no_sec = 時刻（秒なし）
timedatectl.help.fmt.unix_timestamp = Unix タイムスタンプ（秒）
timedatectl.help.fmt.iso8601 = ISO 8601（UTC）

# ヘルプの例の説明
timedatectl.help.ex.status = ステータスを表示
timedatectl.help.ex.set_time = システム時刻を設定
timedatectl.help.ex.set_timezone = タイムゾーンを設定
timedatectl.help.ex.find_timezone = タイムゾーンを検索
timedatectl.help.ex.enable_ntp = NTP 同期を有効化
timedatectl.help.ex.add_server = NTP サーバーを追加
timedatectl.help.ex.sync_status = 同期ステータスを表示
timedatectl.help.ex.statistics = 統計を表示

# プロパティ表示
timedatectl.properties.title = 時刻と日付のプロパティ
timedatectl.properties.time_info = 時刻情報
timedatectl.properties.local_time = ローカル時刻
timedatectl.properties.utc_time = UTC 時刻
timedatectl.properties.timezone_info = タイムゾーン情報
timedatectl.properties.timezone = タイムゾーン
timedatectl.properties.utc_offset = UTC オフセット
timedatectl.properties.dst_active = 夏時間 (DST)
timedatectl.properties.sync_status = 同期ステータス
timedatectl.properties.system_synced = システム時計の同期
timedatectl.properties.ntp_service = NTP サービス
timedatectl.properties.time_source = 時刻ソース
timedatectl.properties.sync_accuracy = 同期精度
timedatectl.properties.last_sync = 最終同期
timedatectl.properties.drift_rate = ドリフト率 (ppm)
timedatectl.properties.leap_info = うるう秒の情報
timedatectl.properties.leap_pending = うるう秒の挿入/削除が保留
timedatectl.properties.ntp_config = NTP 設定
timedatectl.properties.ntp_enabled = NTP 有効
timedatectl.properties.ntp_servers = NTP サーバー
timedatectl.properties.min_poll = 最小ポーリング間隔
timedatectl.properties.max_poll = 最大ポーリング間隔
timedatectl.properties.capabilities = システム機能
timedatectl.properties.tz_changes = タイムゾーン変更
timedatectl.properties.ntp_sync = NTP 同期
timedatectl.properties.rtc_access = RTC アクセス
timedatectl.properties.hw_timestamp = ハードウェアタイムスタンプ

# 共通ラベル
common.yes = はい
common.no = いいえ
common.supported = 対応
common.limited = 制限あり
common.full = 完全
common.available = 利用可

# 単位
units.microseconds = マイクロ秒

# date コマンド（メタデータ/相対表現/エラー/祝日一覧）
date.error.invalid_timezone = 無効なタイムゾーン: {$tz}
date.error.invalid_month = 無効な月: {$month}

date.metadata.unix_timestamp = Unix タイムスタンプ: {$value}
date.metadata.julian_day = ユリウス日: {$value}
date.metadata.day_of_year = 年内通算日: {$value}
date.metadata.week_number = 週番号: {$value}
date.metadata.weekday = 曜日: {$value}
date.metadata.type.weekend = 種別: 週末
date.metadata.type.business = 種別: 平日
date.metadata.astronomical = 天文情報: {$info}

date.relative.now = 現在
date.relative.minutes_ago = {$mins} 分前
date.relative.in_minutes = {$mins} 分後
date.relative.hours_ago = {$hours} 時間前
date.relative.in_hours = {$hours} 時間後
date.relative.days_ago = {$days} 日前
date.relative.in_days = {$days} 日後

date.holiday.none = 対象年 {$year}、地域: {$regions} の祝日は見つかりませんでした
date.holiday.header = 祝日一覧 年: {$year} 地域: {$regions}
date.holiday.separator = =====================================
date.holiday.entry = {$date} - {$name}（{$region}, {$kind}）
date.holiday.total = 合計: {$count} 件の祝日