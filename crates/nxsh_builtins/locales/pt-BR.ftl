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
cat-progress-complete = Complete
cat-error-invalid-file-url = Invalid file URL
cat-error-invalid-base64 = Invalid base64 in data URL: {$error}
cat-error-malformed-data-url = Malformed data URL
cat-error-unsupported-url-scheme = Unsupported URL scheme: {$scheme}
cat-error-http-request-failed = HTTP request failed: {$error}
cat-error-http-feature-missing = URL support requires 'net-http' feature
cat-help-option-e-short-desc = equivalent to -vE
cat-help-option-t-short-desc = equivalent to -vT
cat-help-option-u-ignored = (ignored)
# NexusShell Comandos Integrados - Localização Português Brasileiro

# Mensagens comuns
error-file-not-found = Arquivo não encontrado: {$filename}
error-permission-denied = Permissão negada: {$filename}
error-invalid-option = Opção inválida: {$option}
error-missing-argument = Argumento ausente para a opção: {$option}
error-invalid-argument = Argumento inválido: {$argument}
error-directory-not-found = Diretório não encontrado: {$dirname}
error-not-a-directory = Não é um diretório: {$path}
error-not-a-file = Não é um arquivo: {$path}
error-operation-failed = Operação falhou: {$operation}
error-io-error = Erro de E/S: {$message}

# Comando cat
cat-help-usage = Uso: cat [OPÇÃO]... [ARQUIVO]...
cat-help-description = Concatenar ARQUIVO(s) para a saída padrão.
cat-version = cat (NexusShell) 1.0.0

# Comando ls
ls-help-usage = Uso: ls [OPÇÃO]... [ARQUIVO]...
ls-help-description = Listar informações sobre ARQUIVO(s) (diretório atual por padrão).
ls-permission-read = leitura
ls-permission-write = escrita
ls-permission-execute = execução
ls-type-directory = diretório
ls-type-file = arquivo regular
ls-type-symlink = link simbólico

# Comando grep
grep-help-usage = Uso: grep [OPÇÃO]... PADRÃO [ARQUIVO]...
grep-help-description = Procurar PADRÃO em cada ARQUIVO.
grep-matches-found = {$count} correspondências encontradas
grep-no-matches = Nenhuma correspondência encontrada

# Comando ps
ps-help-usage = Uso: ps [OPÇÃO]...
ps-help-description = Exibir informações sobre processos em execução.
ps-header-pid = PID
ps-header-user = USUÁRIO
ps-header-command = COMANDO

# Comando ping
ping-help-usage = Uso: ping [OPÇÃO]... HOST
ping-help-description = Enviar ICMP ECHO_REQUEST para hosts de rede.
ping-statistics = --- estatísticas de ping de {$host} ---
ping-packets-transmitted = {$transmitted} pacotes transmitidos
ping-packets-received = {$received} recebidos
ping-packet-loss = {$loss}% perda de pacotes

# Operações de arquivo comuns
file-exists = Arquivo existe: {$filename}
file-not-exists = Arquivo não existe: {$filename}
operation-cancelled = Operação cancelada
operation-completed = Operação concluída com sucesso 

# timedatectl status/common/time-sync labels
timedatectl.common.yes = sim
timedatectl.common.no = não
timedatectl.common.enabled = habilitado
timedatectl.common.disabled = desabilitado
timedatectl.common.reachable = alcançável
timedatectl.common.unreachable = inalcançável

timedatectl.msg.time_set_to = Hora definida para:
timedatectl.msg.timezone_set_to = Fuso horário definido para:
timedatectl.msg.rtc_in_local_tz = RTC no fuso horário local:
timedatectl.msg.ntp_sync = Sincronização NTP:
timedatectl.msg.added_ntp_server = Servidor NTP adicionado:
timedatectl.msg.removed_ntp_server = Servidor NTP removido:

timedatectl.timesync.title = Status da sincronização de hora:
timedatectl.timesync.enabled = Habilitado:
timedatectl.timesync.synchronized = Sincronizado:
timedatectl.timesync.last_sync = Última sincronização:
timedatectl.timesync.sync_accuracy = Precisão da sincronização:
timedatectl.timesync.drift_rate = Taxa de deriva:
timedatectl.timesync.poll_interval = Intervalo de sondagem:
timedatectl.timesync.leap_status = Status de segundos intercalares:
timedatectl.timesync.ntp_servers = Servidores NTP:
timedatectl.timesync.stratum = Estrato:
timedatectl.timesync.delay = Atraso:
timedatectl.timesync.offset = Deslocamento:
timedatectl.timesync.summary = Resumo:
timedatectl.timesync.servers_total_reachable = Servidores (total/alcançáveis):
timedatectl.timesync.best_stratum = Melhor estrato:
timedatectl.timesync.preferred_server = Servidor preferido:
timedatectl.timesync.avg_delay = Atraso médio:
timedatectl.timesync.min_delay = Atraso mínimo:
timedatectl.timesync.max_delay = Atraso máximo:
timedatectl.timesync.avg_offset = Deslocamento médio:
timedatectl.timesync.min_offset = Deslocamento mínimo:
timedatectl.timesync.max_offset = Deslocamento máximo:
timedatectl.timesync.avg_jitter = Jitter médio:

# rótulos de status do timedatectl
timedatectl.status.local_time = Hora local

# Cabeçalhos de ajuda do timedatectl
timedatectl.help.title = timedatectl: Gerenciamento de Data e Hora
timedatectl.help.usage = Uso:
timedatectl.help.commands = Comandos:
timedatectl.help.options = Opções:
timedatectl.help.time_formats = Formatos de hora aceitos:
timedatectl.help.examples = Exemplos:
timedatectl.help.timesync_options = Opções para timesync-status:
timedatectl.help.timesync_json_option =   -J, --json            Exibir status e resumo como JSON compacto
timedatectl.help.global_json_option =   Global: alguns comandos aceitam -J/--json para saída JSON
timedatectl.status.universal_time = Tempo universal (UTC)
timedatectl.status.rtc_time = Hora do RTC
timedatectl.status.time_zone = Fuso horário
timedatectl.status.system_clock_synchronized = Relógio do sistema sincronizado
timedatectl.status.ntp_service = Serviço NTP
timedatectl.status.rtc_in_local_tz = RTC no fuso local
timedatectl.status.sync_accuracy = Precisão de sincronização
timedatectl.status.drift_rate = Taxa de deriva
timedatectl.status.last_sync = Última sincronização
timedatectl.status.leap_second = Segundo intercalar
timedatectl.status.pending = pendente

# timedatectl ajuda — comandos
timedatectl.help.cmd.status = Mostrar status de hora atual
timedatectl.help.cmd.show = Mostrar status em JSON
timedatectl.help.cmd.set_time = Definir a hora do sistema
timedatectl.help.cmd.set_timezone = Definir o fuso horário do sistema
timedatectl.help.cmd.list_timezones = Listar fusos horários disponíveis
timedatectl.help.cmd.set_local_rtc = Definir RTC para hora local (true/false)
timedatectl.help.cmd.set_ntp = Habilitar ou desabilitar sincronização NTP (true/false)
timedatectl.help.cmd.timesync_status = Mostrar status da sincronização de hora
timedatectl.help.cmd.show_timesync = Mostrar status de sincronização em JSON
timedatectl.help.cmd.add_ntp_server = Adicionar um servidor NTP
timedatectl.help.cmd.remove_ntp_server = Remover um servidor NTP
timedatectl.help.cmd.statistics = Mostrar estatísticas de tempo
timedatectl.help.cmd.history = Mostrar histórico de ajustes de hora

# timedatectl ajuda — opções
timedatectl.help.opt.help = Mostrar esta ajuda e sair
timedatectl.help.opt.monitor = Executar modo de monitoramento em tempo real
timedatectl.help.opt.all = Mostrar todas as propriedades
timedatectl.help.opt.json = Saída em JSON

# Formatos de hora aceitos
timedatectl.help.fmt.full_datetime = Data e hora completas
timedatectl.help.fmt.datetime_no_sec = Data e hora sem segundos
timedatectl.help.fmt.time_only = Somente hora
timedatectl.help.fmt.time_no_sec = Hora (sem segundos)
timedatectl.help.fmt.unix_timestamp = Timestamp Unix (segundos)
timedatectl.help.fmt.iso8601 = ISO 8601 (UTC)

# Exemplos de ajuda
timedatectl.help.ex.status = mostrar status
timedatectl.help.ex.set_time = definir hora do sistema
timedatectl.help.ex.set_timezone = definir fuso horário
timedatectl.help.ex.find_timezone = encontrar um fuso horário
timedatectl.help.ex.enable_ntp = habilitar sincronização NTP
timedatectl.help.ex.add_server = adicionar servidor NTP
timedatectl.help.ex.sync_status = mostrar status de sincronização
timedatectl.help.ex.statistics = mostrar estatísticas

# Visualização de propriedades
timedatectl.properties.title = Propriedades de Data e Hora
timedatectl.properties.time_info = Informações de tempo
timedatectl.properties.local_time = Hora local
timedatectl.properties.utc_time = Hora UTC
timedatectl.properties.timezone_info = Informações de fuso horário
timedatectl.properties.timezone = Fuso horário
timedatectl.properties.utc_offset = Deslocamento UTC
timedatectl.properties.dst_active = Horário de verão ativo
timedatectl.properties.sync_status = Status de sincronização
timedatectl.properties.system_synced = Relógio do sistema sincronizado
timedatectl.properties.ntp_service = Serviço NTP
timedatectl.properties.time_source = Fonte de tempo
timedatectl.properties.sync_accuracy = Precisão de sincronização
timedatectl.properties.last_sync = Última sincronização
timedatectl.properties.drift_rate = Taxa de deriva (ppm)
timedatectl.properties.leap_info = Informações de segundos intercalares
timedatectl.properties.leap_pending = Segundo intercalar pendente
timedatectl.properties.ntp_config = Configuração NTP
timedatectl.properties.ntp_enabled = NTP habilitado
timedatectl.properties.ntp_servers = Servidores NTP
timedatectl.properties.min_poll = Intervalo mínimo de sondagem
timedatectl.properties.max_poll = Intervalo máximo de sondagem
timedatectl.properties.capabilities = Capacidades do sistema
timedatectl.properties.tz_changes = Mudanças de fuso horário
timedatectl.properties.ntp_sync = Sincronização NTP
timedatectl.properties.rtc_access = Acesso ao RTC
timedatectl.properties.hw_timestamp = Marcação de tempo de HW

# Rótulos genéricos
common.yes = sim
common.no = não
common.supported = suportado
common.limited = limitado
common.full = completo
common.available = disponível

# Unidades
units.microseconds = microssegundos

# comando date (metadados/relativo/feriados)
date.error.invalid_timezone = Fuso horário inválido: {$tz}
date.error.invalid_month = Mês inválido: {$month}

date.metadata.unix_timestamp = Timestamp Unix: {$value}
date.metadata.julian_day = Dia Juliano: {$value}
date.metadata.day_of_year = Dia do ano: {$value}
date.metadata.week_number = Número da semana: {$value}
date.metadata.weekday = Dia da semana: {$value}
date.metadata.type.weekend = Tipo: fim de semana
date.metadata.type.business = Tipo: dia útil
date.metadata.astronomical = Astronômico: {$info}

date.relative.now = agora
date.relative.minutes_ago = há {$mins} minutos
date.relative.in_minutes = em {$mins} minutos
date.relative.hours_ago = há {$hours} horas
date.relative.in_hours = em {$hours} horas
date.relative.days_ago = há {$days} dias
date.relative.in_days = em {$days} dias

date.holiday.none = Nenhum feriado encontrado para o ano {$year} nas regiões: {$regions}
date.holiday.header = Feriados {$year} nas regiões: {$regions}
date.holiday.separator = =====================================
date.holiday.entry = {$date} - {$name} ({$region}, {$kind})
date.holiday.total = Total: {$count} feriados