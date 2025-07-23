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