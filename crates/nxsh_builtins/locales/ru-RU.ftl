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

# timedatectl status/common/time-sync labels
timedatectl.common.yes = да
timedatectl.common.no = нет
timedatectl.common.enabled = включено
timedatectl.common.disabled = отключено
timedatectl.common.reachable = доступен
timedatectl.common.unreachable = недоступен

timedatectl.msg.time_set_to = Время установлено:
timedatectl.msg.timezone_set_to = Часовой пояс установлен:
timedatectl.msg.rtc_in_local_tz = RTC в локальном часовом поясе:
timedatectl.msg.ntp_sync = Синхронизация NTP:
timedatectl.msg.added_ntp_server = Добавлен NTP-сервер:
timedatectl.msg.removed_ntp_server = Удален NTP-сервер:

timedatectl.timesync.title = Статус синхронизации времени:
timedatectl.timesync.enabled = Включено:
timedatectl.timesync.synchronized = Синхронизировано:
timedatectl.timesync.last_sync = Последняя синхронизация:
timedatectl.timesync.sync_accuracy = Точность синхронизации:
timedatectl.timesync.drift_rate = Скорость дрейфа:
timedatectl.timesync.poll_interval = Интервал опроса:
timedatectl.timesync.leap_status = Статус високосной секунды:
timedatectl.timesync.ntp_servers = NTP-серверы:
timedatectl.timesync.stratum = Стратум:
timedatectl.timesync.delay = Задержка:
timedatectl.timesync.offset = Смещение:
timedatectl.timesync.summary = Сводка:
timedatectl.timesync.servers_total_reachable = Серверы (всего/доступно):
timedatectl.timesync.best_stratum = Лучший стратум:
timedatectl.timesync.preferred_server = Предпочитаемый сервер:
timedatectl.timesync.avg_delay = Средняя задержка:
timedatectl.timesync.min_delay = Минимальная задержка:
timedatectl.timesync.max_delay = Максимальная задержка:
timedatectl.timesync.avg_offset = Среднее смещение:
timedatectl.timesync.min_offset = Минимальное смещение:
timedatectl.timesync.max_offset = Максимальное смещение:
timedatectl.timesync.avg_jitter = Средний джиттер:

# Метки статуса timedatectl
timedatectl.status.local_time = Локальное время
timedatectl.status.universal_time = Всемирное время (UTC)
timedatectl.status.rtc_time = Время RTC
timedatectl.status.time_zone = Часовой пояс
timedatectl.status.system_clock_synchronized = Системные часы синхронизированы
timedatectl.status.ntp_service = Служба NTP
timedatectl.status.rtc_in_local_tz = RTC в локальной зоне
timedatectl.status.sync_accuracy = Точность синхронизации
timedatectl.status.drift_rate = Скорость дрейфа
timedatectl.status.last_sync = Последняя синхронизация
timedatectl.status.leap_second = Високосная секунда
timedatectl.status.pending = ожидается

# Справка timedatectl — команды
timedatectl.help.cmd.status = Показать текущий статус времени
timedatectl.help.cmd.show = Показать статус в JSON
timedatectl.help.cmd.set_time = Установить системное время
timedatectl.help.cmd.set_timezone = Установить системный часовой пояс
timedatectl.help.cmd.list_timezones = Показать доступные часовые пояса
timedatectl.help.cmd.set_local_rtc = Установить RTC на локальное время (true/false)
timedatectl.help.cmd.set_ntp = Включить или отключить синхронизацию NTP (true/false)
timedatectl.help.cmd.timesync_status = Показать статус синхронизации времени
timedatectl.help.cmd.show_timesync = Показать статус синхронизации в JSON
timedatectl.help.cmd.add_ntp_server = Добавить сервер NTP
timedatectl.help.cmd.remove_ntp_server = Удалить сервер NTP
timedatectl.help.cmd.statistics = Показать статистику времени
timedatectl.help.cmd.history = Показать историю корректировок времени

# Справка timedatectl — параметры
timedatectl.help.opt.help = Показать эту справку и выйти
timedatectl.help.opt.monitor = Запустить режим мониторинга в реальном времени
timedatectl.help.opt.all = Показать все свойства
timedatectl.help.opt.json = Вывод в JSON

# Принимаемые форматы времени
timedatectl.help.fmt.full_datetime = Полные дата и время
timedatectl.help.fmt.datetime_no_sec = Дата и время без секунд
timedatectl.help.fmt.time_only = Только время
timedatectl.help.fmt.time_no_sec = Время (без секунд)
timedatectl.help.fmt.unix_timestamp = Unix-временная метка (секунды)
timedatectl.help.fmt.iso8601 = ISO 8601 (UTC)

# Примеры справки
timedatectl.help.ex.status = показать статус
timedatectl.help.ex.set_time = установить системное время
timedatectl.help.ex.set_timezone = установить часовой пояс
timedatectl.help.ex.find_timezone = найти часовой пояс
timedatectl.help.ex.enable_ntp = включить синхронизацию NTP
timedatectl.help.ex.add_server = добавить сервер NTP
timedatectl.help.ex.sync_status = показать статус синхронизации
timedatectl.help.ex.statistics = показать статистику

# Просмотр свойств
timedatectl.properties.title = Свойства даты и времени
timedatectl.properties.time_info = Информация о времени
timedatectl.properties.local_time = Локальное время
timedatectl.properties.utc_time = Время UTC
timedatectl.properties.timezone_info = Информация о часовом поясе
timedatectl.properties.timezone = Часовой пояс
timedatectl.properties.utc_offset = Смещение UTC
timedatectl.properties.dst_active = Летнее время активно
timedatectl.properties.sync_status = Статус синхронизации
timedatectl.properties.system_synced = Системные часы синхронизированы
timedatectl.properties.ntp_service = Служба NTP
timedatectl.properties.time_source = Источник времени
timedatectl.properties.sync_accuracy = Точность синхронизации
timedatectl.properties.last_sync = Последняя синхронизация
timedatectl.properties.drift_rate = Скорость дрейфа (ppm)
timedatectl.properties.leap_info = Информация о високосной секунде
timedatectl.properties.leap_pending = Ожидается високосная секунда
timedatectl.properties.ntp_config = Конфигурация NTP
timedatectl.properties.ntp_enabled = NTP включен
timedatectl.properties.ntp_servers = NTP-серверы
timedatectl.properties.min_poll = Минимальный интервал опроса
timedatectl.properties.max_poll = Максимальный интервал опроса
timedatectl.properties.capabilities = Возможности системы
timedatectl.properties.tz_changes = Изменения часового пояса
timedatectl.properties.ntp_sync = Синхронизация NTP
timedatectl.properties.rtc_access = Доступ к RTC
timedatectl.properties.hw_timestamp = Аппаратная временная метка

# Общие метки
common.yes = да
common.no = нет
common.supported = поддерживается
common.limited = ограничено
common.full = полный
common.available = доступно

# Единицы измерения
units.microseconds = микросекунды