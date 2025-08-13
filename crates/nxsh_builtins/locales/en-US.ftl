# NexusShell Builtin Commands - English (US) Localization

# Common messages
error-file-not-found = File not found: {$filename}
error-permission-denied = Permission denied: {$filename}
error-invalid-option = Invalid option: {$option}
error-missing-argument = Missing argument for option: {$option}
error-invalid-argument = Invalid argument: {$argument}
error-directory-not-found = Directory not found: {$dirname}
error-not-a-directory = Not a directory: {$path}
error-not-a-file = Not a file: {$path}
error-operation-failed = Operation failed: {$operation}
error-io-error = I/O error: {$message}

# cat command
cat-help-usage = Usage: cat [OPTION]... [FILE]...
cat-help-description = Concatenate FILE(s) to standard output.
cat-help-no-file = With no FILE, or when FILE is -, read standard input.
cat-help-option-show-all = equivalent to -vET
cat-help-option-number-nonblank = number nonempty output lines, overrides -n
cat-help-option-show-ends = display $ at end of each line
cat-help-option-number = number all output lines
cat-help-option-squeeze-blank = suppress repeated empty output lines
cat-help-option-show-tabs = display TAB characters as ^I
cat-help-option-show-nonprinting = use ^ and M- notation, except for LFD and TAB
cat-help-examples = Examples:
cat-help-example1 = cat f - g  Output f's contents, then standard input, then g's contents.
cat-help-example2 = cat        Copy standard input to standard output.
cat-version = cat (NexusShell) 1.0.0

# ls command
ls-help-usage = Usage: ls [OPTION]... [FILE]...
ls-help-description = List information about the FILEs (the current directory by default).
ls-help-option-all = do not ignore entries starting with .
ls-help-option-almost-all = do not list implied . and ..
ls-help-option-long = use a long listing format
ls-help-option-human-readable = with -l, print human readable sizes
ls-help-option-reverse = reverse order while sorting
ls-help-option-recursive = list subdirectories recursively
ls-help-option-directory = list directories themselves, not their contents
ls-help-option-one-per-line = list one file per line
ls-help-option-color = colorize the output; WHEN can be 'always', 'auto', or 'never'
ls-help-option-classify = append indicator (one of */=>@|) to entries
ls-help-option-inode = print the index number of each file
ls-help-option-size = print the allocated size of each file, in blocks
ls-permission-read = read
ls-permission-write = write
ls-permission-execute = execute
ls-type-directory = directory
ls-type-file = regular file
ls-type-symlink = symbolic link
ls-type-block = block device
ls-type-char = character device
ls-type-fifo = FIFO
ls-type-socket = socket

# grep command
grep-help-usage = Usage: grep [OPTION]... PATTERN [FILE]...
grep-help-description = Search for PATTERN in each FILE.
grep-help-option-extended-regexp = PATTERN is an extended regular expression
grep-help-option-fixed-strings = PATTERN is a set of newline-separated fixed strings
grep-help-option-ignore-case = ignore case distinctions
grep-help-option-invert-match = select non-matching lines
grep-help-option-word-regexp = force PATTERN to match only whole words
grep-help-option-line-regexp = force PATTERN to match only whole lines
grep-help-option-count = print only a count of matching lines per FILE
grep-help-option-files-with-matches = print only names of FILEs containing matches
grep-help-option-line-number = print line number with output lines
grep-help-option-no-filename = suppress the file name prefix on output
grep-help-option-with-filename = print the file name for each match
grep-help-option-quiet = suppress all normal output
grep-help-option-recursive = search directories recursively
grep-help-option-include = search only files that match GLOB
grep-help-option-exclude = skip files and directories matching GLOB
grep-matches-found = {$count} matches found
grep-no-matches = No matches found
grep-binary-file-matches = Binary file {$filename} matches

# ps command
ps-help-usage = Usage: ps [OPTION]...
ps-help-description = Display information about running processes.
ps-help-option-all = show processes for all users
ps-help-option-full = do full-format listing
ps-help-option-long = long format
ps-help-option-user = show processes for the specified user
ps-help-option-pid = show process with specified PID
ps-help-option-command = show processes with specified command name
ps-help-option-forest = show process tree
ps-help-option-sort = sort by specified field
ps-header-pid = PID
ps-header-ppid = PPID
ps-header-user = USER
ps-header-cpu = CPU%
ps-header-mem = MEM%
ps-header-vsz = VSZ
ps-header-rss = RSS
ps-header-tty = TTY
ps-header-stat = STAT
ps-header-start = START
ps-header-time = TIME
ps-header-command = COMMAND

# ping command
ping-help-usage = Usage: ping [OPTION]... HOST
ping-help-description = Send ICMP ECHO_REQUEST to network hosts.
ping-help-option-count = stop after sending COUNT packets
ping-help-option-interval = wait INTERVAL seconds between sending each packet
ping-help-option-size = use SIZE as number of data bytes to be sent
ping-help-option-ttl = define time to live
ping-help-option-timeout = specify a timeout, in seconds, before ping exits
ping-help-option-flood = flood ping
ping-help-option-quiet = quiet output
ping-help-option-verbose = verbose output
ping-help-option-ipv4 = use IPv4 only
ping-help-option-ipv6 = use IPv6 only
ping-help-option-numeric = no attempt to lookup symbolic names for host addresses
ping-statistics = --- {$host} ping statistics ---
ping-packets-transmitted = {$transmitted} packets transmitted
ping-packets-received = {$received} received
ping-packet-loss = {$loss}% packet loss
ping-time-total = time {$time}ms
ping-rtt-stats = rtt min/avg/max/mdev = {$min}/{$avg}/{$max}/{$mdev} ms
ping-reply-from = {$bytes} bytes from {$host} ({$ip}): icmp_seq={$seq} ttl={$ttl} time={$time} ms
ping-destination-unreachable = Destination Host Unreachable
ping-request-timeout = Request timeout for icmp_seq {$seq}

# rm command
rm-help-usage = Usage: rm [OPTION]... [FILE]...
rm-help-description = Remove (unlink) the FILE(s).
rm-help-option-force = ignore nonexistent files and arguments, never prompt
rm-help-option-interactive = prompt before every removal
rm-help-option-recursive = remove directories and their contents recursively
rm-help-option-verbose = explain what is being done
rm-confirm-delete = Remove {$filename}? (y/n): 
rm-removing = removing {$filename}
rm-removed = removed '{$filename}'
rm-cannot-remove = cannot remove '{$filename}': {$reason}

# mkdir command
mkdir-help-usage = Usage: mkdir [OPTION]... DIRECTORY...
mkdir-help-description = Create the DIRECTORY(ies), if they do not already exist.
mkdir-help-option-parents = no error if existing, make parent directories as needed
mkdir-help-option-verbose = print a message for each created directory
mkdir-help-option-mode = set file mode (as in chmod), not a=rwx - umask
mkdir-created = created directory '{$dirname}'
mkdir-cannot-create = cannot create directory '{$dirname}': {$reason}

# mv command
mv-help-usage = Usage: mv [OPTION]... SOURCE... DIRECTORY
mv-help-description = Rename SOURCE to DEST, or move SOURCE(s) to DIRECTORY.
mv-help-option-force = do not prompt before overwriting
mv-help-option-interactive = prompt before overwrite
mv-help-option-no-clobber = do not overwrite an existing file
mv-help-option-verbose = explain what is being done
mv-moving = '{$source}' -> '{$dest}'
mv-cannot-move = cannot move '{$source}' to '{$dest}': {$reason}
mv-overwrite-confirm = overwrite '{$dest}'? (y/n): 

# cp command
cp-help-usage = Usage: cp [OPTION]... SOURCE DEST
cp-help-description = Copy SOURCE to DEST, or multiple SOURCE(s) to DIRECTORY.
cp-help-option-recursive = copy directories recursively
cp-help-option-force = if an existing destination file cannot be opened, remove it and try again
cp-help-option-interactive = prompt before overwrite
cp-help-option-preserve = preserve the specified attributes
cp-help-option-verbose = explain what is being done
cp-copying = '{$source}' -> '{$dest}'
cp-cannot-copy = cannot copy '{$source}' to '{$dest}': {$reason}
cp-overwrite-confirm = overwrite '{$dest}'? (y/n): 

# ln command
ln-help-usage = Usage: ln [OPTION]... TARGET LINK_NAME
ln-help-description = Create links between files.
ln-help-option-symbolic = make symbolic links instead of hard links
ln-help-option-force = remove existing destination files
ln-help-option-interactive = prompt whether to remove destinations
ln-help-option-verbose = print name of each linked file
ln-creating = creating link '{$link}' -> '{$target}'
ln-cannot-create = cannot create link '{$link}': {$reason}

# touch command
touch-help-usage = Usage: touch [OPTION]... FILE...
touch-help-description = Update the access and modification times of each FILE to the current time.
touch-help-option-access = change only the access time
touch-help-option-modify = change only the modification time
touch-help-option-no-create = do not create any files
touch-help-option-reference = use this file's times instead of current time
touch-help-option-time = change the specified time
touch-cannot-touch = cannot touch '{$filename}': {$reason}

# stat command
stat-help-usage = Usage: stat [OPTION]... FILE...
stat-help-description = Display file or file system status.
stat-help-option-format = use the specified FORMAT instead of the default
stat-help-option-filesystem = display file system status instead of file status
stat-help-option-terse = print the information in terse form
stat-file-info = File: {$filename}
stat-size = Size: {$size}
stat-blocks = Blocks: {$blocks}
stat-device = Device: {$device}
stat-inode = Inode: {$inode}
stat-links = Links: {$links}
stat-access-perms = Access: ({$octal}/{$symbolic})
stat-uid-gid = Uid: ({$uid}/{$user})   Gid: ({$gid}/{$group})
stat-access-time = Access: {$atime}
stat-modify-time = Modify: {$mtime}
stat-change-time = Change: {$ctime}
stat-birth-time = Birth: {$btime}

# Common file operations
file-exists = File exists: {$filename}
file-not-exists = File does not exist: {$filename}
directory-exists = Directory exists: {$dirname}
directory-not-exists = Directory does not exist: {$dirname}
operation-cancelled = Operation cancelled
operation-completed = Operation completed successfully
bytes-processed = {$bytes} bytes processed
files-processed = {$count} files processed
progress-complete = Progress: {$percent}% complete

# Error messages
error-out-of-memory = Out of memory
error-disk-full = No space left on device
error-read-only = Read-only file system
error-file-too-large = File too large
error-network-unreachable = Network is unreachable
error-connection-refused = Connection refused
error-timeout = Operation timed out
error-interrupted = Operation interrupted
error-broken-pipe = Broken pipe
error-invalid-utf8 = Invalid UTF-8 sequence 

# schedule command
schedule-help-title = schedule: Simple task scheduler
schedule-help-usage = Usage: schedule [OPTIONS] TIME COMMAND
schedule-help-options-title = Options:
schedule-help-option-list =   -l, --list     List scheduled tasks
schedule-help-option-delete =   -d, --delete   Delete scheduled task
schedule-help-option-help =   -h, --help     Show this help
schedule-help-examples-title = Examples:
schedule-help-example-1 =   schedule 15:30 'echo Hello'
schedule-help-example-2 =   schedule tomorrow 'backup.sh'
schedule-help-example-3 =   schedule '2000-01-01 09:00' 'echo Happy New Year'
schedule-no-tasks = No scheduled tasks
schedule-delete-missing-id = schedule: missing task ID for delete
schedule-deleted = Deleted job
schedule-job-not-found = schedule: job not found
schedule-stats-total = Total Jobs:
schedule-stats-running = Running:
schedule-stats-queued = Queued:
schedule-stats-success-rate = Success Rate:
schedule-stats-avg-exec-ms = Avg Exec Time (ms):
schedule-help-option-list-extended =   -l, --list       List scheduled tasks
schedule-help-option-delete-extended =   -d, --delete ID  Delete scheduled task
schedule-help-option-stats =       --stats      Show scheduler statistics
schedule-help-option-enable =       --enable ID  Enable a disabled job
schedule-help-option-disable =       --disable ID Disable a job
schedule-help-option-interval =       --interval SECS CMD  Schedule interval job
schedule-help-option-at =       --at EPOCH_SECS CMD  Schedule one-shot job
schedule-help-option-help =   -h, --help     Show this help
schedule-missing-command = schedule: missing command
schedule-usage-time-cmd = Usage: schedule TIME COMMAND
schedule-scheduled-as = schedule: scheduled as
schedule-delegating-at = schedule: delegating absolute time to external 'at' if available

# cron daemon
cron-daemon-started = Cron daemon started
cron-daemon-stopped = Cron daemon stopped