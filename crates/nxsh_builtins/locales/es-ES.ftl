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