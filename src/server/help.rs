/// Класс для вывода справки.
pub struct Help;

impl Help {
    /// Выводит справку по использованию программы.
    pub fn display() {
        println!("welsib-httpd - сервер дистрибьюции и активации ключей с web-интерфейсом  [версия 0.1.0.0]");
        println!(
            "\n\
            Использование:\twelsib-httpd [options]\n\
            \t-h, --help\t\tсправка\n\
            \t--nossl\t\t\tне использовать SSL\n\
            \t--nosig\t\tне проверять цифровые подписи статических файлов из директории ./data/web/*\n\
            \t--config=[FILE]\tиспользовать файл конфигурации из файла /path/to/file.json\n\
            \t--host=[IP]\t\tиспользовать IP отличный от 127.0.0.1\n\
            \t--port=[PORT]\t\tиспользовать порт из диапазона 1024..65535\n\n\
            welsib-httpd сервер предоставляющий WGI-коннектор (Welsib Gateway Interface)\n
            и статические web страницы.\n\n\
            Пути к SSL:\n\
            \tpath_cert\t\t/root/.acme.sh/welsib.ru_ecc/welsib.ru.cer\n\
            \tpath_key\t\t/root/.acme.sh/welsib.ru_ecc/welsib.ru.key\n\
            \tpath_ca\t\t\t/root/.acme.sh/welsib.ru_ecc/ca.cer\n\n\
            Пример использования:\n\
            \twelsib-httpd --nossl --config=/etc/welsib-httpd/config.json\n\
            \twelsib-httpd --host=192.168.0.100 --config=/etc/welsib-httpd/config.json\n\
        "
        )
    }
}