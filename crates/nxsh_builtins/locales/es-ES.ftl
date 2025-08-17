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