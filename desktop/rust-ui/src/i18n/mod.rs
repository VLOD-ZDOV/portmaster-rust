//! Lightweight RU/EN i18n dictionary synced with localStorage (`pm_language`).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    En,
    Ru,
}

impl Lang {
    pub fn load() -> Self {
        if let Some(win) = web_sys::window() {
            if let Ok(Some(storage)) = win.local_storage() {
                if let Ok(Some(val)) = storage.get_item("pm_language") {
                    if val == "ru" {
                        return Self::Ru;
                    } else if val == "en" {
                        return Self::En;
                    }
                }
            }
            if let Some(lang) = win.navigator().language() {
                if lang.to_lowercase().starts_with("ru") {
                    return Self::Ru;
                }
            }
        }
        Self::Ru
    }

    pub fn save(self) {
        if let Some(win) = web_sys::window() {
            if let Ok(Some(storage)) = win.local_storage() {
                let code = match self {
                    Self::En => "en",
                    Self::Ru => "ru",
                };
                let _ = storage.set_item("pm_language", code);
            }
        }
    }
}

pub fn t(lang: Lang, key: &'static str) -> &'static str {
    match lang {
        Lang::Ru => match key {
            // Pages & Navigation
            "nav.apps" => "Приложения",
            "nav.monitor" => "Активность",
            "nav.vpn" => "Xray VPN",
            "nav.settings" => "Настройки",
            "nav.dash" => "Панель статуса",
            "nav.pause" => "Пауза защиты",
            "nav.service" => "Управление службой",
            "nav.restart" => "Перезапустить ядро",
            "nav.shutdown" => "Выключить Portmaster",
            "nav.pause_vpn_5" => "Пауза VPN на 5 мин",
            "nav.pause_vpn_15" => "Пауза VPN на 15 мин",
            "nav.pause_vpn_60" => "Пауза VPN на 1 час",
            "nav.pause_all_5" => "Пауза всего на 5 мин",
            "nav.pause_all_15" => "Пауза всего на 15 мин",
            "nav.pause_all_60" => "Пауза всего на 1 час",
            "nav.resume" => "Возобновить защиту",

            // Header
            "page.apps" => "Приложения",
            "page.app_detail" => "Профиль приложения",
            "page.xray" => "Xray VPN и маршрутизация",
            "page.settings" => "Глобальные настройки",
            "page.monitor" => "Сетевая активность",
            "page.support" => "Справка",
            "header.search" => "Поиск приложений, доменов, настроек...",
            "header.vpn_offline" => "VPN: Откл.",
            "header.core" => "Ядро",
            "header.connecting" => "Подкл...",
            "header.offline" => "Оффлайн",

            // SideDash
            "dash.protected" => "Защита активна",
            "dash.paused" => "Защита на паузе",
            "dash.vpn_active" => "Xray VPN подключён",
            "dash.connecting" => "Подключение к ядру...",
            "dash.subsystems" => "Подсистемы",
            "dash.dns" => "Secure DNS",
            "dash.firewall" => "Файрвол",
            "dash.vpn" => "Xray VPN",
            "dash.history" => "История",
            "dash.scout" => "Активные приложения",
            "dash.alerts" => "Уведомления и запросы",
            "dash.clear" => "Очистить",
            "dash.no_alerts" => "Нет активных уведомлений.",
            "dash.allow_all" => "Разрешить все",
            "dash.block_all" => "Заблокировать все",
            "dash.allow" => "Разрешить",
            "dash.block" => "Блокировать",

            // Apps Overview
            "apps.search" => "Поиск приложений...",
            "apps.all" => "Все",
            "apps.active" => "Активные",
            "apps.offline" => "Неактивные",
            "apps.vpn" => "Через VPN",
            "apps.split" => "Split Tunnel",
            "apps.blocked" => "С блокировками",
            "apps.global_vpn" => "Глобальный VPN:",
            "apps.reset_routing" => "Сбросить переопределения",
            "apps.new_profile" => "+ Профиль",
            "apps.import_profile" => "Импорт JSON",
            "apps.empty" => "Нет приложений по выбранному фильтру.",
            "apps.active_group" => "Активные сейчас",
            "apps.offline_group" => "Неактивные профили",
            "apps.permitted" => "Разрешено",
            "apps.blocked_btn" => "Заблокировано",
            "apps.ask_btn" => "Запрос",
            "apps.vpn_auto_on" => "VPN: Авто (ВКЛ)",
            "apps.vpn_auto_off" => "VPN: Авто (ВЫКЛ)",
            "apps.vpn_always" => "VPN: Всегда",
            "apps.vpn_direct" => "VPN: Прямо",
            "apps.badge_blocked" => "Блок интернета",
            "apps.badge_xray" => "Xray",
            "apps.badge_st" => "Split Tun",
            "apps.badge_custom" => "Настроен",

            // App Detail
            "app.back" => "Приложения",
            "app.tab_conns" => "Соединения",
            "app.tab_rules" => "Правила и маршрутизация",
            "app.tab_settings" => "Настройки приложения",
            "app.tab_details" => "Детали и тестирование",
            "app.action_permit" => "Разрешить (Permit)",
            "app.action_ask" => "Спрашивать (Ask)",
            "app.action_block" => "Блокировать интернет",
            "app.export" => "Экспорт JSON",
            "app.delete_profile" => "Удалить профиль",
            "app.promote_global" => "В глобальные правила",
            "app.path_tester" => "Проверка пути (Fingerprint Tester)",
            "app.path_tester_hint" => "Введите путь бинарника и нажмите «Проверить», чтобы убедиться, подходит ли он к профилю.",
            "app.test_btn" => "Проверить",

            // Monitor
            "mon.title" => "Сетевая активность",
            "mon.clear" => "Очистить историю",
            "mon.scope_all" => "Все",
            "mon.scope_inet" => "Интернет",
            "mon.scope_lan" => "LAN",
            "mon.scope_local" => "Локальные",
            "mon.scope_in" => "Входящие",
            "mon.verdict_all" => "Все вердикты",
            "mon.verdict_ok" => "Разрешённые",
            "mon.verdict_blk" => "Блокированные",
            "mon.col_status" => "Статус",
            "mon.col_target" => "Домен / IP",
            "mon.col_loc" => "Страна",
            "mon.col_app" => "Приложение",
            "mon.col_proto" => "Протокол",
            "mon.col_traffic" => "Трафик",
            "mon.col_time" => "Время",

            // Xray
            "xray.connected" => "Активен",
            "xray.offline" => "Отключён",
            "xray.connect" => "Подключить",
            "xray.disconnect" => "Отключить",
            "xray.download_bin" => "Скачать / Обновить xray-rust",
            "xray.download_geo" => "Скачать GeoIP & GeoSite",
            "xray.add_sub" => "Добавить подписку",
            "xray.add_manual" => "Добавить ссылку напрямую",
            "xray.subs" => "Подписки и ключи",
            "xray.nodes" => "Серверы Xray",
            "xray.rules" => "Активные правила маршрутизации",
            "xray.map_title" => "Интерактивная карта узлов",

            // Settings & UI Switcher
            "set.title" => "Глобальные настройки",
            "set.search" => "Поиск настройки...",
            "set.expertise" => "Уровень:",
            "set.lang" => "Язык интерфейса",
            "set.export" => "Экспорт настроек",
            "set.import" => "Импорт настроек",
            "set.append_quick" => "Добавлять в конец списка (не заменять текущие)",
            "set.ui_title" => "Интерфейсы и ОЗУ",
            "set.port_open" => "Держать веб-порт :817 открытым постоянно",
            "set.port_listening" => "Слушает",
            "set.port_closed" => "Порт закрыт — работает Unix-сокет IPC",
            "set.live_ram" => "Сейчас ОЗУ",
            "set.bench_ram" => "База замера",
            "set.activate" => "Сделать основным",
            "set.active" => "Основной",
            "set.not_installed" => "Не установлен",
            _ => key,
        },
        Lang::En => match key {
            // Pages & Navigation
            "nav.apps" => "Apps",
            "nav.monitor" => "Monitor",
            "nav.vpn" => "Xray VPN",
            "nav.settings" => "Settings",
            "nav.dash" => "Status Dash",
            "nav.pause" => "Pause Protection",
            "nav.service" => "Service Control",
            "nav.restart" => "Restart Core",
            "nav.shutdown" => "Shutdown Portmaster",
            "nav.pause_vpn_5" => "Pause VPN for 5 min",
            "nav.pause_vpn_15" => "Pause VPN for 15 min",
            "nav.pause_vpn_60" => "Pause VPN for 1 hour",
            "nav.pause_all_5" => "Pause All for 5 min",
            "nav.pause_all_15" => "Pause All for 15 min",
            "nav.pause_all_60" => "Pause All for 1 hour",
            "nav.resume" => "Resume Protection",

            // Header
            "page.apps" => "Applications",
            "page.app_detail" => "Application Profile",
            "page.xray" => "VPN (Xray)",
            "page.settings" => "Global Settings",
            "page.monitor" => "Network Activity",
            "page.support" => "Support",
            "header.search" => "Search apps, domains, settings...",
            "header.vpn_offline" => "VPN: Offline",
            "header.core" => "Core",
            "header.connecting" => "Connecting",
            "header.offline" => "Offline",

            // SideDash
            "dash.protected" => "Secured",
            "dash.paused" => "Protection Paused",
            "dash.vpn_active" => "Xray VPN Connected",
            "dash.connecting" => "Connecting to Core...",
            "dash.subsystems" => "Subsystems",
            "dash.dns" => "Secure DNS",
            "dash.firewall" => "Privacy Filter",
            "dash.vpn" => "Xray VPN",
            "dash.history" => "History",
            "dash.scout" => "Active Applications",
            "dash.alerts" => "Notifications & Prompts",
            "dash.clear" => "Clear",
            "dash.no_alerts" => "No active notifications.",
            "dash.allow_all" => "Allow All",
            "dash.block_all" => "Block All",
            "dash.allow" => "Allow",
            "dash.block" => "Block",

            // Apps Overview
            "apps.search" => "Search applications...",
            "apps.all" => "All",
            "apps.active" => "Active",
            "apps.offline" => "Offline",
            "apps.vpn" => "VPN",
            "apps.split" => "Split Tunnel",
            "apps.blocked" => "Blocked",
            "apps.global_vpn" => "Global VPN:",
            "apps.reset_routing" => "Reset Overrides",
            "apps.new_profile" => "+ New Profile",
            "apps.import_profile" => "Import JSON",
            "apps.empty" => "No applications found matching the filter.",
            "apps.active_group" => "Active Now",
            "apps.offline_group" => "Inactive Profiles",
            "apps.permitted" => "Permitted",
            "apps.blocked_btn" => "Blocked",
            "apps.ask_btn" => "Ask",
            "apps.vpn_auto_on" => "VPN: Auto (ON)",
            "apps.vpn_auto_off" => "VPN: Auto (OFF)",
            "apps.vpn_always" => "VPN: Always ON",
            "apps.vpn_direct" => "VPN: Direct",
            "apps.badge_blocked" => "Internet Blocked",
            "apps.badge_xray" => "Xray",
            "apps.badge_st" => "Split Tun",
            "apps.badge_custom" => "Custom",

            // App Detail
            "app.back" => "Applications",
            "app.tab_conns" => "Connections",
            "app.tab_rules" => "Rules & Routing",
            "app.tab_settings" => "App Settings",
            "app.tab_details" => "Details & Tester",
            "app.action_permit" => "Permit",
            "app.action_ask" => "Prompt (Ask)",
            "app.action_block" => "Block Internet",
            "app.export" => "Export JSON",
            "app.delete_profile" => "Delete Profile",
            "app.promote_global" => "Promote to Global",
            "app.path_tester" => "Executable Path Rule Tester",
            "app.path_tester_hint" => "Test whether a binary path matches this profile's linked path or fingerprints.",
            "app.test_btn" => "Test Path",

            // Monitor
            "mon.title" => "Network Activity",
            "mon.clear" => "Clear History",
            "mon.scope_all" => "All",
            "mon.scope_inet" => "Internet",
            "mon.scope_lan" => "LAN",
            "mon.scope_local" => "Local",
            "mon.scope_in" => "Inbound",
            "mon.verdict_all" => "All Verdicts",
            "mon.verdict_ok" => "Permitted",
            "mon.verdict_blk" => "Blocked",
            "mon.col_status" => "Status",
            "mon.col_target" => "Target Domain / IP",
            "mon.col_loc" => "Location",
            "mon.col_app" => "Application",
            "mon.col_proto" => "Protocol",
            "mon.col_traffic" => "Traffic",
            "mon.col_time" => "Time",

            // Xray
            "xray.connected" => "Active",
            "xray.offline" => "Offline",
            "xray.connect" => "Connect VPN",
            "xray.disconnect" => "Disconnect",
            "xray.download_bin" => "Download / Update xray-rust",
            "xray.download_geo" => "Download GeoIP & GeoSite",
            "xray.add_sub" => "Add Subscription",
            "xray.add_manual" => "Add Direct Link",
            "xray.subs" => "Subscriptions & Gateway Keys",
            "xray.nodes" => "Xray Server Nodes",
            "xray.rules" => "Active Routing Rules",
            "xray.map_title" => "Interactive Node Map",

            // Settings & UI Switcher
            "set.title" => "Global Settings",
            "set.search" => "Search settings...",
            "set.expertise" => "Level:",
            "set.lang" => "Language",
            "set.export" => "Export Settings",
            "set.import" => "Import Settings",
            "set.append_quick" => "Append to bottom (do not replace existing)",
            "set.ui_title" => "User Interfaces & RAM Footprint",
            "set.port_open" => "Keep Web API Port :817 Continuously Open",
            "set.port_listening" => "Listening on",
            "set.port_closed" => "Port Closed — Unix Socket IPC Active",
            "set.live_ram" => "Live RAM",
            "set.bench_ram" => "Measured",
            "set.activate" => "Set Default",
            "set.active" => "Active",
            "set.not_installed" => "Not Installed",
            _ => key,
        },
    }
}
