# NexusShell 内置命令 - 简体中文本地化

# 通用消息
error-file-not-found = 文件未找到: {$filename}
error-permission-denied = 权限被拒绝: {$filename}
error-invalid-option = 无效选项: {$option}
error-missing-argument = 选项缺少参数: {$option}
error-invalid-argument = 无效参数: {$argument}
error-directory-not-found = 目录未找到: {$dirname}
error-not-a-directory = 不是目录: {$path}
error-not-a-file = 不是文件: {$path}
error-operation-failed = 操作失败: {$operation}
error-io-error = I/O错误: {$message}

# cat 命令
cat-help-usage = 用法: cat [选项]... [文件]...
cat-help-description = 将文件连接并输出到标准输出。
cat-help-no-file = 没有文件或文件为 - 时，从标准输入读取。
cat-help-option-show-all = 等同于 -vET
cat-help-option-number-nonblank = 对非空输出行编号，覆盖 -n
cat-help-option-show-ends = 在每行末尾显示 $
cat-help-option-number = 对所有输出行编号
cat-help-option-squeeze-blank = 抑制重复的空行
cat-help-option-show-tabs = 将TAB字符显示为 ^I
cat-help-option-show-nonprinting = 使用 ^ 和 M- 记号，除了LFD和TAB
cat-help-examples = 示例:
cat-help-example1 = cat f - g  输出f的内容，然后是标准输入，然后是g的内容。
cat-help-example2 = cat        将标准输入复制到标准输出。
cat-version = cat (NexusShell) 1.0.0

# ls 命令
ls-help-usage = 用法: ls [选项]... [文件]...
ls-help-description = 列出文件信息（默认为当前目录）。
ls-help-option-all = 不忽略以 . 开头的条目
ls-help-option-almost-all = 不列出隐含的 . 和 ..
ls-help-option-long = 使用长列表格式
ls-help-option-human-readable = 与 -l 一起使用，以人类可读的大小显示
ls-help-option-reverse = 反转排序顺序
ls-help-option-recursive = 递归列出子目录
ls-help-option-directory = 列出目录本身，而不是其内容
ls-help-option-one-per-line = 每行列出一个文件
ls-help-option-color = 为输出着色；WHEN 可以是 'always'、'auto' 或 'never'
ls-help-option-classify = 在条目后添加指示符（*/=>@| 之一）
ls-help-option-inode = 打印每个文件的索引号
ls-help-option-size = 以块为单位打印每个文件的分配大小
ls-permission-read = 读取
ls-permission-write = 写入
ls-permission-execute = 执行
ls-type-directory = 目录
ls-type-file = 常规文件
ls-type-symlink = 符号链接
ls-type-block = 块设备
ls-type-char = 字符设备
ls-type-fifo = FIFO
ls-type-socket = 套接字

# grep 命令
grep-help-usage = 用法: grep [选项]... 模式 [文件]...
grep-help-description = 在每个文件中搜索模式。
grep-help-option-extended-regexp = 模式是扩展正则表达式
grep-help-option-fixed-strings = 模式是换行符分隔的固定字符串集合
grep-help-option-ignore-case = 忽略大小写区别
grep-help-option-invert-match = 选择不匹配的行
grep-help-option-word-regexp = 强制模式仅匹配整个单词
grep-help-option-line-regexp = 强制模式仅匹配整行
grep-help-option-count = 仅打印每个文件的匹配行数
grep-help-option-files-with-matches = 仅打印包含匹配的文件名
grep-help-option-line-number = 与输出行一起打印行号
grep-help-option-no-filename = 在输出中抑制文件名前缀
grep-help-option-with-filename = 为每个匹配打印文件名
grep-help-option-quiet = 抑制所有正常输出
grep-help-option-recursive = 递归搜索目录
grep-help-option-include = 仅搜索匹配GLOB的文件
grep-help-option-exclude = 跳过匹配GLOB的文件和目录
grep-matches-found = 找到 {$count} 个匹配
grep-no-matches = 未找到匹配
grep-binary-file-matches = 二进制文件 {$filename} 匹配

# ps 命令
ps-help-usage = 用法: ps [选项]...
ps-help-description = 显示正在运行的进程信息。
ps-help-option-all = 显示所有用户的进程
ps-help-option-full = 完整格式列表
ps-help-option-long = 长格式
ps-help-option-user = 显示指定用户的进程
ps-help-option-pid = 显示指定PID的进程
ps-help-option-command = 显示指定命令名的进程
ps-help-option-forest = 显示进程树
ps-help-option-sort = 按指定字段排序
ps-header-pid = PID
ps-header-ppid = PPID
ps-header-user = 用户
ps-header-cpu = CPU%
ps-header-mem = MEM%
ps-header-vsz = VSZ
ps-header-rss = RSS
ps-header-tty = TTY
ps-header-stat = STAT
ps-header-start = 开始
ps-header-time = 时间
ps-header-command = 命令

# ping 命令
ping-help-usage = 用法: ping [选项]... 主机
ping-help-description = 向网络主机发送ICMP ECHO_REQUEST。
ping-help-option-count = 发送COUNT个数据包后停止
ping-help-option-interval = 每个数据包发送间隔的秒数
ping-help-option-size = 使用SIZE作为要发送的数据字节数
ping-help-option-ttl = 定义生存时间
ping-help-option-timeout = 指定ping退出前的超时时间（秒）
ping-help-option-flood = 洪水ping
ping-help-option-quiet = 安静输出
ping-help-option-verbose = 详细输出
ping-help-option-ipv4 = 仅使用IPv4
ping-help-option-ipv6 = 仅使用IPv6
ping-help-option-numeric = 不尝试查找主机地址的符号名
ping-statistics = --- {$host} ping统计 ---
ping-packets-transmitted = {$transmitted} 个数据包已传输
ping-packets-received = {$received} 个已接收
ping-packet-loss = {$loss}% 数据包丢失
ping-time-total = 时间 {$time}ms
ping-rtt-stats = rtt 最小/平均/最大/标准差 = {$min}/{$avg}/{$max}/{$mdev} ms
ping-reply-from = 来自 {$host} ({$ip}) 的 {$bytes} 字节: icmp_seq={$seq} ttl={$ttl} time={$time} ms
ping-destination-unreachable = 目标主机不可达
ping-request-timeout = icmp_seq {$seq} 请求超时

# rm 命令
rm-help-usage = 用法: rm [选项]... [文件]...
rm-help-description = 删除（取消链接）文件。
rm-help-option-force = 忽略不存在的文件和参数，从不提示
rm-help-option-interactive = 每次删除前提示
rm-help-option-recursive = 递归删除目录及其内容
rm-help-option-verbose = 解释正在执行的操作
rm-confirm-delete = 删除 {$filename}？ (y/n): 
rm-removing = 正在删除 {$filename}
rm-removed = 已删除 '{$filename}'
rm-cannot-remove = 无法删除 '{$filename}': {$reason}

# mkdir 命令
mkdir-help-usage = 用法: mkdir [选项]... 目录...
mkdir-help-description = 如果目录不存在，则创建目录。
mkdir-help-option-parents = 如果存在则无错误，根据需要创建父目录
mkdir-help-option-verbose = 为每个创建的目录打印消息
mkdir-help-option-mode = 设置文件模式（如chmod），不是 a=rwx - umask
mkdir-created = 已创建目录 '{$dirname}'
mkdir-cannot-create = 无法创建目录 '{$dirname}': {$reason}

# mv 命令
mv-help-usage = 用法: mv [选项]... 源... 目录
mv-help-description = 将源重命名为目标，或将源移动到目录。
mv-help-option-force = 覆盖前不提示
mv-help-option-interactive = 覆盖前提示
mv-help-option-no-clobber = 不覆盖现有文件
mv-help-option-verbose = 解释正在执行的操作
mv-moving = '{$source}' -> '{$dest}'
mv-cannot-move = 无法将 '{$source}' 移动到 '{$dest}': {$reason}
mv-overwrite-confirm = 覆盖 '{$dest}'？ (y/n): 

# cp 命令
cp-help-usage = 用法: cp [选项]... 源 目标
cp-help-description = 将源复制到目标，或将多个源复制到目录。
cp-help-option-recursive = 递归复制目录
cp-help-option-force = 如果无法打开现有目标文件，删除它并重试
cp-help-option-interactive = 覆盖前提示
cp-help-option-preserve = 保留指定的属性
cp-help-option-verbose = 解释正在执行的操作
cp-copying = '{$source}' -> '{$dest}'
cp-cannot-copy = 无法将 '{$source}' 复制到 '{$dest}': {$reason}
cp-overwrite-confirm = 覆盖 '{$dest}'？ (y/n): 

# ln 命令
ln-help-usage = 用法: ln [选项]... 目标 链接名
ln-help-description = 在文件之间创建链接。
ln-help-option-symbolic = 创建符号链接而不是硬链接
ln-help-option-force = 删除现有的目标文件
ln-help-option-interactive = 提示是否删除目标
ln-help-option-verbose = 打印每个链接文件的名称
ln-creating = 正在创建链接 '{$link}' -> '{$target}'
ln-cannot-create = 无法创建链接 '{$link}': {$reason}

# touch 命令
touch-help-usage = 用法: touch [选项]... 文件...
touch-help-description = 将每个文件的访问和修改时间更新为当前时间。
touch-help-option-access = 仅更改访问时间
touch-help-option-modify = 仅更改修改时间
touch-help-option-no-create = 不创建任何文件
touch-help-option-reference = 使用此文件的时间而不是当前时间
touch-help-option-time = 更改指定的时间
touch-cannot-touch = 无法touch '{$filename}': {$reason}

# stat 命令
stat-help-usage = 用法: stat [选项]... 文件...
stat-help-description = 显示文件或文件系统状态。
stat-help-option-format = 使用指定的格式而不是默认格式
stat-help-option-filesystem = 显示文件系统状态而不是文件状态
stat-help-option-terse = 以简洁形式打印信息
stat-file-info = 文件: {$filename}
stat-size = 大小: {$size}
stat-blocks = 块: {$blocks}
stat-device = 设备: {$device}
stat-inode = Inode: {$inode}
stat-links = 链接: {$links}
stat-access-perms = 访问: ({$octal}/{$symbolic})
stat-uid-gid = Uid: ({$uid}/{$user})   Gid: ({$gid}/{$group})
stat-access-time = 访问: {$atime}
stat-modify-time = 修改: {$mtime}
stat-change-time = 更改: {$ctime}
stat-birth-time = 创建: {$btime}

# 通用文件操作
file-exists = 文件存在: {$filename}
file-not-exists = 文件不存在: {$filename}
directory-exists = 目录存在: {$dirname}
directory-not-exists = 目录不存在: {$dirname}
operation-cancelled = 操作已取消
operation-completed = 操作成功完成
bytes-processed = 已处理 {$bytes} 字节
files-processed = 已处理 {$count} 个文件
progress-complete = 进度: {$percent}% 完成

# 错误消息
error-out-of-memory = 内存不足
error-disk-full = 设备上没有剩余空间
error-read-only = 只读文件系统
error-file-too-large = 文件太大
error-network-unreachable = 网络不可达
error-connection-refused = 连接被拒绝
error-timeout = 操作超时
error-interrupted = 操作被中断
error-broken-pipe = 管道损坏
error-invalid-utf8 = 无效的UTF-8序列 

# timedatectl status/common/time-sync labels
timedatectl.common.yes = 是
timedatectl.common.no = 否
timedatectl.common.enabled = 已启用
timedatectl.common.disabled = 已禁用
timedatectl.common.reachable = 可达
timedatectl.common.unreachable = 不可达

timedatectl.msg.time_set_to = 时间已设置为：
timedatectl.msg.timezone_set_to = 时区已设置为：
timedatectl.msg.rtc_in_local_tz = 本地时区的RTC：
timedatectl.msg.ntp_sync = NTP同步：
timedatectl.msg.added_ntp_server = 已添加NTP服务器：
timedatectl.msg.removed_ntp_server = 已移除NTP服务器：

timedatectl.timesync.title = 时间同步状态：
timedatectl.timesync.enabled = 已启用：
timedatectl.timesync.synchronized = 已同步：
timedatectl.timesync.last_sync = 上次同步：
timedatectl.timesync.sync_accuracy = 同步精度：
timedatectl.timesync.drift_rate = 漂移率：
timedatectl.timesync.poll_interval = 轮询间隔：
timedatectl.timesync.leap_status = 闰秒状态：
timedatectl.timesync.ntp_servers = NTP服务器：
timedatectl.timesync.stratum = 层级：
timedatectl.timesync.delay = 延迟：
timedatectl.timesync.offset = 偏移：
timedatectl.timesync.summary = 摘要：
timedatectl.timesync.servers_total_reachable = 服务器（总数/可达）：
timedatectl.timesync.best_stratum = 最佳层级：
timedatectl.timesync.preferred_server = 首选服务器：
timedatectl.timesync.avg_delay = 平均延迟：
timedatectl.timesync.min_delay = 最小延迟：
timedatectl.timesync.max_delay = 最大延迟：
timedatectl.timesync.avg_offset = 平均偏移：
timedatectl.timesync.min_offset = 最小偏移：
timedatectl.timesync.max_offset = 最大偏移：
timedatectl.timesync.avg_jitter = 平均抖动：

# timedatectl 状态标签
timedatectl.status.local_time = 本地时间
timedatectl.status.universal_time = 世界时 (UTC)
timedatectl.status.rtc_time = RTC 时间
timedatectl.status.time_zone = 时区
timedatectl.status.system_clock_synchronized = 系统时钟已同步
timedatectl.status.ntp_service = NTP 服务
timedatectl.status.rtc_in_local_tz = 本地时区的 RTC
timedatectl.status.sync_accuracy = 同步精度
timedatectl.status.drift_rate = 漂移率
timedatectl.status.last_sync = 上次同步
timedatectl.status.leap_second = 闰秒
timedatectl.status.pending = 待定

# timedatectl 帮助 — 命令
timedatectl.help.cmd.status = 显示当前时间状态
timedatectl.help.cmd.show = 以 JSON 显示状态
timedatectl.help.cmd.set_time = 设置系统时间
timedatectl.help.cmd.set_timezone = 设置系统时区
timedatectl.help.cmd.list_timezones = 列出可用时区
timedatectl.help.cmd.set_local_rtc = 将 RTC 设为本地时间（true/false）
timedatectl.help.cmd.set_ntp = 启用或禁用 NTP 同步（true/false）
timedatectl.help.cmd.timesync_status = 显示时间同步状态
timedatectl.help.cmd.show_timesync = 以 JSON 显示同步状态
timedatectl.help.cmd.add_ntp_server = 添加 NTP 服务器
timedatectl.help.cmd.remove_ntp_server = 删除 NTP 服务器
timedatectl.help.cmd.statistics = 显示时间统计
timedatectl.help.cmd.history = 显示时间调整历史

# timedatectl 帮助 — 选项
timedatectl.help.opt.help = 显示此帮助并退出
timedatectl.help.opt.monitor = 运行实时监控模式
timedatectl.help.opt.all = 显示所有属性
timedatectl.help.opt.json = 以 JSON 输出

# 接受的时间格式
timedatectl.help.fmt.full_datetime = 完整日期和时间
timedatectl.help.fmt.datetime_no_sec = 不含秒的日期和时间
timedatectl.help.fmt.time_only = 仅时间
timedatectl.help.fmt.time_no_sec = 时间（无秒）
timedatectl.help.fmt.unix_timestamp = Unix 时间戳（秒）
timedatectl.help.fmt.iso8601 = ISO 8601（UTC）

# 帮助示例
timedatectl.help.ex.status = 显示状态
timedatectl.help.ex.set_time = 设置系统时间
timedatectl.help.ex.set_timezone = 设置时区
timedatectl.help.ex.find_timezone = 查找时区
timedatectl.help.ex.enable_ntp = 启用 NTP 同步
timedatectl.help.ex.add_server = 添加 NTP 服务器
timedatectl.help.ex.sync_status = 显示同步状态
timedatectl.help.ex.statistics = 显示统计

# 属性视图
timedatectl.properties.title = 时间和日期属性
timedatectl.properties.time_info = 时间信息
timedatectl.properties.local_time = 本地时间
timedatectl.properties.utc_time = UTC 时间
timedatectl.properties.timezone_info = 时区信息
timedatectl.properties.timezone = 时区
timedatectl.properties.utc_offset = UTC 偏移
timedatectl.properties.dst_active = 夏令时启用
timedatectl.properties.sync_status = 同步状态
timedatectl.properties.system_synced = 系统时钟已同步
timedatectl.properties.ntp_service = NTP 服务
timedatectl.properties.time_source = 时间源
timedatectl.properties.sync_accuracy = 同步精度
timedatectl.properties.last_sync = 上次同步
timedatectl.properties.drift_rate = 漂移率 (ppm)
timedatectl.properties.leap_info = 闰秒信息
timedatectl.properties.leap_pending = 闰秒待定
timedatectl.properties.ntp_config = NTP 配置
timedatectl.properties.ntp_enabled = NTP 已启用
timedatectl.properties.ntp_servers = NTP 服务器
timedatectl.properties.min_poll = 最小轮询间隔
timedatectl.properties.max_poll = 最大轮询间隔
timedatectl.properties.capabilities = 系统功能
timedatectl.properties.tz_changes = 时区更改
timedatectl.properties.ntp_sync = NTP 同步
timedatectl.properties.rtc_access = RTC 访问
timedatectl.properties.hw_timestamp = 硬件时间戳

# 通用标签
common.yes = 是
common.no = 否
common.supported = 支持
common.limited = 受限
common.full = 完整
common.available = 可用

# 单位
units.microseconds = 微秒