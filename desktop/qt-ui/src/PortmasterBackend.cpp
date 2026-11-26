#include "PortmasterBackend.h"

#include <QApplication>
#include <QDateTime>
#include <QIcon>
#include <QJsonArray>
#include <QJsonDocument>
#include <QMenu>
#include <QPainter>
#include <QPixmap>
#include <QPolygon>
#include <QRegularExpression>
#include <QSettings>
#include <QUrl>
#include <QtMath>

namespace {
struct CountryCenter { const char* code; double lat; double lon; };
static const CountryCenter kCountryCenters[] = {
    {"AD", 42.54, 1.58}, {"AE", 23.42, 53.85}, {"AL", 41.15, 20.17}, {"AM", 40.07, 45.04},
    {"AR", -38.42, -63.62}, {"AT", 47.52, 14.55}, {"AU", -25.27, 133.78}, {"AZ", 40.14, 47.58},
    {"BA", 43.92, 17.68}, {"BE", 50.50, 4.47}, {"BG", 42.73, 25.49}, {"BH", 26.07, 50.55},
    {"BR", -14.24, -51.93}, {"BY", 53.71, 27.95}, {"CA", 56.13, -106.35}, {"CH", 46.82, 8.23},
    {"CL", -35.68, -71.54}, {"CN", 35.86, 104.20}, {"CO", 4.57, -74.30}, {"CR", 9.75, -83.75},
    {"CY", 35.13, 33.43}, {"CZ", 49.82, 15.47}, {"DE", 51.17, 10.45}, {"DK", 56.26, 9.50},
    {"DO", 18.74, -70.16}, {"DZ", 28.03, 1.66}, {"EC", -1.83, -78.18}, {"EE", 58.60, 25.01},
    {"EG", 26.82, 30.80}, {"ES", 40.46, -3.75}, {"FI", 61.92, 25.75}, {"FR", 46.23, 2.21},
    {"GB", 54.00, -2.00}, {"UK", 54.00, -2.00}, {"GE", 42.32, 43.36}, {"GR", 39.07, 21.82},
    {"HK", 22.32, 114.17}, {"HR", 45.10, 15.20}, {"HU", 47.16, 19.50}, {"ID", -2.55, 118.02},
    {"IE", 53.41, -8.24}, {"IL", 31.05, 34.85}, {"IN", 20.59, 78.96}, {"IQ", 33.22, 43.68},
    {"IR", 32.43, 53.69}, {"IS", 64.96, -19.02}, {"IT", 41.87, 12.57}, {"JP", 36.20, 138.25},
    {"KE", 0.02, 37.91}, {"KG", 41.20, 74.77}, {"KR", 35.91, 127.77}, {"KZ", 48.02, 67.88},
    {"LT", 55.17, 23.88}, {"LU", 49.82, 6.13}, {"LV", 56.95, 24.11}, {"MD", 47.41, 28.37},
    {"MX", 23.63, -102.55}, {"MY", 4.21, 101.98}, {"NG", 9.08, 8.68}, {"NL", 52.13, 5.29},
    {"NO", 60.47, 8.47}, {"NZ", -40.90, 174.89}, {"PH", 12.88, 121.77}, {"PK", 30.38, 69.35},
    {"PL", 51.92, 19.15}, {"PT", 39.40, -8.22}, {"RO", 45.94, 24.97}, {"RS", 44.02, 21.01},
    {"RU", 61.52, 105.32}, {"SA", 23.89, 45.08}, {"SE", 60.13, 18.64}, {"SG", 1.35, 103.82},
    {"SI", 46.15, 14.99}, {"SK", 48.67, 19.70}, {"TH", 15.87, 100.99}, {"TR", 38.96, 35.24},
    {"TW", 23.70, 120.96}, {"UA", 48.38, 31.17}, {"US", 37.09, -95.71}, {"UZ", 41.38, 64.59},
    {"VN", 14.06, 108.28}, {"ZA", -30.56, 22.94}
};

bool lookupCountry(const QString& code, double& lat, double& lon) {
    const QString up = code.trimmed().toUpper();
    for (const auto& c : kCountryCenters) {
        if (up == QLatin1String(c.code)) {
            lat = c.lat;
            lon = c.lon;
            return true;
        }
    }
    return false;
}

QPointF projectNormalized(double lat, double lon) {
    double nx = qBound(0.0, (lon + 180.0) / 360.0, 1.0);
    double ny = qBound(0.0, (84.0 - lat) / 145.0, 1.0);
    return QPointF(nx, ny);
}

QJsonValue effectiveCfgVal(const QJsonObject& s) {
    if (s.contains(QLatin1String("Value")) && !s.value(QLatin1String("Value")).isNull()) {
        return s.value(QLatin1String("Value"));
    }
    return s.value(QLatin1String("DefaultValue"));
}

// Portmaster stores app Config both flat ("spn/use") and nested ({"spn":{"use":...}}).
QJsonValue readProfileCfg(const QJsonObject& cfg, const QString& path) {
    if (cfg.contains(path)) {
        const QJsonValue direct = cfg.value(path);
        if (!direct.isUndefined()) return direct;
    }
    const QStringList parts = path.split('/');
    QJsonValue cur(cfg);
    for (const QString& part : parts) {
        if (!cur.isObject()) return QJsonValue::Undefined;
        cur = cur.toObject().value(part);
        if (cur.isUndefined()) return QJsonValue::Undefined;
    }
    return cur;
}

void writeProfileCfg(QJsonObject& cfg, const QString& path, const QJsonValue& value) {
    if (value.isUndefined() || value.isNull() || path.contains('/')) {
        cfg.remove(path);
    }
    const QStringList parts = path.split('/');
    if (parts.isEmpty()) return;

    auto setRec = [&](auto& self, QJsonObject& obj, int idx) -> void {
        const QString& part = parts[idx];
        if (idx == parts.size() - 1) {
            if (value.isUndefined() || value.isNull()) obj.remove(part);
            else obj.insert(part, value);
            return;
        }
        QJsonValue sub = obj.value(part);
        if (!sub.isObject()) {
            if (value.isUndefined() || value.isNull()) return;
            QJsonObject child;
            self(self, child, idx + 1);
            obj.insert(part, child);
            return;
        }
        QJsonObject child = sub.toObject();
        self(self, child, idx + 1);
        if (child.isEmpty() && (value.isUndefined() || value.isNull())) obj.remove(part);
        else obj.insert(part, child);
    };
    setRec(setRec, cfg, 0);
}

QString protoNameFromVal(const QJsonValue& v) {
    if (v.isString()) return v.toString().toUpper();
    switch (v.toInt(-1)) {
        case 6: return QStringLiteral("TCP");
        case 17: return QStringLiteral("UDP");
        case 1: return QStringLiteral("ICMP");
        case 58: return QStringLiteral("ICMPv6");
        case 2: return QStringLiteral("IGMP");
        default: return v.toInt(0) > 0 ? QString::number(v.toInt()) : QString();
    }
}

QString verdictSlugFromVal(const QJsonValue& v, bool tunneled) {
    if (v.isString()) {
        QString s = v.toString().toLower();
        return s.isEmpty() ? (tunneled ? QStringLiteral("tunneled") : QStringLiteral("accept")) : s;
    }
    switch (v.toInt(-1)) {
        case 0:
        case 1: return QStringLiteral("ask");
        case 2: return tunneled ? QStringLiteral("tunneled") : QStringLiteral("accept");
        case 3: return QStringLiteral("block");
        case 4: return QStringLiteral("drop");
        case 5: return QStringLiteral("dns");
        case 6: return QStringLiteral("tunneled");
        case 7: return QStringLiteral("failed");
        case 8: return QStringLiteral("splittun");
        default: return QStringLiteral("accept");
    }
}

QString scopeSlugFromVal(const QJsonValue& v) {
    if (v.isString()) return v.toString().toLower();
    switch (v.toInt(-1)) {
        case 0:
        case 1: return QStringLiteral("local");
        case 2:
        case 3:
        case -2:
        case -3: return QStringLiteral("lan");
        default: return QStringLiteral("internet");
    }
}

QJsonObject normalizeConn(const QJsonObject& raw) {
    // Handles both HTTP SQLite rows (snake_case) and live WS runtime:netquery/data/ (PascalCase).
    auto pick = [&](const char* snake, const char* pascal) -> QJsonValue {
        if (raw.contains(QLatin1String(snake))) return raw.value(QLatin1String(snake));
        if (raw.contains(QLatin1String(pascal))) return raw.value(QLatin1String(pascal));
        return QJsonValue::Undefined;
    };
    const bool tunneled = pick("tunneled", "Tunneled").toBool(false);
    QJsonValue vVal = raw.contains(QLatin1String("verdict"))
                          ? raw.value(QLatin1String("verdict"))
                          : raw.value(QLatin1String("ActiveVerdict"));
    const QString verdict = verdictSlugFromVal(vVal, tunneled);

    const QJsonObject extra = pick("extra_data", "ExtraData").toObject();
    QString reason = raw.value(QLatin1String("verdict_reason")).toString();
    if (reason.isEmpty()) {
        QJsonValue rObj = extra.contains(QLatin1String("reason"))
                              ? extra.value(QLatin1String("reason"))
                              : extra.value(QLatin1String("Reason"));
        if (rObj.isObject()) {
            reason = rObj.toObject().value(QLatin1String("Msg")).toString();
        }
    }

    const QString profRaw = pick("profile", "ProfileID").toString();
    QString dbKey;
    if (!profRaw.isEmpty()) {
        dbKey = profRaw.startsWith(QLatin1String("core:"))
                    ? profRaw
                    : QStringLiteral("core:profiles/%1").arg(profRaw.contains('/') ? profRaw : QStringLiteral("local/") + profRaw);
    }

    return QJsonObject{
        {QStringLiteral("id"), pick("id", "ID").toString()},
        {QStringLiteral("profile"), profRaw},
        {QStringLiteral("profileKey"), dbKey},
        {QStringLiteral("profile_name"), pick("profile_name", "ProfileName").toString()},
        {QStringLiteral("domain"), pick("domain", "Domain").toString()},
        {QStringLiteral("remote_ip"), pick("remote_ip", "RemoteIP").toString()},
        {QStringLiteral("remote_port"), pick("remote_port", "RemotePort").toInt()},
        {QStringLiteral("ip_protocol"), protoNameFromVal(pick("ip_protocol", "IPProtocol"))},
        {QStringLiteral("direction"), pick("direction", "Direction").toString(QStringLiteral("outbound")).toLower()},
        {QStringLiteral("scope"), scopeSlugFromVal(pick("scope", "Scope"))},
        {QStringLiteral("country"), pick("country", "Country").toString()},
        {QStringLiteral("as_owner"), pick("as_owner", "ASOwner").toString()},
        {QStringLiteral("verdict"), verdict},
        {QStringLiteral("tunneled"), tunneled || verdict == QLatin1String("tunneled")},
        {QStringLiteral("reason"), reason},
        {QStringLiteral("bytes_received"), static_cast<double>(pick("bytes_received", "BytesReceived").toDouble(0))},
        {QStringLiteral("bytes_sent"), static_cast<double>(pick("bytes_sent", "BytesSent").toDouble(0))},
        {QStringLiteral("active"), pick("active", "Active").toBool(false)},
    };
}
} // namespace

PortmasterBackend::PortmasterBackend(bool startBackground, bool withTray, QObject* parent)
    : QObject(parent),
      m_api(new PortApiClient(this)),
      m_appsTimer(new QTimer(this)),
      m_settingsTimer(new QTimer(this)),
      m_connsTimer(new QTimer(this)),
      m_withTray(withTray) {
    QSettings qs(QStringLiteral("portmaster"), QStringLiteral("qt-ui"));
    m_ru = qs.value(QStringLiteral("lang"), QStringLiteral("ru")).toString() == QLatin1String("ru");

    m_appsTimer->setSingleShot(true);
    m_appsTimer->setInterval(60);
    connect(m_appsTimer, &QTimer::timeout, this, [this]() {
        rebuildAppsCache();
        if (!m_selectedProfileKey.isEmpty()) rebuildSelectedAppCache();
    });

    m_settingsTimer->setSingleShot(true);
    m_settingsTimer->setInterval(60);
    connect(m_settingsTimer, &QTimer::timeout, this, &PortmasterBackend::rebuildSettingsCache);

    m_connsTimer->setSingleShot(true);
    m_connsTimer->setInterval(120);
    connect(m_connsTimer, &QTimer::timeout, this, [this]() {
        rebuildConnectionsCache();
        rebuildAppsCache();
        if (!m_selectedProfileKey.isEmpty()) rebuildSelectedAppCache();
    });

    if (m_withTray) setupTray();
    connect(m_api, &PortApiClient::connectionChanged, this, &PortmasterBackend::onConnectionChanged);
    m_api->start();

    auto* poll = new QTimer(this);
    poll->setInterval(3000);
    connect(poll, &QTimer::timeout, this, &PortmasterBackend::refreshHttpData);
    poll->start();
    Q_UNUSED(startBackground);
}

QString PortmasterBackend::formatBytes(qint64 bytes) const {
    if (bytes >= 1024LL * 1024 * 1024) return QStringLiteral("%1 GB").arg(bytes / (1024.0 * 1024.0 * 1024.0), 0, 'f', 2);
    if (bytes >= 1024LL * 1024) return QStringLiteral("%1 MB").arg(bytes / (1024.0 * 1024.0), 0, 'f', 1);
    if (bytes >= 1024) return QStringLiteral("%1 KB").arg(bytes / 1024.0, 0, 'f', 0);
    return QStringLiteral("%1 B").arg(bytes);
}

QJsonValue PortmasterBackend::effectiveValue(const QString& key) const {
    return effectiveCfgVal(m_settings.value(key));
}

bool PortmasterBackend::globalVpn() const {
    return effectiveValue(QStringLiteral("spn/use")).toBool(false);
}

bool PortmasterBackend::globalSplit() const {
    return effectiveValue(QStringLiteral("splittun/use")).toBool(false);
}

bool PortmasterBackend::autoConnectXray() const {
    return effectiveValue(QStringLiteral("xray/autoConnect")).toBool(false);
}

bool PortmasterBackend::historyEnabled() const {
    return effectiveValue(QStringLiteral("history/enable")).toBool(true);
}

void PortmasterBackend::setLanguage(const QString& code) {
    bool next = (code == QLatin1String("ru"));
    if (m_ru == next) return;
    m_ru = next;
    QSettings qs(QStringLiteral("portmaster"), QStringLiteral("qt-ui"));
    qs.setValue(QStringLiteral("lang"), code);
    emit langChanged();
    rebuildAppsCache();
    rebuildSettingsCache();
}

QString PortmasterBackend::t(const QString& key) const {
    static const QMap<QString, QString> ru = {
        {"nav.apps", "Приложения"}, {"nav.monitor", "Монитор активности"},
        {"nav.xray", "Xray VPN"}, {"nav.settings", "Настройки"},
        {"nav.help", "Справка"}, {"nav.tools", "Инструменты"},
        {"nav.pause", "Пауза"}, {"nav.power", "Питание"},
        {"apps.title", "Приложения"}, {"apps.search", "Поиск приложений..."},
        {"apps.active", "Активные"}, {"apps.all", "Все приложения"},
        {"apps.globalRouting", "Глобальная маршрутизация:"},
        {"apps.selective", "Выборочно"}, {"apps.allVpn", "Всё через Xray"},
        {"apps.resetDirect", "Сбросить переопределения"},
        {"apps.newProfile", "+ Создать профиль"}, {"apps.import", "Импорт JSON"},
        {"apps.filterAll", "Все"}, {"apps.filterActive", "Активные"},
        {"apps.filterOffline", "Неактивные"}, {"apps.filterVpn", "Xray"},
        {"apps.filterBlocked", "Блок."},
        {"app.back", "← Приложения"}, {"app.permit", "Разрешить"},
        {"app.ask", "Запрос"}, {"app.block", "Блокировать"},
        {"app.connections", "Соединения"}, {"app.rules", "Правила и сеть"},
        {"app.settings", "Настройки профиля"}, {"app.details", "Детали и тест пути"},
        {"app.promote", "В глобальные"}, {"app.export", "Экспорт JSON"},
        {"app.delete", "Удалить профиль"},
        {"mon.title", "Сетевая активность"}, {"mon.clear", "Очистить историю"},
        {"mon.all", "Все"}, {"mon.internet", "Интернет"}, {"mon.lan", "LAN"},
        {"mon.local", "Локальные"}, {"mon.inbound", "Входящие"},
        {"mon.permitted", "Разрешённые"}, {"mon.blocked", "Блокированные"},
        {"xray.title", "Xray VPN Шлюз и Маршрутизация"},
        {"xray.connect", "Подключить"}, {"xray.disconnect", "Отключить"},
        {"xray.dlBin", "Скачать xray-rust"}, {"xray.dlGeo", "Скачать GeoIP & GeoSite"},
        {"xray.addSub", "Добавить подписку"}, {"xray.addManual", "Добавить ссылку"},
        {"xray.defaultVpn", "Весь трафик через Xray по умолчанию"},
        {"xray.autoConnect", "Автоподключение Xray при запуске"},
        {"set.title", "Глобальные настройки"}, {"set.search", "Поиск настройки..."},
        {"set.uiTitle", "Интерфейс, ОЗУ и безопасность веб-порта :817"},
        {"set.keepPort", "Держать веб-порт :817 открытым постоянно"},
        {"set.portOpen", "Порт открыт"},
        {"set.portClosed", "Порт закрыт — работает только локальный Unix-сокет"},
        {"set.export", "Экспорт"}, {"set.import", "Импорт"},
        {"set.appendDns", "Добавлять в конец (не заменять текущие)"},
        {"dash.protected", "Защита активна"}, {"dash.paused", "На паузе"},
        {"dash.offline", "Нет связи с ядром"},
        {"dash.prompts", "Ожидающие запросы"}, {"dash.alerts", "Уведомления"},
        {"dash.allowAll", "Разрешить все"}, {"dash.blockAll", "Блокировать все"},
    };
    static const QMap<QString, QString> en = {
        {"nav.apps", "Apps"}, {"nav.monitor", "Monitor"},
        {"nav.xray", "Xray VPN"}, {"nav.settings", "Settings"},
        {"nav.help", "Support"}, {"nav.tools", "Tools"},
        {"nav.pause", "Pause"}, {"nav.power", "Power"},
        {"apps.title", "Applications"}, {"apps.search", "Search applications..."},
        {"apps.active", "Active"}, {"apps.all", "All Applications"},
        {"apps.globalRouting", "Global Routing:"},
        {"apps.selective", "Selective"}, {"apps.allVpn", "All via Xray"},
        {"apps.resetDirect", "Reset Overrides"},
        {"apps.newProfile", "+ New Profile"}, {"apps.import", "Import JSON"},
        {"apps.filterAll", "All"}, {"apps.filterActive", "Active"},
        {"apps.filterOffline", "Offline"}, {"apps.filterVpn", "Xray"},
        {"apps.filterBlocked", "Blocked"},
        {"app.back", "← Applications"}, {"app.permit", "Permit"},
        {"app.ask", "Ask"}, {"app.block", "Block"},
        {"app.connections", "Connections"}, {"app.rules", "Rules & Routing"},
        {"app.settings", "App Settings"}, {"app.details", "Details & Path Tester"},
        {"app.promote", "Promote Global"}, {"app.export", "Export JSON"},
        {"app.delete", "Delete Profile"},
        {"mon.title", "Network Activity"}, {"mon.clear", "Clear History"},
        {"mon.all", "All"}, {"mon.internet", "Internet"}, {"mon.lan", "LAN"},
        {"mon.local", "Local"}, {"mon.inbound", "Inbound"},
        {"mon.permitted", "Permitted"}, {"mon.blocked", "Blocked"},
        {"xray.title", "Xray VPN Gateway & Routing"},
        {"xray.connect", "Connect"}, {"xray.disconnect", "Disconnect"},
        {"xray.dlBin", "Download xray-rust"}, {"xray.dlGeo", "Download GeoIP & GeoSite"},
        {"xray.addSub", "Add Subscription"}, {"xray.addManual", "Add Direct Link"},
        {"xray.defaultVpn", "Route All Apps via Xray by Default"},
        {"xray.autoConnect", "Auto-Connect Xray on Startup"},
        {"set.title", "Global Settings"}, {"set.search", "Search settings..."},
        {"set.uiTitle", "User Interfaces, RAM Footprint & Port :817 Security"},
        {"set.keepPort", "Keep Web API Port :817 Continuously Open"},
        {"set.portOpen", "Listening"},
        {"set.portClosed", "Closed — Unix Socket IPC Active"},
        {"set.export", "Export"}, {"set.import", "Import"},
        {"set.appendDns", "Append to bottom (do not replace existing)"},
        {"dash.protected", "Protected"}, {"dash.paused", "Paused"},
        {"dash.offline", "Connecting to Core..."},
        {"dash.prompts", "Connection Prompts"}, {"dash.alerts", "Live Alerts"},
        {"dash.allowAll", "Allow All"}, {"dash.blockAll", "Block All"},
    };
    const auto& dict = m_ru ? ru : en;
    return dict.value(key, key);
}

QIcon PortmasterBackend::buildShieldIcon(const QColor& c) const {
    QPixmap pm(64, 64);
    pm.fill(Qt::transparent);
    QPainter p(&pm);
    p.setRenderHint(QPainter::Antialiasing);
    QPolygon poly;
    poly << QPoint(32, 6) << QPoint(54, 15) << QPoint(54, 33) << QPoint(32, 57) << QPoint(10, 33) << QPoint(10, 15);
    p.setBrush(c);
    p.setPen(QPen(Qt::white, 2));
    p.drawPolygon(poly);
    p.setPen(QPen(Qt::white, 4, Qt::SolidLine, Qt::RoundCap, Qt::RoundJoin));
    p.drawLine(22, 33, 29, 41);
    p.drawLine(29, 41, 43, 24);
    return QIcon(pm);
}

void PortmasterBackend::setupTray() {
    if (!QSystemTrayIcon::isSystemTrayAvailable()) return;
    m_tray = new QSystemTrayIcon(buildShieldIcon(QColor(0xd1, 0x2e, 0x2e)), this);
    auto* menu = new QMenu();
    auto* showAct = menu->addAction(QStringLiteral("Show Portmaster"));
    connect(showAct, &QAction::triggered, this, &PortmasterBackend::showWindowRequested);
    auto* vpnAct = menu->addAction(QStringLiteral("Toggle Xray VPN"));
    connect(vpnAct, &QAction::triggered, this, &PortmasterBackend::toggleXrayConnect);
    menu->addSeparator();
    auto* quitAct = menu->addAction(QStringLiteral("Quit Qt UI"));
    connect(quitAct, &QAction::triggered, qApp, &QApplication::quit);
    m_tray->setContextMenu(menu);
    connect(m_tray, &QSystemTrayIcon::activated, this, [this](QSystemTrayIcon::ActivationReason r) {
        if (r == QSystemTrayIcon::Trigger || r == QSystemTrayIcon::DoubleClick) {
            emit showWindowRequested();
        }
    });
    m_tray->show();
}

void PortmasterBackend::updateTrayIcon() {
    if (!m_tray) return;
    QColor c(0xd1, 0x2e, 0x2e);
    if (m_api->isConnected()) {
        if (m_xrayStatusMap.value(QStringLiteral("connected")).toBool()) c = QColor(0x4e, 0x97, 0xfa);
        else c = QColor(0x1d, 0xe9, 0x66);
    }
    m_tray->setIcon(buildShieldIcon(c));
}

void PortmasterBackend::onConnectionChanged(bool connected, const QString&) {
    emit connectionChanged();
    updateTrayIcon();
    if (connected) {
        startSubscriptions();
        refreshHttpData();
        if (!m_worldLoaded) fetchWorldMap();
    }
}

void PortmasterBackend::scheduleAppsRebuild() {
    if (!m_appsTimer->isActive()) m_appsTimer->start();
}

void PortmasterBackend::scheduleSettingsRebuild() {
    if (!m_settingsTimer->isActive()) m_settingsTimer->start();
}

void PortmasterBackend::scheduleConnsRebuild() {
    if (!m_connsTimer->isActive()) m_connsTimer->start();
}

void PortmasterBackend::startSubscriptions() {
    if (m_subscribed) return;
    m_subscribed = true;

    m_api->qsub(QStringLiteral("query core:profiles/"), [this](const QString& op, const QString& key, const QJsonValue& val) {
        if (op == QLatin1String("del")) m_profiles.remove(key);
        else if (val.isObject()) m_profiles.insert(key, val.toObject());
        scheduleAppsRebuild();
    });

    m_api->qsub(QStringLiteral("query config:"), [this](const QString& op, const QString& key, const QJsonValue& val) {
        if (op == QLatin1String("del") || !val.isObject()) return;
        QString k = key;
        k.remove(QStringLiteral("config:"));
        m_settings.insert(k, val.toObject());
        scheduleSettingsRebuild();
        if (k == QLatin1String("spn/use") || k == QLatin1String("splittun/use")) {
            scheduleAppsRebuild();
        }
    });

    m_api->qsub(QStringLiteral("query notifications:all/"), [this](const QString& op, const QString& key, const QJsonValue& val) {
        if (op == QLatin1String("del")) m_notifications.remove(key);
        else if (val.isObject()) m_notifications.insert(key, val.toObject());
        rebuildNotificationsCache();
    });

    m_api->qsub(QStringLiteral("query runtime:netquery/data/"), [this](const QString& op, const QString&, const QJsonValue& val) {
        if (op == QLatin1String("del") || !val.isObject()) return;
        QJsonObject obj = normalizeConn(val.toObject());
        const QString id = obj.value(QStringLiteral("id")).toString();
        bool found = false;
        for (int i = 0; i < m_connections.size(); ++i) {
            if (m_connections[i].value(QStringLiteral("id")).toString() == id) {
                m_connections[i] = obj;
                found = true;
                break;
            }
        }
        if (!found) {
            m_connections.prepend(obj);
            if (m_connections.size() > 400) m_connections.resize(400);
        }
        scheduleConnsRebuild();
    });
}

void PortmasterBackend::refreshHttpData() {
    if (!m_api->isConnected()) return;

    m_api->httpGet(QStringLiteral("/api/v1/ui/flavors"), [this](int code, const QByteArray& body, const QString&) {
        if (code == 200) {
            m_uiReportMap = QJsonDocument::fromJson(body).object().toVariantMap();
            emit uiReportChanged();
        }
    });
    m_api->httpGet(QStringLiteral("/api/v1/xray/status"), [this](int code, const QByteArray& body, const QString&) {
        if (code == 200) {
            QVariantMap st = QJsonDocument::fromJson(body).object().toVariantMap();
            if (!st.contains(QStringLiteral("nodeName"))) st.insert(QStringLiteral("nodeName"), st.value(QStringLiteral("node_name")));
            if (!st.contains(QStringLiteral("exitCountry"))) st.insert(QStringLiteral("exitCountry"), st.value(QStringLiteral("country")));
            if (!st.contains(QStringLiteral("inboundAddress"))) st.insert(QStringLiteral("inboundAddress"), st.value(QStringLiteral("inbound_address")));
            m_xrayStatusMap = st;
            rebuildMapPins();
            emit xrayChanged();
            updateTrayIcon();
        }
    });
    m_api->httpGet(QStringLiteral("/api/v1/xray/nodes"), [this](int code, const QByteArray& body, const QString&) {
        if (code == 200) {
            m_xrayNodesList.clear();
            for (const auto& v : QJsonDocument::fromJson(body).array()) {
                if (v.isObject()) m_xrayNodesList.append(v.toObject().toVariantMap());
            }
            rebuildMapPins();
            emit xrayChanged();
        }
    });
    m_api->httpGet(QStringLiteral("/api/v1/xray/subscriptions"), [this](int code, const QByteArray& body, const QString&) {
        if (code == 200) {
            m_xraySubsList.clear();
            for (const auto& v : QJsonDocument::fromJson(body).array()) {
                if (v.isObject()) {
                    QVariantMap m = v.toObject().toVariantMap();
                    m.insert(QStringLiteral("name"), QUrl::fromPercentEncoding(m.value(QStringLiteral("name")).toString().toUtf8()));
                    m_xraySubsList.append(m);
                }
            }
            emit xrayChanged();
        }
    });
    if (m_connections.isEmpty()) {
        const QByteArray payload = R"({"databases":["main"],"orderBy":[{"field":"started","desc":true}],"pageSize":250})";
        m_api->httpPost(QStringLiteral("/api/v1/netquery/query"), payload, QStringLiteral("application/json"),
            [this](int code, const QByteArray& body, const QString&) {
                if (code == 200) {
                    const QJsonArray arr = QJsonDocument::fromJson(body).object().value(QStringLiteral("results")).toArray();
                    m_connections.clear();
                    for (const auto& v : arr) {
                        if (v.isObject()) m_connections.append(normalizeConn(v.toObject()));
                    }
                    if (m_connections.size() > 400) m_connections.resize(400);
                    rebuildConnectionsCache();
                    rebuildAppsCache();
                    if (!m_selectedProfileKey.isEmpty()) rebuildSelectedAppCache();
                }
            });
    }
}

void PortmasterBackend::fetchWorldMap() {
    m_api->httpGet(QStringLiteral("/assets/world-50m.json"), [this](int code, const QByteArray& body, const QString&) {
        if (code != 200) return;
        const QJsonObject root = QJsonDocument::fromJson(body).object();
        const QJsonObject tr = root.value(QStringLiteral("transform")).toObject();
        const QJsonArray sc = tr.value(QStringLiteral("scale")).toArray();
        const QJsonArray tl = tr.value(QStringLiteral("translate")).toArray();
        if (sc.size() < 2 || tl.size() < 2) return;
        const double sx = sc[0].toDouble(), sy = sc[1].toDouble();
        const double tx = tl[0].toDouble(), ty = tl[1].toDouble();

        const QJsonArray arcs = root.value(QStringLiteral("arcs")).toArray();
        const QJsonArray landArcs = root.value(QStringLiteral("objects")).toObject().value(QStringLiteral("land")).toObject().value(QStringLiteral("arcs")).toArray();

        QVariantList polygons;
        for (const auto& polyVal : landArcs) {
            const QJsonArray rings = polyVal.toArray();
            if (rings.isEmpty()) continue;
            // Outer ring for clean fast Continent outline
            const QJsonArray ring = rings[0].toArray();
            QVariantList polyPts;
            double prevLon = 0.0;
            bool hasPrev = false;

            auto flushPoly = [&]() {
                if (polyPts.size() >= 3) polygons.append(QVariant::fromValue(polyPts));
                polyPts.clear();
            };

            for (const auto& idxVal : ring) {
                int idx = idxVal.toInt();
                bool rev = idx < 0;
                int realIdx = rev ? ~idx : idx;
                if (realIdx < 0 || realIdx >= arcs.size()) continue;
                const QJsonArray raw = arcs[realIdx].toArray();
                QVector<QPair<double, double>> pts;
                pts.reserve(raw.size());
                qint64 qx = 0, qy = 0;
                for (const auto& pVal : raw) {
                    const QJsonArray pa = pVal.toArray();
                    qx += pa[0].toInteger();
                    qy += pa[1].toInteger();
                    pts.append({qy * sy + ty, qx * sx + tx});
                }
                if (rev) std::reverse(pts.begin(), pts.end());
                for (const auto& ll : pts) {
                    if (ll.first < -58.0) {
                        flushPoly();
                        hasPrev = false;
                        continue;
                    }
                    if (hasPrev && std::abs(ll.second - prevLon) > 100.0) {
                        flushPoly();
                    }
                    QPointF np = projectNormalized(ll.first, ll.second);
                    polyPts.append(QVariantMap{{QStringLiteral("x"), np.x()}, {QStringLiteral("y"), np.y()}});
                    prevLon = ll.second;
                    hasPrev = true;
                }
            }
            flushPoly();
        }
        m_worldPolygons = polygons;
        m_worldLoaded = true;
        emit worldMapReady();
    });
}

void PortmasterBackend::rebuildMapPins() {
    QMap<QString, QVariantList> byC;
    QString activeId = m_xrayStatusMap.value(QStringLiteral("nodeId")).toString();
    bool isConn = m_xrayStatusMap.value(QStringLiteral("connected")).toBool();

    for (const auto& nv : m_xrayNodesList) {
        QVariantMap n = nv.toMap();
        QString cc = n.value(QStringLiteral("country")).toString().trimmed().toUpper();
        if (cc.isEmpty()) cc = QStringLiteral("??");
        byC[cc].append(n);
    }
    m_mapPinsCache.clear();
    for (auto it = byC.cbegin(); it != byC.cend(); ++it) {
        const QVariantList& list = it.value();
        double lat = 0, lon = 0;
        const QVariantMap firstNode = list.first().toMap();
        double nLat = firstNode.value(QStringLiteral("latitude")).toDouble();
        double nLon = firstNode.value(QStringLiteral("longitude")).toDouble();
        if (qAbs(nLat) > 0.01 || qAbs(nLon) > 0.01) {
            lat = nLat;
            lon = nLon;
        } else if (!lookupCountry(it.key(), lat, lon)) {
            continue;
        }
        QPointF p = projectNormalized(lat, lon);
        bool active = false;
        QString firstId = firstNode.value(QStringLiteral("id")).toString();
        for (const auto& nv : list) {
            if (isConn && nv.toMap().value(QStringLiteral("id")).toString() == activeId) active = true;
        }
        m_mapPinsCache.append(QVariantMap{
            {QStringLiteral("country"), it.key()},
            {QStringLiteral("count"), list.size()},
            {QStringLiteral("firstNodeId"), firstId},
            {QStringLiteral("active"), active},
            {QStringLiteral("nx"), p.x()},
            {QStringLiteral("ny"), p.y()},
        });
    }
}

void PortmasterBackend::rebuildAppsCache() {
    bool gVpn = globalVpn();
    bool gSplit = globalSplit();
    m_activeAppsCache.clear();
    m_offlineAppsCache.clear();
    m_allAppsCache.clear();

    QVector<QPair<QString, QJsonObject>> sorted;
    for (auto it = m_profiles.cbegin(); it != m_profiles.cend(); ++it) {
        const QJsonObject& p = it.value();
        const QString src = p.value(QStringLiteral("Source")).toString();
        const QString pid = p.value(QStringLiteral("ID")).toString();
        if (src == QLatin1String("special") || pid == QLatin1String("global-config") || pid.isEmpty()) {
            continue;
        }
        sorted.append({it.key(), p});
    }
    std::sort(sorted.begin(), sorted.end(), [](const auto& a, const auto& b) {
        QString na = a.second.value(QStringLiteral("Name")).toString(a.second.value(QStringLiteral("ID")).toString()).toLower();
        QString nb = b.second.value(QStringLiteral("Name")).toString(b.second.value(QStringLiteral("ID")).toString()).toLower();
        return na < nb;
    });

    for (const auto& pair : sorted) {
        const QString& key = pair.first;
        const QJsonObject& p = pair.second;
        QString id = p.value(QStringLiteral("ID")).toString();
        QString name = p.value(QStringLiteral("Name")).toString(id);
        QString path = p.value(QStringLiteral("PresentationPath")).toString();
        if (path.isEmpty()) path = p.value(QStringLiteral("LinkedPath")).toString();
        QJsonObject cfg = p.value(QStringLiteral("Config")).toObject();

        QString defAct = readProfileCfg(cfg, QStringLiteral("filter/defaultAction")).toString(QStringLiteral("permit"));
        QJsonValue vpnVal = readProfileCfg(cfg, QStringLiteral("spn/use"));
        QString vpnMode = vpnVal.isBool() ? (vpnVal.toBool() ? QStringLiteral("vpn") : QStringLiteral("direct")) : QStringLiteral("default");
        bool effectiveVpn = vpnVal.isBool() ? vpnVal.toBool() : gVpn;

        QJsonValue stVal = readProfileCfg(cfg, QStringLiteral("splittun/use"));
        QString splitMode = stVal.isBool() ? (stVal.toBool() ? QStringLiteral("on") : QStringLiteral("off")) : QStringLiteral("default");
        bool effectiveSplit = stVal.isBool() ? stVal.toBool() : gSplit;

        int liveActive = p.value(QStringLiteral("Stats")).toObject().value(QStringLiteral("ConnectionsCount")).toInt(0);
        qint64 rx = static_cast<qint64>(p.value(QStringLiteral("Stats")).toObject().value(QStringLiteral("BytesReceived")).toDouble());
        qint64 tx = static_cast<qint64>(p.value(QStringLiteral("Stats")).toObject().value(QStringLiteral("BytesSent")).toDouble());
        int totalSeen = 0, blockedSeen = 0;

        for (const auto& c : m_connections) {
            QString cp = c.value(QStringLiteral("profile")).toString();
            if (cp == id || cp.endsWith('/' + id) || (!name.isEmpty() && c.value(QStringLiteral("profile_name")).toString().compare(name, Qt::CaseInsensitive) == 0)) {
                ++totalSeen;
                rx = qMax(rx, static_cast<qint64>(c.value(QStringLiteral("bytes_received")).toDouble()));
                tx = qMax(tx, static_cast<qint64>(c.value(QStringLiteral("bytes_sent")).toDouble()));
                if (c.value(QStringLiteral("active")).toBool()) ++liveActive;
                QString vd = c.value(QStringLiteral("verdict")).toString();
                if (vd == QLatin1String("block") || vd == QLatin1String("drop")) ++blockedSeen;
            }
        }

        QVariantMap item{
            {QStringLiteral("key"), key},
            {QStringLiteral("id"), id},
            {QStringLiteral("source"), p.value(QStringLiteral("Source")).toString(QStringLiteral("local"))},
            {QStringLiteral("name"), name},
            {QStringLiteral("initials"), name.left(2).toUpper()},
            {QStringLiteral("path"), path},
            {QStringLiteral("action"), defAct},
            {QStringLiteral("vpnMode"), vpnMode},
            {QStringLiteral("effectiveVpn"), effectiveVpn},
            {QStringLiteral("splitMode"), splitMode},
            {QStringLiteral("effectiveSplit"), effectiveSplit},
            {QStringLiteral("hasCustom"), !cfg.isEmpty()},
            {QStringLiteral("internal"), p.value(QStringLiteral("Internal")).toBool(false)},
            {QStringLiteral("alive"), liveActive},
            {QStringLiteral("totalConns"), totalSeen},
            {QStringLiteral("blockedConns"), blockedSeen},
            {QStringLiteral("rxStr"), formatBytes(rx)},
            {QStringLiteral("txStr"), formatBytes(tx)},
        };
        m_allAppsCache.append(item);
        if (liveActive > 0) m_activeAppsCache.append(item);
        else m_offlineAppsCache.append(item);
    }
    std::sort(m_activeAppsCache.begin(), m_activeAppsCache.end(), [](const QVariant& a, const QVariant& b) {
        const auto ma = a.toMap();
        const auto mb = b.toMap();
        int aa = ma.value(QStringLiteral("alive")).toInt();
        int ab = mb.value(QStringLiteral("alive")).toInt();
        if (aa != ab) return aa > ab;
        return ma.value(QStringLiteral("name")).toString().toLower() < mb.value(QStringLiteral("name")).toString().toLower();
    });
    emit appsChanged();
}

void PortmasterBackend::selectApp(const QString& scopedKey) {
    m_selectedProfileKey = scopedKey;
    rebuildSelectedAppCache();
}

void PortmasterBackend::rebuildSelectedAppCache() {
    m_selectedAppConns.clear();
    if (!m_profiles.contains(m_selectedProfileKey)) {
        m_selectedAppMap.clear();
        emit selectedAppChanged();
        return;
    }
    const QJsonObject& p = m_profiles.value(m_selectedProfileKey);
    QString id = p.value(QStringLiteral("ID")).toString();
    QString name = p.value(QStringLiteral("Name")).toString(id);
    QJsonObject cfg = p.value(QStringLiteral("Config")).toObject();

    QVariantList rules;
    for (const auto& rv : readProfileCfg(cfg, QStringLiteral("filter/endpoints")).toArray()) {
        QString r = rv.toString();
        rules.append(QVariantMap{
            {QStringLiteral("raw"), r},
            {QStringLiteral("allow"), r.trimmed().startsWith('+')},
            {QStringLiteral("pattern"), r.trimmed().mid(1).trimmed()},
        });
    }
    QStringList dnsList;
    for (const auto& dv : readProfileCfg(cfg, QStringLiteral("dns/servers")).toArray()) {
        dnsList << dv.toString();
    }

    QJsonValue vpnVal = readProfileCfg(cfg, QStringLiteral("spn/use"));
    QJsonValue stVal = readProfileCfg(cfg, QStringLiteral("splittun/use"));
    qint64 rx = 0, tx = 0;
    int alive = 0, blocked = 0;
    for (const auto& c : m_connections) {
        QString cp = c.value(QStringLiteral("profile")).toString();
        if (cp == id || cp.endsWith('/' + id) || (!name.isEmpty() && c.value(QStringLiteral("profile_name")).toString().compare(name, Qt::CaseInsensitive) == 0)) {
            qint64 crx = static_cast<qint64>(c.value(QStringLiteral("bytes_received")).toDouble());
            qint64 ctx = static_cast<qint64>(c.value(QStringLiteral("bytes_sent")).toDouble());
            rx += crx; tx += ctx;
            if (c.value(QStringLiteral("active")).toBool()) ++alive;
            QString vd = c.value(QStringLiteral("verdict")).toString().toLower();
            if (vd == QLatin1String("block") || vd == QLatin1String("drop")) ++blocked;
            m_selectedAppConns.append(QVariantMap{
                {QStringLiteral("id"), c.value(QStringLiteral("id")).toString()},
                {QStringLiteral("domain"), c.value(QStringLiteral("domain")).toString().isEmpty() ? c.value(QStringLiteral("remote_ip")).toString() : c.value(QStringLiteral("domain")).toString()},
                {QStringLiteral("ip"), c.value(QStringLiteral("remote_ip")).toString()},
                {QStringLiteral("port"), c.value(QStringLiteral("remote_port")).toInt()},
                {QStringLiteral("proto"), c.value(QStringLiteral("ip_protocol")).toString()},
                {QStringLiteral("country"), c.value(QStringLiteral("country")).toString()},
                {QStringLiteral("verdict"), vd},
                {QStringLiteral("tunneled"), c.value(QStringLiteral("tunneled")).toBool()},
                {QStringLiteral("reason"), c.value(QStringLiteral("reason")).toString()},
                {QStringLiteral("bytes"), formatBytes(crx + ctx)},
            });
        }
    }

    QString path = p.value(QStringLiteral("PresentationPath")).toString();
    if (path.isEmpty()) path = p.value(QStringLiteral("LinkedPath")).toString();

    m_selectedAppMap = QVariantMap{
        {QStringLiteral("key"), m_selectedProfileKey},
        {QStringLiteral("id"), id},
        {QStringLiteral("source"), p.value(QStringLiteral("Source")).toString()},
        {QStringLiteral("name"), name},
        {QStringLiteral("initials"), name.left(2).toUpper()},
        {QStringLiteral("path"), path},
        {QStringLiteral("action"), readProfileCfg(cfg, QStringLiteral("filter/defaultAction")).toString(QStringLiteral("permit"))},
        {QStringLiteral("vpnMode"), vpnVal.isBool() ? (vpnVal.toBool() ? QStringLiteral("vpn") : QStringLiteral("direct")) : QStringLiteral("default")},
        {QStringLiteral("splitMode"), stVal.isBool() ? (stVal.toBool() ? QStringLiteral("on") : QStringLiteral("off")) : QStringLiteral("default")},
        {QStringLiteral("blockLan"), readProfileCfg(cfg, QStringLiteral("filter/blockLAN")).toBool(false)},
        {QStringLiteral("customDns"), dnsList.join(QStringLiteral(", "))},
        {QStringLiteral("rules"), rules},
        {QStringLiteral("alive"), alive},
        {QStringLiteral("blocked"), blocked},
        {QStringLiteral("rxStr"), formatBytes(rx)},
        {QStringLiteral("txStr"), formatBytes(tx)},
        {QStringLiteral("fpCount"), p.value(QStringLiteral("Fingerprints")).toArray().size()},
    };
    emit selectedAppChanged();
}

void PortmasterBackend::rebuildConnectionsCache() {
    m_connectionsCache.clear();
    m_connectionsCache.reserve(qMin(250, m_connections.size()));
    for (int i = 0; i < m_connections.size() && i < 250; ++i) {
        const auto& c = m_connections[i];
        QString vd = c.value(QStringLiteral("verdict")).toString().toLower();
        qint64 rx = static_cast<qint64>(c.value(QStringLiteral("bytes_received")).toDouble());
        qint64 tx = static_cast<qint64>(c.value(QStringLiteral("bytes_sent")).toDouble());
        QString dom = c.value(QStringLiteral("domain")).toString();
        m_connectionsCache.append(QVariantMap{
            {QStringLiteral("id"), c.value(QStringLiteral("id")).toString()},
            {QStringLiteral("profileKey"), c.value(QStringLiteral("profileKey")).toString()},
            {QStringLiteral("target"), dom.isEmpty() ? c.value(QStringLiteral("remote_ip")).toString() : dom},
            {QStringLiteral("domain"), dom},
            {QStringLiteral("ip"), c.value(QStringLiteral("remote_ip")).toString()},
            {QStringLiteral("port"), c.value(QStringLiteral("remote_port")).toInt()},
            {QStringLiteral("proto"), c.value(QStringLiteral("ip_protocol")).toString()},
            {QStringLiteral("country"), c.value(QStringLiteral("country")).toString()},
            {QStringLiteral("app"), c.value(QStringLiteral("profile_name")).toString()},
            {QStringLiteral("scope"), c.value(QStringLiteral("scope")).toString()},
            {QStringLiteral("direction"), c.value(QStringLiteral("direction")).toString()},
            {QStringLiteral("verdict"), vd},
            {QStringLiteral("tunneled"), c.value(QStringLiteral("tunneled")).toBool()},
            {QStringLiteral("reason"), c.value(QStringLiteral("reason")).toString()},
            {QStringLiteral("asOwner"), c.value(QStringLiteral("as_owner")).toString()},
            {QStringLiteral("bytes"), formatBytes(rx + tx)},
        });
    }
    emit connectionsChanged();
}

void PortmasterBackend::rebuildSettingsCache() {
    m_settingsCache.clear();
    for (auto it = m_settings.cbegin(); it != m_settings.cend(); ++it) {
        const auto& s = it.value();
        QString key = s.value(QStringLiteral("Key")).toString();
        QJsonValue eff = effectiveCfgVal(s);
        bool modified = s.contains(QLatin1String("Value")) && !s.value(QLatin1String("Value")).isNull();
        QString strVal;
        QStringList arrayItems;
        if (eff.isBool()) strVal = eff.toBool() ? QStringLiteral("true") : QStringLiteral("false");
        else if (eff.isDouble()) strVal = QString::number(static_cast<qint64>(eff.toDouble()));
        else if (eff.isString()) strVal = eff.toString();
        else if (eff.isArray()) {
            for (const auto& a : eff.toArray()) arrayItems << a.toString();
            strVal = arrayItems.join(QStringLiteral(", "));
        }
        QVariantList choices;
        for (const auto& pv : s.value(QStringLiteral("PossibleValues")).toArray()) {
            choices.append(pv.toObject().toVariantMap());
        }
        m_settingsCache.append(QVariantMap{
            {QStringLiteral("key"), key},
            {QStringLiteral("subsystem"), key.section('/', 0, 0)},
            {QStringLiteral("name"), s.value(QStringLiteral("Name")).toString()},
            {QStringLiteral("description"), s.value(QStringLiteral("Description")).toString()},
            {QStringLiteral("optType"), s.value(QStringLiteral("OptType")).toInt(1)},
            {QStringLiteral("expertise"), s.value(QStringLiteral("ExpertiseLevel")).toInt(0)},
            {QStringLiteral("modified"), modified},
            {QStringLiteral("valueStr"), strVal},
            {QStringLiteral("boolVal"), eff.toBool(false)},
            {QStringLiteral("intVal"), static_cast<int>(eff.toDouble(0))},
            {QStringLiteral("arrayItems"), arrayItems},
            {QStringLiteral("choices"), choices},
        });
    }
    emit settingsChanged();
}

void PortmasterBackend::rebuildNotificationsCache() {
    m_alertsCache.clear();
    m_promptsCache.clear();
    m_paused = false;
    m_vpnPaused = false;
    m_pauseUntil.clear();

    for (auto it = m_notifications.cbegin(); it != m_notifications.cend(); ++it) {
        const auto& n = it.value();
        QString id = n.value(QStringLiteral("EventID")).toString();
        int nType = n.value(QStringLiteral("Type")).toInt(0);
        if (id == QLatin1String("control:paused")) {
            QJsonObject data = n.value(QStringLiteral("EventData")).toObject();
            m_paused = true;
            m_vpnPaused = data.value(QStringLiteral("SPN")).toBool() && !data.value(QStringLiteral("Interception")).toBool();
            m_pauseUntil = data.value(QStringLiteral("TillTime")).toString();
            continue;
        }
        QVariantMap item{
            {QStringLiteral("id"), id},
            {QStringLiteral("title"), n.value(QStringLiteral("Title")).toString()},
            {QStringLiteral("message"), n.value(QStringLiteral("Message")).toString()},
            {QStringLiteral("type"), nType},
        };
        if (nType == 100) m_promptsCache.append(item);
        else m_alertsCache.append(item);
    }
    emit notificationsChanged();
}

void PortmasterBackend::setAppAction(const QString& scopedKey, const QString& action) {
    if (!m_profiles.contains(scopedKey)) return;
    QJsonObject p = m_profiles.value(scopedKey);
    QJsonObject cfg = p.value(QStringLiteral("Config")).toObject();
    writeProfileCfg(cfg, QStringLiteral("filter/defaultAction"), action);
    p.insert(QStringLiteral("Config"), cfg);
    m_profiles.insert(scopedKey, p);
    m_api->sendPutFrame(QStringLiteral("update"), scopedKey, p);
    rebuildAppsCache();
    rebuildSelectedAppCache();
}

void PortmasterBackend::setAppVpnMode(const QString& scopedKey, const QString& mode) {
    if (!m_profiles.contains(scopedKey)) return;
    QJsonObject p = m_profiles.value(scopedKey);
    QJsonObject cfg = p.value(QStringLiteral("Config")).toObject();
    if (mode == QLatin1String("vpn")) writeProfileCfg(cfg, QStringLiteral("spn/use"), true);
    else if (mode == QLatin1String("direct")) writeProfileCfg(cfg, QStringLiteral("spn/use"), false);
    else writeProfileCfg(cfg, QStringLiteral("spn/use"), QJsonValue::Undefined);
    p.insert(QStringLiteral("Config"), cfg);
    m_profiles.insert(scopedKey, p);
    m_api->sendPutFrame(QStringLiteral("update"), scopedKey, p);
    rebuildAppsCache();
    rebuildSelectedAppCache();
}

void PortmasterBackend::setAppSplitMode(const QString& scopedKey, const QString& mode) {
    if (!m_profiles.contains(scopedKey)) return;
    QJsonObject p = m_profiles.value(scopedKey);
    QJsonObject cfg = p.value(QStringLiteral("Config")).toObject();
    if (mode == QLatin1String("on")) writeProfileCfg(cfg, QStringLiteral("splittun/use"), true);
    else if (mode == QLatin1String("off")) writeProfileCfg(cfg, QStringLiteral("splittun/use"), false);
    else writeProfileCfg(cfg, QStringLiteral("splittun/use"), QJsonValue::Undefined);
    p.insert(QStringLiteral("Config"), cfg);
    m_profiles.insert(scopedKey, p);
    m_api->sendPutFrame(QStringLiteral("update"), scopedKey, p);
    rebuildAppsCache();
    rebuildSelectedAppCache();
}

void PortmasterBackend::setAppBlockLan(const QString& scopedKey, bool blocked) {
    if (!m_profiles.contains(scopedKey)) return;
    QJsonObject p = m_profiles.value(scopedKey);
    QJsonObject cfg = p.value(QStringLiteral("Config")).toObject();
    writeProfileCfg(cfg, QStringLiteral("filter/blockLAN"), blocked);
    p.insert(QStringLiteral("Config"), cfg);
    m_profiles.insert(scopedKey, p);
    m_api->sendPutFrame(QStringLiteral("update"), scopedKey, p);
    rebuildSelectedAppCache();
}

void PortmasterBackend::setAppCustomDns(const QString& scopedKey, const QString& serversCsv) {
    if (!m_profiles.contains(scopedKey)) return;
    QJsonObject p = m_profiles.value(scopedKey);
    QJsonObject cfg = p.value(QStringLiteral("Config")).toObject();
    QJsonArray arr;
    for (const auto& s : serversCsv.split(',', Qt::SkipEmptyParts)) {
        if (!s.trimmed().isEmpty()) arr.append(s.trimmed());
    }
    if (arr.isEmpty()) writeProfileCfg(cfg, QStringLiteral("dns/servers"), QJsonValue::Undefined);
    else writeProfileCfg(cfg, QStringLiteral("dns/servers"), arr);
    p.insert(QStringLiteral("Config"), cfg);
    m_profiles.insert(scopedKey, p);
    m_api->sendPutFrame(QStringLiteral("update"), scopedKey, p);
    rebuildSelectedAppCache();
}

void PortmasterBackend::addAppRule(const QString& scopedKey, const QString& rule) {
    if (!m_profiles.contains(scopedKey) || rule.trimmed().isEmpty()) return;
    QJsonObject p = m_profiles.value(scopedKey);
    QJsonObject cfg = p.value(QStringLiteral("Config")).toObject();
    QJsonArray arr = readProfileCfg(cfg, QStringLiteral("filter/endpoints")).toArray();
    for (const auto& v : arr) {
        if (v.toString() == rule.trimmed()) return;
    }
    arr.append(rule.trimmed());
    writeProfileCfg(cfg, QStringLiteral("filter/endpoints"), arr);
    p.insert(QStringLiteral("Config"), cfg);
    m_profiles.insert(scopedKey, p);
    m_api->sendPutFrame(QStringLiteral("update"), scopedKey, p);
    rebuildSelectedAppCache();
}

void PortmasterBackend::removeAppRule(const QString& scopedKey, const QString& rule) {
    if (!m_profiles.contains(scopedKey)) return;
    QJsonObject p = m_profiles.value(scopedKey);
    QJsonObject cfg = p.value(QStringLiteral("Config")).toObject();
    QJsonArray arr = readProfileCfg(cfg, QStringLiteral("filter/endpoints")).toArray();
    QJsonArray next;
    for (const auto& v : arr) {
        if (v.toString() != rule) next.append(v);
    }
    writeProfileCfg(cfg, QStringLiteral("filter/endpoints"), next);
    p.insert(QStringLiteral("Config"), cfg);
    m_profiles.insert(scopedKey, p);
    m_api->sendPutFrame(QStringLiteral("update"), scopedKey, p);
    rebuildSelectedAppCache();
}

void PortmasterBackend::promoteRuleToGlobal(const QString& cfgKey, const QString& rule) {
    applyPreset(cfgKey, {rule}, true);
}

QString PortmasterBackend::testAppPath(const QString& scopedKey, const QString& testPath) const {
    if (!m_profiles.contains(scopedKey)) return {};
    const QString input = testPath.trimmed().replace('\\', '/').toLower();
    if (input.isEmpty()) return {};
    const auto p = m_profiles.value(scopedKey);
    if (p.value(QStringLiteral("LinkedPath")).toString().replace('\\', '/').toLower() == input ||
        p.value(QStringLiteral("PresentationPath")).toString().replace('\\', '/').toLower() == input) {
        return QStringLiteral("MATCH (Exact LinkedPath)");
    }
    for (const auto& v : p.value(QStringLiteral("Fingerprints")).toArray()) {
        const auto fp = v.toObject();
        if (fp.value(QStringLiteral("Type")).toString() == QLatin1String("path")) {
            QString val = fp.value(QStringLiteral("Value")).toString().replace('\\', '/').toLower();
            QString op = fp.value(QStringLiteral("Operation")).toString();
            bool hit = (op == QLatin1String("prefix")) ? input.startsWith(val)
                     : (op == QLatin1String("regex")) ? QRegularExpression(val).match(input).hasMatch()
                     : (input == val);
            if (hit) return QStringLiteral("MATCH (%1)").arg(fp.value(QStringLiteral("Value")).toString());
        }
    }
    return QStringLiteral("NO MATCH");
}

void PortmasterBackend::createProfile(const QString& name, const QString& path) {
    QString trimmedName = name.trimmed();
    QString trimmedPath = path.trimmed();
    if (trimmedName.isEmpty()) return;
    QString slug = trimmedName.toLower().replace(QRegularExpression(QStringLiteral("[^a-z0-9]+")), QStringLiteral("-"));
    QString id = QStringLiteral("%1-%2").arg(slug).arg(QDateTime::currentSecsSinceEpoch() % 10000);
    QJsonObject fp{{QStringLiteral("Type"), QStringLiteral("path")}, {QStringLiteral("Operation"), QStringLiteral("equal")}, {QStringLiteral("Value"), trimmedPath}};
    QJsonObject p{
        {QStringLiteral("ID"), id},
        {QStringLiteral("Source"), QStringLiteral("local")},
        {QStringLiteral("Name"), trimmedName},
        {QStringLiteral("LinkedPath"), trimmedPath},
        {QStringLiteral("PresentationPath"), trimmedPath},
        {QStringLiteral("Fingerprints"), trimmedPath.isEmpty() ? QJsonArray{} : QJsonArray{fp}},
    };
    QString key = QStringLiteral("core:profiles/local/%1").arg(id);
    m_api->sendPutFrame(QStringLiteral("create"), key, p);
}

void PortmasterBackend::deleteProfile(const QString& scopedKey) {
    m_api->sendDeleteFrame(scopedKey);
    m_profiles.remove(scopedKey);
    if (m_selectedProfileKey == scopedKey) {
        m_selectedProfileKey.clear();
        rebuildSelectedAppCache();
    }
    rebuildAppsCache();
}

void PortmasterBackend::exportProfile(const QString& scopedKey) {
    QString clean = scopedKey;
    clean.remove(QStringLiteral("core:profiles/"));
    m_api->httpGet(QStringLiteral("/api/v1/sync/profile/export?id=") + QString::fromUtf8(QUrl::toPercentEncoding(clean)),
        [this, clean](int code, const QByteArray& body, const QString&) {
            if (code == 200) {
                m_modalTitle = QStringLiteral("Export Profile: ") + clean;
                m_modalBody = QString::fromUtf8(body);
                emit modalChanged();
            }
        });
}

void PortmasterBackend::importProfile(const QString& rawPayload) {
    m_api->httpPost(QStringLiteral("/api/v1/sync/profile/import?allowReplace=true"), rawPayload.toUtf8(), QStringLiteral("text/plain"), nullptr);
}

void PortmasterBackend::allowOrBlockConnection(const QString& connId, bool allow) {
    for (const auto& c : m_connections) {
        if (c.value(QStringLiteral("id")).toString() != connId) continue;
        QString prof = c.value(QStringLiteral("profile")).toString();
        QString dbKey = prof.startsWith(QLatin1String("core:")) ? prof : QStringLiteral("core:profiles/%1").arg(prof.contains('/') ? prof : QStringLiteral("local/") + prof);
        QString dom = c.value(QStringLiteral("domain")).toString().trimmed();
        QString ip = c.value(QStringLiteral("remote_ip")).toString().trimmed();
        QString pat = dom.isEmpty() ? ip : (QStringLiteral(".") + dom);
        if (!pat.isEmpty()) addAppRule(dbKey, QStringLiteral("%1 %2").arg(allow ? QStringLiteral("+") : QStringLiteral("-"), pat));
        return;
    }
}

void PortmasterBackend::clearHistory() {
    m_api->httpPost(QStringLiteral("/api/v1/netquery/history/clear"), "{}", QStringLiteral("application/json"), [this](int code, const QByteArray&, const QString&) {
        if (code >= 200 && code < 300) {
            m_connections.clear();
            rebuildConnectionsCache();
        }
    });
}

void PortmasterBackend::toggleGlobalVpn() {
    updateSetting(QStringLiteral("spn/use"), !globalVpn());
}

void PortmasterBackend::resetAllAppOverrides() {
    m_api->httpPost(QStringLiteral("/api/v1/profiles/reset-routing"), "{}", QStringLiteral("application/json"), nullptr);
}

void PortmasterBackend::toggleXrayConnect() {
    bool conn = m_xrayStatusMap.value(QStringLiteral("connected")).toBool();
    const QString ep = conn ? QStringLiteral("/api/v1/xray/disconnect") : QStringLiteral("/api/v1/xray/connect-last");
    m_api->httpPost(ep, "{}", QStringLiteral("application/json"), [this](int, const QByteArray&, const QString&) {
        refreshHttpData();
    });
}

void PortmasterBackend::connectNode(const QString& nodeId) {
    QByteArray body = QJsonDocument(QJsonObject{{QStringLiteral("nodeId"), nodeId}}).toJson();
    m_api->httpPost(QStringLiteral("/api/v1/xray/connect"), body, QStringLiteral("application/json"), [this](int, const QByteArray&, const QString&) {
        refreshHttpData();
    });
}

void PortmasterBackend::addSubscription(const QString& url) {
    QByteArray body = QJsonDocument(QJsonObject{{QStringLiteral("url"), url.trimmed()}}).toJson();
    m_api->httpPost(QStringLiteral("/api/v1/xray/subscriptions"), body, QStringLiteral("application/json"), [this](int, const QByteArray&, const QString&) {
        refreshHttpData();
    });
}

void PortmasterBackend::addManualLink(const QString& links) {
    QByteArray body = QJsonDocument(QJsonObject{{QStringLiteral("links"), links.trimmed()}}).toJson();
    m_api->httpPost(QStringLiteral("/api/v1/xray/nodes/manual"), body, QStringLiteral("application/json"), [this](int, const QByteArray&, const QString&) {
        refreshHttpData();
    });
}

void PortmasterBackend::refreshSubscription(const QString& id) {
    QByteArray body = QJsonDocument(QJsonObject{{QStringLiteral("id"), id}}).toJson();
    m_api->httpPut(QStringLiteral("/api/v1/xray/subscriptions"), body, QStringLiteral("application/json"), [this](int, const QByteArray&, const QString&) {
        refreshHttpData();
    });
}

void PortmasterBackend::deleteSubscription(const QString& id) {
    m_api->httpDelete(QStringLiteral("/api/v1/xray/subscriptions/") + QString::fromUtf8(QUrl::toPercentEncoding(id)), [this](int, const QByteArray&, const QString&) {
        refreshHttpData();
    });
}

void PortmasterBackend::downloadXrayBinary() {
    m_api->httpPost(QStringLiteral("/api/v1/xray/download-binary"), "{}", QStringLiteral("application/json"), [this](int code, const QByteArray&, const QString&) {
        emit toastMessage(QStringLiteral("Xray"), code == 200 ? QStringLiteral("xray-rust downloaded/updated.") : QStringLiteral("Failed to download xray-rust."), code != 200);
        refreshHttpData();
    });
}

void PortmasterBackend::downloadGeodata() {
    m_api->httpPost(QStringLiteral("/api/v1/xray/download-geodata"), "{}", QStringLiteral("application/json"), [this](int code, const QByteArray&, const QString&) {
        emit toastMessage(QStringLiteral("GeoIP & GeoSite"), code == 200 ? QStringLiteral("geoip.dat & geosite.dat updated.") : QStringLiteral("Failed to update geodata."), code != 200);
    });
}

void PortmasterBackend::updateSetting(const QString& key, const QVariant& val) {
    QJsonObject payload = m_settings.contains(key) ? m_settings.value(key) : QJsonObject{{QStringLiteral("Key"), key}};
    payload.insert(QStringLiteral("Key"), key);
    payload.insert(QStringLiteral("Value"), QJsonValue::fromVariant(val));
    m_settings.insert(key, payload);
    rebuildSettingsCache();
    m_api->sendPutFrame(QStringLiteral("update"), QStringLiteral("config:") + key, payload);
}

void PortmasterBackend::resetSetting(const QString& key) {
    QJsonObject payload = m_settings.contains(key) ? m_settings.value(key) : QJsonObject{{QStringLiteral("Key"), key}};
    payload.insert(QStringLiteral("Key"), key);
    payload.remove(QStringLiteral("Value"));
    m_settings.insert(key, payload);
    rebuildSettingsCache();
    m_api->sendPutFrame(QStringLiteral("update"), QStringLiteral("config:") + key, payload);
}

void PortmasterBackend::applyPreset(const QString& key, const QStringList& values, bool append) {
    QStringList next;
    if (append) {
        const QJsonArray cur = effectiveValue(key).toArray();
        for (const auto& v : cur) next << v.toString();
    }
    for (const auto& v : values) {
        if (!next.contains(v)) next << v;
    }
    updateSetting(key, next);
}

void PortmasterBackend::switchUiFlavor(const QString& flavorId) {
    QByteArray body = QJsonDocument(QJsonObject{{QStringLiteral("flavor"), flavorId}}).toJson();
    m_api->httpPost(QStringLiteral("/api/v1/ui/flavor"), body, QStringLiteral("application/json"), [this](int code, const QByteArray& resp, const QString&) {
        if (code == 200) {
            m_uiReportMap = QJsonDocument::fromJson(resp).object().toVariantMap();
            emit uiReportChanged();
        }
    });
}

void PortmasterBackend::setKeepApiPortOpen(bool keep) {
    QByteArray body = QJsonDocument(QJsonObject{{QStringLiteral("keepOpen"), keep}}).toJson();
    m_api->httpPost(QStringLiteral("/api/v1/core/api-port"), body, QStringLiteral("application/json"), [this](int, const QByteArray&, const QString&) {
        QTimer::singleShot(450, this, &PortmasterBackend::refreshHttpData);
    });
}

void PortmasterBackend::exportSettings() {
    m_api->httpGet(QStringLiteral("/api/v1/sync/settings/export?from=core:config/global"), [this](int code, const QByteArray& body, const QString&) {
        if (code == 200) {
            m_modalTitle = QStringLiteral("Export Global Settings");
            m_modalBody = QString::fromUtf8(body);
            emit modalChanged();
        }
    });
}

void PortmasterBackend::importSettings(const QString& rawPayload) {
    m_api->httpPost(QStringLiteral("/api/v1/sync/settings/import?to=core:config/global"), rawPayload.toUtf8(), QStringLiteral("text/plain"), nullptr);
}

void PortmasterBackend::closeModal() {
    m_modalTitle.clear();
    m_modalBody.clear();
    emit modalChanged();
}

void PortmasterBackend::resolvePrompt(const QString& promptId, bool allow) {
    QJsonObject payload{{QStringLiteral("EventID"), promptId}, {QStringLiteral("SelectedActionID"), allow ? QStringLiteral("allow-domain-all") : QStringLiteral("block-domain-all")}};
    m_api->sendPutFrame(QStringLiteral("update"), QStringLiteral("notifications:all/") + promptId, payload);
}

void PortmasterBackend::resolveAllPrompts(bool allow) {
    for (const auto& pv : m_promptsCache) {
        resolvePrompt(pv.toMap().value(QStringLiteral("id")).toString(), allow);
    }
}

void PortmasterBackend::clearNotifications() {
    for (const auto& av : m_alertsCache) {
        m_api->sendDeleteFrame(QStringLiteral("notifications:all/") + av.toMap().value(QStringLiteral("id")).toString());
    }
}

void PortmasterBackend::pauseProtection(int minutes, bool vpnOnly) {
    QByteArray body = QJsonDocument(QJsonObject{{QStringLiteral("duration"), minutes * 60}, {QStringLiteral("only"), vpnOnly ? QStringLiteral("spn") : QStringLiteral("")}}).toJson();
    m_api->httpPost(QStringLiteral("/api/v1/control/pause"), body, QStringLiteral("application/json"), nullptr);
}

void PortmasterBackend::resumeProtection() {
    m_api->httpPost(QStringLiteral("/api/v1/control/resume"), "{}", QStringLiteral("application/json"), nullptr);
}

void PortmasterBackend::clearDnsCache() {
    m_api->httpPost(QStringLiteral("/api/v1/dns/clear"), "{}", QStringLiteral("application/json"), nullptr);
}

void PortmasterBackend::checkUpdates() {
    m_api->httpPost(QStringLiteral("/api/v1/updates/check"), "{}", QStringLiteral("application/json"), nullptr);
}

void PortmasterBackend::restartCore() {
    m_api->httpPost(QStringLiteral("/api/v1/core/restart"), "{}", QStringLiteral("application/json"), nullptr);
}

void PortmasterBackend::shutdownCore() {
    m_api->httpPost(QStringLiteral("/api/v1/core/shutdown"), "{}", QStringLiteral("application/json"), nullptr);
}

void PortmasterBackend::quitApp() {
    qApp->quit();
}
