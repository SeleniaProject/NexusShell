# NexusShell Comandos Integrados - Localização Português Brasileiro

# Mensagens comuns
error-file-not-found = Arquivo não encontrado: {$filename}
error-permission-denied = Permissão negada: {$filename}
error-invalid-option = Opção inválida: {$option}
error-missing-argument = Argumento ausente para a opção: {$option}
error-invalid-argument = Argumento inválido: {$argument}
error-directory-not-found = Diretório não encontrado: {$dirname}
error-not-a-directory = Não é um diretório: {$path}
error-not-a-file = Não é um arquivo: {$path}
error-operation-failed = Operação falhou: {$operation}
error-io-error = Erro de E/S: {$message}

# Comando cat
cat-help-usage = Uso: cat [OPÇÃO]... [ARQUIVO]...
cat-help-description = Concatenar ARQUIVO(s) para a saída padrão.
cat-version = cat (NexusShell) 1.0.0

# Comando ls
ls-help-usage = Uso: ls [OPÇÃO]... [ARQUIVO]...
ls-help-description = Listar informações sobre ARQUIVO(s) (diretório atual por padrão).
ls-permission-read = leitura
ls-permission-write = escrita
ls-permission-execute = execução
ls-type-directory = diretório
ls-type-file = arquivo regular
ls-type-symlink = link simbólico

# Comando grep
grep-help-usage = Uso: grep [OPÇÃO]... PADRÃO [ARQUIVO]...
grep-help-description = Procurar PADRÃO em cada ARQUIVO.
grep-matches-found = {$count} correspondências encontradas
grep-no-matches = Nenhuma correspondência encontrada

# Comando ps
ps-help-usage = Uso: ps [OPÇÃO]...
ps-help-description = Exibir informações sobre processos em execução.
ps-header-pid = PID
ps-header-user = USUÁRIO
ps-header-command = COMANDO

# Comando ping
ping-help-usage = Uso: ping [OPÇÃO]... HOST
ping-help-description = Enviar ICMP ECHO_REQUEST para hosts de rede.
ping-statistics = --- estatísticas de ping de {$host} ---
ping-packets-transmitted = {$transmitted} pacotes transmitidos
ping-packets-received = {$received} recebidos
ping-packet-loss = {$loss}% perda de pacotes

# Operações de arquivo comuns
file-exists = Arquivo existe: {$filename}
file-not-exists = Arquivo não existe: {$filename}
operation-cancelled = Operação cancelada
operation-completed = Operação concluída com sucesso 

# timedatectl status/common/time-sync labels
timedatectl.common.yes = sim
timedatectl.common.no = não
timedatectl.common.enabled = habilitado
timedatectl.common.disabled = desabilitado
timedatectl.common.reachable = alcançável
timedatectl.common.unreachable = inalcançável

timedatectl.msg.time_set_to = Hora definida para:
timedatectl.msg.timezone_set_to = Fuso horário definido para:
timedatectl.msg.rtc_in_local_tz = RTC no fuso horário local:
timedatectl.msg.ntp_sync = Sincronização NTP:
timedatectl.msg.added_ntp_server = Servidor NTP adicionado:
timedatectl.msg.removed_ntp_server = Servidor NTP removido:

timedatectl.timesync.title = Status da sincronização de hora:
timedatectl.timesync.enabled = Habilitado:
timedatectl.timesync.synchronized = Sincronizado:
timedatectl.timesync.last_sync = Última sincronização:
timedatectl.timesync.sync_accuracy = Precisão da sincronização:
timedatectl.timesync.drift_rate = Taxa de deriva:
timedatectl.timesync.poll_interval = Intervalo de sondagem:
timedatectl.timesync.leap_status = Status de segundos intercalares:
timedatectl.timesync.ntp_servers = Servidores NTP:
timedatectl.timesync.stratum = Estrato:
timedatectl.timesync.delay = Atraso:
timedatectl.timesync.offset = Deslocamento:
timedatectl.timesync.summary = Resumo:
timedatectl.timesync.servers_total_reachable = Servidores (total/alcançáveis):
timedatectl.timesync.best_stratum = Melhor estrato:
timedatectl.timesync.preferred_server = Servidor preferido:
timedatectl.timesync.avg_delay = Atraso médio:
timedatectl.timesync.min_delay = Atraso mínimo:
timedatectl.timesync.max_delay = Atraso máximo:
timedatectl.timesync.avg_offset = Deslocamento médio:
timedatectl.timesync.min_offset = Deslocamento mínimo:
timedatectl.timesync.max_offset = Deslocamento máximo:
timedatectl.timesync.avg_jitter = Jitter médio:

# rótulos de status do timedatectl
timedatectl.status.local_time = Hora local

# Cabeçalhos de ajuda do timedatectl
timedatectl.help.title = timedatectl: Gerenciamento de Data e Hora
timedatectl.help.usage = Uso:
timedatectl.help.commands = Comandos:
timedatectl.help.options = Opções:
timedatectl.help.time_formats = Formatos de hora aceitos:
timedatectl.help.examples = Exemplos:
timedatectl.help.timesync_options = Opções para timesync-status:
timedatectl.help.timesync_json_option =   -J, --json            Exibir status e resumo como JSON compacto
timedatectl.help.global_json_option =   Global: alguns comandos aceitam -J/--json para saída JSON
timedatectl.status.universal_time = Tempo universal (UTC)
timedatectl.status.rtc_time = Hora do RTC
timedatectl.status.time_zone = Fuso horário
timedatectl.status.system_clock_synchronized = Relógio do sistema sincronizado
timedatectl.status.ntp_service = Serviço NTP
timedatectl.status.rtc_in_local_tz = RTC no fuso local
timedatectl.status.sync_accuracy = Precisão de sincronização
timedatectl.status.drift_rate = Taxa de deriva
timedatectl.status.last_sync = Última sincronização
timedatectl.status.leap_second = Segundo intercalar
timedatectl.status.pending = pendente

# timedatectl ajuda — comandos
timedatectl.help.cmd.status = Mostrar status de hora atual
timedatectl.help.cmd.show = Mostrar status em JSON
timedatectl.help.cmd.set_time = Definir a hora do sistema
timedatectl.help.cmd.set_timezone = Definir o fuso horário do sistema
timedatectl.help.cmd.list_timezones = Listar fusos horários disponíveis
timedatectl.help.cmd.set_local_rtc = Definir RTC para hora local (true/false)
timedatectl.help.cmd.set_ntp = Habilitar ou desabilitar sincronização NTP (true/false)
timedatectl.help.cmd.timesync_status = Mostrar status da sincronização de hora
timedatectl.help.cmd.show_timesync = Mostrar status de sincronização em JSON
timedatectl.help.cmd.add_ntp_server = Adicionar um servidor NTP
timedatectl.help.cmd.remove_ntp_server = Remover um servidor NTP
timedatectl.help.cmd.statistics = Mostrar estatísticas de tempo
timedatectl.help.cmd.history = Mostrar histórico de ajustes de hora

# timedatectl ajuda — opções
timedatectl.help.opt.help = Mostrar esta ajuda e sair
timedatectl.help.opt.monitor = Executar modo de monitoramento em tempo real
timedatectl.help.opt.all = Mostrar todas as propriedades
timedatectl.help.opt.json = Saída em JSON

# Formatos de hora aceitos
timedatectl.help.fmt.full_datetime = Data e hora completas
timedatectl.help.fmt.datetime_no_sec = Data e hora sem segundos
timedatectl.help.fmt.time_only = Somente hora
timedatectl.help.fmt.time_no_sec = Hora (sem segundos)
timedatectl.help.fmt.unix_timestamp = Timestamp Unix (segundos)
timedatectl.help.fmt.iso8601 = ISO 8601 (UTC)

# Exemplos de ajuda
timedatectl.help.ex.status = mostrar status
timedatectl.help.ex.set_time = definir hora do sistema
timedatectl.help.ex.set_timezone = definir fuso horário
timedatectl.help.ex.find_timezone = encontrar um fuso horário
timedatectl.help.ex.enable_ntp = habilitar sincronização NTP
timedatectl.help.ex.add_server = adicionar servidor NTP
timedatectl.help.ex.sync_status = mostrar status de sincronização
timedatectl.help.ex.statistics = mostrar estatísticas

# Visualização de propriedades
timedatectl.properties.title = Propriedades de Data e Hora
timedatectl.properties.time_info = Informações de tempo
timedatectl.properties.local_time = Hora local
timedatectl.properties.utc_time = Hora UTC
timedatectl.properties.timezone_info = Informações de fuso horário
timedatectl.properties.timezone = Fuso horário
timedatectl.properties.utc_offset = Deslocamento UTC
timedatectl.properties.dst_active = Horário de verão ativo
timedatectl.properties.sync_status = Status de sincronização
timedatectl.properties.system_synced = Relógio do sistema sincronizado
timedatectl.properties.ntp_service = Serviço NTP
timedatectl.properties.time_source = Fonte de tempo
timedatectl.properties.sync_accuracy = Precisão de sincronização
timedatectl.properties.last_sync = Última sincronização
timedatectl.properties.drift_rate = Taxa de deriva (ppm)
timedatectl.properties.leap_info = Informações de segundos intercalares
timedatectl.properties.leap_pending = Segundo intercalar pendente
timedatectl.properties.ntp_config = Configuração NTP
timedatectl.properties.ntp_enabled = NTP habilitado
timedatectl.properties.ntp_servers = Servidores NTP
timedatectl.properties.min_poll = Intervalo mínimo de sondagem
timedatectl.properties.max_poll = Intervalo máximo de sondagem
timedatectl.properties.capabilities = Capacidades do sistema
timedatectl.properties.tz_changes = Mudanças de fuso horário
timedatectl.properties.ntp_sync = Sincronização NTP
timedatectl.properties.rtc_access = Acesso ao RTC
timedatectl.properties.hw_timestamp = Marcação de tempo de HW

# Rótulos genéricos
common.yes = sim
common.no = não
common.supported = suportado
common.limited = limitado
common.full = completo
common.available = disponível

# Unidades
units.microseconds = microssegundos