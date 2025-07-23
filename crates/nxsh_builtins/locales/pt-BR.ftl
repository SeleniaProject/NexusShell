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