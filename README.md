# cli-modbus-viewer

CLI утилита для опроса Modbus TCP устройств с отображением регистров в табличной форме.

![demo](https://raw.githubusercontent.com/Lolita1001/cli-modbus-viewer/assets/media/cli-modbus-viewer.png)

Английская версия: [README.en.md](README.en.md)

## Возможности

- Подключение к Modbus TCP устройствам
- Табличное отображение регистров
- Множественные форматы: Hex, Int16, UInt16, Binary, Bool
- Поддержка всех типов регистров: Holding, Input, Coils, Discrete
- Watch-режим для непрерывного мониторинга

## Установка

```bash
cargo build --release
```

Запуск после сборки:

```bash
./target/release/cli-modbus-viewer --help
```

## Использование

`-h/--host` принимает IP или hostname (например, `localhost`).

```bash
# Базовый опрос
cli-modbus-viewer -h 192.168.1.100 --holding 0-10

# Дефолтный тип (holding), если типы регистров не указаны
cli-modbus-viewer -h 192.168.1.100 0-10

# Опрос разных типов регистров
cli-modbus-viewer -h 192.168.1.100 --holding 0-5 --input 10-15 --coils 0-7

# Порт / unit id / таймаут
cli-modbus-viewer -h 192.168.1.100 -p 502 -u 1 -t 2000 --holding 0-20

# Watch-режим
cli-modbus-viewer -h 192.168.1.100 --holding 0-10 -w --interval 500
```


## Структура проекта

```
modbus-viewer/
├── Cargo.toml          # Зависимости проекта
├── Cargo.lock          # Lock-файл зависимостей
├── README.md           # Описание проекта (RU)
├── README.en.md        # Описание проекта (EN)
├── src/
│   ├── main.rs         # Точка входа
│   ├── cli.rs          # CLI (clap) и валидация аргументов
│   ├── addr.rs         # Парсер адресов регистров
│   ├── modbus.rs       # Modbus TCP: подключение и чтение регистров
│   └── render.rs       # Таблица и форматирование значений
└── target/             # Артефакты сборки
```
