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
# NexusShell Comandos Integrados - Localización en Español

# Mensajes comunes
error-file-not-found = Archivo no encontrado: {$filename}
error-permission-denied = Permiso denegado: {$filename}
error-invalid-option = Opción inválida: {$option}
error-missing-argument = Falta argumento para la opción: {$option}
error-invalid-argument = Argumento inválido: {$argument}
error-directory-not-found = Directorio no encontrado: {$dirname}
error-not-a-directory = No es un directorio: {$path}
error-not-a-file = No es un archivo: {$path}
error-operation-failed = Operación fallida: {$operation}
error-io-error = Error de E/S: {$message}

# Comando cat
cat-help-usage = Uso: cat [OPCIÓN]... [ARCHIVO]...
cat-help-description = Concatenar ARCHIVO(s) a la salida estándar.
cat-version = cat (NexusShell) 1.0.0

# Comando ls
ls-help-usage = Uso: ls [OPCIÓN]... [ARCHIVO]...
ls-help-description = Listar información sobre los ARCHIVO(s) (el directorio actual por defecto).
ls-permission-read = lectura
ls-permission-write = escritura
ls-permission-execute = ejecución
ls-type-directory = directorio
ls-type-file = archivo regular
ls-type-symlink = enlace simbólico

# Comando grep
grep-help-usage = Uso: grep [OPCIÓN]... PATRÓN [ARCHIVO]...
grep-help-description = Buscar PATRÓN en cada ARCHIVO.
grep-matches-found = {$count} coincidencias encontradas
grep-no-matches = No se encontraron coincidencias

# Comando ps
ps-help-usage = Uso: ps [OPCIÓN]...
ps-help-description = Mostrar información sobre procesos en ejecución.
ps-header-pid = PID
ps-header-user = USUARIO
ps-header-command = COMANDO

# Comando ping
ping-help-usage = Uso: ping [OPCIÓN]... HOST
ping-help-description = Enviar ICMP ECHO_REQUEST a hosts de red.
ping-statistics = --- estadísticas de ping de {$host} ---
ping-packets-transmitted = {$transmitted} paquetes transmitidos
ping-packets-received = {$received} recibidos
ping-packet-loss = {$loss}% pérdida de paquetes

# Operaciones de archivo comunes
file-exists = El archivo existe: {$filename}
file-not-exists = El archivo no existe: {$filename}
operation-cancelled = Operación cancelada
operation-completed = Operación completada exitosamente 

# timedatectl status/common/time-sync labels
timedatectl.common.yes = sí
timedatectl.common.no = no
timedatectl.common.enabled = habilitado
timedatectl.common.disabled = deshabilitado
timedatectl.common.reachable = alcanzable
timedatectl.common.unreachable = inalcanzable

timedatectl.msg.time_set_to = Hora establecida a:
timedatectl.msg.timezone_set_to = Zona horaria establecida a:
timedatectl.msg.rtc_in_local_tz = RTC en zona horaria local:
timedatectl.msg.ntp_sync = Sincronización NTP:
timedatectl.msg.added_ntp_server = Servidor NTP agregado:
timedatectl.msg.removed_ntp_server = Servidor NTP eliminado:

timedatectl.timesync.title = Estado de sincronización de tiempo:
timedatectl.timesync.enabled = Habilitado:
timedatectl.timesync.synchronized = Sincronizado:
timedatectl.timesync.last_sync = Última sincronización:
timedatectl.timesync.sync_accuracy = Precisión de sincronización:
timedatectl.timesync.drift_rate = Tasa de deriva:
timedatectl.timesync.poll_interval = Intervalo de sondeo:
timedatectl.timesync.leap_status = Estado de segundos intercalares:
timedatectl.timesync.ntp_servers = Servidores NTP:
timedatectl.timesync.stratum = Estrato:
timedatectl.timesync.delay = Retardo:
timedatectl.timesync.offset = Desfase:
timedatectl.timesync.summary = Resumen:
timedatectl.timesync.servers_total_reachable = Servidores (total/alcanzables):
timedatectl.timesync.best_stratum = Mejor estrato:
timedatectl.timesync.preferred_server = Servidor preferido:
timedatectl.timesync.avg_delay = Retardo promedio:
timedatectl.timesync.min_delay = Retardo mínimo:
timedatectl.timesync.max_delay = Retardo máximo:
timedatectl.timesync.avg_offset = Desfase promedio:
timedatectl.timesync.min_offset = Desfase mínimo:
timedatectl.timesync.max_offset = Desfase máximo:
timedatectl.timesync.avg_jitter = Jitter promedio:

# timedatectl etiquetas de estado
timedatectl.help.title = timedatectl: Gestión de Hora y Fecha
timedatectl.help.usage = Uso:
timedatectl.help.commands = Comandos:
timedatectl.help.options = Opciones:
timedatectl.help.time_formats = Formatos de hora aceptados:
timedatectl.help.examples = Ejemplos:
timedatectl.help.timesync_options = Opciones para timesync-status:
timedatectl.help.timesync_json_option =   -J, --json            Mostrar estado y resumen como JSON compacto
timedatectl.help.global_json_option =   Global: algunos comandos aceptan -J/--json para salida JSON
timedatectl.status.local_time = Hora local
timedatectl.status.universal_time = Hora universal (UTC)
timedatectl.status.rtc_time = Hora RTC
timedatectl.status.time_zone = Zona horaria
timedatectl.status.system_clock_synchronized = Reloj del sistema sincronizado
timedatectl.status.ntp_service = Servicio NTP
timedatectl.status.rtc_in_local_tz = RTC en zona local
timedatectl.status.sync_accuracy = Precisión de sincronización
timedatectl.status.drift_rate = Tasa de deriva
timedatectl.status.last_sync = Última sincronización
timedatectl.status.leap_second = Segundo intercalar
timedatectl.status.pending = pendiente

# timedatectl ayuda — comandos
timedatectl.help.cmd.status = Mostrar el estado de la hora actual
timedatectl.help.cmd.show = Mostrar el estado en JSON
timedatectl.help.cmd.set_time = Establecer la hora del sistema
timedatectl.help.cmd.set_timezone = Establecer la zona horaria del sistema
timedatectl.help.cmd.list_timezones = Listar zonas horarias disponibles
timedatectl.help.cmd.set_local_rtc = Establecer RTC a hora local (true/false)
timedatectl.help.cmd.set_ntp = Habilitar o deshabilitar la sincronización NTP (true/false)
timedatectl.help.cmd.timesync_status = Mostrar el estado de sincronización de hora
timedatectl.help.cmd.show_timesync = Mostrar el estado de sincronización en JSON
timedatectl.help.cmd.add_ntp_server = Agregar un servidor NTP
timedatectl.help.cmd.remove_ntp_server = Eliminar un servidor NTP
timedatectl.help.cmd.statistics = Mostrar estadísticas de tiempo
timedatectl.help.cmd.history = Mostrar historial de ajustes de hora

# timedatectl ayuda — opciones
timedatectl.help.opt.help = Mostrar esta ayuda y salir
timedatectl.help.opt.monitor = Ejecutar modo de monitoreo en tiempo real
timedatectl.help.opt.all = Mostrar todas las propiedades
timedatectl.help.opt.json = Salida en JSON

# Formatos de hora aceptados
timedatectl.help.fmt.full_datetime = Fecha y hora completa
timedatectl.help.fmt.datetime_no_sec = Fecha y hora sin segundos
timedatectl.help.fmt.time_only = Solo hora
timedatectl.help.fmt.time_no_sec = Hora (sin segundos)
timedatectl.help.fmt.unix_timestamp = Marca de tiempo Unix (segundos)
timedatectl.help.fmt.iso8601 = ISO 8601 (UTC)

# Ejemplos de ayuda
timedatectl.help.ex.status = mostrar estado
timedatectl.help.ex.set_time = establecer hora del sistema
timedatectl.help.ex.set_timezone = establecer zona horaria
timedatectl.help.ex.find_timezone = encontrar una zona horaria
timedatectl.help.ex.enable_ntp = habilitar sincronización NTP
timedatectl.help.ex.add_server = agregar servidor NTP
timedatectl.help.ex.sync_status = mostrar estado de sincronización
timedatectl.help.ex.statistics = mostrar estadísticas

# Vista de propiedades
timedatectl.properties.title = Propiedades de Fecha y Hora
timedatectl.properties.time_info = Información de tiempo
timedatectl.properties.local_time = Hora local
timedatectl.properties.utc_time = Hora UTC
timedatectl.properties.timezone_info = Información de zona horaria
timedatectl.properties.timezone = Zona horaria
timedatectl.properties.utc_offset = Desfase UTC
timedatectl.properties.dst_active = Horario de verano activo
timedatectl.properties.sync_status = Estado de sincronización
timedatectl.properties.system_synced = Reloj del sistema sincronizado
timedatectl.properties.ntp_service = Servicio NTP
timedatectl.properties.time_source = Fuente de tiempo
timedatectl.properties.sync_accuracy = Precisión de sincronización
timedatectl.properties.last_sync = Última sincronización
timedatectl.properties.drift_rate = Tasa de deriva (ppm)
timedatectl.properties.leap_info = Información de segundo intercalar
timedatectl.properties.leap_pending = Segundo intercalar pendiente
timedatectl.properties.ntp_config = Configuración de NTP
timedatectl.properties.ntp_enabled = NTP habilitado
timedatectl.properties.ntp_servers = Servidores NTP
timedatectl.properties.min_poll = Intervalo mínimo de sondeo
timedatectl.properties.max_poll = Intervalo máximo de sondeo
timedatectl.properties.capabilities = Capacidades del sistema
timedatectl.properties.tz_changes = Cambios de zona horaria
timedatectl.properties.ntp_sync = Sincronización NTP
timedatectl.properties.rtc_access = Acceso a RTC
timedatectl.properties.hw_timestamp = Marcas de tiempo de HW

# Etiquetas genéricas
common.yes = sí
common.no = no
common.supported = compatible
common.limited = limitado
common.full = completo
common.available = disponible

# Unidades
units.microseconds = microsegundos

# comando date (metadatos/relativo/feriados)
date.error.invalid_timezone = Zona horaria inválida: {$tz}
date.error.invalid_month = Mes inválido: {$month}

date.metadata.unix_timestamp = Marca de tiempo Unix: {$value}
date.metadata.julian_day = Día juliano: {$value}
date.metadata.day_of_year = Día del año: {$value}
date.metadata.week_number = Número de semana: {$value}
date.metadata.weekday = Día de la semana: {$value}
date.metadata.type.weekend = Tipo: fin de semana
date.metadata.type.business = Tipo: día laborable
date.metadata.astronomical = Astronómico: {$info}

date.relative.now = ahora
date.relative.minutes_ago = hace {$mins} minutos
date.relative.in_minutes = en {$mins} minutos
date.relative.hours_ago = hace {$hours} horas
date.relative.in_hours = en {$hours} horas
date.relative.days_ago = hace {$days} días
date.relative.in_days = en {$days} días

date.holiday.none = No se encontraron feriados para el año {$year} en las regiones: {$regions}
date.holiday.header = Feriados {$year} en las regiones: {$regions}
date.holiday.separator = =====================================
date.holiday.entry = {$date} - {$name} ({$region}, {$kind})
date.holiday.total = Total: {$count} feriados