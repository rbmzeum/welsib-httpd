# Build welsib-httpd with static library of verify

```bash
RUSTFLAGS='-L staticlib' cargo build --release
mkdir /root/software # or any other directory
cp target/release/welsib-httpd /root/software/welsib-httpd
```

Веб-сервер, использующий ГОСТ 34.10-2018.  
Пользователь самостоятельно может проверить ЭП.

---

## Welsib Web Server

Многопоточный HTTPS сервер с поддержкой:
- Статических файлов
- API запросов
- WebSocket соединений
- Цифровых подписей ГОСТ Р 34.10-2018

---

### Основные компоненты

1. **Server** - основной класс сервера
   - Инициализация конфигурации
   - Управление SSL/TLS
   - Обработка входящих соединений

2. **Context** - контекст обработки запроса
   - Состояния (`WelsibState`)
   - Обработчики запросов
   - Управление ресурсами

3. **Resource** - управление статическими ресурсами
   - Кэширование файлов
   - Сжатие (gzip)
   - Цифровые подписи

4. **Dispatcher** - диспетчеризация запросов
   - Маршрутизация
   - Управление потоками

5. **Channel** - двунаправленный канал связи
   - WebSocket коммуникация
   - Асинхронный обмен сообщениями

---

### Особенности реализации
- **Thread-safe** с использованием `Arc<Mutex<>>`
- Конфигурируемый через JSON
- Поддержка hot-reload SSL сертификатов
- Расширяемая архитектура обработчиков

---

### Основные зависимости
- **openssl**: SSL/TLS поддержка
- **serde**: сериализация конфигурации
- **flate2**: сжатие данных
- **num-bigint**: работа с большими числами для криптографии


### Конфигурация

В примере конфигурационного файла указан параметр available_static_file_uri содержащий список URI-адресов доступных статических файлов.  
В данном случае при запросе к URI-адресу `/` возвращается статический файл `home.html`.
Для предоставления доступа к другим статическим файлам необходимо перечислить их в список URI-адресов, например:

```
{
  "available_static_file_uri": [
    "/", // home.html
    "/about.html",
    "/privacy.html"
  ]
}
```

### Установка и запуск

Создайте файл /usr/lib/systemd/system/welsib-httpd.service
с содержимым:

```
[Unit]
Description=Welsib web server
After=network.target

[Service]
WorkingDirectory=/root/software
ExecStart=/root/software/welsib-httpd --host=127.0.0.1 > /var/log/welsib-httpd.log
Restart=always
RestartSec=10s
Type=simple

[Install]
WantedBy=multi-user.target


```


Включение и запуск сервиса:

```
systemctl enable welsib-httpd
systemctl start welsib-httpd
```

### Параметры командной строки

```
welsib-httpd --help
```

Для создания цифровых подписей к файлам необходимо использовать совместимую библиотеку с поддержкой ГОСТ 34.10-2018 на основе OID: 1.2.643.7.1.2.1.2.1, TC26: id-tc26-gost-3410-12-512-paramSetA, сохранив две 64 байтные (512 битные) компоненты в виде бинарного файла размером 128 байт в файл с именем совпадающим со статическим файлом с добавлением расширения sig, например: home.html.sig, about.html.sig и privacy.html.sig и разместите их там же где и файл с данными.
