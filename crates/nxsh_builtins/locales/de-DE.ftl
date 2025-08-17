# at help detailed sections (fallback to English)
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
# NexusShell Eingebaute Befehle - Deutsche Lokalisierung

# Allgemeine Nachrichten
error-file-not-found = Datei nicht gefunden: {$filename}
error-permission-denied = Berechtigung verweigert: {$filename}
error-invalid-option = Ungültige Option: {$option}
error-missing-argument = Fehlendes Argument für Option: {$option}
error-invalid-argument = Ungültiges Argument: {$argument}
error-directory-not-found = Verzeichnis nicht gefunden: {$dirname}
error-not-a-directory = Ist kein Verzeichnis: {$path}
error-not-a-file = Ist keine Datei: {$path}
error-operation-failed = Operation fehlgeschlagen: {$operation}
error-io-error = E/A-Fehler: {$message}

# cat Befehl
cat-help-usage = Verwendung: cat [OPTION]... [DATEI]...
cat-help-description = DATEI(en) zur Standardausgabe verketten.
cat-version = cat (NexusShell) 1.0.0

# ls Befehl
ls-help-usage = Verwendung: ls [OPTION]... [DATEI]...
ls-help-description = Informationen über DATEI(en) auflisten (standardmäßig aktuelles Verzeichnis).
ls-permission-read = lesen
ls-permission-write = schreiben
ls-permission-execute = ausführen
ls-type-directory = Verzeichnis
ls-type-file = reguläre Datei
ls-type-symlink = symbolischer Link

# grep Befehl
grep-help-usage = Verwendung: grep [OPTION]... MUSTER [DATEI]...
grep-help-description = Nach MUSTER in jeder DATEI suchen.
grep-matches-found = {$count} Übereinstimmungen gefunden
grep-no-matches = Keine Übereinstimmungen gefunden

# ps Befehl
ps-help-usage = Verwendung: ps [OPTION]...
ps-help-description = Informationen über laufende Prozesse anzeigen.
ps-header-pid = PID
ps-header-user = BENUTZER
ps-header-command = BEFEHL

# ping Befehl
ping-help-usage = Verwendung: ping [OPTION]... HOST
ping-help-description = ICMP ECHO_REQUEST an Netzwerk-Hosts senden.
ping-statistics = --- {$host} Ping-Statistiken ---
ping-packets-transmitted = {$transmitted} Pakete übertragen
ping-packets-received = {$received} empfangen
ping-packet-loss = {$loss}% Paketverlust

# Allgemeine Dateioperationen
file-exists = Datei existiert: {$filename}
file-not-exists = Datei existiert nicht: {$filename}
operation-cancelled = Operation abgebrochen
operation-completed = Operation erfolgreich abgeschlossen 

# timedatectl status/common/time-sync labels
timedatectl.common.yes = ja
timedatectl.common.no = nein
timedatectl.common.enabled = aktiviert
timedatectl.common.disabled = deaktiviert
timedatectl.common.reachable = erreichbar
timedatectl.common.unreachable = nicht erreichbar

timedatectl.msg.time_set_to = Zeit eingestellt auf:
timedatectl.msg.timezone_set_to = Zeitzone eingestellt auf:
timedatectl.msg.rtc_in_local_tz = RTC in lokaler Zeitzone:
timedatectl.msg.ntp_sync = NTP-Synchronisierung:
timedatectl.msg.added_ntp_server = NTP-Server hinzugefügt:
timedatectl.msg.removed_ntp_server = NTP-Server entfernt:

timedatectl.timesync.title = Zeitsynchronisationsstatus:
timedatectl.timesync.enabled = Aktiviert:
timedatectl.timesync.synchronized = Synchronisiert:
timedatectl.timesync.last_sync = Letzte Synchronisierung:
timedatectl.timesync.sync_accuracy = Synchronisationsgenauigkeit:
timedatectl.timesync.drift_rate = Drift-Rate:
timedatectl.timesync.poll_interval = Abfrageintervall:
timedatectl.timesync.leap_status = Schaltsekundenstatus:
timedatectl.timesync.ntp_servers = NTP-Server:
timedatectl.timesync.stratum = Stratum:
timedatectl.timesync.delay = Verzögerung:
timedatectl.timesync.offset = Offset:
timedatectl.timesync.summary = Zusammenfassung:
timedatectl.timesync.servers_total_reachable = Server (gesamt/erreichbar):
timedatectl.timesync.best_stratum = Bestes Stratum:
timedatectl.timesync.preferred_server = Bevorzugter Server:
timedatectl.timesync.avg_delay = Durchschnittliche Verzögerung:
timedatectl.timesync.min_delay = Minimale Verzögerung:
timedatectl.timesync.max_delay = Maximale Verzögerung:
timedatectl.timesync.avg_offset = Durchschnittlicher Offset:
timedatectl.timesync.min_offset = Minimaler Offset:
timedatectl.timesync.max_offset = Maximaler Offset:
timedatectl.timesync.avg_jitter = Durchschnittliches Jitter:

# timedatectl Statusanzeigen
timedatectl.help.title = timedatectl: Zeit- und Datumsverwaltung
timedatectl.help.usage = Verwendung:
timedatectl.help.commands = Befehle:
timedatectl.help.options = Optionen:
timedatectl.help.time_formats = Akzeptierte Zeitformate:
timedatectl.help.examples = Beispiele:
timedatectl.help.timesync_options = Optionen für timesync-status:
timedatectl.help.timesync_json_option =   -J, --json            Status und Zusammenfassung als kompaktes JSON ausgeben
timedatectl.help.global_json_option =   Global: Einige Befehle akzeptieren -J/--json für JSON-Ausgabe
timedatectl.status.local_time = Lokale Zeit
timedatectl.status.universal_time = Weltzeit (UTC)
timedatectl.status.rtc_time = RTC-Zeit
timedatectl.status.time_zone = Zeitzone
timedatectl.status.system_clock_synchronized = Systemuhr synchronisiert
timedatectl.status.ntp_service = NTP-Dienst
timedatectl.status.rtc_in_local_tz = RTC in lokaler Zeitzone
timedatectl.status.sync_accuracy = Synchronisationsgenauigkeit
timedatectl.status.drift_rate = Drift-Rate
timedatectl.status.last_sync = Letzte Synchronisierung
timedatectl.status.leap_second = Schaltsekunde
timedatectl.status.pending = ausstehend

# timedatectl Hilfe – Befehle
timedatectl.help.cmd.status = Aktuellen Zeitstatus anzeigen
timedatectl.help.cmd.show = Status als JSON anzeigen
timedatectl.help.cmd.set_time = Systemzeit einstellen
timedatectl.help.cmd.set_timezone = Systemzeitzone einstellen
timedatectl.help.cmd.list_timezones = Verfügbare Zeitzonen auflisten
timedatectl.help.cmd.set_local_rtc = RTC auf lokale Zeit setzen (true/false)
timedatectl.help.cmd.set_ntp = NTP-Synchronisierung aktivieren/deaktivieren (true/false)
timedatectl.help.cmd.timesync_status = Status der Zeitsynchronisierung anzeigen
timedatectl.help.cmd.show_timesync = Zeitsynchronisationsstatus als JSON anzeigen
timedatectl.help.cmd.add_ntp_server = NTP-Server hinzufügen
timedatectl.help.cmd.remove_ntp_server = NTP-Server entfernen
timedatectl.help.cmd.statistics = Zeitstatistiken anzeigen
timedatectl.help.cmd.history = Verlauf der Zeitanpassungen anzeigen

# timedatectl Hilfe – Optionen
timedatectl.help.opt.help = Diese Hilfe anzeigen und beenden
timedatectl.help.opt.monitor = Echtzeit-Überwachungsmodus starten
timedatectl.help.opt.all = Alle Eigenschaften anzeigen
timedatectl.help.opt.json = In JSON ausgeben

# Akzeptierte Zeitformate
timedatectl.help.fmt.full_datetime = Volles Datum und Uhrzeit
timedatectl.help.fmt.datetime_no_sec = Datum und Uhrzeit ohne Sekunden
timedatectl.help.fmt.time_only = Nur Uhrzeit
timedatectl.help.fmt.time_no_sec = Uhrzeit (ohne Sekunden)
timedatectl.help.fmt.unix_timestamp = Unix-Zeitstempel (Sekunden)
timedatectl.help.fmt.iso8601 = ISO 8601 (UTC)

# Hilfe Beispiele
timedatectl.help.ex.status = Status anzeigen
timedatectl.help.ex.set_time = Systemzeit einstellen
timedatectl.help.ex.set_timezone = Zeitzone einstellen
timedatectl.help.ex.find_timezone = Zeitzone suchen
timedatectl.help.ex.enable_ntp = NTP-Synchronisierung aktivieren
timedatectl.help.ex.add_server = NTP-Server hinzufügen
timedatectl.help.ex.sync_status = Synchronisationsstatus anzeigen
timedatectl.help.ex.statistics = Statistiken anzeigen

# Eigenschaftenansicht
timedatectl.properties.title = Zeit- und Datums-Eigenschaften
timedatectl.properties.time_info = Zeitinformationen
timedatectl.properties.local_time = Lokale Zeit
timedatectl.properties.utc_time = UTC-Zeit
timedatectl.properties.timezone_info = Zeitzoneninformationen
timedatectl.properties.timezone = Zeitzone
timedatectl.properties.utc_offset = UTC-Versatz
timedatectl.properties.dst_active = Sommerzeit aktiv
timedatectl.properties.sync_status = Synchronisationsstatus
timedatectl.properties.system_synced = Systemuhr synchronisiert
timedatectl.properties.ntp_service = NTP-Dienst
timedatectl.properties.time_source = Zeitquelle
timedatectl.properties.sync_accuracy = Synchronisationsgenauigkeit
timedatectl.properties.last_sync = Letzte Synchronisierung
timedatectl.properties.drift_rate = Drift-Rate (ppm)
timedatectl.properties.leap_info = Schaltsekundeninformationen
timedatectl.properties.leap_pending = Schaltsekunde ausstehend
timedatectl.properties.ntp_config = NTP-Konfiguration
timedatectl.properties.ntp_enabled = NTP aktiviert
timedatectl.properties.ntp_servers = NTP-Server
timedatectl.properties.min_poll = Minimales Abfrageintervall
timedatectl.properties.max_poll = Maximales Abfrageintervall
timedatectl.properties.capabilities = Systemfunktionen
timedatectl.properties.tz_changes = Zeitzonenänderungen
timedatectl.properties.ntp_sync = NTP-Synchronisierung
timedatectl.properties.rtc_access = RTC-Zugriff
timedatectl.properties.hw_timestamp = HW-Zeitstempelung

# Allgemeine Labels (generisch)
common.yes = ja
common.no = nein
common.supported = unterstützt
common.limited = eingeschränkt
common.full = vollständig
common.available = verfügbar

# Einheiten
units.microseconds = Mikrosekunden

# date Befehl (Metadaten/relativ/Feiertage)
date.error.invalid_timezone = Ungültige Zeitzone: {$tz}
date.error.invalid_month = Ungültiger Monat: {$month}

date.metadata.unix_timestamp = Unix-Zeitstempel: {$value}
date.metadata.julian_day = Julianischer Tag: {$value}
date.metadata.day_of_year = Tag des Jahres: {$value}
date.metadata.week_number = Kalenderwoche: {$value}
date.metadata.weekday = Wochentag: {$value}
date.metadata.type.weekend = Typ: Wochenende
date.metadata.type.business = Typ: Werktag
date.metadata.astronomical = Astronomisch: {$info}

date.relative.now = jetzt
date.relative.minutes_ago = vor {$mins} Minuten
date.relative.in_minutes = in {$mins} Minuten
date.relative.hours_ago = vor {$hours} Stunden
date.relative.in_hours = in {$hours} Stunden
date.relative.days_ago = vor {$days} Tagen
date.relative.in_days = in {$days} Tagen

date.holiday.none = Keine Feiertage für das Jahr {$year} in Regionen: {$regions} gefunden
date.holiday.header = Feiertage {$year} in Regionen: {$regions}
date.holiday.separator = =====================================
date.holiday.entry = {$date} - {$name} ({$region}, {$kind})
date.holiday.total = Gesamt: {$count} Feiertage