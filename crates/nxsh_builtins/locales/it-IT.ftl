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
# NexusShell Comandi Integrati - Localizzazione Italiana

# Messaggi comuni
error-file-not-found = File non trovato: {$filename}
error-permission-denied = Permesso negato: {$filename}
error-invalid-option = Opzione non valida: {$option}
error-missing-argument = Argomento mancante per l'opzione: {$option}
error-invalid-argument = Argomento non valido: {$argument}
error-directory-not-found = Directory non trovata: {$dirname}
error-not-a-directory = Non è una directory: {$path}
error-not-a-file = Non è un file: {$path}
error-operation-failed = Operazione fallita: {$operation}
error-io-error = Errore I/O: {$message}

# Comando cat
cat-help-usage = Utilizzo: cat [OPZIONE]... [FILE]...
cat-help-description = Concatenare FILE verso l'output standard.
cat-version = cat (NexusShell) 1.0.0

# Comando ls
ls-help-usage = Utilizzo: ls [OPZIONE]... [FILE]...
ls-help-description = Elencare informazioni sui FILE (directory corrente per impostazione predefinita).
ls-permission-read = lettura
ls-permission-write = scrittura
ls-permission-execute = esecuzione
ls-type-directory = directory
ls-type-file = file regolare
ls-type-symlink = collegamento simbolico

# Comando grep
grep-help-usage = Utilizzo: grep [OPZIONE]... PATTERN [FILE]...
grep-help-description = Cercare PATTERN in ogni FILE.
grep-matches-found = {$count} corrispondenze trovate
grep-no-matches = Nessuna corrispondenza trovata

# Comando ps
ps-help-usage = Utilizzo: ps [OPZIONE]...
ps-help-description = Mostrare informazioni sui processi in esecuzione.
ps-header-pid = PID
ps-header-user = UTENTE
ps-header-command = COMANDO

# Comando ping
ping-help-usage = Utilizzo: ping [OPZIONE]... HOST
ping-help-description = Inviare ICMP ECHO_REQUEST agli host di rete.
ping-statistics = --- statistiche ping di {$host} ---
ping-packets-transmitted = {$transmitted} pacchetti trasmessi
ping-packets-received = {$received} ricevuti
ping-packet-loss = {$loss}% perdita pacchetti

# Operazioni file comuni
file-exists = Il file esiste: {$filename}
file-not-exists = Il file non esiste: {$filename}
operation-cancelled = Operazione annullata
operation-completed = Operazione completata con successo 

# timedatectl status/common/time-sync labels
timedatectl.common.yes = sì
timedatectl.common.no = no
timedatectl.common.enabled = abilitato
timedatectl.common.disabled = disabilitato
timedatectl.common.reachable = raggiungibile
timedatectl.common.unreachable = irraggiungibile

timedatectl.msg.time_set_to = Ora impostata su:
timedatectl.msg.timezone_set_to = Fuso orario impostato su:
timedatectl.msg.rtc_in_local_tz = RTC nel fuso orario locale:
timedatectl.msg.ntp_sync = Sincronizzazione NTP:
timedatectl.msg.added_ntp_server = Server NTP aggiunto:
timedatectl.msg.removed_ntp_server = Server NTP rimosso:

timedatectl.timesync.title = Stato sincronizzazione oraria:
timedatectl.timesync.enabled = Abilitato:
timedatectl.timesync.synchronized = Sincronizzato:
timedatectl.timesync.last_sync = Ultima sincronizzazione:
timedatectl.timesync.sync_accuracy = Precisione di sincronizzazione:
timedatectl.timesync.drift_rate = Tasso di deriva:
timedatectl.timesync.poll_interval = Intervallo di polling:
timedatectl.timesync.leap_status = Stato secondi intercalari:
timedatectl.timesync.ntp_servers = Server NTP:
timedatectl.timesync.stratum = Stratum:
timedatectl.timesync.delay = Ritardo:
timedatectl.timesync.offset = Scarto:
timedatectl.timesync.summary = Riepilogo:
timedatectl.timesync.servers_total_reachable = Server (totale/raggiungibili):
timedatectl.timesync.best_stratum = Miglior stratum:
timedatectl.timesync.preferred_server = Server preferito:
timedatectl.timesync.avg_delay = Ritardo medio:
timedatectl.timesync.min_delay = Ritardo minimo:
timedatectl.timesync.max_delay = Ritardo massimo:
timedatectl.timesync.avg_offset = Scarto medio:
timedatectl.timesync.min_offset = Scarto minimo:
timedatectl.timesync.max_offset = Scarto massimo:
timedatectl.timesync.avg_jitter = Jitter medio:

# Etichette stato timedatectl
timedatectl.help.title = timedatectl: Gestione di Data e Ora
timedatectl.help.usage = Utilizzo:
timedatectl.help.commands = Comandi:
timedatectl.help.options = Opzioni:
timedatectl.help.time_formats = Formati orari accettati:
timedatectl.help.examples = Esempi:
timedatectl.help.timesync_options = Opzioni per timesync-status:
timedatectl.help.timesync_json_option =   -J, --json            Mostra stato e riepilogo come JSON compatto
timedatectl.help.global_json_option =   Globale: alcuni comandi accettano -J/--json per l'output JSON
timedatectl.status.local_time = Ora locale
timedatectl.status.universal_time = Tempo universale (UTC)
timedatectl.status.rtc_time = Ora RTC
timedatectl.status.time_zone = Fuso orario
timedatectl.status.system_clock_synchronized = Orologio di sistema sincronizzato
timedatectl.status.ntp_service = Servizio NTP
timedatectl.status.rtc_in_local_tz = RTC nel fuso locale
timedatectl.status.sync_accuracy = Precisione di sincronizzazione
timedatectl.status.drift_rate = Tasso di deriva
timedatectl.status.last_sync = Ultima sincronizzazione
timedatectl.status.leap_second = Secondo intercalare
timedatectl.status.pending = in sospeso

# timedatectl guida — comandi
timedatectl.help.cmd.status = Mostra stato orario corrente
timedatectl.help.cmd.show = Mostra stato in JSON
timedatectl.help.cmd.set_time = Imposta l'ora di sistema
timedatectl.help.cmd.set_timezone = Imposta il fuso orario di sistema
timedatectl.help.cmd.list_timezones = Elenca i fusi orari disponibili
timedatectl.help.cmd.set_local_rtc = Imposta RTC all'ora locale (true/false)
timedatectl.help.cmd.set_ntp = Abilita o disabilita la sincronizzazione NTP (true/false)
timedatectl.help.cmd.timesync_status = Mostra stato sincronizzazione oraria
timedatectl.help.cmd.show_timesync = Mostra stato di sincronizzazione in JSON
timedatectl.help.cmd.add_ntp_server = Aggiungi un server NTP
timedatectl.help.cmd.remove_ntp_server = Rimuovi un server NTP
timedatectl.help.cmd.statistics = Mostra statistiche temporali
timedatectl.help.cmd.history = Mostra cronologia delle regolazioni orarie

# timedatectl guida — opzioni
timedatectl.help.opt.help = Mostra questo aiuto ed esce
timedatectl.help.opt.monitor = Esegui in modalità monitoraggio in tempo reale
timedatectl.help.opt.all = Mostra tutte le proprietà
timedatectl.help.opt.json = Output in JSON

# Formati orari accettati
timedatectl.help.fmt.full_datetime = Data e ora complete
timedatectl.help.fmt.datetime_no_sec = Data e ora senza secondi
timedatectl.help.fmt.time_only = Solo ora
timedatectl.help.fmt.time_no_sec = Ora (senza secondi)
timedatectl.help.fmt.unix_timestamp = Timestamp Unix (secondi)
timedatectl.help.fmt.iso8601 = ISO 8601 (UTC)

# Esempi di guida
timedatectl.help.ex.status = mostra stato
timedatectl.help.ex.set_time = imposta ora di sistema
timedatectl.help.ex.set_timezone = imposta fuso orario
timedatectl.help.ex.find_timezone = trova un fuso orario
timedatectl.help.ex.enable_ntp = abilita sincronizzazione NTP
timedatectl.help.ex.add_server = aggiungi server NTP
timedatectl.help.ex.sync_status = mostra stato di sincronizzazione
timedatectl.help.ex.statistics = mostra statistiche

# Vista proprietà
timedatectl.properties.title = Proprietà Data e Ora
timedatectl.properties.time_info = Informazioni sull'ora
timedatectl.properties.local_time = Ora locale
timedatectl.properties.utc_time = Ora UTC
timedatectl.properties.timezone_info = Informazioni sul fuso orario
timedatectl.properties.timezone = Fuso orario
timedatectl.properties.utc_offset = Offset UTC
timedatectl.properties.dst_active = Ora legale attiva
timedatectl.properties.sync_status = Stato di sincronizzazione
timedatectl.properties.system_synced = Orologio di sistema sincronizzato
timedatectl.properties.ntp_service = Servizio NTP
timedatectl.properties.time_source = Sorgente oraria
timedatectl.properties.sync_accuracy = Precisione di sincronizzazione
timedatectl.properties.last_sync = Ultima sincronizzazione
timedatectl.properties.drift_rate = Tasso di deriva (ppm)
timedatectl.properties.leap_info = Informazioni sui secondi intercalari
timedatectl.properties.leap_pending = Secondo intercalare in sospeso
timedatectl.properties.ntp_config = Configurazione NTP
timedatectl.properties.ntp_enabled = NTP abilitato
timedatectl.properties.ntp_servers = Server NTP
timedatectl.properties.min_poll = Intervallo minimo di polling
timedatectl.properties.max_poll = Intervallo massimo di polling
timedatectl.properties.capabilities = Capacità del sistema
timedatectl.properties.tz_changes = Cambi di fuso orario
timedatectl.properties.ntp_sync = Sincronizzazione NTP
timedatectl.properties.rtc_access = Accesso RTC
timedatectl.properties.hw_timestamp = Marcatura temporale HW

# Etichette generiche
common.yes = sì
common.no = no
common.supported = supportato
common.limited = limitato
common.full = completo
common.available = disponibile

# Unità
units.microseconds = microsecondi

# comando date (metadati/relativo/festività)
date.error.invalid_timezone = Fuso orario non valido: {$tz}
date.error.invalid_month = Mese non valido: {$month}

date.metadata.unix_timestamp = Timestamp Unix: {$value}
date.metadata.julian_day = Giorno giuliano: {$value}
date.metadata.day_of_year = Giorno dell'anno: {$value}
date.metadata.week_number = Numero settimana: {$value}
date.metadata.weekday = Giorno della settimana: {$value}
date.metadata.type.weekend = Tipo: weekend
date.metadata.type.business = Tipo: giorno lavorativo
date.metadata.astronomical = Astronomico: {$info}

date.relative.now = ora
date.relative.minutes_ago = {$mins} minuti fa
date.relative.in_minutes = tra {$mins} minuti
date.relative.hours_ago = {$hours} ore fa
date.relative.in_hours = tra {$hours} ore
date.relative.days_ago = {$days} giorni fa
date.relative.in_days = tra {$days} giorni

date.holiday.none = Nessuna festività trovata per l'anno {$year} nelle regioni: {$regions}
date.holiday.header = Festività {$year} nelle regioni: {$regions}
date.holiday.separator = =====================================
date.holiday.entry = {$date} - {$name} ({$region}, {$kind})
date.holiday.total = Totale: {$count} festività