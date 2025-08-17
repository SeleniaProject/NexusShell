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