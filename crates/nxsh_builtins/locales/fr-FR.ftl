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