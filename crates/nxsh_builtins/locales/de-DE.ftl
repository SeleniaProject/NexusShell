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