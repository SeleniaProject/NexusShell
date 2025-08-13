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
schedule-delegating-at = schedule: 絶対時刻は外部 'at' があれば委譲します

# cron デーモン
cron-daemon-started = Cron デーモンを開始しました
cron-daemon-stopped = Cron デーモンを停止しました