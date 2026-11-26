#include "PortmasterBackend.h"

#include <QApplication>
#include <QDateTime>
#include <QDir>
#include <QDirIterator>
#include <QFile>
#include <QFileInfo>
#include <QIcon>
#include <QJsonArray>
#include <QJsonDocument>
#include <QMenu>
#include <QMutexLocker>
#include <QPainter>
#include <QPixmap>
#include <QPointer>
#include <QPolygon>
#include <QRegularExpression>
#include <QSettings>
#include <QStandardPaths>
#include <QTextStream>
#include <QUrl>
#include <QtMath>
#include <cmath>

namespace {
double mercatorLatToY(double latDeg) {
    const double clamped = qBound(-60.0, latDeg, 78.0);
    return std::asinh(std::tan(qDegreesToRadians(clamped)));
}

QPointF projectNormalized(double lat, double lon) {
    static const double yMax = mercatorLatToY(78.0);
    static const double yMin = mercatorLatToY(-60.0);
    static const double ySpan = yMax - yMin;
    const double nx = qBound(0.0, (lon + 180.0) / 360.0, 1.0);
    const double ny = qBound(0.0, (yMax - mercatorLatToY(lat)) / ySpan, 1.0);
    return QPointF(nx, ny);
}

QString appAvatarColor(const QString& seed) {
    static const char* kPalette[] = {
        "#d32f2f", "#c2185b", "#7b1fa2", "#512da8", "#303f9f",
        "#1976d2", "#0288d1", "#0097a7", "#00796b", "#388e3c",
        "#689f38", "#827717", "#f57f17", "#ef6c00", "#d84315",
        "#5d4037", "#455a64", "#283593", "#00695c"
    };
    quint32 sum = 0;
    for (const QChar ch : seed) sum += ch.unicode();
    return QLatin1String(kPalette[sum % 19]);
}

QString localizeChoiceName(const QString& rawName, const QString& val, bool ru) {
    if (!ru) return rawName;
    if (val == QLatin1String("permit") || rawName == QLatin1String("Allow")) return QStringLiteral("Разрешить");
    if (val == QLatin1String("block") || rawName == QLatin1String("Block")) return QStringLiteral("Блокировать");
    if (val == QLatin1String("ask") || rawName == QLatin1String("Prompt")) return QStringLiteral("Запрашивать");
    if (val == QLatin1String("angular")) return QStringLiteral("Angular (Веб-интерфейс)");
    if (val == QLatin1String("rust-wasm")) return QStringLiteral("Rust WASM (Лёгкий веб)");
    if (val == QLatin1String("qt")) return QStringLiteral("Qt 6 Native (Без браузера)");
    if (val.isEmpty() && rawName.startsWith(QLatin1String("Auto-detect"))) return QStringLiteral("Автопоиск (рекомендуется)");
    return rawName;
}

QStringList moveArrayElement(const QJsonArray& arr, int index, int delta) {
    QStringList list;
    for (const auto& v : arr) list << v.toString();
    const int target = index + delta;
    if (index >= 0 && index < list.size() && target >= 0 && target < list.size() && delta != 0) {
        list.swapItemsAt(index, target);
    }
    return list;
}
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

QJsonValue effectiveCfgVal(const QJsonObject& s) {
    if (s.contains(QLatin1String("Value")) && !s.value(QLatin1String("Value")).isNull()) {
        return s.value(QLatin1String("Value"));
    }
    return s.value(QLatin1String("DefaultValue"));
}

bool jsonToBool(const QJsonValue& v, bool fallback = false) {
    if (v.isBool()) return v.toBool();
    if (v.isDouble()) return v.toDouble() != 0.0;
    return fallback;
}

QJsonValue variantToJsonValue(const QVariant& val) {
    if (val.metaType().id() == QMetaType::QStringList) {
        return QJsonArray::fromStringList(val.toStringList());
    }
    return QJsonValue::fromVariant(val);
}

QStringList mergePresetValues(const QJsonArray& cur, const QStringList& values, bool append) {
    QStringList next;
    if (append) {
        for (const auto& v : cur) next << v.toString();
    }
    for (const auto& v : values) {
        if (!next.contains(v)) next << v;
    }
    return next;
}

QString summarizeDnsList(const QStringList& servers, bool ru) {
    if (servers.isEmpty()) return ru ? QStringLiteral("Системный / По умолч.") : QStringLiteral("System / Default");
    const QString first = servers.first();
    QString tag;
    if (first.contains(QLatin1String("cloudflare"), Qt::CaseInsensitive) || first.contains(QLatin1String("1.1.1."))) {
        tag = first.startsWith(QLatin1String("dot://")) ? QStringLiteral("Cloudflare DoT") : QStringLiteral("Cloudflare DoH");
    } else if (first.contains(QLatin1String("quad9"), Qt::CaseInsensitive) || first.contains(QLatin1String("9.9.9.9"))) {
        tag = first.startsWith(QLatin1String("dot://")) ? QStringLiteral("Quad9 DoT") : QStringLiteral("Quad9 DoH");
    } else if (first.contains(QLatin1String("adguard"), Qt::CaseInsensitive) || first.contains(QLatin1String("94.140.14."))) {
        tag = first.startsWith(QLatin1String("dot://")) ? QStringLiteral("AdGuard DoT") : QStringLiteral("AdGuard DoH");
    } else if (first.contains(QLatin1String("mullvad"), Qt::CaseInsensitive)) {
        tag = QStringLiteral("Mullvad DoH");
    } else {
        QUrl u(first);
        tag = !u.host().isEmpty() ? u.host() : first.left(24);
    }
    if (servers.size() > 1) {
        return QStringLiteral("%1 (+%2)").arg(tag).arg(servers.size() - 1);
    }
    return tag;
}

QString formatSettingDisplay(const QJsonValue& v, int optType, const QVariantList& choices, bool ru) {
    if (optType == 4) {
        return jsonToBool(v) ? (ru ? QStringLiteral("Вкл") : QStringLiteral("On"))
                             : (ru ? QStringLiteral("Выкл") : QStringLiteral("Off"));
    }
    if (v.isArray()) {
        const auto arr = v.toArray();
        if (arr.isEmpty()) return ru ? QStringLiteral("Пусто (0)") : QStringLiteral("Empty (0)");
        if (arr.size() == 1) return arr.first().toString();
        return QStringLiteral("%1 (%2)").arg(arr.first().toString()).arg(arr.size());
    }
    QString raw = v.isDouble() ? QString::number(static_cast<qint64>(v.toDouble())) : v.toString();
    for (const auto& chv : choices) {
        const auto m = chv.toMap();
        if (m.value(QStringLiteral("Value")).toString() == raw) {
            QString name = m.value(QStringLiteral("Name")).toString();
            if (!name.isEmpty()) return name;
        }
    }
    return raw.isEmpty() ? QStringLiteral("-") : raw;
}

void sortSettingsByOrder(QVariantList& list) {
    std::sort(list.begin(), list.end(), [](const QVariant& a, const QVariant& b) {
        const auto ma = a.toMap();
        const auto mb = b.toMap();
        int oa = ma.value(QStringLiteral("order"), 9999).toInt();
        int ob = mb.value(QStringLiteral("order"), 9999).toInt();
        if (oa != ob) return oa < ob;
        return ma.value(QStringLiteral("key")).toString() < mb.value(QStringLiteral("key")).toString();
    });
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

    const QJsonValue stVal = pick("started", "Started");
    qint64 startedTs = 0;
    if (stVal.isString()) {
        QDateTime dt = QDateTime::fromString(stVal.toString(), Qt::ISODateWithMs);
        if (!dt.isValid()) dt = QDateTime::fromString(stVal.toString(), Qt::ISODate);
        if (dt.isValid()) startedTs = dt.toSecsSinceEpoch();
    } else if (stVal.isDouble()) {
        qint64 rawTs = static_cast<qint64>(stVal.toDouble(0));
        startedTs = (rawTs > 100000000000LL) ? (rawTs / 1000) : rawTs;
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
        {QStringLiteral("started_ts"), static_cast<double>(startedTs)},
        {QStringLiteral("active"), pick("active", "Active").toBool(false)},
    };
}
} // namespace

PortmasterIconProvider::PortmasterIconProvider(QSharedPointer<PortmasterIconStore> store)
    : QQuickImageProvider(QQuickImageProvider::Image), m_store(std::move(store)) {}

QImage PortmasterIconProvider::requestImage(const QString& id, QSize* size, const QSize& requestedSize) {
    const QString key = QUrl::fromPercentEncoding(id.section('?', 0, 0).toUtf8());
    QImage img;
    {
        QMutexLocker lock(&m_store->mutex);
        img = m_store->images.value(key);
    }
    if (img.isNull()) {
        static const QImage kEmptyPx = []() {
            QImage px(1, 1, QImage::Format_ARGB32_Premultiplied);
            px.fill(Qt::transparent);
            return px;
        }();
        if (size) *size = kEmptyPx.size();
        return kEmptyPx;
    }
    if (requestedSize.width() > 0 && requestedSize.height() > 0) {
        img = img.scaled(requestedSize, Qt::KeepAspectRatio, Qt::SmoothTransformation);
    }
    if (size) *size = img.size();
    return img;
}

PortmasterBackend::PortmasterBackend(bool startBackground, bool withTray, QObject* parent)
    : QObject(parent),
      m_api(new PortApiClient(this)),
      m_appsTimer(new QTimer(this)),
      m_appsTickTimer(new QTimer(this)),
      m_settingsTimer(new QTimer(this)),
      m_connsTimer(new QTimer(this)),
      m_withTray(withTray),
      m_iconStore(QSharedPointer<PortmasterIconStore>::create()),
      m_iconBatchTimer(new QTimer(this)),
      m_iconNotifyTimer(new QTimer(this)) {
    QSettings qs(QStringLiteral("portmaster"), QStringLiteral("qt-ui"));
    m_ru = qs.value(QStringLiteral("lang"), QStringLiteral("ru")).toString() == QLatin1String("ru");
    m_hwAccel = qs.value(QStringLiteral("hwAccel"), true).toBool();
    m_historyLimit = qBound(250, qs.value(QStringLiteral("historyLimit"), 2000).toInt(), 10000);
    loadI18nCatalog();
    buildDesktopIconIndex();

    m_iconBatchTimer->setInterval(20);
    connect(m_iconBatchTimer, &QTimer::timeout, this, &PortmasterBackend::processNextIconBatch);

    m_iconNotifyTimer->setSingleShot(true);
    m_iconNotifyTimer->setInterval(300);
    connect(m_iconNotifyTimer, &QTimer::timeout, this, [this]() {
        ++m_iconVersion;
        emit iconsChanged();
        if (m_iconQueue.isEmpty() && m_iconPending.isEmpty()) {
            scheduleAppsRebuild();
        }
    });

    m_appsTimer->setSingleShot(true);
    m_appsTimer->setInterval(60);
    connect(m_appsTimer, &QTimer::timeout, this, [this]() {
        rebuildAppsCache();
        if (!m_selectedProfileKey.isEmpty()) rebuildSelectedAppCache();
    });

    m_appsTickTimer->setSingleShot(true);
    m_appsTickTimer->setInterval(1000);
    connect(m_appsTickTimer, &QTimer::timeout, this, [this]() {
        rebuildAppsCache();
        if (!m_selectedProfileKey.isEmpty()) rebuildSelectedAppCache();
    });

    m_settingsTimer->setSingleShot(true);
    m_settingsTimer->setInterval(60);
    connect(m_settingsTimer, &QTimer::timeout, this, [this]() {
        rebuildSettingsCache();
        if (!m_selectedProfileKey.isEmpty()) rebuildSelectedAppCache();
    });

    m_connsTimer->setSingleShot(true);
    m_connsTimer->setInterval(250);
    connect(m_connsTimer, &QTimer::timeout, this, [this]() {
        rebuildConnectionsCache();
        scheduleAppsTickRebuild();
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

PortmasterBackend::~PortmasterBackend() {
    if (m_iconBatchTimer) m_iconBatchTimer->stop();
    if (m_iconNotifyTimer) m_iconNotifyTimer->stop();
    if (m_appsTimer) m_appsTimer->stop();
    if (m_appsTickTimer) m_appsTickTimer->stop();
    if (m_settingsTimer) m_settingsTimer->stop();
    if (m_connsTimer) m_connsTimer->stop();
}

QQuickImageProvider* PortmasterBackend::createIconProvider() const {
    return new PortmasterIconProvider(m_iconStore);
}

void PortmasterBackend::loadI18nCatalog() {
    QFile f(QStringLiteral(":/qt/qml/Portmaster/cfg_i18n.json"));
    if (!f.open(QIODevice::ReadOnly)) return;
    const QJsonObject root = QJsonDocument::fromJson(f.readAll()).object();
    const QJsonObject ru = root.value(QStringLiteral("ru")).toObject();
    const QJsonObject en = root.value(QStringLiteral("en")).toObject();
    for (auto it = ru.cbegin(); it != ru.cend(); ++it) {
        m_i18nRu.insert(it.key(), it.value().toString());
    }
    for (auto it = en.cbegin(); it != en.cend(); ++it) {
        m_i18nEn.insert(it.key(), it.value().toString());
    }
}

void PortmasterBackend::buildDesktopIconIndex() {
    QStringList dirs = QStandardPaths::standardLocations(QStandardPaths::ApplicationsLocation);
    const QString flatpakSys = QStringLiteral("/var/lib/flatpak/exports/share/applications");
    if (QDir(flatpakSys).exists() && !dirs.contains(flatpakSys)) dirs.append(flatpakSys);

    for (const QString& dir : dirs) {
        QDirIterator it(dir, QStringList{QStringLiteral("*.desktop")}, QDir::Files, QDirIterator::Subdirectories);
        while (it.hasNext()) {
            QFile f(it.next());
            if (!f.open(QIODevice::ReadOnly | QIODevice::Text)) continue;
            QString execLine, iconName, dispName, wmClass;
            bool inEntry = false;
            QTextStream in(&f);
            while (!in.atEnd()) {
                const QString line = in.readLine().trimmed();
                if (line.isEmpty() || line.startsWith('#')) continue;
                if (line.startsWith('[')) {
                    inEntry = (line == QLatin1String("[Desktop Entry]"));
                    continue;
                }
                if (!inEntry) continue;
                if (line.startsWith(QLatin1String("Exec="))) execLine = line.mid(5).trimmed();
                else if (line.startsWith(QLatin1String("Icon="))) iconName = line.mid(5).trimmed();
                else if (line.startsWith(QLatin1String("Name="))) dispName = line.mid(5).trimmed();
                else if (line.startsWith(QLatin1String("StartupWMClass="))) wmClass = line.mid(15).trimmed();
            }
            if (iconName.isEmpty()) continue;

            auto putIfEmpty = [&](const QString& k, const QString& v) {
                if (!k.isEmpty() && !m_desktopIcons.contains(k)) m_desktopIcons.insert(k, v);
            };
            if (!execLine.isEmpty()) {
                QString tok;
                const QStringList parts = execLine.split(QRegularExpression(QStringLiteral("\\s+")), Qt::SkipEmptyParts);
                for (const QString& raw : parts) {
                    QString p = raw;
                    p.remove('"').remove('\'');
                    if (p.isEmpty() || p.contains('=')) continue;
                    const QString base = QFileInfo(p).fileName().toLower();
                    if (base == QLatin1String("env") || base == QLatin1String("sh") || base == QLatin1String("bash") || base == QLatin1String("gtk-launch")) continue;
                    tok = base;
                    break;
                }
                if (!tok.isEmpty()) m_desktopIcons.insert(tok, iconName);
            }
            putIfEmpty(wmClass.toLower(), iconName);
            putIfEmpty(dispName.toLower(), iconName);
            putIfEmpty(QFileInfo(it.filePath()).completeBaseName().toLower(), iconName);
        }
    }
}

bool PortmasterBackend::hasCachedIcon(const QString& profileKey) const {
    QMutexLocker lock(&m_iconStore->mutex);
    return !m_iconStore->images.value(profileKey).isNull();
}

void PortmasterBackend::queueIconResolve(const QString& profileKey, bool prioritize) {
    if (m_iconAttempted.contains(profileKey) || m_iconQueued.contains(profileKey)) return;
    m_iconQueued.insert(profileKey);
    if (prioritize) m_iconQueue.prepend(profileKey);
    else m_iconQueue.append(profileKey);
    if (m_iconBatchTimer && !m_iconBatchTimer->isActive()) {
        m_iconBatchTimer->start();
    }
}

void PortmasterBackend::processNextIconBatch() {
    if (m_iconQueue.isEmpty()) {
        if (m_iconBatchTimer) m_iconBatchTimer->stop();
        return;
    }
    int resolvedAny = 0;
    const int batchLimit = 10;
    for (int processed = 0; processed < batchLimit && !m_iconQueue.isEmpty(); ++processed) {
        const QString profileKey = m_iconQueue.takeFirst();
        m_iconQueued.remove(profileKey);
        if (m_iconAttempted.contains(profileKey) || !m_profiles.contains(profileKey)) continue;
        m_iconAttempted.insert(profileKey);

        const QJsonObject profile = m_profiles.value(profileKey);
        QString path = profile.value(QStringLiteral("PresentationPath")).toString();
        if (path.isEmpty()) path = profile.value(QStringLiteral("LinkedPath")).toString();
        const QString name = profile.value(QStringLiteral("Name")).toString(profile.value(QStringLiteral("ID")).toString());

        const QFileInfo fi(path);
        QString base = fi.fileName().toLower();
        if (base.endsWith(QLatin1String(".exe"))) base.chop(4);
        QString baseNoBin = base;
        if (baseNoBin.endsWith(QLatin1String("-bin"))) baseNoBin.chop(4);

        QStringList candidates;
        auto addCand = [&](const QString& s) {
            if (!s.isEmpty() && !candidates.contains(s)) candidates.append(s);
        };
        addCand(m_desktopIcons.value(base));
        addCand(m_desktopIcons.value(baseNoBin));
        addCand(m_desktopIcons.value(name.toLower()));
        addCand(base);
        addCand(baseNoBin);
        addCand(name.toLower());

        QIcon resolved;
        for (const QString& cand : candidates) {
            if (QFileInfo::exists(cand) && QFileInfo(cand).isFile()) {
                QIcon fileIcon(cand);
                if (!fileIcon.isNull()) { resolved = fileIcon; break; }
            }
            if (QIcon::hasThemeIcon(cand)) {
                resolved = QIcon::fromTheme(cand);
                if (!resolved.isNull()) break;
            }
        }

        if (!resolved.isNull()) {
            const QImage img = resolved.pixmap(48, 48).toImage();
            if (!img.isNull()) {
                {
                    QMutexLocker lock(&m_iconStore->mutex);
                    m_iconStore->images.insert(profileKey, img);
                }
                ++resolvedAny;
                continue;
            }
        }

        const QJsonArray icons = profile.value(QStringLiteral("Icons")).toArray();
        if (!icons.isEmpty() && m_iconPending.size() < 4) {
            const QJsonObject first = icons.first().toObject();
            if (first.value(QStringLiteral("Type")).toString() == QLatin1String("api")) {
                const QString val = first.value(QStringLiteral("Value")).toString();
                if (!val.isEmpty() && !m_iconPending.contains(profileKey)) {
                    m_iconPending.insert(profileKey);
                    QPointer<PortmasterBackend> self(this);
                    m_api->httpGet(QStringLiteral("/api/v1/profile/icon/") + QString::fromUtf8(QUrl::toPercentEncoding(val)),
                        [self, profileKey](int code, const QByteArray& body, const QString&) {
                            if (!self) return;
                            self->m_iconPending.remove(profileKey);
                            if (code != 200 || body.isEmpty()) return;
                            QImage img = QImage::fromData(body);
                            if (img.isNull()) return;
                            {
                                QMutexLocker lock(&self->m_iconStore->mutex);
                                self->m_iconStore->images.insert(profileKey, img.scaled(64, 64, Qt::KeepAspectRatio, Qt::SmoothTransformation));
                            }
                            if (self->m_iconNotifyTimer) self->m_iconNotifyTimer->start();
                        });
                }
            }
        }
    }
    if (resolvedAny > 0 || (m_iconQueue.isEmpty() && m_iconPending.isEmpty())) {
        if (m_iconNotifyTimer) m_iconNotifyTimer->start();
    }
}

QString PortmasterBackend::cfgText(const QString& key) const {
    const auto& dict = m_ru ? m_i18nRu : m_i18nEn;
    if (dict.contains(key)) return dict.value(key);
    return key;
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

bool PortmasterBackend::blockInternet() const {
    return jsonToBool(effectiveValue(QStringLiteral("filter/blockInternet")));
}

bool PortmasterBackend::blockLan() const {
    return jsonToBool(effectiveValue(QStringLiteral("filter/blockLAN")));
}

bool PortmasterBackend::blockP2p() const {
    return jsonToBool(effectiveValue(QStringLiteral("filter/blockP2P")));
}

bool PortmasterBackend::blockInbound() const {
    return jsonToBool(effectiveValue(QStringLiteral("filter/blockInbound")), true);
}

bool PortmasterBackend::preventDnsBypass() const {
    return jsonToBool(effectiveValue(QStringLiteral("filter/preventBypassing")), true);
}

bool PortmasterBackend::dnsNoInsecure() const {
    return jsonToBool(effectiveValue(QStringLiteral("dns/noInsecureProtocols")));
}

bool PortmasterBackend::dnsIgnoreSystem() const {
    return jsonToBool(effectiveValue(QStringLiteral("dns/noAssignedNameservers")));
}

bool PortmasterBackend::dnsUseStaleCache() const {
    return jsonToBool(effectiveValue(QStringLiteral("dns/useStaleCache")));
}

QStringList PortmasterBackend::dnsServers() const {
    QStringList out;
    for (const auto& v : effectiveValue(QStringLiteral("dns/nameservers")).toArray()) {
        const QString s = v.toString().trimmed();
        if (!s.isEmpty()) out << s;
    }
    return out;
}

QString PortmasterBackend::dnsSummary() const {
    return summarizeDnsList(dnsServers(), m_ru);
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
    if (!m_selectedProfileKey.isEmpty()) rebuildSelectedAppCache();
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
        c = m_paused ? QColor(0xe9, 0xd8, 0x1d) : QColor(0x1d, 0xe9, 0x66);
    }
    m_tray->setIcon(buildShieldIcon(c));
}

void PortmasterBackend::onConnectionChanged(bool connected, const QString&) {
    emit connectionChanged();
    updateTrayIcon();
    if (connected) {
        m_uiPollTick = 0;
        startSubscriptions();
        refreshHttpData();
        if (!m_worldLoaded) fetchWorldMap();
    } else {
        m_historyLoaded = false;
        m_historyFetching = false;
    }
}

void PortmasterBackend::scheduleAppsRebuild() {
    if (!m_appsTimer->isActive()) m_appsTimer->start();
}

void PortmasterBackend::scheduleAppsTickRebuild() {
    if (!m_appsTickTimer->isActive()) m_appsTickTimer->start();
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
        if (k == QLatin1String("spn/use") || k == QLatin1String("splittun/use") || k == QLatin1String("filter/defaultAction") || k == QLatin1String("filter/blockInternet")) {
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
            const int cap = qMax(250, m_historyLimit);
            if (m_connections.size() > cap) m_connections.resize(cap);
        }
        scheduleConnsRebuild();
    });
}

void PortmasterBackend::refreshHttpData() {
    if (!m_api->isConnected()) return;

    // Poll /api/v1/ui/flavors at most every ~15s (5 x 3s) to avoid frequent /proc scans.
    if ((m_uiPollTick % 5) == 0) {
        m_api->httpGet(QStringLiteral("/api/v1/ui/flavors"), [this](int code, const QByteArray& body, const QString&) {
            if (code == 200) {
                m_uiReportMap = QJsonDocument::fromJson(body).object().toVariantMap();
                emit uiReportChanged();
            }
        });
    }
    ++m_uiPollTick;

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
    if (!m_historyLoaded && !m_historyFetching) {
        loadHistoryFromDb(false);
    }
}

void PortmasterBackend::loadHistoryFromDb(bool forceReload) {
    if (!m_api->isConnected()) return;
    if (m_historyFetching && !forceReload) return;
    m_historyFetching = true;
    ++m_historyFetchGen;
    emit historyConfigChanged();
    QVector<QJsonObject> acc;
    acc.reserve(qMin(m_historyLimit, 2000));
    fetchHistoryPage(m_historyFetchGen, QStringLiteral("main"), 0, m_historyLimit, std::move(acc));
}

void PortmasterBackend::fetchHistoryPage(int gen, const QString& db, int page, int targetLimit, QVector<QJsonObject> acc) {
    if (gen != m_historyFetchGen || !m_api->isConnected()) {
        if (gen == m_historyFetchGen) {
            m_historyFetching = false;
            emit historyConfigChanged();
        }
        return;
    }
    const int pageSize = 100;
    const QJsonObject req{
        {QStringLiteral("databases"), QJsonArray{db}},
        {QStringLiteral("orderBy"), QJsonArray{QJsonObject{{QStringLiteral("field"), QStringLiteral("started")}, {QStringLiteral("desc"), true}}}},
        {QStringLiteral("page"), page},
        {QStringLiteral("pageSize"), pageSize},
    };
    const QByteArray payload = QJsonDocument(req).toJson(QJsonDocument::Compact);
    QPointer<PortmasterBackend> self(this);
    m_api->httpPost(QStringLiteral("/api/v1/netquery/query"), payload, QStringLiteral("application/json"),
        [self, gen, db, page, targetLimit, acc = std::move(acc)](int code, const QByteArray& body, const QString& err) mutable {
            if (!self || gen != self->m_historyFetchGen) return;
            if (code != 200) {
                qWarning("netquery/query (%s p%d) failed: http=%d err=%s", qPrintable(db), page, code, qPrintable(err));
                self->m_historyFetching = false;
                if (!acc.isEmpty()) {
                    self->m_historyLoaded = true;
                    self->mergeDbConnections(acc);
                }
                emit self->historyConfigChanged();
                return;
            }
            const QJsonArray arr = QJsonDocument::fromJson(body).object().value(QStringLiteral("results")).toArray();
            for (const auto& v : arr) {
                if (v.isObject()) acc.append(normalizeConn(v.toObject()));
            }
            if (!arr.isEmpty() && (page == 0 || acc.size() % 500 < 100)) {
                self->mergeDbConnections(acc);
            }
            if (arr.size() >= 100 && acc.size() < targetLimit) {
                self->fetchHistoryPage(gen, db, page + 1, targetLimit, std::move(acc));
                return;
            }
            if (db == QLatin1String("main") && acc.size() < targetLimit) {
                self->fetchHistoryPage(gen, QStringLiteral("history"), 0, targetLimit, std::move(acc));
                return;
            }
            self->m_historyLoaded = true;
            self->m_historyFetching = false;
            self->mergeDbConnections(acc);
            emit self->historyConfigChanged();
        });
}

void PortmasterBackend::mergeDbConnections(const QVector<QJsonObject>& rows) {
    QHash<QString, int> idxById;
    idxById.reserve(m_connections.size() + rows.size());
    for (int i = 0; i < m_connections.size(); ++i) {
        const QString id = m_connections[i].value(QStringLiteral("id")).toString();
        if (!id.isEmpty()) idxById.insert(id, i);
    }
    for (const auto& r : rows) {
        const QString id = r.value(QStringLiteral("id")).toString();
        if (id.isEmpty()) {
            m_connections.append(r);
            continue;
        }
        auto it = idxById.constFind(id);
        if (it == idxById.constEnd()) {
            idxById.insert(id, m_connections.size());
            m_connections.append(r);
        }
    }
    std::stable_sort(m_connections.begin(), m_connections.end(), [](const QJsonObject& a, const QJsonObject& b) {
        const qint64 ta = static_cast<qint64>(a.value(QStringLiteral("started_ts")).toDouble(0));
        const qint64 tb = static_cast<qint64>(b.value(QStringLiteral("started_ts")).toDouble(0));
        return ta > tb;
    });
    const int cap = qMax(250, m_historyLimit);
    if (m_connections.size() > cap) {
        m_connections.resize(cap);
        m_connections.squeeze();
    }
    rebuildConnectionsCache();
    scheduleAppsTickRebuild();
    if (!m_selectedProfileKey.isEmpty()) rebuildSelectedAppCache();
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

        auto decodeRings = [&](const QJsonArray& rings, double& bMinX, double& bMinY, double& bMaxX, double& bMaxY) -> QVariantList {
            QVariantList out;
            if (rings.isEmpty()) return out;
            const QJsonArray ring = rings[0].toArray();
            QVariantList polyPts;
            double prevLon = 0.0;
            bool hasPrev = false;

            auto flushPoly = [&]() {
                // Packed flat [x0, y0, x1, y1, ...] avoids ~70k heap QVariantMap allocations.
                if (polyPts.size() >= 6) out.append(QVariant::fromValue(polyPts));
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
                    if (ll.first < -60.0) {
                        flushPoly();
                        hasPrev = false;
                        continue;
                    }
                    if (hasPrev && std::abs(ll.second - prevLon) > 100.0) {
                        flushPoly();
                    }
                    QPointF np = projectNormalized(ll.first, ll.second);
                    polyPts.append(np.x());
                    polyPts.append(np.y());
                    bMinX = qMin(bMinX, np.x());
                    bMinY = qMin(bMinY, np.y());
                    bMaxX = qMax(bMaxX, np.x());
                    bMaxY = qMax(bMaxY, np.y());
                    prevLon = ll.second;
                    hasPrev = true;
                }
            }
            flushPoly();
            return out;
        };

        m_countryNames.clear();
        QVariantList countries;
        const QJsonArray geoms = root.value(QStringLiteral("objects")).toObject().value(QStringLiteral("countries")).toObject().value(QStringLiteral("geometries")).toArray();
        for (const auto& gVal : geoms) {
            const QJsonObject g = gVal.toObject();
            const QJsonObject props = g.value(QStringLiteral("properties")).toObject();
            const QString iso = props.value(QStringLiteral("iso_a2")).toString().trimmed().toUpper();
            if (iso == QLatin1String("AQ")) continue;
            const QString name = props.value(QStringLiteral("name")).toString();
            if (!iso.isEmpty() && iso != QLatin1String("-99") && !name.isEmpty()) {
                m_countryNames.insert(iso, name);
            }
            const QString gtype = g.value(QStringLiteral("type")).toString();
            const QJsonArray gArcs = g.value(QStringLiteral("arcs")).toArray();

            double bMinX = 1.0, bMinY = 1.0, bMaxX = 0.0, bMaxY = 0.0;
            QVariantList cPolys;
            if (gtype == QLatin1String("Polygon")) {
                cPolys.append(decodeRings(gArcs, bMinX, bMinY, bMaxX, bMaxY));
            } else if (gtype == QLatin1String("MultiPolygon")) {
                for (const auto& polyVal : gArcs) {
                    cPolys.append(decodeRings(polyVal.toArray(), bMinX, bMinY, bMaxX, bMaxY));
                }
            }
            if (!cPolys.isEmpty()) {
                countries.append(QVariantMap{
                    {QStringLiteral("code"), iso},
                    {QStringLiteral("name"), name},
                    {QStringLiteral("minX"), bMinX},
                    {QStringLiteral("minY"), bMinY},
                    {QStringLiteral("maxX"), bMaxX},
                    {QStringLiteral("maxY"), bMaxY},
                    {QStringLiteral("polygons"), cPolys},
                });
            }
        }

        m_worldPolygons.clear();
        m_worldPolygons.squeeze();
        m_worldCountries = countries;
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

    // Single linear pass over connections so we avoid O(profiles * connections) scans.
    struct ConnAgg {
        int total = 0;
        int alive = 0;
        int blocked = 0;
        qint64 maxRx = 0;
        qint64 maxTx = 0;
    };
    QHash<QString, ConnAgg> aggById;
    QHash<QString, ConnAgg> aggByName;
    aggById.reserve(128);
    aggByName.reserve(128);

    for (const auto& c : m_connections) {
        const QString cp = c.value(QStringLiteral("profile")).toString();
        const QString bareCp = cp.contains('/') ? cp.section('/', -1) : cp;
        const QString pName = c.value(QStringLiteral("profile_name")).toString().toLower();
        const qint64 rxVal = static_cast<qint64>(c.value(QStringLiteral("bytes_received")).toDouble());
        const qint64 txVal = static_cast<qint64>(c.value(QStringLiteral("bytes_sent")).toDouble());
        const bool isAlive = c.value(QStringLiteral("active")).toBool();
        const QString vd = c.value(QStringLiteral("verdict")).toString();
        const bool isBlk = (vd == QLatin1String("block") || vd == QLatin1String("drop") || vd == QLatin1String("failed"));

        auto accum = [&](ConnAgg& a) {
            ++a.total;
            a.maxRx = qMax(a.maxRx, rxVal);
            a.maxTx = qMax(a.maxTx, txVal);
            if (isAlive) ++a.alive;
            if (isBlk) ++a.blocked;
        };
        if (!bareCp.isEmpty()) accum(aggById[bareCp]);
        else if (!pName.isEmpty()) accum(aggByName[pName]);
    }

    QVector<QPair<QString, QJsonObject>> sorted;
    sorted.reserve(m_profiles.size());
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

        QString globalAction = effectiveValue(QStringLiteral("filter/defaultAction")).toString(QStringLiteral("permit"));
        QString defAct = readProfileCfg(cfg, QStringLiteral("filter/defaultAction")).toString(globalAction);
        QJsonValue blkNetVal = readProfileCfg(cfg, QStringLiteral("filter/blockInternet"));
        bool appBlkNet = blkNetVal.isUndefined() || blkNetVal.isNull() ? blockInternet() : jsonToBool(blkNetVal);
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

        auto idIt = aggById.constFind(id);
        if (idIt != aggById.constEnd()) {
            totalSeen = idIt->total;
            blockedSeen = idIt->blocked;
            liveActive += idIt->alive;
            rx = qMax(rx, idIt->maxRx);
            tx = qMax(tx, idIt->maxTx);
        } else if (!name.isEmpty()) {
            auto nIt = aggByName.constFind(name.toLower());
            if (nIt != aggByName.constEnd()) {
                totalSeen = nIt->total;
                blockedSeen = nIt->blocked;
                liveActive += nIt->alive;
                rx = qMax(rx, nIt->maxRx);
                tx = qMax(tx, nIt->maxTx);
            }
        }

        const bool hasIcon = hasCachedIcon(key);
        if (!hasIcon) queueIconResolve(key, liveActive > 0);
        const QString iconKey = QString::fromUtf8(QUrl::toPercentEncoding(key));
        const QString avatarColor = appAvatarColor(id.isEmpty() ? name : id);

        QVariantMap item{
            {QStringLiteral("key"), key},
            {QStringLiteral("id"), id},
            {QStringLiteral("source"), p.value(QStringLiteral("Source")).toString(QStringLiteral("local"))},
            {QStringLiteral("name"), name},
            {QStringLiteral("initials"), name.left(2).toUpper()},
            {QStringLiteral("hasIcon"), hasIcon},
            {QStringLiteral("iconKey"), iconKey},
            {QStringLiteral("avatarColor"), avatarColor},
            {QStringLiteral("path"), path},
            {QStringLiteral("action"), defAct},
            {QStringLiteral("blockInternet"), appBlkNet},
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
    m_selectedAppSettings.clear();
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

    int customSettingsCount = 0;
    for (auto it = m_settings.cbegin(); it != m_settings.cend(); ++it) {
        QVariantMap item = buildSettingMap(it.value(), &cfg);
        if (!item.value(QStringLiteral("settablePerApp")).toBool()) continue;
        if (item.value(QStringLiteral("modified")).toBool()) ++customSettingsCount;
        m_selectedAppSettings.append(item);
    }
    sortSettingsByOrder(m_selectedAppSettings);

    auto readScopeBool = [&](const QString& cfgKey, bool globalFallback, bool& overridden) -> bool {
        QJsonValue local = readProfileCfg(cfg, cfgKey);
        overridden = !local.isUndefined() && !local.isNull();
        return overridden ? jsonToBool(local, globalFallback) : globalFallback;
    };
    bool ovInternet = false, ovLan = false, ovP2p = false, ovInbound = false;
    bool appBlkInternet = readScopeBool(QStringLiteral("filter/blockInternet"), blockInternet(), ovInternet);
    bool appBlkLan = readScopeBool(QStringLiteral("filter/blockLAN"), blockLan(), ovLan);
    bool appBlkP2p = readScopeBool(QStringLiteral("filter/blockP2P"), blockP2p(), ovP2p);
    bool appBlkInbound = readScopeBool(QStringLiteral("filter/blockInbound"), blockInbound(), ovInbound);

    QString path = p.value(QStringLiteral("PresentationPath")).toString();
    if (path.isEmpty()) path = p.value(QStringLiteral("LinkedPath")).toString();
    const bool hasIcon = hasCachedIcon(m_selectedProfileKey);
    if (!hasIcon) queueIconResolve(m_selectedProfileKey, true);
    const QString iconKey = QString::fromUtf8(QUrl::toPercentEncoding(m_selectedProfileKey));
    const QString avatarColor = appAvatarColor(id.isEmpty() ? name : id);

    m_selectedAppMap = QVariantMap{
        {QStringLiteral("key"), m_selectedProfileKey},
        {QStringLiteral("id"), id},
        {QStringLiteral("source"), p.value(QStringLiteral("Source")).toString()},
        {QStringLiteral("name"), name},
        {QStringLiteral("initials"), name.left(2).toUpper()},
        {QStringLiteral("hasIcon"), hasIcon},
        {QStringLiteral("iconKey"), iconKey},
        {QStringLiteral("avatarColor"), avatarColor},
        {QStringLiteral("path"), path},
        {QStringLiteral("action"), readProfileCfg(cfg, QStringLiteral("filter/defaultAction")).toString(effectiveValue(QStringLiteral("filter/defaultAction")).toString(QStringLiteral("permit")))},
        {QStringLiteral("actionOverridden"), !readProfileCfg(cfg, QStringLiteral("filter/defaultAction")).isUndefined()},
        {QStringLiteral("vpnMode"), vpnVal.isBool() ? (vpnVal.toBool() ? QStringLiteral("vpn") : QStringLiteral("direct")) : QStringLiteral("default")},
        {QStringLiteral("splitMode"), stVal.isBool() ? (stVal.toBool() ? QStringLiteral("on") : QStringLiteral("off")) : QStringLiteral("default")},
        {QStringLiteral("blockInternet"), appBlkInternet},
        {QStringLiteral("blockLan"), appBlkLan},
        {QStringLiteral("blockP2p"), appBlkP2p},
        {QStringLiteral("blockInbound"), appBlkInbound},
        {QStringLiteral("overriddenInternet"), ovInternet},
        {QStringLiteral("overriddenLan"), ovLan},
        {QStringLiteral("overriddenP2p"), ovP2p},
        {QStringLiteral("overriddenInbound"), ovInbound},
        {QStringLiteral("customCount"), customSettingsCount},
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
    const int limit = qMin(qMax(250, m_historyLimit), m_connections.size());
    m_connectionsCache.reserve(limit);

    int allowedTotal = 0;
    int blockedTotal = 0;
    int internetBlockedTotal = 0;
    int activeTotal = 0;
    int tunneledTotal = 0;
    qint64 totalRx = 0;
    qint64 totalTx = 0;

    qint64 minTs = 0;
    qint64 maxTs = 0;
    int recentCount = 0;
    const qint64 nowSec = QDateTime::currentSecsSinceEpoch();

    QHash<QString, int> countryCounts;
    QHash<QString, int> blockedAppCounts;
    QHash<QString, QString> blockedAppNames;

    for (int i = 0; i < m_connections.size(); ++i) {
        const auto& c = m_connections[i];
        const QString vd = c.value(QStringLiteral("verdict")).toString().toLower();
        const bool isBlk = (vd == QLatin1String("block") || vd == QLatin1String("drop") || vd == QLatin1String("failed"));
        const QString sc = c.value(QStringLiteral("scope")).toString().toLower();
        const bool isActive = c.value(QStringLiteral("active")).toBool();
        const bool isTun = c.value(QStringLiteral("tunneled")).toBool() || vd == QLatin1String("tunneled");
        const qint64 rx = static_cast<qint64>(c.value(QStringLiteral("bytes_received")).toDouble());
        const qint64 tx = static_cast<qint64>(c.value(QStringLiteral("bytes_sent")).toDouble());
        const qint64 ts = static_cast<qint64>(c.value(QStringLiteral("started_ts")).toDouble(0));
        const QString dom = c.value(QStringLiteral("domain")).toString();
        const QString ip = c.value(QStringLiteral("remote_ip")).toString();
        const QString cc = c.value(QStringLiteral("country")).toString().trimmed().toUpper();
        const QString profKey = c.value(QStringLiteral("profileKey")).toString();
        const QString profName = c.value(QStringLiteral("profile_name")).toString();

        if (isBlk) {
            ++blockedTotal;
            if (sc.contains(QLatin1String("internet"))) ++internetBlockedTotal;
            if (!profKey.isEmpty()) {
                blockedAppCounts[profKey] += 1;
                if (!profName.isEmpty()) blockedAppNames[profKey] = profName;
            }
        } else {
            ++allowedTotal;
        }
        if (isActive) ++activeTotal;
        if (isTun) ++tunneledTotal;
        totalRx += rx;
        totalTx += tx;

        if (!isBlk && !cc.isEmpty() && cc != QLatin1String("-99")) {
            countryCounts[cc] += 1;
        }

        if (ts > 0) {
            if (minTs == 0 || ts < minTs) minTs = ts;
            if (ts > maxTs) maxTs = ts;
            if (ts >= nowSec - 600) ++recentCount;
        }

        if (i < limit) {
            m_connectionsCache.append(QVariantMap{
                {QStringLiteral("id"), c.value(QStringLiteral("id")).toString()},
                {QStringLiteral("profileKey"), profKey},
                {QStringLiteral("target"), dom.isEmpty() ? ip : dom},
                {QStringLiteral("domain"), dom},
                {QStringLiteral("ip"), ip},
                {QStringLiteral("port"), c.value(QStringLiteral("remote_port")).toInt()},
                {QStringLiteral("proto"), c.value(QStringLiteral("ip_protocol")).toString()},
                {QStringLiteral("country"), cc},
                {QStringLiteral("app"), profName},
                {QStringLiteral("scope"), sc},
                {QStringLiteral("direction"), c.value(QStringLiteral("direction")).toString()},
                {QStringLiteral("verdict"), vd},
                {QStringLiteral("tunneled"), isTun},
                {QStringLiteral("reason"), c.value(QStringLiteral("reason")).toString()},
                {QStringLiteral("asOwner"), c.value(QStringLiteral("as_owner")).toString()},
                {QStringLiteral("bytes"), formatBytes(rx + tx)},
                {QStringLiteral("active"), isActive},
            });
        }
    }

    m_connAllowedTotal = allowedTotal;
    m_connBlockedTotal = blockedTotal;
    m_connInternetBlockedTotal = internetBlockedTotal;
    m_connActiveTotal = activeTotal;
    m_connTunneledTotal = tunneledTotal;
    m_connRxStr = formatBytes(totalRx);
    m_connTxStr = formatBytes(totalTx);

    // 36-bin green/red activity chart (10-minute rolling window when active, else recent history window).
    constexpr int kBins = 36;
    QVector<int> okBins(kBins, 0);
    QVector<int> blkBins(kBins, 0);
    qint64 winEnd = (recentCount >= 3) ? nowSec : (maxTs > 0 ? maxTs : nowSec);
    qint64 winStart = (recentCount >= 3) ? (nowSec - 600) : (minTs > 0 ? qMax(minTs, winEnd - 3600) : (winEnd - 600));
    if (winEnd <= winStart) winStart = winEnd - 600;
    const double span = static_cast<double>(winEnd - winStart);

    const int chartSampleMax = qMin(m_connections.size(), 600);
    for (int i = 0; i < chartSampleMax; ++i) {
        const auto& c = m_connections[i];
        const qint64 ts = static_cast<qint64>(c.value(QStringLiteral("started_ts")).toDouble(0));
        int bin = -1;
        if (ts > 0 && ts >= winStart && ts <= winEnd + 10) {
            double norm = qBound(0.0, (static_cast<double>(ts) - winStart) / span, 0.9999);
            bin = static_cast<int>(norm * kBins);
        } else if (ts == 0) {
            bin = qBound(0, kBins - 1 - (i * kBins) / qMax(1, chartSampleMax), kBins - 1);
        }
        if (bin < 0 || bin >= kBins) continue;
        const QString vd = c.value(QStringLiteral("verdict")).toString().toLower();
        if (vd == QLatin1String("block") || vd == QLatin1String("drop") || vd == QLatin1String("failed")) {
            ++blkBins[bin];
        } else {
            ++okBins[bin];
        }
    }
    m_connChartBins.clear();
    m_connChartBins.reserve(kBins);
    for (int i = 0; i < kBins; ++i) {
        m_connChartBins.append(QVariantMap{
            {QStringLiteral("allowed"), okBins[i]},
            {QStringLiteral("blocked"), blkBins[i]},
            {QStringLiteral("total"), okBins[i] + blkBins[i]},
        });
    }

    QVector<QPair<QString, int>> cList;
    cList.reserve(countryCounts.size());
    for (auto it = countryCounts.cbegin(); it != countryCounts.cend(); ++it) {
        cList.append({it.key(), it.value()});
    }
    std::sort(cList.begin(), cList.end(), [](const auto& a, const auto& b) {
        if (a.second != b.second) return a.second > b.second;
        return a.first < b.first;
    });
    m_connCountriesSummary.clear();
    for (int i = 0; i < cList.size() && i < 24; ++i) {
        const QString& cc = cList[i].first;
        m_connCountriesSummary.append(QVariantMap{
            {QStringLiteral("code"), cc},
            {QStringLiteral("name"), countryName(cc)},
            {QStringLiteral("count"), cList[i].second},
        });
    }

    QVector<QPair<QString, int>> bList;
    bList.reserve(blockedAppCounts.size());
    for (auto it = blockedAppCounts.cbegin(); it != blockedAppCounts.cend(); ++it) {
        bList.append({it.key(), it.value()});
    }
    std::sort(bList.begin(), bList.end(), [](const auto& a, const auto& b) {
        if (a.second != b.second) return a.second > b.second;
        return a.first < b.first;
    });
    m_blockedAppsSummary.clear();
    for (int i = 0; i < bList.size() && i < 18; ++i) {
        const QString& pk = bList[i].first;
        QString nm = blockedAppNames.value(pk);
        if (m_profiles.contains(pk)) {
            const auto& p = m_profiles.value(pk);
            const QString pn = p.value(QStringLiteral("Name")).toString();
            if (!pn.isEmpty()) nm = pn;
        }
        if (nm.isEmpty()) nm = pk.section('/', -1);
        m_blockedAppsSummary.append(QVariantMap{
            {QStringLiteral("profileKey"), pk},
            {QStringLiteral("name"), nm},
            {QStringLiteral("initials"), nm.left(2).toUpper()},
            {QStringLiteral("hasIcon"), hasCachedIcon(pk)},
            {QStringLiteral("iconKey"), QString::fromUtf8(QUrl::toPercentEncoding(pk))},
            {QStringLiteral("avatarColor"), appAvatarColor(nm)},
            {QStringLiteral("count"), bList[i].second},
        });
    }

    emit connectionsChanged();
}

QVariantMap PortmasterBackend::buildSettingMap(const QJsonObject& s, const QJsonObject* appCfg) const {
    const QString key = s.value(QStringLiteral("Key")).toString();
    const int optType = s.value(QStringLiteral("OptType")).toInt(1);
    const QJsonObject ann = s.value(QStringLiteral("Annotations")).toObject();
    const int order = ann.value(QStringLiteral("safing/portbase:ui:order")).toInt(9999);
    const QString category = ann.value(QStringLiteral("safing/portbase:ui:category")).toString();
    const bool settablePerApp = ann.value(QStringLiteral("safing/portmaster:settable-per-app")).toBool(false);

    QVariantList choices;
    for (const auto& pv : s.value(QStringLiteral("PossibleValues")).toArray()) {
        QVariantMap cm = pv.toObject().toVariantMap();
        const QString rawVal = cm.value(QStringLiteral("Value")).toString();
        const QString rawName = cm.value(QStringLiteral("Name")).toString();
        cm.insert(QStringLiteral("Name"), localizeChoiceName(rawName, rawVal, m_ru));
        choices.append(cm);
    }

    const auto& dict = m_ru ? m_i18nRu : m_i18nEn;
    const QString locName = dict.value(QStringLiteral("cfg.%1.name").arg(key), s.value(QStringLiteral("Name")).toString());
    const QString locDesc = dict.value(QStringLiteral("cfg.%1.desc").arg(key), s.value(QStringLiteral("Description")).toString());

    const QJsonValue globalEff = effectiveCfgVal(s);
    QJsonValue eff = globalEff;
    bool modified = s.contains(QLatin1String("Value")) && !s.value(QLatin1String("Value")).isNull();

    if (appCfg != nullptr) {
        const QJsonValue localVal = readProfileCfg(*appCfg, key);
        modified = !localVal.isUndefined() && !localVal.isNull();
        if (modified) eff = localVal;
    }

    QString strVal;
    QStringList arrayItems;
    if (eff.isBool()) strVal = eff.toBool() ? QStringLiteral("true") : QStringLiteral("false");
    else if (eff.isDouble()) strVal = QString::number(static_cast<qint64>(eff.toDouble()));
    else if (eff.isString()) strVal = eff.toString();
    else if (eff.isArray()) {
        for (const auto& a : eff.toArray()) arrayItems << a.toString();
        strVal = arrayItems.join(QStringLiteral(", "));
    }

    return QVariantMap{
        {QStringLiteral("key"), key},
        {QStringLiteral("subsystem"), key.section('/', 0, 0)},
        {QStringLiteral("category"), category},
        {QStringLiteral("order"), order},
        {QStringLiteral("settablePerApp"), settablePerApp},
        {QStringLiteral("name"), locName},
        {QStringLiteral("description"), locDesc},
        {QStringLiteral("optType"), optType},
        {QStringLiteral("expertise"), s.value(QStringLiteral("ExpertiseLevel")).toInt(0)},
        {QStringLiteral("modified"), modified},
        {QStringLiteral("valueStr"), strVal},
        {QStringLiteral("boolVal"), jsonToBool(eff)},
        {QStringLiteral("intVal"), static_cast<int>(eff.toDouble(0))},
        {QStringLiteral("arrayItems"), arrayItems},
        {QStringLiteral("choices"), choices},
        {QStringLiteral("globalDisplay"), formatSettingDisplay(globalEff, optType, choices, m_ru)},
    };
}

void PortmasterBackend::rebuildSettingsCache() {
    m_settingsCache.clear();
    for (auto it = m_settings.cbegin(); it != m_settings.cend(); ++it) {
        m_settingsCache.append(buildSettingMap(it.value(), nullptr));
    }
    sortSettingsByOrder(m_settingsCache);
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
    setAppSetting(scopedKey, QStringLiteral("filter/defaultAction"), action);
}

void PortmasterBackend::setAppVpnMode(const QString& scopedKey, const QString& mode) {
    if (mode == QLatin1String("vpn")) setAppSetting(scopedKey, QStringLiteral("spn/use"), true);
    else if (mode == QLatin1String("direct")) setAppSetting(scopedKey, QStringLiteral("spn/use"), false);
    else resetAppSetting(scopedKey, QStringLiteral("spn/use"));
}

void PortmasterBackend::setAppSplitMode(const QString& scopedKey, const QString& mode) {
    if (mode == QLatin1String("on")) setAppSetting(scopedKey, QStringLiteral("splittun/use"), true);
    else if (mode == QLatin1String("off")) setAppSetting(scopedKey, QStringLiteral("splittun/use"), false);
    else resetAppSetting(scopedKey, QStringLiteral("splittun/use"));
}

void PortmasterBackend::setAppBlockLan(const QString& scopedKey, bool blocked) {
    setAppSetting(scopedKey, QStringLiteral("filter/blockLAN"), blocked);
}

void PortmasterBackend::setAppSetting(const QString& scopedKey, const QString& cfgKey, const QVariant& val) {
    if (!m_profiles.contains(scopedKey) || cfgKey.isEmpty()) return;
    QJsonObject p = m_profiles.value(scopedKey);
    QJsonObject cfg = p.value(QStringLiteral("Config")).toObject();
    writeProfileCfg(cfg, cfgKey, variantToJsonValue(val));
    p.insert(QStringLiteral("Config"), cfg);
    m_profiles.insert(scopedKey, p);
    m_api->sendPutFrame(QStringLiteral("update"), scopedKey, p);
    rebuildAppsCache();
    if (scopedKey == m_selectedProfileKey) rebuildSelectedAppCache();
}

void PortmasterBackend::resetAppSetting(const QString& scopedKey, const QString& cfgKey) {
    if (!m_profiles.contains(scopedKey) || cfgKey.isEmpty()) return;
    QJsonObject p = m_profiles.value(scopedKey);
    QJsonObject cfg = p.value(QStringLiteral("Config")).toObject();
    writeProfileCfg(cfg, cfgKey, QJsonValue::Undefined);
    p.insert(QStringLiteral("Config"), cfg);
    m_profiles.insert(scopedKey, p);
    m_api->sendPutFrame(QStringLiteral("update"), scopedKey, p);
    rebuildAppsCache();
    if (scopedKey == m_selectedProfileKey) rebuildSelectedAppCache();
}

void PortmasterBackend::applyAppPreset(const QString& scopedKey, const QString& cfgKey, const QStringList& values, bool append) {
    if (!m_profiles.contains(scopedKey) || cfgKey.isEmpty()) return;
    const QJsonObject cfg = m_profiles.value(scopedKey).value(QStringLiteral("Config")).toObject();
    QJsonValue baseVal = readProfileCfg(cfg, cfgKey);
    if (baseVal.isUndefined() || baseVal.isNull()) baseVal = effectiveValue(cfgKey);
    setAppSetting(scopedKey, cfgKey, mergePresetValues(baseVal.toArray(), values, append));
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
    setAppSetting(scopedKey, QStringLiteral("filter/endpoints"), arr.toVariantList());
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
    setAppSetting(scopedKey, QStringLiteral("filter/endpoints"), next.toVariantList());
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
            m_historyLoaded = true;
            rebuildConnectionsCache();
            scheduleAppsTickRebuild();
        }
    });
}

void PortmasterBackend::setHistoryLimit(int limit) {
    const int clamped = qBound(250, limit, 10000);
    if (m_historyLimit == clamped) return;
    m_historyLimit = clamped;
    QSettings qs(QStringLiteral("portmaster"), QStringLiteral("qt-ui"));
    qs.setValue(QStringLiteral("historyLimit"), m_historyLimit);
    emit historyConfigChanged();
    if (m_connections.size() > m_historyLimit) {
        m_connections.resize(m_historyLimit);
        m_connections.squeeze();
        rebuildConnectionsCache();
    } else {
        loadHistoryFromDb(true);
    }
}

void PortmasterBackend::refreshHistory() {
    loadHistoryFromDb(true);
}

QString PortmasterBackend::countryFlagUrl(const QString& code) const {
    QString up = code.trimmed().toUpper();
    if (up == QLatin1String("UK")) up = QStringLiteral("GB");
    if (up.size() != 2) return QString();
    const QString rcPath = QStringLiteral(":/qt/qml/Portmaster/flags/%1.png").arg(up);
    return QFile::exists(rcPath) ? (QStringLiteral("qrc") + rcPath) : QString();
}

QString PortmasterBackend::countryName(const QString& code) const {
    const QString up = code.trimmed().toUpper();
    if (up.isEmpty()) return QString();
    if (m_countryNames.contains(up)) return m_countryNames.value(up);
    if (up == QLatin1String("UK") && m_countryNames.contains(QStringLiteral("GB"))) {
        return m_countryNames.value(QStringLiteral("GB"));
    }
    return up;
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
    payload.insert(QStringLiteral("Value"), variantToJsonValue(val));
    m_settings.insert(key, payload);
    rebuildSettingsCache();
    if (!m_selectedProfileKey.isEmpty()) rebuildSelectedAppCache();
    if (key.startsWith(QLatin1String("filter/")) || key == QLatin1String("spn/use") || key == QLatin1String("splittun/use")) {
        rebuildAppsCache();
    }
    m_api->sendPutFrame(QStringLiteral("update"), QStringLiteral("config:") + key, payload);
}

void PortmasterBackend::resetSetting(const QString& key) {
    QJsonObject payload = m_settings.contains(key) ? m_settings.value(key) : QJsonObject{{QStringLiteral("Key"), key}};
    payload.insert(QStringLiteral("Key"), key);
    payload.remove(QStringLiteral("Value"));
    m_settings.insert(key, payload);
    rebuildSettingsCache();
    if (!m_selectedProfileKey.isEmpty()) rebuildSelectedAppCache();
    if (key.startsWith(QLatin1String("filter/")) || key == QLatin1String("spn/use") || key == QLatin1String("splittun/use")) {
        rebuildAppsCache();
    }
    m_api->sendPutFrame(QStringLiteral("update"), QStringLiteral("config:") + key, payload);
}

void PortmasterBackend::applyPreset(const QString& key, const QStringList& values, bool append) {
    updateSetting(key, mergePresetValues(effectiveValue(key).toArray(), values, append));
}

void PortmasterBackend::moveSettingArrayItem(const QString& key, int index, int delta) {
    updateSetting(key, moveArrayElement(effectiveValue(key).toArray(), index, delta));
}

void PortmasterBackend::moveAppArrayItem(const QString& scopedKey, const QString& cfgKey, int index, int delta) {
    if (!m_profiles.contains(scopedKey) || cfgKey.isEmpty()) return;
    const QJsonObject cfg = m_profiles.value(scopedKey).value(QStringLiteral("Config")).toObject();
    QJsonValue baseVal = readProfileCfg(cfg, cfgKey);
    if (baseVal.isUndefined() || baseVal.isNull()) baseVal = effectiveValue(cfgKey);
    setAppSetting(scopedKey, cfgKey, moveArrayElement(baseVal.toArray(), index, delta));
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
        m_uiPollTick = 0;
        QTimer::singleShot(450, this, &PortmasterBackend::refreshHttpData);
    });
}

void PortmasterBackend::setHwAccel(bool enabled) {
    if (m_hwAccel == enabled) return;
    m_hwAccel = enabled;
    QSettings qs(QStringLiteral("portmaster"), QStringLiteral("qt-ui"));
    qs.setValue(QStringLiteral("hwAccel"), m_hwAccel);
    emit uiReportChanged();
    emit toastMessage(
        m_ru ? QStringLiteral("Аппаратное ускорение") : QStringLiteral("Hardware Acceleration"),
        m_ru ? QStringLiteral("Настройка сохранена. Перезапустите Qt UI, чтобы применить режим рендеринга.")
             : QStringLiteral("Setting saved. Restart Qt UI to apply the graphics backend change."),
        false);
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
