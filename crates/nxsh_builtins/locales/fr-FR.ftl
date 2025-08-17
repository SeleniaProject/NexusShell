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
# NexusShell Commandes Intégrées - Localisation Française

# Messages communs
error-file-not-found = Fichier non trouvé : {$filename}
error-permission-denied = Permission refusée : {$filename}
error-invalid-option = Option invalide : {$option}
error-missing-argument = Argument manquant pour l'option : {$option}
error-invalid-argument = Argument invalide : {$argument}
error-directory-not-found = Répertoire non trouvé : {$dirname}
error-not-a-directory = N'est pas un répertoire : {$path}
error-not-a-file = N'est pas un fichier : {$path}
error-operation-failed = Opération échouée : {$operation}
error-io-error = Erreur E/S : {$message}

# Commande cat
cat-help-usage = Usage : cat [OPTION]... [FICHIER]...
cat-help-description = Concaténer FICHIER(s) vers la sortie standard.
cat-version = cat (NexusShell) 1.0.0

# Commande ls
ls-help-usage = Usage : ls [OPTION]... [FICHIER]...
ls-help-description = Lister les informations sur les FICHIER(s) (le répertoire courant par défaut).
ls-permission-read = lecture
ls-permission-write = écriture
ls-permission-execute = exécution
ls-type-directory = répertoire
ls-type-file = fichier régulier
ls-type-symlink = lien symbolique

# Commande grep
grep-help-usage = Usage : grep [OPTION]... MOTIF [FICHIER]...
grep-help-description = Rechercher MOTIF dans chaque FICHIER.
grep-matches-found = {$count} correspondances trouvées
grep-no-matches = Aucune correspondance trouvée

# Commande ps
ps-help-usage = Usage : ps [OPTION]...
ps-help-description = Afficher les informations sur les processus en cours d'exécution.
ps-header-pid = PID
ps-header-user = UTILISATEUR
ps-header-command = COMMANDE

# Commande ping
ping-help-usage = Usage : ping [OPTION]... HÔTE
ping-help-description = Envoyer ICMP ECHO_REQUEST aux hôtes réseau.
ping-statistics = --- statistiques ping de {$host} ---
ping-packets-transmitted = {$transmitted} paquets transmis
ping-packets-received = {$received} reçus
ping-packet-loss = {$loss}% perte de paquets

# Opérations de fichier communes
file-exists = Le fichier existe : {$filename}
file-not-exists = Le fichier n'existe pas : {$filename}
operation-cancelled = Opération annulée
operation-completed = Opération terminée avec succès 

# timedatectl status/common/time-sync labels
timedatectl.common.yes = oui
timedatectl.common.no = non
timedatectl.common.enabled = activé
timedatectl.common.disabled = désactivé
timedatectl.common.reachable = joignable
timedatectl.common.unreachable = injoignable

timedatectl.msg.time_set_to = Heure définie sur :
timedatectl.msg.timezone_set_to = Fuseau horaire défini sur :
timedatectl.msg.rtc_in_local_tz = RTC en fuseau horaire local :
timedatectl.msg.ntp_sync = Synchronisation NTP :
timedatectl.msg.added_ntp_server = Serveur NTP ajouté :
timedatectl.msg.removed_ntp_server = Serveur NTP supprimé :

timedatectl.timesync.title = État de la synchronisation de l'heure :
timedatectl.timesync.enabled = Activé :
timedatectl.timesync.synchronized = Synchronisé :
timedatectl.timesync.last_sync = Dernière synchronisation :
timedatectl.timesync.sync_accuracy = Précision de synchronisation :
timedatectl.timesync.drift_rate = Taux de dérive :
timedatectl.timesync.poll_interval = Intervalle d'interrogation :
timedatectl.timesync.leap_status = État des secondes intercalaires :
timedatectl.timesync.ntp_servers = Serveurs NTP :
timedatectl.timesync.stratum = Stratum :
timedatectl.timesync.delay = Délai :
timedatectl.timesync.offset = Décalage :
timedatectl.timesync.summary = Résumé :
timedatectl.timesync.servers_total_reachable = Serveurs (total/joignables) :
timedatectl.timesync.best_stratum = Meilleur stratum :
timedatectl.timesync.preferred_server = Serveur préféré :
timedatectl.timesync.avg_delay = Délai moyen :
timedatectl.timesync.min_delay = Délai minimal :
timedatectl.timesync.max_delay = Délai maximal :
timedatectl.timesync.avg_offset = Décalage moyen :
timedatectl.timesync.min_offset = Décalage minimal :
timedatectl.timesync.max_offset = Décalage maximal :
timedatectl.timesync.avg_jitter = Gigue moyenne :

# timedatectl étiquettes d'état
timedatectl.help.title = timedatectl : Gestion de l'heure et de la date
timedatectl.help.usage = Utilisation :
timedatectl.help.commands = Commandes :
timedatectl.help.options = Options :
timedatectl.help.time_formats = Formats d'heure acceptés :
timedatectl.help.examples = Exemples :
timedatectl.help.timesync_options = Options pour timesync-status :
timedatectl.help.timesync_json_option =   -J, --json            Afficher l'état et le résumé en JSON compact
timedatectl.help.global_json_option =   Global : certains commandes acceptent -J/--json pour la sortie JSON
timedatectl.status.local_time = Heure locale
timedatectl.status.universal_time = Temps universel (UTC)
timedatectl.status.rtc_time = Heure RTC
timedatectl.status.time_zone = Fuseau horaire
timedatectl.status.system_clock_synchronized = Horloge système synchronisée
timedatectl.status.ntp_service = Service NTP
timedatectl.status.rtc_in_local_tz = RTC en fuseau local
timedatectl.status.sync_accuracy = Précision de synchronisation
timedatectl.status.drift_rate = Taux de dérive
timedatectl.status.last_sync = Dernière synchronisation
timedatectl.status.leap_second = Seconde intercalaire
timedatectl.status.pending = en attente

# timedatectl aide — commandes
timedatectl.help.cmd.status = Afficher l'état actuel de l'heure
timedatectl.help.cmd.show = Afficher l'état en JSON
timedatectl.help.cmd.set_time = Définir l'heure du système
timedatectl.help.cmd.set_timezone = Définir le fuseau horaire du système
timedatectl.help.cmd.list_timezones = Lister les fuseaux horaires disponibles
timedatectl.help.cmd.set_local_rtc = Définir la RTC en heure locale (true/false)
timedatectl.help.cmd.set_ntp = Activer/désactiver la synchronisation NTP (true/false)
timedatectl.help.cmd.timesync_status = Afficher l'état de synchronisation de l'heure
timedatectl.help.cmd.show_timesync = Afficher l'état de synchronisation en JSON
timedatectl.help.cmd.add_ntp_server = Ajouter un serveur NTP
timedatectl.help.cmd.remove_ntp_server = Supprimer un serveur NTP
timedatectl.help.cmd.statistics = Afficher les statistiques de temps
timedatectl.help.cmd.history = Afficher l'historique des ajustements d'heure

# timedatectl aide — options
timedatectl.help.opt.help = Afficher cette aide et quitter
timedatectl.help.opt.monitor = Exécuter en mode surveillance temps réel
timedatectl.help.opt.all = Afficher toutes les propriétés
timedatectl.help.opt.json = Sortie en JSON

# Formats d'heure acceptés
timedatectl.help.fmt.full_datetime = Date et heure complètes
timedatectl.help.fmt.datetime_no_sec = Date et heure sans secondes
timedatectl.help.fmt.time_only = Heure seule
timedatectl.help.fmt.time_no_sec = Heure (sans secondes)
timedatectl.help.fmt.unix_timestamp = Timestamp Unix (secondes)
timedatectl.help.fmt.iso8601 = ISO 8601 (UTC)

# Exemples d'aide
timedatectl.help.ex.status = afficher l'état
timedatectl.help.ex.set_time = définir l'heure système
timedatectl.help.ex.set_timezone = définir le fuseau horaire
timedatectl.help.ex.find_timezone = trouver un fuseau horaire
timedatectl.help.ex.enable_ntp = activer la synchronisation NTP
timedatectl.help.ex.add_server = ajouter un serveur NTP
timedatectl.help.ex.sync_status = afficher l'état de synchronisation
timedatectl.help.ex.statistics = afficher les statistiques

# Vue des propriétés
timedatectl.properties.title = Propriétés de l'heure et de la date
timedatectl.properties.time_info = Informations sur l'heure
timedatectl.properties.local_time = Heure locale
timedatectl.properties.utc_time = Heure UTC
timedatectl.properties.timezone_info = Informations sur le fuseau horaire
timedatectl.properties.timezone = Fuseau horaire
timedatectl.properties.utc_offset = Décalage UTC
timedatectl.properties.dst_active = Heure d'été active
timedatectl.properties.sync_status = État de synchronisation
timedatectl.properties.system_synced = Horloge système synchronisée
timedatectl.properties.ntp_service = Service NTP
timedatectl.properties.time_source = Source de temps
timedatectl.properties.sync_accuracy = Précision de synchronisation
timedatectl.properties.last_sync = Dernière synchronisation
timedatectl.properties.drift_rate = Taux de dérive (ppm)
timedatectl.properties.leap_info = Informations sur les secondes intercalaires
timedatectl.properties.leap_pending = Seconde intercalaire en attente
timedatectl.properties.ntp_config = Configuration NTP
timedatectl.properties.ntp_enabled = NTP activé
timedatectl.properties.ntp_servers = Serveurs NTP
timedatectl.properties.min_poll = Intervalle d'interrogation min
timedatectl.properties.max_poll = Intervalle d'interrogation max
timedatectl.properties.capabilities = Capacités du système
timedatectl.properties.tz_changes = Changements de fuseau
timedatectl.properties.ntp_sync = Synchronisation NTP
timedatectl.properties.rtc_access = Accès RTC
timedatectl.properties.hw_timestamp = Horodatage matériel

# Libellés génériques
common.yes = oui
common.no = non
common.supported = pris en charge
common.limited = limité
common.full = complet
common.available = disponible

# Unités
units.microseconds = microsecondes

# commande date (métadonnées/relatif/jours fériés)
date.error.invalid_timezone = Fuseau horaire invalide : {$tz}
date.error.invalid_month = Mois invalide : {$month}

date.metadata.unix_timestamp = Timestamp Unix : {$value}
date.metadata.julian_day = Jour julien : {$value}
date.metadata.day_of_year = Jour de l'année : {$value}
date.metadata.week_number = Numéro de semaine : {$value}
date.metadata.weekday = Jour de la semaine : {$value}
date.metadata.type.weekend = Type : week-end
date.metadata.type.business = Type : jour ouvrable
date.metadata.astronomical = Astronomique : {$info}

date.relative.now = maintenant
date.relative.minutes_ago = il y a {$mins} minutes
date.relative.in_minutes = dans {$mins} minutes
date.relative.hours_ago = il y a {$hours} heures
date.relative.in_hours = dans {$hours} heures
date.relative.days_ago = il y a {$days} jours
date.relative.in_days = dans {$days} jours

date.holiday.none = Aucun jour férié trouvé pour l'année {$year} dans les régions : {$regions}
date.holiday.header = Jours fériés {$year} dans les régions : {$regions}
date.holiday.separator = =====================================
date.holiday.entry = {$date} - {$name} ({$region}, {$kind})
date.holiday.total = Total : {$count} jours fériés