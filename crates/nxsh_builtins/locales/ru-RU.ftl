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
# NexusShell Встроенные Команды - Русская Локализация

# Общие сообщения
error-file-not-found = Файл не найден: {$filename}
error-permission-denied = Доступ запрещен: {$filename}
error-invalid-option = Неверная опция: {$option}
error-missing-argument = Отсутствует аргумент для опции: {$option}
error-invalid-argument = Неверный аргумент: {$argument}
error-directory-not-found = Каталог не найден: {$dirname}
error-not-a-directory = Не является каталогом: {$path}
error-not-a-file = Не является файлом: {$path}
error-operation-failed = Операция не удалась: {$operation}
error-io-error = Ошибка ввода-вывода: {$message}

# Команда cat
cat-help-usage = Использование: cat [ОПЦИЯ]... [ФАЙЛ]...
cat-help-description = Объединить ФАЙЛ(ы) в стандартный вывод.
cat-version = cat (NexusShell) 1.0.0

# Команда ls
ls-help-usage = Использование: ls [ОПЦИЯ]... [ФАЙЛ]...
ls-help-description = Вывести информацию о ФАЙЛ(ах) (по умолчанию текущий каталог).
ls-permission-read = чтение
ls-permission-write = запись
ls-permission-execute = выполнение
ls-type-directory = каталог
ls-type-file = обычный файл
ls-type-symlink = символическая ссылка

# Команда grep
grep-help-usage = Использование: grep [ОПЦИЯ]... ШАБЛОН [ФАЙЛ]...
grep-help-description = Поиск ШАБЛОНА в каждом ФАЙЛЕ.
grep-matches-found = Найдено {$count} совпадений
grep-no-matches = Совпадений не найдено

# Команда ps
ps-help-usage = Использование: ps [ОПЦИЯ]...
ps-help-description = Показать информацию о запущенных процессах.
ps-header-pid = PID
ps-header-user = ПОЛЬЗОВАТЕЛЬ
ps-header-command = КОМАНДА

# Команда ping
ping-help-usage = Использование: ping [ОПЦИЯ]... ХОСТ
ping-help-description = Отправить ICMP ECHO_REQUEST сетевым хостам.
ping-statistics = --- статистика ping {$host} ---
ping-packets-transmitted = {$transmitted} пакетов передано
ping-packets-received = {$received} получено
ping-packet-loss = {$loss}% потеря пакетов

# Общие файловые операции
file-exists = Файл существует: {$filename}
file-not-exists = Файл не существует: {$filename}
operation-cancelled = Операция отменена
operation-completed = Операция успешно завершена 

# timedatectl status/common/time-sync labels
timedatectl.common.yes = да
timedatectl.common.no = нет
timedatectl.common.enabled = включено
timedatectl.common.disabled = отключено
timedatectl.common.reachable = доступен
timedatectl.common.unreachable = недоступен

timedatectl.msg.time_set_to = Время установлено:
timedatectl.msg.timezone_set_to = Часовой пояс установлен:
timedatectl.msg.rtc_in_local_tz = RTC в локальном часовом поясе:
timedatectl.msg.ntp_sync = Синхронизация NTP:
timedatectl.msg.added_ntp_server = Добавлен NTP-сервер:
timedatectl.msg.removed_ntp_server = Удален NTP-сервер:

timedatectl.timesync.title = Статус синхронизации времени:
timedatectl.timesync.enabled = Включено:
timedatectl.timesync.synchronized = Синхронизировано:
timedatectl.timesync.last_sync = Последняя синхронизация:
timedatectl.timesync.sync_accuracy = Точность синхронизации:
timedatectl.timesync.drift_rate = Скорость дрейфа:
timedatectl.timesync.poll_interval = Интервал опроса:
timedatectl.timesync.leap_status = Статус високосной секунды:
timedatectl.timesync.ntp_servers = NTP-серверы:
timedatectl.timesync.stratum = Стратум:
timedatectl.timesync.delay = Задержка:
timedatectl.timesync.offset = Смещение:
timedatectl.timesync.summary = Сводка:
timedatectl.timesync.servers_total_reachable = Серверы (всего/доступно):
timedatectl.timesync.best_stratum = Лучший стратум:
timedatectl.timesync.preferred_server = Предпочитаемый сервер:
timedatectl.timesync.avg_delay = Средняя задержка:
timedatectl.timesync.min_delay = Минимальная задержка:
timedatectl.timesync.max_delay = Максимальная задержка:
timedatectl.timesync.avg_offset = Среднее смещение:
timedatectl.timesync.min_offset = Минимальное смещение:
timedatectl.timesync.max_offset = Максимальное смещение:
timedatectl.timesync.avg_jitter = Средний джиттер:

# Метки статуса timedatectl
timedatectl.status.local_time = Локальное время

# Заголовки раздела справки timedatectl
timedatectl.help.title = timedatectl: Управление временем и датой
timedatectl.help.usage = Использование:
timedatectl.help.commands = Команды:
timedatectl.help.options = Параметры:
timedatectl.help.time_formats = Допустимые форматы времени:
timedatectl.help.examples = Примеры:
timedatectl.help.timesync_options = Параметры для timesync-status:
timedatectl.help.timesync_json_option =   -J, --json            Показать статус и сводку в компактном JSON
timedatectl.help.global_json_option =   Глобально: некоторые команды поддерживают -J/--json для вывода JSON
timedatectl.status.universal_time = Всемирное время (UTC)
timedatectl.status.rtc_time = Время RTC
timedatectl.status.time_zone = Часовой пояс
timedatectl.status.system_clock_synchronized = Системные часы синхронизированы
timedatectl.status.ntp_service = Служба NTP
timedatectl.status.rtc_in_local_tz = RTC в локальной зоне
timedatectl.status.sync_accuracy = Точность синхронизации
timedatectl.status.drift_rate = Скорость дрейфа
timedatectl.status.last_sync = Последняя синхронизация
timedatectl.status.leap_second = Високосная секунда
timedatectl.status.pending = ожидается

# Справка timedatectl — команды
timedatectl.help.cmd.status = Показать текущий статус времени
timedatectl.help.cmd.show = Показать статус в JSON
timedatectl.help.cmd.set_time = Установить системное время
timedatectl.help.cmd.set_timezone = Установить системный часовой пояс
timedatectl.help.cmd.list_timezones = Показать доступные часовые пояса
timedatectl.help.cmd.set_local_rtc = Установить RTC на локальное время (true/false)
timedatectl.help.cmd.set_ntp = Включить или отключить синхронизацию NTP (true/false)
timedatectl.help.cmd.timesync_status = Показать статус синхронизации времени
timedatectl.help.cmd.show_timesync = Показать статус синхронизации в JSON
timedatectl.help.cmd.add_ntp_server = Добавить сервер NTP
timedatectl.help.cmd.remove_ntp_server = Удалить сервер NTP
timedatectl.help.cmd.statistics = Показать статистику времени
timedatectl.help.cmd.history = Показать историю корректировок времени

# Справка timedatectl — параметры
timedatectl.help.opt.help = Показать эту справку и выйти
timedatectl.help.opt.monitor = Запустить режим мониторинга в реальном времени
timedatectl.help.opt.all = Показать все свойства
timedatectl.help.opt.json = Вывод в JSON

# Принимаемые форматы времени
timedatectl.help.fmt.full_datetime = Полные дата и время
timedatectl.help.fmt.datetime_no_sec = Дата и время без секунд
timedatectl.help.fmt.time_only = Только время
timedatectl.help.fmt.time_no_sec = Время (без секунд)
timedatectl.help.fmt.unix_timestamp = Unix-временная метка (секунды)
timedatectl.help.fmt.iso8601 = ISO 8601 (UTC)

# Примеры справки
timedatectl.help.ex.status = показать статус
timedatectl.help.ex.set_time = установить системное время
timedatectl.help.ex.set_timezone = установить часовой пояс
timedatectl.help.ex.find_timezone = найти часовой пояс
timedatectl.help.ex.enable_ntp = включить синхронизацию NTP
timedatectl.help.ex.add_server = добавить сервер NTP
timedatectl.help.ex.sync_status = показать статус синхронизации
timedatectl.help.ex.statistics = показать статистику

# Просмотр свойств
timedatectl.properties.title = Свойства даты и времени
timedatectl.properties.time_info = Информация о времени
timedatectl.properties.local_time = Локальное время
timedatectl.properties.utc_time = Время UTC
timedatectl.properties.timezone_info = Информация о часовом поясе
timedatectl.properties.timezone = Часовой пояс
timedatectl.properties.utc_offset = Смещение UTC
timedatectl.properties.dst_active = Летнее время активно
timedatectl.properties.sync_status = Статус синхронизации
timedatectl.properties.system_synced = Системные часы синхронизированы
timedatectl.properties.ntp_service = Служба NTP
timedatectl.properties.time_source = Источник времени
timedatectl.properties.sync_accuracy = Точность синхронизации
timedatectl.properties.last_sync = Последняя синхронизация
timedatectl.properties.drift_rate = Скорость дрейфа (ppm)
timedatectl.properties.leap_info = Информация о високосной секунде
timedatectl.properties.leap_pending = Ожидается високосная секунда
timedatectl.properties.ntp_config = Конфигурация NTP
timedatectl.properties.ntp_enabled = NTP включен
timedatectl.properties.ntp_servers = NTP-серверы
timedatectl.properties.min_poll = Минимальный интервал опроса
timedatectl.properties.max_poll = Максимальный интервал опроса
timedatectl.properties.capabilities = Возможности системы
timedatectl.properties.tz_changes = Изменения часового пояса
timedatectl.properties.ntp_sync = Синхронизация NTP
timedatectl.properties.rtc_access = Доступ к RTC
timedatectl.properties.hw_timestamp = Аппаратная временная метка

# Общие метки
common.yes = да
common.no = нет
common.supported = поддерживается
common.limited = ограничено
common.full = полный
common.available = доступно

# Единицы измерения
units.microseconds = микросекунды

# команда date (метаданные/относительное/праздники)
date.error.invalid_timezone = Неверный часовой пояс: {$tz}
date.error.invalid_month = Неверный месяц: {$month}

date.metadata.unix_timestamp = Unix-временная метка: {$value}
date.metadata.julian_day = Юлианский день: {$value}
date.metadata.day_of_year = День года: {$value}
date.metadata.week_number = Номер недели: {$value}
date.metadata.weekday = День недели: {$value}
date.metadata.type.weekend = Тип: выходной
date.metadata.type.business = Тип: рабочий день
date.metadata.astronomical = Астрономическое: {$info}

date.relative.now = сейчас
date.relative.minutes_ago = {$mins} минут назад
date.relative.in_minutes = через {$mins} минут
date.relative.hours_ago = {$hours} часов назад
date.relative.in_hours = через {$hours} часов
date.relative.days_ago = {$days} дней назад
date.relative.in_days = через {$days} дней

date.holiday.none = Праздники за {$year} год в регионах {$regions} не найдены
date.holiday.header = Праздники {$year} года в регионах: {$regions}
date.holiday.separator = =====================================
date.holiday.entry = {$date} - {$name} ({$region}, {$kind})
date.holiday.total = Итого: {$count} праздников