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