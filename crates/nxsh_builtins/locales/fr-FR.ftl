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