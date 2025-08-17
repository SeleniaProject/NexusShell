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