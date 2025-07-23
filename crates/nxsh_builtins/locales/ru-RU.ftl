# NexusShell Встроенные Команды - Русская Локализация

# Общие сообщения
error-file-not-found = Файл не найден: {$filename}
error-permission-denied = Доступ запрещен: {$filename}
error-invalid-option = Неверная опция: {$option}
error-missing-argument = Отсутствует аргумент для опции: {$option}
error-invalid-argument = Неверный аргумент: {$argument}
error-directory-not-found = Каталог не найден: {$dirname}
error-not-a-directory = Не является каталогом: {$path}
error-not-a-file = Не является файлом: {$path}
error-operation-failed = Операция не удалась: {$operation}
error-io-error = Ошибка ввода-вывода: {$message}

# Команда cat
cat-help-usage = Использование: cat [ОПЦИЯ]... [ФАЙЛ]...
cat-help-description = Объединить ФАЙЛ(ы) в стандартный вывод.
cat-version = cat (NexusShell) 1.0.0

# Команда ls
ls-help-usage = Использование: ls [ОПЦИЯ]... [ФАЙЛ]...
ls-help-description = Вывести информацию о ФАЙЛ(ах) (по умолчанию текущий каталог).
ls-permission-read = чтение
ls-permission-write = запись
ls-permission-execute = выполнение
ls-type-directory = каталог
ls-type-file = обычный файл
ls-type-symlink = символическая ссылка

# Команда grep
grep-help-usage = Использование: grep [ОПЦИЯ]... ШАБЛОН [ФАЙЛ]...
grep-help-description = Поиск ШАБЛОНА в каждом ФАЙЛЕ.
grep-matches-found = Найдено {$count} совпадений
grep-no-matches = Совпадений не найдено

# Команда ps
ps-help-usage = Использование: ps [ОПЦИЯ]...
ps-help-description = Показать информацию о запущенных процессах.
ps-header-pid = PID
ps-header-user = ПОЛЬЗОВАТЕЛЬ
ps-header-command = КОМАНДА

# Команда ping
ping-help-usage = Использование: ping [ОПЦИЯ]... ХОСТ
ping-help-description = Отправить ICMP ECHO_REQUEST сетевым хостам.
ping-statistics = --- статистика ping {$host} ---
ping-packets-transmitted = {$transmitted} пакетов передано
ping-packets-received = {$received} получено
ping-packet-loss = {$loss}% потеря пакетов

# Общие файловые операции
file-exists = Файл существует: {$filename}
file-not-exists = Файл не существует: {$filename}
operation-cancelled = Операция отменена
operation-completed = Операция успешно завершена 