# at help detailed sections (fallback)
at.help.usage-line =     at [OPTIONS] time [command...]
at.help.time_formats.details =     HH:MM [AM/PM] [date]    - Specific time (e.g., '14:30', '2:30 PM tomorrow')\n    HHMM [AM/PM] [date]     - Numeric format (e.g., '1430', '230 PM')\n    noon/midnight [date]    - Named times\n    now + N units           - Relative time (e.g., 'now + 2 hours')\n    in N units              - Alternative relative (e.g., 'in 30 minutes')\n    tomorrow at time        - Next day scheduling\n    next weekday [at time]  - Next occurrence of weekday\n    ISO-8601 format         - Full timestamp\n    @timestamp              - Unix timestamp
at.help.options.list =     -h, --help              Show this help message\n    -l, --list              List scheduled jobs\n    -r, --remove ID         Remove job by ID\n    -q, --queue QUEUE       Specify job queue (default: 'a')\n    -m, --mail              Send mail when job completes\n    -M, --no-mail           Don't send mail\n    -f, --file FILE         Read commands from file\n    -t, --time TIME         Time specification\n    --priority LEVEL        Set priority (low, normal, high, critical)\n    --output FILE           Redirect stdout to file\n    --error FILE            Redirect stderr to file\n    --max-runtime SECS      Maximum runtime in seconds\n    --retry COUNT           Number of retries on failure\n    --tag TAG               Add tag to job
at.help.examples.list =     at 14:30 tomorrow       # Schedule for 2:30 PM tomorrow\n    at 'now + 1 hour'       # Schedule for one hour from now\n    at 'next friday at 9am' # Schedule for next Friday at 9 AM\n    at --queue b --priority high 16:00 # High priority job in queue b\n    echo 'backup.sh' | at midnight # Schedule backup at midnight\n    at -l -q a               # List jobs in queue 'a'\n    at -r at_123             # Remove job with ID 'at_123'
at.help.title = at: One-time job scheduler
at.help.usage = Usage:
at.help.time_formats = Accepted time formats:
at.help.options = Options:
at.help.examples = Examples:
at.help.inline-usage = at: usage: at [OPTIONS] time [command...]
at.error.unable-parse-time = Unable to parse time specification: { $input }
at.error.invalid-time = Invalid time: { $hour }:{ $minute }
at.error.invalid-date-time-combo = Invalid date/time combination
at.error.ambiguous-local-time = Ambiguous local time
at.error.invalid-numeric-time = Invalid numeric time format
at.error.unknown-named-time = Unknown named time: { $name }
at.error.unknown-time-unit = Unknown time unit: { $unit }
at.error.unknown-day = Unknown day: { $day }
at.error.unknown-weekday = Unknown weekday: { $weekday }
at.error.parse-iso = Failed to parse ISO format
at.error.invalid-unix-timestamp = Invalid Unix timestamp: { $timestamp }
at.error.unable-parse-date = Unable to parse date: { $date }
at.error.in-future = Scheduled time must be in the future
at.error.job-not-found = Job not found: { $id }
at.error.user-not-allowed = User { $user } is not allowed to use at
at.error.user-denied = User { $user } is denied access to at
at.error.missing-id-for-remove = -r requires a job ID
at.error.missing-queue-name = -q requires a queue name
at.error.read-file = Failed to read file: { $filename }
at.error.missing-filename = -f requires a filename
at.error.missing-time-spec = -t requires a time specification
at.error.invalid-priority = Invalid priority: { $value }
at.error.missing-priority = --priority requires a priority level
at.error.missing-output-filename = --output requires a filename
at.error.missing-error-filename = --error requires a filename
at.error.invalid-max-runtime = Invalid max runtime
at.error.missing-max-runtime = --max-runtime requires seconds
at.error.invalid-retry-count = Invalid retry count
at.error.missing-retry-count = --retry requires a count
at.error.missing-tag-name = --tag requires a tag name
at.error.unknown-option = Unknown option: { $option }
at.list.no-jobs = No jobs scheduled
at.list.header.job-id = Job ID
at.list.header.scheduled-time = Scheduled Time
at.list.header.status = Status
at.list.header.queue = Queue
at.list.header.command = Command
at.remove.removed = Job { $id } removed
at.remove.failed = Failed to remove job { $id }: { $error }
at.error.time-spec-required = Time specification required
at.error.read-stdin = Failed to read from stdin: { $error }
at.error.no-command = No command specified
at.schedule.scheduled = job { $id } at { $time }
# cat statistics/help (fallback in English)
cat-stats-header = === Statistics for { $filename } ===
cat-stats-total-header = === Total Statistics ===
cat-stats-bytes-read = Bytes read
cat-stats-lines-processed = Lines processed
cat-stats-processing-time = Processing time
cat-stats-encoding-detected = Encoding detected
cat-stats-file-type = File type
cat-stats-compression = Compression
cat-stats-throughput = Throughput
cat-binary-skipped = cat: { $filename }: binary file skipped
cat-error-file = cat: { $filename }: { $error }
cat-warn-bzip2-missing = Warning: bzip2 decompression not available, reading as regular file
cat-warn-xz-missing = Warning: XZ decompression not available, reading as regular file
cat-warn-zstd-missing = Warning: zstd decompression not available, reading as regular file
cat-help-advanced-title = Advanced options:
cat-help-advanced-options =       --progress           show progress bar for large files\n      --parallel           process multiple files in parallel\n      --threads N          number of threads for parallel processing\n      --encoding ENC       force specific encoding (utf-8, utf-16le, etc.)\n      --binary             treat all files as binary\n      --text               treat all files as text\n      --skip-binary        skip binary files\n      --format FMT         output format (raw, hex, base64, json)\n      --color WHEN         colorize output (always, never, auto)\n      --statistics         show processing statistics\n      --buffer-size N      buffer size for I/O operations\n      --no-mmap            disable memory mapping for large files\n      --no-decompress      disable automatic decompression\n      --no-follow-symlinks don't follow symbolic links\n      --timeout N          network timeout in seconds\n      --help               display this help and exit\n      --version            output version information and exit
cat-help-advanced-examples-title = Advanced examples:
cat-help-advanced-example1 =   cat --parallel --progress *.log    Process log files in parallel with progress
cat-help-advanced-example2 =   cat --format hex data.bin          Output binary file as hexadecimal
cat-help-advanced-example3 =   cat --statistics --encoding utf-16le file.txt  Show stats with specific encoding
cat-help-report-bugs = Report cat bugs to <bug-reports@nexusshell.org>
# NexusShell 내장 명령어 - 한국어 현지화

# 공통 메시지
error-file-not-found = 파일을 찾을 수 없습니다: {$filename}
error-permission-denied = 권한이 거부되었습니다: {$filename}
error-invalid-option = 잘못된 옵션: {$option}
error-missing-argument = 옵션에 인수가 없습니다: {$option}
error-invalid-argument = 잘못된 인수: {$argument}
error-directory-not-found = 디렉토리를 찾을 수 없습니다: {$dirname}
error-not-a-directory = 디렉토리가 아닙니다: {$path}
error-not-a-file = 파일이 아닙니다: {$path}
error-operation-failed = 작업이 실패했습니다: {$operation}
error-io-error = I/O 오류: {$message}

# cat 명령어
cat-help-usage = 사용법: cat [옵션]... [파일]...
cat-help-description = 파일을 표준 출력으로 연결합니다.
cat-version = cat (NexusShell) 1.0.0

# ls 명령어
ls-help-usage = 사용법: ls [옵션]... [파일]...
ls-help-description = 파일 정보를 나열합니다(기본값은 현재 디렉토리).
ls-permission-read = 읽기
ls-permission-write = 쓰기
ls-permission-execute = 실행
ls-type-directory = 디렉토리
ls-type-file = 일반 파일
ls-type-symlink = 심볼릭 링크

# grep 명령어
grep-help-usage = 사용법: grep [옵션]... 패턴 [파일]...
grep-help-description = 각 파일에서 패턴을 검색합니다.
grep-matches-found = {$count}개의 일치 항목을 찾았습니다
grep-no-matches = 일치하는 항목이 없습니다

# ps 명령어
ps-help-usage = 사용법: ps [옵션]...
ps-help-description = 실행 중인 프로세스 정보를 표시합니다.
ps-header-pid = PID
ps-header-user = 사용자
ps-header-command = 명령어

# ping 명령어
ping-help-usage = 사용법: ping [옵션]... 호스트
ping-help-description = 네트워크 호스트에 ICMP ECHO_REQUEST를 보냅니다.
ping-statistics = --- {$host} ping 통계 ---
ping-packets-transmitted = {$transmitted}개 패킷 전송됨
ping-packets-received = {$received}개 수신됨
ping-packet-loss = {$loss}% 패킷 손실

# 공통 파일 작업
file-exists = 파일이 존재합니다: {$filename}
file-not-exists = 파일이 존재하지 않습니다: {$filename}
operation-cancelled = 작업이 취소되었습니다
operation-completed = 작업이 성공적으로 완료되었습니다 

# timedatectl status/common/time-sync labels
timedatectl.common.yes = 예
timedatectl.common.no = 아니오
timedatectl.common.enabled = 사용
timedatectl.common.disabled = 사용 안 함
timedatectl.common.reachable = 도달 가능
timedatectl.common.unreachable = 도달 불가

timedatectl.msg.time_set_to = 시간 설정:
timedatectl.msg.timezone_set_to = 시간대 설정:
timedatectl.msg.rtc_in_local_tz = 로컬 시간대의 RTC:
timedatectl.msg.ntp_sync = NTP 동기화:
timedatectl.msg.added_ntp_server = NTP 서버 추가:
timedatectl.msg.removed_ntp_server = NTP 서버 제거:

timedatectl.timesync.title = 시간 동기화 상태:
timedatectl.timesync.enabled = 사용:
timedatectl.timesync.synchronized = 동기화됨:
timedatectl.timesync.last_sync = 최근 동기화:
timedatectl.timesync.sync_accuracy = 동기화 정확도:
timedatectl.timesync.drift_rate = 드리프트율:
timedatectl.timesync.poll_interval = 폴링 간격:
timedatectl.timesync.leap_status = 윤초 상태:
timedatectl.timesync.ntp_servers = NTP 서버:
timedatectl.timesync.stratum = 스트라텀:
timedatectl.timesync.delay = 지연:
timedatectl.timesync.offset = 오프셋:
timedatectl.timesync.summary = 요약:
timedatectl.timesync.servers_total_reachable = 서버 (전체/도달 가능):
timedatectl.timesync.best_stratum = 최적 스트라텀:
timedatectl.timesync.preferred_server = 우선 서버:
timedatectl.timesync.avg_delay = 평균 지연:
timedatectl.timesync.min_delay = 최소 지연:
timedatectl.timesync.max_delay = 최대 지연:
timedatectl.timesync.avg_offset = 평균 오프셋:
timedatectl.timesync.min_offset = 최소 오프셋:
timedatectl.timesync.max_offset = 최대 오프셋:
timedatectl.timesync.avg_jitter = 평균 지터:

# timedatectl 상태 레이블
timedatectl.status.local_time = 로컬 시간

# timedatectl 도움말 섹션 헤더
timedatectl.help.title = timedatectl: 날짜와 시간 관리
timedatectl.help.usage = 사용법:
timedatectl.help.commands = 명령:
timedatectl.help.options = 옵션:
timedatectl.help.time_formats = 허용되는 시간 형식:
timedatectl.help.examples = 예시:
timedatectl.help.timesync_options = timesync-status 옵션:
timedatectl.help.timesync_json_option =   -J, --json            상태와 요약을 간결한 JSON으로 출력
timedatectl.help.global_json_option =   전역: 일부 명령은 -J/--json 으로 JSON 출력 지원
timedatectl.status.universal_time = 세계시 (UTC)
timedatectl.status.rtc_time = RTC 시간
timedatectl.status.time_zone = 시간대
timedatectl.status.system_clock_synchronized = 시스템 시계 동기화됨
timedatectl.status.ntp_service = NTP 서비스
timedatectl.status.rtc_in_local_tz = 로컬 TZ의 RTC
timedatectl.status.sync_accuracy = 동기화 정확도
timedatectl.status.drift_rate = 드리프트율
timedatectl.status.last_sync = 마지막 동기화
timedatectl.status.leap_second = 윤초
timedatectl.status.pending = 보류 중

# timedatectl 도움말 — 명령
timedatectl.help.cmd.status = 현재 시간 상태 표시
timedatectl.help.cmd.show = 상태를 JSON으로 표시
timedatectl.help.cmd.set_time = 시스템 시간을 설정
timedatectl.help.cmd.set_timezone = 시스템 시간대를 설정
timedatectl.help.cmd.list_timezones = 사용 가능한 시간대 나열
timedatectl.help.cmd.set_local_rtc = RTC를 로컬 시간으로 설정 (true/false)
timedatectl.help.cmd.set_ntp = NTP 동기화 활성/비활성 (true/false)
timedatectl.help.cmd.timesync_status = 시간 동기화 상태 표시
timedatectl.help.cmd.show_timesync = 시간 동기화 상태를 JSON으로 표시
timedatectl.help.cmd.add_ntp_server = NTP 서버 추가
timedatectl.help.cmd.remove_ntp_server = NTP 서버 제거
timedatectl.help.cmd.statistics = 시간 관련 통계 표시
timedatectl.help.cmd.history = 시간 조정 이력 표시

# timedatectl 도움말 — 옵션
timedatectl.help.opt.help = 이 도움말을 표시하고 종료
timedatectl.help.opt.monitor = 실시간 모니터링 모드 실행
timedatectl.help.opt.all = 모든 속성 표시
timedatectl.help.opt.json = JSON으로 출력

# 허용되는 시간 형식
timedatectl.help.fmt.full_datetime = 전체 날짜 및 시간
timedatectl.help.fmt.datetime_no_sec = 초 없이 날짜 및 시간
timedatectl.help.fmt.time_only = 시간만
timedatectl.help.fmt.time_no_sec = 시간 (초 없음)
timedatectl.help.fmt.unix_timestamp = 유닉스 타임스탬프 (초)
timedatectl.help.fmt.iso8601 = ISO 8601 (UTC)

# 도움말 예시
timedatectl.help.ex.status = 상태 표시
timedatectl.help.ex.set_time = 시스템 시간 설정
timedatectl.help.ex.set_timezone = 시간대 설정
timedatectl.help.ex.find_timezone = 시간대 찾기
timedatectl.help.ex.enable_ntp = NTP 동기화 활성화
timedatectl.help.ex.add_server = NTP 서버 추가
timedatectl.help.ex.sync_status = 동기화 상태 표시
timedatectl.help.ex.statistics = 통계 표시

# 속성 보기
timedatectl.properties.title = 시간 및 날짜 속성
timedatectl.properties.time_info = 시간 정보
timedatectl.properties.local_time = 로컬 시간
timedatectl.properties.utc_time = UTC 시간
timedatectl.properties.timezone_info = 시간대 정보
timedatectl.properties.timezone = 시간대
timedatectl.properties.utc_offset = UTC 오프셋
timedatectl.properties.dst_active = 서머타임 활성
timedatectl.properties.sync_status = 동기화 상태
timedatectl.properties.system_synced = 시스템 시계 동기화됨
timedatectl.properties.ntp_service = NTP 서비스
timedatectl.properties.time_source = 시간 소스
timedatectl.properties.sync_accuracy = 동기화 정확도
timedatectl.properties.last_sync = 마지막 동기화
timedatectl.properties.drift_rate = 드리프트율 (ppm)
timedatectl.properties.leap_info = 윤초 정보
timedatectl.properties.leap_pending = 윤초 보류 중
timedatectl.properties.ntp_config = NTP 구성
timedatectl.properties.ntp_enabled = NTP 활성화
timedatectl.properties.ntp_servers = NTP 서버
timedatectl.properties.min_poll = 최소 폴링 간격
timedatectl.properties.max_poll = 최대 폴링 간격
timedatectl.properties.capabilities = 시스템 기능
timedatectl.properties.tz_changes = 시간대 변경
timedatectl.properties.ntp_sync = NTP 동기화
timedatectl.properties.rtc_access = RTC 액세스
timedatectl.properties.hw_timestamp = 하드웨어 타임스탬핑

# 일반 레이블
common.yes = 예
common.no = 아니오
common.supported = 지원됨
common.limited = 제한적
common.full = 전체
common.available = 사용 가능

# 단위
units.microseconds = 마이크로초

# date 명령어 (메타데이터/상대/공휴일)
date.error.invalid_timezone = 잘못된 시간대: {$tz}
date.error.invalid_month = 잘못된 월: {$month}

date.metadata.unix_timestamp = Unix 타임스탬프: {$value}
date.metadata.julian_day = 율리우스일: {$value}
date.metadata.day_of_year = 연중 일수: {$value}
date.metadata.week_number = 주 번호: {$value}
date.metadata.weekday = 요일: {$value}
date.metadata.type.weekend = 유형: 주말
date.metadata.type.business = 유형: 평일
date.metadata.astronomical = 천문: {$info}

date.relative.now = 지금
date.relative.minutes_ago = {$mins}분 전
date.relative.in_minutes = {$mins}분 후
date.relative.hours_ago = {$hours}시간 전
date.relative.in_hours = {$hours}시간 후
date.relative.days_ago = {$days}일 전
date.relative.in_days = {$days}일 후

date.holiday.none = {$year}년, 지역: {$regions}의 공휴일을 찾을 수 없습니다
date.holiday.header = 공휴일 목록 {$year}년 지역: {$regions}
date.holiday.separator = =====================================
date.holiday.entry = {$date} - {$name} ({$region}, {$kind})
date.holiday.total = 합계: {$count}건의 공휴일