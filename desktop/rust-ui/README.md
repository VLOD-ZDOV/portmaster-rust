# Portmaster Native Rust UI (Leptos CSR)

Быстрый, легковесный интерфейс Portmaster, написанный на Rust (**Leptos 0.8 + Trunk + TailwindCSS v4**) и компилируемый в WebAssembly.

Размер собранного WASM-бандла составляет всего **~750 КБ** (в ~20 раз легче Angular-бандла), обеспечивая мгновенный запуск без пауз сборщика мусора (GC).

---

## Как запустить для тестирования

### 1. Запуск в режиме разработки (Dev Server)
Для запуска локального сервера с горячей перезагрузкой (Hot-Reload):

```bash
cd desktop/rust-ui
trunk serve
```

После запуска откройте в браузере:
👉 **`http://127.0.0.1:8080`**

*(Если порт 8080 занят, можно указать любой другой: `trunk serve --port 3000`)*

Интерфейс автоматически подключится к запущенному ядру Portmaster по адресу `ws://127.0.0.1:817/v1/database/ws`.
Если ядро временно остановлено, интерфейс переходит в безопасный offline/seed-режим и автоматически восстанавливает связь при старте ядра.

---

### 2. Сборка релизного бандла
Собрать оптимизированный статичный WebAssembly-бандл:

```bash
cd desktop/rust-ui
trunk build --release
```

Файлы сборки помещаются в `desktop/rust-ui/dist/` (`index.html`, `.wasm`, `.js`, `.css`).

---

### Необходимые инструменты
- `rustup target add wasm32-unknown-unknown`
- `trunk` (`cargo install trunk`)
- `tailwindcss` CLI (`cargo install tailwindcss-extra` или бинарник Tailwind v4)
