#include "PortmasterBackend.h"

#include <QApplication>
#include <QCoreApplication>
#include <QDateTime>
#include <QDesktopServices>
#include <QDir>
#include <QDirIterator>
#include <QFile>
#include <QFileInfo>
#include <QGuiApplication>
#include <QHostAddress>
#include <QIcon>
#include <QJsonArray>
#include <QJsonDocument>
#include <QMenu>
#include <QMutexLocker>
#include <QPainter>
#include <QPixmap>
#include <QPointer>
#include <QPolygon>
#include <QProcess>
#include <QRegularExpression>
#include <QSettings>
#include <QStandardPaths>
#if QT_VERSION >= QT_VERSION_CHECK(6, 5, 0)
#include <QStyleHints>
#endif
#include <QTextStream>
#include <QTimeZone>
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

QString formatBytesStatic(qint64 bytes) {
    if (bytes >= 1024LL * 1024 * 1024) return QStringLiteral("%1 GB").arg(bytes / (1024.0 * 1024.0 * 1024.0), 0, 'f', 2);
    if (bytes >= 1024LL * 1024) return QStringLiteral("%1 MB").arg(bytes / (1024.0 * 1024.0), 0, 'f', 1);
    if (bytes >= 1024) return QStringLiteral("%1 KB").arg(bytes / 1024.0, 0, 'f', 0);
    return QStringLiteral("%1 B").arg(bytes);
}

bool isKnownDnsResolverIp(const QString& ip) {
    static const QSet<QString> kIps = {
        QStringLiteral("9.9.9.9"), QStringLiteral("9.9.9.10"), QStringLiteral("9.9.9.11"),
        QStringLiteral("149.112.112.112"), QStringLiteral("149.112.112.11"), QStringLiteral("149.112.112.10"),
        QStringLiteral("1.1.1.1"), QStringLiteral("1.0.0.1"), QStringLiteral("1.1.1.2"), QStringLiteral("1.0.0.2"),
        QStringLiteral("8.8.8.8"), QStringLiteral("8.8.4.4"),
        QStringLiteral("94.140.14.14"), QStringLiteral("94.140.15.15"),
        QStringLiteral("2606:4700:4700::1111"), QStringLiteral("2606:4700:4700::1001"),
        QStringLiteral("2620:fe::fe"), QStringLiteral("2620:fe::9"),
        QStringLiteral("2001:4860:4860::8888"), QStringLiteral("2001:4860:4860::8844"),
    };
    return kIps.contains(ip);
}

bool isKnownDohDomain(const QString& domain) {
    if (domain.isEmpty()) return false;
    QString d = domain.toLower();
    while (d.endsWith('.')) d.chop(1);
    return d == QLatin1String("dns.quad9.net") || d == QLatin1String("cloudflare-dns.com")
        || d == QLatin1String("mozilla.cloudflare-dns.com") || d == QLatin1String("dns.google")
        || d == QLatin1String("dns.adguard-dns.com") || d == QLatin1String("dns.nextdns.io")
        || d == QLatin1String("dns.mullvad.net") || d == QLatin1String("freedns.controld.com")
        || d.startsWith(QLatin1String("doh."));
}

bool isDnsNoiseConn(const ConnRecord& c) {
    if (c.port == 53 || c.port == 853 || c.port == 5353) return true;
    if (c.connType == QLatin1String("dns")) return true;
    if (c.profileKey.endsWith(QLatin1String("_system-resolver"))) return true;
    if (c.profileKey.endsWith(QLatin1String("_portmaster")) &&
        (c.internal || c.domain.isEmpty() || isKnownDnsResolverIp(c.ip))) {
        return true;
    }
    if ((c.port == 443 || c.port == 853) && (isKnownDnsResolverIp(c.ip) || isKnownDohDomain(c.domain))) {
        return true;
    }
    return false;
}

bool isLocalhostConn(const ConnRecord& c) {
    return c.scope.contains(QLatin1String("local"))
        || c.ip == QLatin1String("127.0.0.1")
        || c.ip == QLatin1String("::1")
        || c.ip == QLatin1String("::")
        || c.ip.startsWith(QLatin1String("127."));
}

ConnRecord normalizeConnRecord(const QJsonObject& raw, bool session) {
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
        if (dt.isValid()) {
            // SQLite history rows without a Z/+offset suffix are stored in UTC.
            if (dt.timeSpec() == Qt::LocalTime) dt.setTimeZone(QTimeZone::utc());
            startedTs = dt.toSecsSinceEpoch();
        }
    } else if (stVal.isDouble()) {
        qint64 rawTs = static_cast<qint64>(stVal.toDouble(0));
        startedTs = (rawTs > 100000000000LL) ? (rawTs / 1000) : rawTs;
    }

    const qint64 rx = static_cast<qint64>(pick("bytes_received", "BytesReceived").toDouble(0));
    const qint64 tx = static_cast<qint64>(pick("bytes_sent", "BytesSent").toDouble(0));

    ConnRecord rec;
    rec.id = pick("id", "ID").toString();
    rec.profile = profRaw;
    rec.profileKey = dbKey;
    rec.profileName = pick("profile_name", "ProfileName").toString();
    rec.domain = pick("domain", "Domain").toString();
    rec.ip = pick("remote_ip", "RemoteIP").toString();
    rec.port = pick("remote_port", "RemotePort").toInt();
    rec.proto = protoNameFromVal(pick("ip_protocol", "IPProtocol"));
    rec.direction = pick("direction", "Direction").toString(QStringLiteral("outbound")).toLower();
    rec.scope = scopeSlugFromVal(pick("scope", "Scope"));
    rec.country = pick("country", "Country").toString().trimmed().toUpper();
    rec.asOwner = pick("as_owner", "ASOwner").toString();
    rec.verdict = verdict;
    rec.reason = reason;
    rec.connType = pick("type", "Type").toString().toLower();
    rec.rx = rx;
    rec.tx = tx;
    rec.bytesStr = formatBytesStatic(rx + tx);
    rec.startedTs = startedTs;
    rec.tunneled = tunneled || verdict == QLatin1String("tunneled");
    rec.active = pick("active", "Active").toBool(false);
    rec.internal = pick("internal", "Internal").toBool(false);
    rec.session = session;
    return rec;
}

struct ChartSample {
    qint64 ts = 0;
    bool blocked = false;
    bool internet = false;
    qint64 rx = 0;
    qint64 tx = 0;
};

QVariantList buildChartBinsHelper(const QVector<ChartSample>& samples,
                                  bool internetOnly,
                                  bool preferRecentWindow,
                                  qint64 nowSec,
                                  const QHash<qint64, QPair<qint64, qint64>>* liveBwBins = nullptr) {
    constexpr int kBins = 36;
    constexpr qint64 kLiveStepSec = 20; // Fixed 20s wall-clock buckets (36 * 20s = 12 min) prevent edge jitter.
    QVector<int> okBins(kBins, 0);
    QVector<int> blkBins(kBins, 0);
    QVector<qint64> rxBins(kBins, 0);
    QVector<qint64> txBins(kBins, 0);

    const qint64 curLiveBucket = (nowSec / kLiveStepSec) * kLiveStepSec;
    const qint64 liveWinEnd = curLiveBucket + kLiveStepSec;
    const qint64 liveWinStart = liveWinEnd - kBins * kLiveStepSec;

    qint64 minTs = 0;
    qint64 maxTs = 0;
    bool hasRecent = false;
    int filteredCount = 0;
    bool haveTs = false;
    for (const auto& s : samples) {
        if (internetOnly && !s.internet) continue;
        ++filteredCount;
        if (s.ts > 0) {
            if (!haveTs) {
                minTs = maxTs = s.ts;
                haveTs = true;
            } else {
                if (s.ts < minTs) minTs = s.ts;
                if (s.ts > maxTs) maxTs = s.ts;
            }
            if (s.ts >= liveWinStart) hasRecent = true;
        }
    }
    if (liveBwBins && !hasRecent) {
        for (auto it = liveBwBins->cbegin(); it != liveBwBins->cend(); ++it) {
            if (it.key() >= liveWinStart && (it.value().first > 0 || it.value().second > 0)) {
                hasRecent = true;
                break;
            }
        }
    }

    qint64 stepSec = kLiveStepSec;
    qint64 winEnd = liveWinEnd;
    qint64 winStart = liveWinStart;
    if (preferRecentWindow) {
        if (!hasRecent && haveTs) {
            stepSec = 60;
            winEnd = ((maxTs / stepSec) + 1) * stepSec;
            winStart = winEnd - kBins * stepSec;
        }
    } else if (haveTs) {
        const qint64 span = qMax<qint64>(1, maxTs - minTs);
        stepSec = qMax<qint64>(kLiveStepSec, ((span / kBins) / 10 + 1) * 10);
        if (stepSec > 20 && stepSec < 60) stepSec = 60;
        winEnd = ((maxTs / stepSec) + 1) * stepSec;
        winStart = winEnd - kBins * stepSec;
    }

    const qint64 spanSec = kBins * stepSec;
    const QString tsFmt = (spanSec <= 180)
        ? QStringLiteral("HH:mm:ss")
        : (spanSec > 36 * 3600 ? QStringLiteral("MM-dd HH:mm") : QStringLiteral("HH:mm"));

    int sampleIdx = 0;
    for (const auto& s : samples) {
        if (internetOnly && !s.internet) continue;
        int bin = -1;
        if (haveTs && s.ts > 0) {
            if (s.ts < winStart || s.ts > winEnd + stepSec) {
                ++sampleIdx;
                continue;
            }
            bin = qBound(0, static_cast<int>((s.ts - winStart) / stepSec), kBins - 1);
        } else {
            bin = qBound(0, kBins - 1 - (sampleIdx * kBins) / qMax(1, filteredCount), kBins - 1);
        }
        if (s.blocked) ++blkBins[bin];
        else ++okBins[bin];
        rxBins[bin] += s.rx;
        txBins[bin] += s.tx;
        ++sampleIdx;
    }

    if (liveBwBins) {
        for (auto it = liveBwBins->cbegin(); it != liveBwBins->cend(); ++it) {
            const qint64 bt = it.key();
            if (bt < winStart || bt >= winEnd) continue;
            const int bin = qBound(0, static_cast<int>((bt - winStart) / stepSec), kBins - 1);
            rxBins[bin] += it.value().first;
            txBins[bin] += it.value().second;
        }
    }

    QVariantList out;
    out.reserve(kBins);
    for (int i = 0; i < kBins; ++i) {
        const qint64 binTs = winStart + static_cast<qint64>(i) * stepSec;
        out.append(QVariantMap{
            {QStringLiteral("allowed"), okBins[i]},
            {QStringLiteral("blocked"), blkBins[i]},
            {QStringLiteral("total"), okBins[i] + blkBins[i]},
            {QStringLiteral("rxBytes"), rxBins[i]},
            {QStringLiteral("txBytes"), txBins[i]},
            {QStringLiteral("dataBytes"), rxBins[i] + txBins[i]},
            {QStringLiteral("ts"), binTs},
            {QStringLiteral("timeLabel"), QDateTime::fromSecsSinceEpoch(binTs).toString(tsFmt)},
        });
    }
    return out;
}
} // namespace

ConnectionListModel::ConnectionListModel(const std::deque<ConnRecord>* source, QObject* parent)
    : QAbstractListModel(parent), m_source(source) {}

int ConnectionListModel::rowCount(const QModelIndex& parent) const {
    if (parent.isValid()) return 0;
    return m_filtered.size();
}

QVariant ConnectionListModel::data(const QModelIndex& index, int role) const {
    if (!index.isValid() || !m_source || index.row() < 0 || index.row() >= m_filtered.size()) {
        return QVariant();
    }
    const int srcIdx = m_filtered.at(index.row());
    if (srcIdx < 0 || static_cast<size_t>(srcIdx) >= m_source->size()) return QVariant();
    const ConnRecord& c = (*m_source)[static_cast<size_t>(srcIdx)];
    switch (role) {
        case ConnIdRole: return c.id;
        case ProfileKeyRole: return c.profileKey;
        case TargetRole: return c.domain.isEmpty() ? c.ip : c.domain;
        case DomainRole: return c.domain;
        case IpRole: return c.ip;
        case PortRole: return c.port;
        case ProtoRole: return c.proto;
        case CountryRole: return c.country;
        case AppRole: return c.profileName;
        case ScopeRole: return c.scope;
        case DirectionRole: return c.direction;
        case VerdictRole: return c.verdict;
        case TunneledRole: return c.tunneled;
        case ReasonRole: return c.reason;
        case AsOwnerRole: return c.asOwner;
        case BytesRole: return c.bytesStr;
        case ActiveRole: return c.active;
        case InternalRole: return c.internal;
        case ConnTypeRole: return c.connType;
        default: return QVariant();
    }
}

QHash<int, QByteArray> ConnectionListModel::roleNames() const {
    static const QHash<int, QByteArray> roles{
        {ConnIdRole, "connId"},
        {ProfileKeyRole, "profileKey"},
        {TargetRole, "target"},
        {DomainRole, "domain"},
        {IpRole, "ip"},
        {PortRole, "port"},
        {ProtoRole, "proto"},
        {CountryRole, "country"},
        {AppRole, "app"},
        {ScopeRole, "scope"},
        {DirectionRole, "direction"},
        {VerdictRole, "verdict"},
        {TunneledRole, "tunneled"},
        {ReasonRole, "reason"},
        {AsOwnerRole, "asOwner"},
        {BytesRole, "bytes"},
        {ActiveRole, "active"},
        {InternalRole, "internal"},
        {ConnTypeRole, "connType"},
    };
    return roles;
}

void ConnectionListModel::setSearchQuery(const QString& q) {
    if (m_searchQuery == q) return;
    m_searchQuery = q;
    emit filterChanged();
    refilter(false);
}

void ConnectionListModel::setScopeInternet(bool v) {
    if (m_scopeInternet == v) return;
    m_scopeInternet = v;
    emit filterChanged();
    refilter(false);
}

void ConnectionListModel::setScopeLan(bool v) {
    if (m_scopeLan == v) return;
    m_scopeLan = v;
    emit filterChanged();
    refilter(false);
}

void ConnectionListModel::setScopeLocal(bool v) {
    if (m_scopeLocal == v) return;
    m_scopeLocal = v;
    emit filterChanged();
    refilter(false);
}

void ConnectionListModel::setDirFilter(const QString& v) {
    if (m_dirFilter == v) return;
    m_dirFilter = v;
    emit filterChanged();
    refilter(false);
}

void ConnectionListModel::setVerdictFilter(const QString& v) {
    if (m_verdictFilter == v) return;
    m_verdictFilter = v;
    emit filterChanged();
    refilter(false);
}

void ConnectionListModel::setHideDnsNoise(bool v) {
    if (m_hideDnsNoise == v) return;
    m_hideDnsNoise = v;
    emit filterChanged();
    refilter(false);
}

void ConnectionListModel::setHideLocalhost(bool v) {
    if (m_hideLocalhost == v) return;
    m_hideLocalhost = v;
    emit filterChanged();
    refilter(false);
}

void ConnectionListModel::syncFrom(int limit) {
    m_limit = limit;
    refilter(true);
}

void ConnectionListModel::refilter(bool preserveOnSameShape) {
    if (!m_source) return;
    const QString q = m_searchQuery.trimmed().toLower();
    const bool anyScope = m_scopeInternet || m_scopeLan || m_scopeLocal;
    const int cap = qMin(m_limit, static_cast<int>(m_source->size()));

    QVector<int> next;
    next.reserve(cap);
    for (int i = 0; i < cap; ++i) {
        const ConnRecord& c = (*m_source)[static_cast<size_t>(i)];
        if (!q.isEmpty()) {
            const QString target = c.domain.isEmpty() ? c.ip : c.domain;
            if (!target.contains(q, Qt::CaseInsensitive) &&
                !c.ip.contains(q, Qt::CaseInsensitive) &&
                !c.profileName.contains(q, Qt::CaseInsensitive) &&
                !c.country.contains(q, Qt::CaseInsensitive) &&
                !c.asOwner.contains(q, Qt::CaseInsensitive)) {
                continue;
            }
        }
        if (m_dirFilter != QLatin1String("all") && c.direction != m_dirFilter) continue;
        if (anyScope) {
            const bool hit = (m_scopeInternet && c.scope.contains(QLatin1String("internet")))
                          || (m_scopeLan && c.scope.contains(QLatin1String("lan")))
                          || (m_scopeLocal && c.scope.contains(QLatin1String("local")));
            if (!hit) continue;
        }
        if (m_verdictFilter != QLatin1String("all")) {
            const bool isBlk = (c.verdict == QLatin1String("block") || c.verdict == QLatin1String("drop") || c.verdict == QLatin1String("failed"));
            if (m_verdictFilter == QLatin1String("block") && !isBlk) continue;
            if (m_verdictFilter == QLatin1String("xray") && !c.tunneled) continue;
            if (m_verdictFilter == QLatin1String("permit") && isBlk) continue;
        }
        if (m_hideDnsNoise && isDnsNoiseConn(c)) continue;
        if (m_hideLocalhost && !m_scopeLocal && isLocalhostConn(c)) continue;
        next.append(i);
    }

    if (preserveOnSameShape && next.size() == m_filtered.size() && next == m_filtered) {
        if (!m_filtered.isEmpty()) {
            emit dataChanged(index(0), index(m_filtered.size() - 1));
        }
        return;
    }

    const bool countChangedFlag = (next.size() != m_filtered.size());
    beginResetModel();
    m_filtered = std::move(next);
    endResetModel();
    if (countChangedFlag) emit countChanged();
}

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

PortmasterBackend::PortmasterBackend(bool startBackground, bool withTray, bool demoMode, QObject* parent)
    : QObject(parent),
      m_demoMode(demoMode),
      m_uiWindowVisible(!startBackground || demoMode),
      m_api(new PortApiClient(this)),
      m_appsTimer(new QTimer(this)),
      m_appsTickTimer(new QTimer(this)),
      m_settingsTimer(new QTimer(this)),
      m_connsTimer(new QTimer(this)),
      m_chartTickTimer(new QTimer(this)),
      m_connModel(new ConnectionListModel(&m_connections, this)),
      m_withTray(withTray),
      m_iconStore(QSharedPointer<PortmasterIconStore>::create()),
      m_iconBatchTimer(new QTimer(this)),
      m_iconNotifyTimer(new QTimer(this)) {
    if (m_demoMode) {
        m_ru = false;
        m_hwAccel = true;
        m_onlyNowApps = false;
        m_dashboardChartInternetOnly = true;
        m_chartDataMode = false;
        m_historyLimit = 50000;
    } else {
        QSettings qs(QStringLiteral("portmaster"), QStringLiteral("qt-ui"));
        m_ru = qs.value(QStringLiteral("lang"), QStringLiteral("ru")).toString() == QLatin1String("ru");
        m_hwAccel = qs.value(QStringLiteral("hwAccel"), true).toBool();
        m_onlyNowApps = qs.value(QStringLiteral("onlyNowApps"), false).toBool();
        m_dashboardChartInternetOnly = qs.value(QStringLiteral("dashboardChartInternetOnly"), true).toBool();
        m_chartDataMode = qs.value(QStringLiteral("chartDataMode"), false).toBool();
        m_historyLimit = qBound(500, qs.value(QStringLiteral("historyLimit"), 2000).toInt(), 50000);
    }
    loadI18nCatalog();
    if (!m_demoMode) buildDesktopIconIndex();

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
    m_appsTickTimer->setInterval(2500);
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

    m_chartTickTimer->setInterval(10000);
    connect(m_chartTickTimer, &QTimer::timeout, this, [this]() {
        rebuildChartBinsOnly();
    });

    if (m_withTray) setupTray();
    if (m_demoMode) {
        loadDemoData();
        return;
    }
    m_chartTickTimer->start();
    connect(m_api, &PortApiClient::connectionChanged, this, &PortmasterBackend::onConnectionChanged);
    m_api->start();

    auto* poll = new QTimer(this);
    poll->setInterval(3000);
    connect(poll, &QTimer::timeout, this, &PortmasterBackend::refreshHttpData);
    poll->start();
}

PortmasterBackend::~PortmasterBackend() {
    if (m_iconBatchTimer) m_iconBatchTimer->stop();
    if (m_iconNotifyTimer) m_iconNotifyTimer->stop();
    if (m_appsTimer) m_appsTimer->stop();
    if (m_appsTickTimer) m_appsTickTimer->stop();
    if (m_settingsTimer) m_settingsTimer->stop();
    if (m_connsTimer) m_connsTimer->stop();
    if (m_chartTickTimer) m_chartTickTimer->stop();
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
    for (auto it = ru.constBegin(); it != ru.constEnd(); ++it) {
        m_i18nRu.insert(it.key(), it.value().toString());
    }
    for (auto it = en.constBegin(); it != en.constEnd(); ++it) {
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
    return formatBytesStatic(bytes);
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
    if (!m_demoMode) {
        QSettings qs(QStringLiteral("portmaster"), QStringLiteral("qt-ui"));
        qs.setValue(QStringLiteral("lang"), code);
    }
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

QIcon PortmasterBackend::loadTrayShieldIcon(const QString& colorName) const {
    bool useDarkGlyph = false;
    if (m_trayTheme == QLatin1String("light")) {
        useDarkGlyph = true;
    } else if (m_trayTheme == QLatin1String("dark")) {
        useDarkGlyph = false;
    } else {
#if QT_VERSION >= QT_VERSION_CHECK(6, 5, 0)
        if (const auto* hints = QGuiApplication::styleHints()) {
            useDarkGlyph = (hints->colorScheme() == Qt::ColorScheme::Light);
        }
#endif
    }
    const QString variant = useDarkGlyph ? QStringLiteral("dark") : QStringLiteral("light");
    const QString resPath = QStringLiteral(":/qt/qml/Portmaster/icons/pm_%1_%2_64.png").arg(variant, colorName);
    QIcon icon(resPath);
    if (!icon.isNull()) return icon;
    if (colorName == QLatin1String("blue")) return buildShieldIcon(QColor(0x4e, 0x97, 0xfa));
    if (colorName == QLatin1String("green")) return buildShieldIcon(QColor(0x1d, 0xe9, 0x66));
    if (colorName == QLatin1String("yellow")) return buildShieldIcon(QColor(0xe9, 0xd8, 0x1d));
    return buildShieldIcon(QColor(0xd1, 0x2e, 0x2e));
}

void PortmasterBackend::setupTray() {
    if (!QSystemTrayIcon::isSystemTrayAvailable()) return;
    QSettings uiPrefs(QStringLiteral("portmaster"), QStringLiteral("qt-ui"));
    m_trayTheme = uiPrefs.value(QStringLiteral("trayIconTheme"), QStringLiteral("system")).toString();

    m_tray = new QSystemTrayIcon(loadTrayShieldIcon(QStringLiteral("red")), this);
    m_trayMenu = new QMenu();

    m_trayStatusAct = m_trayMenu->addAction(QStringLiteral("Status: unknown"));
    m_trayStatusAct->setEnabled(false);
    m_trayMenu->addSeparator();

    m_trayPauseInfoAct = m_trayMenu->addAction(QStringLiteral("Portmaster is paused"));
    m_trayPauseInfoAct->setEnabled(false);
    m_trayPauseInfoAct->setVisible(false);
    m_trayResumeAct = m_trayMenu->addAction(QStringLiteral("Resume now"));
    connect(m_trayResumeAct, &QAction::triggered, this, &PortmasterBackend::resumeProtection);
    m_trayResumeAct->setVisible(false);

    m_trayPauseSubmenu = m_trayMenu->addMenu(QStringLiteral("Pause"));
    m_trayPauseVpn5Act = m_trayPauseSubmenu->addAction(QStringLiteral("Pause Xray VPN for 5 minutes"));
    connect(m_trayPauseVpn5Act, &QAction::triggered, this, [this]() { pauseProtection(5, true); });
    m_trayPauseVpn15Act = m_trayPauseSubmenu->addAction(QStringLiteral("Pause Xray VPN for 15 minutes"));
    connect(m_trayPauseVpn15Act, &QAction::triggered, this, [this]() { pauseProtection(15, true); });
    m_trayPauseVpn60Act = m_trayPauseSubmenu->addAction(QStringLiteral("Pause Xray VPN for 1 hour"));
    connect(m_trayPauseVpn60Act, &QAction::triggered, this, [this]() { pauseProtection(60, true); });
    m_trayPauseSubmenu->addSeparator();
    connect(m_trayPauseSubmenu->addAction(QStringLiteral("Pause for 5 minutes")), &QAction::triggered, this, [this]() { pauseProtection(5, false); });
    connect(m_trayPauseSubmenu->addAction(QStringLiteral("Pause for 15 minutes")), &QAction::triggered, this, [this]() { pauseProtection(15, false); });
    connect(m_trayPauseSubmenu->addAction(QStringLiteral("Pause for 1 hour")), &QAction::triggered, this, [this]() { pauseProtection(60, false); });
    m_trayMenu->addSeparator();

    m_trayVpnStatusAct = m_trayMenu->addAction(QStringLiteral("Xray VPN: Disconnected"));
    m_trayVpnStatusAct->setEnabled(false);
    m_trayVpnToggleAct = m_trayMenu->addAction(QStringLiteral("Connect Xray VPN"));
    connect(m_trayVpnToggleAct, &QAction::triggered, this, &PortmasterBackend::toggleXrayConnect);
    m_trayVpnAllAppsAct = m_trayMenu->addAction(QStringLiteral("Route All Apps via Xray VPN"));
    m_trayVpnAllAppsAct->setCheckable(true);
    connect(m_trayVpnAllAppsAct, &QAction::triggered, this, &PortmasterBackend::toggleGlobalVpn);
    m_trayMenu->addSeparator();

    auto* themeMenu = m_trayMenu->addMenu(QStringLiteral("Icon Theme"));
    m_trayThemeSysAct = themeMenu->addAction(QStringLiteral("System"));
    m_trayThemeLightAct = themeMenu->addAction(QStringLiteral("Light"));
    m_trayThemeDarkAct = themeMenu->addAction(QStringLiteral("Dark"));
    for (auto* a : {m_trayThemeSysAct, m_trayThemeLightAct, m_trayThemeDarkAct}) a->setCheckable(true);
    auto setTheme = [this](const QString& th) {
        m_trayTheme = th;
        QSettings prefs(QStringLiteral("portmaster"), QStringLiteral("qt-ui"));
        prefs.setValue(QStringLiteral("trayIconTheme"), th);
        updateTrayIcon();
    };
    connect(m_trayThemeSysAct, &QAction::triggered, this, [setTheme]() { setTheme(QStringLiteral("system")); });
    connect(m_trayThemeLightAct, &QAction::triggered, this, [setTheme]() { setTheme(QStringLiteral("light")); });
    connect(m_trayThemeDarkAct, &QAction::triggered, this, [setTheme]() { setTheme(QStringLiteral("dark")); });
    m_trayMenu->addSeparator();

    auto* showAct = m_trayMenu->addAction(QStringLiteral("Open Portmaster"));
    connect(showAct, &QAction::triggered, this, &PortmasterBackend::showWindowRequested);
    auto* openBrowserAct = m_trayMenu->addAction(QStringLiteral("Open in Web Browser"));
    connect(openBrowserAct, &QAction::triggered, this, [this]() {
        m_api->httpPost(QStringLiteral("/api/v1/core/api-port"), "{\"lease\":\"open\"}", QStringLiteral("application/json"), nullptr);
        QDesktopServices::openUrl(QUrl(QStringLiteral("http://127.0.0.1:817")));
    });
    m_trayMenu->addSeparator();

    auto* quitAct = m_trayMenu->addAction(QStringLiteral("Exit UI"));
    connect(quitAct, &QAction::triggered, qApp, &QApplication::quit);
    auto* shutAct = m_trayMenu->addAction(QStringLiteral("Shut Down Portmaster"));
    connect(shutAct, &QAction::triggered, this, &PortmasterBackend::shutdownCore);

    m_tray->setContextMenu(m_trayMenu);
    connect(m_tray, &QSystemTrayIcon::activated, this, [this](QSystemTrayIcon::ActivationReason r) {
        if (r == QSystemTrayIcon::Trigger || r == QSystemTrayIcon::DoubleClick) {
            emit showWindowRequested();
        }
    });
    updateTrayMenu();
    m_tray->show();
}

void PortmasterBackend::updateTrayMenu() {
    if (!m_tray || !m_trayMenu) return;
    const bool conn = m_api->isConnected();
    const bool xrayConn = m_xrayStatusMap.value(QStringLiteral("connected")).toBool();
    const bool xrayEnabled = effectiveValue(QStringLiteral("spn/enable")).toBool(false);
    const bool xrayConnecting = !xrayConn && xrayEnabled;

    if (m_trayStatusAct) {
        if (!conn) {
            m_trayStatusAct->setText(QStringLiteral("Status: unknown"));
        } else if (m_paused) {
            m_trayStatusAct->setText(QStringLiteral("Status: Secured (PAUSED)"));
        } else {
            m_trayStatusAct->setText(QStringLiteral("Status: Secured"));
        }
    }
    const bool anyPaused = m_paused || m_vpnPaused;
    if (m_trayPauseInfoAct) {
        if (m_paused && m_vpnPaused) m_trayPauseInfoAct->setText(QStringLiteral("Portmaster and Xray VPN are paused"));
        else if (m_paused) m_trayPauseInfoAct->setText(QStringLiteral("Portmaster is paused"));
        else if (m_vpnPaused) m_trayPauseInfoAct->setText(QStringLiteral("Xray VPN is paused"));
        m_trayPauseInfoAct->setVisible(anyPaused);
    }
    if (m_trayResumeAct) m_trayResumeAct->setVisible(anyPaused);

    const bool vpnPauseAvail = (xrayConn || xrayConnecting || m_vpnPaused) && !m_paused;
    for (auto* a : {m_trayPauseVpn5Act, m_trayPauseVpn15Act, m_trayPauseVpn60Act}) {
        if (a) a->setVisible(vpnPauseAvail);
    }

    if (m_trayVpnStatusAct && m_trayVpnToggleAct) {
        if (xrayConn) {
            const QString node = m_xrayStatusMap.value(QStringLiteral("nodeName")).toString();
            m_trayVpnStatusAct->setText(node.isEmpty() ? QStringLiteral("Xray VPN: Connected") : QStringLiteral("Xray VPN: Connected (%1)").arg(node));
            m_trayVpnToggleAct->setText(QStringLiteral("Disconnect Xray VPN"));
        } else if (xrayConnecting) {
            m_trayVpnStatusAct->setText(QStringLiteral("Xray VPN: Connecting..."));
            m_trayVpnToggleAct->setText(QStringLiteral("Cancel Xray VPN"));
        } else {
            m_trayVpnStatusAct->setText(QStringLiteral("Xray VPN: Disconnected"));
            m_trayVpnToggleAct->setText(QStringLiteral("Connect Xray VPN"));
        }
    }
    if (m_trayVpnAllAppsAct) {
        m_trayVpnAllAppsAct->setChecked(globalVpn());
    }
    if (m_trayThemeSysAct) m_trayThemeSysAct->setChecked(m_trayTheme == QLatin1String("system"));
    if (m_trayThemeLightAct) m_trayThemeLightAct->setChecked(m_trayTheme == QLatin1String("light"));
    if (m_trayThemeDarkAct) m_trayThemeDarkAct->setChecked(m_trayTheme == QLatin1String("dark"));
}

void PortmasterBackend::updateTrayIcon() {
    if (!m_tray) return;
    QString colorName = QStringLiteral("red");
    if (m_api->isConnected()) {
        if (m_paused) {
            colorName = QStringLiteral("yellow");
        } else {
            const bool xrayConn = m_xrayStatusMap.value(QStringLiteral("connected")).toBool();
            const bool xrayEnabled = effectiveValue(QStringLiteral("spn/enable")).toBool(false);
            colorName = (xrayConn || xrayEnabled) ? QStringLiteral("blue") : QStringLiteral("green");
        }
    }
    m_tray->setIcon(loadTrayShieldIcon(colorName));
    updateTrayMenu();
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
        ConnRecord rec = normalizeConnRecord(val.toObject(), true);
        const QString id = rec.id;
        if (!id.isEmpty()) {
            auto it = m_connSeqById.constFind(id);
            if (it != m_connSeqById.cend()) {
                const qint64 idx = m_connHeadSeq - it.value();
                if (idx >= 0 && static_cast<size_t>(idx) < m_connections.size()) {
                    const ConnRecord& prev = m_connections[static_cast<size_t>(idx)];
                    const qint64 drx = qMax<qint64>(0, rec.rx - prev.rx);
                    const qint64 dtx = qMax<qint64>(0, rec.tx - prev.tx);
                    rec.accountedLiveRx = qMin(rec.rx, prev.accountedLiveRx + drx);
                    rec.accountedLiveTx = qMin(rec.tx, prev.accountedLiveTx + dtx);
                    if (drx > 0 || dtx > 0) {
                        const qint64 bucketTs = (QDateTime::currentSecsSinceEpoch() / 20) * 20;
                        auto& slot = m_liveBwBins[bucketTs];
                        slot.first += drx;
                        slot.second += dtx;
                        if (rec.scope.contains(QLatin1String("internet"))) {
                            auto& islot = m_liveInetBwBins[bucketTs];
                            islot.first += drx;
                            islot.second += dtx;
                        }
                        if (m_liveBwBins.size() > 400) {
                            const qint64 cutoff = bucketTs - 7200;
                            for (auto bit = m_liveBwBins.begin(); bit != m_liveBwBins.end();) {
                                if (bit.key() < cutoff) bit = m_liveBwBins.erase(bit);
                                else ++bit;
                            }
                            for (auto bit = m_liveInetBwBins.begin(); bit != m_liveInetBwBins.end();) {
                                if (bit.key() < cutoff) bit = m_liveInetBwBins.erase(bit);
                                else ++bit;
                            }
                        }
                    }
                    m_connections[static_cast<size_t>(idx)] = std::move(rec);
                    scheduleConnsRebuild();
                    return;
                }
            }
        }
        ++m_connHeadSeq;
        m_connections.push_front(std::move(rec));
        if (!id.isEmpty()) m_connSeqById.insert(id, m_connHeadSeq);
        const size_t cap = static_cast<size_t>(qMax(500, m_historyLimit));
        while (m_connections.size() > cap) {
            const qint64 tailSeq = m_connHeadSeq - static_cast<qint64>(m_connections.size() - 1);
            const QString& tailId = m_connections.back().id;
            if (!tailId.isEmpty() && m_connSeqById.value(tailId) == tailSeq) {
                m_connSeqById.remove(tailId);
            }
            m_connections.pop_back();
        }
        scheduleConnsRebuild();
    });

    m_api->qsub(QStringLiteral("query runtime:modules/core/event/restart-ui-process"), [](const QString& op, const QString&, const QJsonValue&) {
        if (op == QLatin1String("new") || op == QLatin1String("upd")) {
            QStringList args = QCoreApplication::arguments().mid(1);
            if (QProcess::startDetached(QCoreApplication::applicationFilePath(), args)) {
                qApp->quit();
            }
        }
    });
}

void PortmasterBackend::refreshHttpData() {
    if (m_demoMode || !m_api->isConnected()) return;
    const bool uiVis = m_uiWindowVisible;
    const bool needTrayStatus = hasTray();
    if (!uiVis && !needTrayStatus) return;

    // Poll /api/v1/ui/flavors and auth/geodata every ~15s (5 x 3s) to avoid frequent scans.
    if (uiVis && (m_uiPollTick % 5) == 0) {
        m_api->httpGet(QStringLiteral("/api/v1/ui/flavors"), [this](int code, const QByteArray& body, const QString&) {
            if (code == 200 && body != m_lastUiReportBytes) {
                m_lastUiReportBytes = body;
                m_uiReportMap = QJsonDocument::fromJson(body).object().toVariantMap();
                emit uiReportChanged();
            }
        });
        refreshAdminLock();
        if (m_geodataHints.isEmpty()) fetchGeodataHints();
    }
    ++m_uiPollTick;

    m_api->httpGet(QStringLiteral("/api/v1/xray/status"), [this](int code, const QByteArray& body, const QString&) {
        if (code == 200 && body != m_lastXrayStatusBytes) {
            m_lastXrayStatusBytes = body;
            QVariantMap st = QJsonDocument::fromJson(body).object().toVariantMap();
            if (!st.contains(QStringLiteral("nodeName"))) st.insert(QStringLiteral("nodeName"), st.value(QStringLiteral("node_name")));
            if (!st.contains(QStringLiteral("exitCountry"))) st.insert(QStringLiteral("exitCountry"), st.value(QStringLiteral("country")));
            if (!st.contains(QStringLiteral("inboundAddress"))) st.insert(QStringLiteral("inboundAddress"), st.value(QStringLiteral("inbound_address")));
            m_xrayStatusMap = std::move(st);
            rebuildMapPins();
            emit xrayChanged();
            updateTrayIcon();
        }
    });
    if (!uiVis) return;

    m_api->httpGet(QStringLiteral("/api/v1/xray/nodes"), [this](int code, const QByteArray& body, const QString&) {
        if (code == 200 && body != m_lastXrayNodesBytes) {
            m_lastXrayNodesBytes = body;
            QVariantList out;
            const QJsonArray arr = QJsonDocument::fromJson(body).array();
            out.reserve(arr.size());
            for (const auto& v : arr) {
                if (v.isObject()) out.append(v.toObject().toVariantMap());
            }
            m_xrayNodesList = std::move(out);
            rebuildMapPins();
            emit xrayChanged();
        }
    });
    m_api->httpGet(QStringLiteral("/api/v1/xray/subscriptions"), [this](int code, const QByteArray& body, const QString&) {
        if (code == 200 && body != m_lastXraySubsBytes) {
            m_lastXraySubsBytes = body;
            QVariantList out;
            const QJsonArray arr = QJsonDocument::fromJson(body).array();
            out.reserve(arr.size());
            for (const auto& v : arr) {
                if (v.isObject()) {
                    QVariantMap m = v.toObject().toVariantMap();
                    m.insert(QStringLiteral("name"), QUrl::fromPercentEncoding(m.value(QStringLiteral("name")).toString().toUtf8()));
                    out.append(std::move(m));
                }
            }
            m_xraySubsList = std::move(out);
            emit xrayChanged();
        }
    });
    if (!m_historyLoaded && !m_historyFetching) {
        loadHistoryFromDb(false);
    }
}

void PortmasterBackend::loadHistoryFromDb(bool forceReload) {
    if (m_demoMode || !m_api->isConnected()) return;
    if (m_historyFetching && !forceReload) return;
    m_historyFetching = true;
    ++m_historyFetchGen;
    emit historyConfigChanged();
    QVector<ConnRecord> acc;
    acc.reserve(qMin(m_historyLimit, 2000));
    fetchHistoryPage(m_historyFetchGen, QStringLiteral("main"), 0, m_historyLimit, std::move(acc));
}

void PortmasterBackend::fetchHistoryPage(int gen, const QString& db, int page, int targetLimit, QVector<ConnRecord> acc) {
    if (gen != m_historyFetchGen || !m_api->isConnected()) {
        if (gen == m_historyFetchGen) {
            m_historyFetching = false;
            emit historyConfigChanged();
        }
        return;
    }
    const int pageSize = 1000;
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
            const bool isSessionDb = (db == QLatin1String("main"));
            const QJsonArray arr = QJsonDocument::fromJson(body).object().value(QStringLiteral("results")).toArray();
            for (const auto& v : arr) {
                if (v.isObject()) {
                    acc.append(normalizeConnRecord(v.toObject(), isSessionDb));
                }
            }
            if (!arr.isEmpty() && page == 0) {
                self->mergeDbConnections(acc);
            }
            if (arr.size() >= 1000 && acc.size() < targetLimit) {
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

void PortmasterBackend::mergeDbConnections(const QVector<ConnRecord>& rows) {
    QHash<QString, int> idxById;
    idxById.reserve(static_cast<int>(m_connections.size()) + rows.size());
    for (size_t i = 0; i < m_connections.size(); ++i) {
        const QString& id = m_connections[i].id;
        if (!id.isEmpty()) idxById.insert(id, static_cast<int>(i));
    }
    for (const auto& r : rows) {
        if (r.id.isEmpty()) {
            m_connections.push_back(r);
            continue;
        }
        auto it = idxById.constFind(r.id);
        if (it == idxById.constEnd()) {
            idxById.insert(r.id, static_cast<int>(m_connections.size()));
            m_connections.push_back(r);
        }
    }
    std::stable_sort(m_connections.begin(), m_connections.end(), [](const ConnRecord& a, const ConnRecord& b) {
        return a.startedTs > b.startedTs;
    });
    const int cap = qMax(250, m_historyLimit);
    if (m_connections.size() > static_cast<size_t>(cap)) {
        m_connections.resize(static_cast<size_t>(cap));
        m_connections.shrink_to_fit();
    }
    m_connSeqById.clear();
    m_connHeadSeq = static_cast<qint64>(m_connections.size());
    m_connSeqById.reserve(static_cast<int>(m_connections.size()));
    for (size_t i = 0; i < m_connections.size(); ++i) {
        const QString& id = m_connections[i].id;
        if (!id.isEmpty()) m_connSeqById.insert(id, m_connHeadSeq - static_cast<qint64>(i));
    }
    rebuildConnectionsCache();
    scheduleAppsTickRebuild();
    if (!m_selectedProfileKey.isEmpty()) rebuildSelectedAppCache();
}

void PortmasterBackend::fetchWorldMap() {
    QFile resFile(QStringLiteral(":/qt/qml/Portmaster/world-50m.json"));
    if (resFile.open(QIODevice::ReadOnly)) {
        const QByteArray bytes = resFile.readAll();
        if (!bytes.isEmpty()) {
            parseWorldMapJson(bytes);
            if (m_worldLoaded) return;
        }
    }
    if (m_demoMode) return;
    m_api->httpGet(QStringLiteral("/assets/world-50m.json"), [this](int code, const QByteArray& body, const QString&) {
        if (code == 200) parseWorldMapJson(body);
    });
}

void PortmasterBackend::parseWorldMapJson(const QByteArray& body) {
    const QJsonObject root = QJsonDocument::fromJson(body).object();
    const QJsonObject tr = root.value(QStringLiteral("transform")).toObject();
    const QJsonArray sc = tr.value(QStringLiteral("scale")).toArray();
    const QJsonArray tl = tr.value(QStringLiteral("translate")).toArray();
    if (sc.size() < 2 || tl.size() < 2) return;
    const double sx = sc[0].toDouble(), sy = sc[1].toDouble();
    const double tx = tl[0].toDouble(), ty = tl[1].toDouble();

    const QJsonArray arcs = root.value(QStringLiteral("arcs")).toArray();

    auto decodeRing = [&](const QJsonArray& ring, double& bMinX, double& bMinY, double& bMaxX, double& bMaxY) -> QVariantList {
        QVariantList out;
        QVariantList polyPts;
        double prevLon = 0.0;
        bool hasPrev = false;

        auto flushPoly = [&]() {
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

    auto decodeRings = [&](const QJsonArray& rings, double& bMinX, double& bMinY, double& bMaxX, double& bMaxY) -> QVariantList {
        // Decode every ring (outer boundary and interior holes such as Lesotho inside ZA)
        // so complex boundaries never lose enclaves or split coastlines.
        QVariantList allParts;
        for (const auto& rv : rings) {
            const QVariantList parts = decodeRing(rv.toArray(), bMinX, bMinY, bMaxX, bMaxY);
            for (const auto& p : parts) allParts.append(p);
        }
        return allParts;
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
        const QString hitKey = (iso.isEmpty() || iso == QLatin1String("-99"))
            ? (QStringLiteral("_N_") + name)
            : iso;
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
                {QStringLiteral("hitKey"), hitKey},
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
    m_worldHitIndex.clear();
    m_worldHitIndex.reserve(countries.size());
    for (const QVariant& cv : countries) {
        const QVariantMap cm = cv.toMap();
        WorldCountryHit entry;
        entry.code = cm.value(QStringLiteral("code")).toString();
        entry.hitKey = cm.value(QStringLiteral("hitKey")).toString();
        entry.name = cm.value(QStringLiteral("name")).toString();
        entry.minX = cm.value(QStringLiteral("minX")).toDouble();
        entry.maxX = cm.value(QStringLiteral("maxX")).toDouble();
        entry.minY = cm.value(QStringLiteral("minY")).toDouble();
        entry.maxY = cm.value(QStringLiteral("maxY")).toDouble();
        const QVariantList polys = cm.value(QStringLiteral("polygons")).toList();
        entry.polygons.reserve(polys.size());
        for (const QVariant& pv : polys) {
            const QVariantList ringList = pv.toList();
            QVector<QVector<double>> rings;
            if (!ringList.isEmpty() && ringList.first().metaType().id() != QMetaType::QVariantList) {
                QVector<double> pts;
                pts.reserve(ringList.size());
                for (const QVariant& x : ringList) pts.append(x.toDouble());
                rings.append(std::move(pts));
            } else {
                rings.reserve(ringList.size());
                for (const QVariant& rv : ringList) {
                    const QVariantList ptsList = rv.toList();
                    QVector<double> pts;
                    pts.reserve(ptsList.size());
                    for (const QVariant& x : ptsList) pts.append(x.toDouble());
                    rings.append(std::move(pts));
                }
            }
            entry.polygons.append(std::move(rings));
        }
        m_worldHitIndex.append(std::move(entry));
    }
    m_worldLoaded = true;
    emit worldMapReady();
}

QVariantMap PortmasterBackend::hitTestMapCountry(double nx, double ny) const {
    for (const auto& c : m_worldHitIndex) {
        if (nx < c.minX || nx > c.maxX || ny < c.minY || ny > c.maxY) continue;
        for (const auto& polyRings : c.polygons) {
            int hits = 0;
            for (const auto& pts : polyRings) {
                const int n = pts.size();
                if (n < 6) continue;
                bool inside = false;
                for (int i = 0, j = n - 2; i < n; j = i, i += 2) {
                    const double xi = pts[i], yi = pts[i + 1];
                    const double xj = pts[j], yj = pts[j + 1];
                    const double dy = yj - yi;
                    const bool intersect = ((yi > ny) != (yj > ny)) &&
                        (nx < (xj - xi) * (ny - yi) / (std::abs(dy) < 1e-12 ? 1e-12 : dy) + xi);
                    if (intersect) inside = !inside;
                }
                if (inside) ++hits;
            }
            if ((hits % 2) == 1) {
                return QVariantMap{
                    {QStringLiteral("code"), c.code},
                    {QStringLiteral("hitKey"), c.hitKey},
                    {QStringLiteral("name"), c.name},
                };
            }
        }
    }
    return QVariantMap{};
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

void PortmasterBackend::setOnlyNowApps(bool onlyNow) {
    if (m_onlyNowApps == onlyNow) return;
    m_onlyNowApps = onlyNow;
    if (!m_demoMode) QSettings(QStringLiteral("portmaster"), QStringLiteral("qt-ui")).setValue(QStringLiteral("onlyNowApps"), m_onlyNowApps);
    rebuildAppsCache();
}

void PortmasterBackend::setDashboardChartInternetOnly(bool internetOnly) {
    if (m_dashboardChartInternetOnly == internetOnly) return;
    m_dashboardChartInternetOnly = internetOnly;
    if (!m_demoMode) QSettings(QStringLiteral("portmaster"), QStringLiteral("qt-ui")).setValue(QStringLiteral("dashboardChartInternetOnly"), m_dashboardChartInternetOnly);
    rebuildConnectionsCache();
}

void PortmasterBackend::setChartDataMode(bool dataMode) {
    if (m_chartDataMode == dataMode) return;
    m_chartDataMode = dataMode;
    if (!m_demoMode) QSettings(QStringLiteral("portmaster"), QStringLiteral("qt-ui")).setValue(QStringLiteral("chartDataMode"), m_chartDataMode);
    emit chartDataModeChanged();
}

void PortmasterBackend::setAppChartInternetOnly(bool internetOnly) {
    if (m_appChartInternetOnly == internetOnly) return;
    m_appChartInternetOnly = internetOnly;
    if (!m_selectedProfileKey.isEmpty()) rebuildSelectedAppCache();
}

void PortmasterBackend::setUiWindowVisible(bool visible) {
    if (m_uiWindowVisible == visible) return;
    m_uiWindowVisible = visible;
    if (visible) {
        if (m_appsDirty) {
            m_appsDirty = false;
            rebuildAppsCache();
        }
        if (m_connsDirty) {
            m_connsDirty = false;
            rebuildConnectionsCache();
        } else if (!m_demoMode) {
            rebuildChartBinsOnly();
        }
        if (!m_demoMode && m_api->isConnected()) {
            refreshHttpData();
        }
    }
}

void PortmasterBackend::rebuildAppsCache() {
    if (!m_uiWindowVisible) {
        m_appsDirty = true;
        return;
    }
    m_appsDirty = false;
    bool gVpn = globalVpn();
    bool gSplit = globalSplit();
    m_activeAppsCache.clear();
    m_offlineAppsCache.clear();
    m_allAppsCache.clear();

    // Single linear pass over connections so we avoid O(profiles * connections) scans.
    struct ConnAgg {
        int total = 0;
        int sessionTotal = 0;
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
        const QString& cp = c.profile;
        const QString bareCp = cp.contains('/') ? cp.section('/', -1) : cp;
        const QString pName = c.profileName.toLower();
        const qint64 rxVal = c.rx;
        const qint64 txVal = c.tx;
        const bool isAlive = c.active;
        const bool inSession = isAlive || c.session;
        const QString& vd = c.verdict;
        const bool isBlk = (vd == QLatin1String("block") || vd == QLatin1String("drop") || vd == QLatin1String("failed"));

        auto accum = [&](ConnAgg& a) {
            ++a.total;
            if (inSession) ++a.sessionTotal;
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
        int totalSeen = 0, sessionSeen = 0, blockedSeen = 0;

        auto idIt = aggById.constFind(id);
        if (idIt != aggById.constEnd()) {
            totalSeen = idIt->total;
            sessionSeen = idIt->sessionTotal;
            blockedSeen = idIt->blocked;
            liveActive += idIt->alive;
            rx = qMax(rx, idIt->maxRx);
            tx = qMax(tx, idIt->maxTx);
        } else if (!name.isEmpty()) {
            auto nIt = aggByName.constFind(name.toLower());
            if (nIt != aggByName.constEnd()) {
                totalSeen = nIt->total;
                sessionSeen = nIt->sessionTotal;
                blockedSeen = nIt->blocked;
                liveActive += nIt->alive;
                rx = qMax(rx, nIt->maxRx);
                tx = qMax(tx, nIt->maxTx);
            }
        }
        const bool isSessionActive = liveActive > 0 || sessionSeen > 0;
        const bool showInActive = m_onlyNowApps ? (liveActive > 0) : isSessionActive;

        const bool hasIcon = hasCachedIcon(key);
        if (!hasIcon) queueIconResolve(key, showInActive);
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
            {QStringLiteral("sessionConns"), sessionSeen},
            {QStringLiteral("totalConns"), totalSeen},
            {QStringLiteral("blockedConns"), blockedSeen},
            {QStringLiteral("rxStr"), formatBytes(rx)},
            {QStringLiteral("txStr"), formatBytes(tx)},
        };
        m_allAppsCache.append(item);
        if (showInActive) m_activeAppsCache.append(item);
        else m_offlineAppsCache.append(item);
    }
    std::sort(m_activeAppsCache.begin(), m_activeAppsCache.end(), [](const QVariant& a, const QVariant& b) {
        const auto ma = a.toMap();
        const auto mb = b.toMap();
        int aa = ma.value(QStringLiteral("alive")).toInt();
        int ab = mb.value(QStringLiteral("alive")).toInt();
        if ((aa > 0) != (ab > 0)) return aa > ab;
        if (aa != ab) return aa > ab;
        int sa = ma.value(QStringLiteral("sessionConns")).toInt();
        int sb = mb.value(QStringLiteral("sessionConns")).toInt();
        if (sa != sb) return sa > sb;
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
    m_selectedAppChartBins.clear();
    m_selectedAppInetAllowed = 0;
    m_selectedAppInetBlocked = 0;
    m_selectedAppAllAllowed = 0;
    m_selectedAppAllBlocked = 0;
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
    qint64 inetRx = 0, inetTx = 0;
    int alive = 0, blocked = 0;
    QVector<ChartSample> chartSamples;
    chartSamples.reserve(256);
    for (const auto& c : m_connections) {
        const QString& cp = c.profile;
        if (cp == id || cp.endsWith('/' + id) || (!name.isEmpty() && c.profileName.compare(name, Qt::CaseInsensitive) == 0)) {
            rx += c.rx;
            tx += c.tx;
            if (c.active) ++alive;
            const QString& vd = c.verdict;
            const bool isBlk = (vd == QLatin1String("block") || vd == QLatin1String("drop") || vd == QLatin1String("failed"));
            if (vd == QLatin1String("block") || vd == QLatin1String("drop")) ++blocked;
            const bool isInet = c.scope.contains(QLatin1String("internet"));
            if (isInet) { inetRx += c.rx; inetTx += c.tx; }
            if (isBlk) { ++m_selectedAppAllBlocked; if (isInet) ++m_selectedAppInetBlocked; }
            else       { ++m_selectedAppAllAllowed; if (isInet) ++m_selectedAppInetAllowed; }
            if (chartSamples.size() < 1500) {
                chartSamples.append({c.startedTs, isBlk, isInet, c.rx, c.tx});
            }
            if (m_selectedAppConns.size() < 1000) {
                m_selectedAppConns.append(QVariantMap{
                    {QStringLiteral("id"), c.id},
                    {QStringLiteral("domain"), c.domain.isEmpty() ? c.ip : c.domain},
                    {QStringLiteral("ip"), c.ip},
                    {QStringLiteral("port"), c.port},
                    {QStringLiteral("proto"), c.proto},
                    {QStringLiteral("country"), c.country},
                    {QStringLiteral("verdict"), vd},
                    {QStringLiteral("tunneled"), c.tunneled},
                    {QStringLiteral("reason"), c.reason},
                    {QStringLiteral("bytes"), c.bytesStr},
                });
            }
        }
    }

    m_selectedAppChartBins = buildChartBinsHelper(
        chartSamples, m_appChartInternetOnly, false, QDateTime::currentSecsSinceEpoch());

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
        {QStringLiteral("rxInetStr"), formatBytes(inetRx)},
        {QStringLiteral("txInetStr"), formatBytes(inetTx)},
        {QStringLiteral("fpCount"), p.value(QStringLiteral("Fingerprints")).toArray().size()},
    };
    emit selectedAppChanged();
}

void PortmasterBackend::rebuildConnectionsCache() {
    if (!m_uiWindowVisible) {
        m_connsDirty = true;
        return;
    }
    m_connsDirty = false;
    const int limit = qMin(qMax(500, m_historyLimit), static_cast<int>(m_connections.size()));

    int allowedTotal = 0;
    int blockedTotal = 0;
    int inetAllowedTotal = 0;
    int inetBlockedTotal = 0;
    int internetBlockedTotal = 0;
    int activeTotal = 0;
    int tunneledTotal = 0;
    qint64 totalRx = 0;
    qint64 totalTx = 0;
    qint64 inetRx = 0;
    qint64 inetTx = 0;

    const qint64 nowSec = QDateTime::currentSecsSinceEpoch();
    QVector<ChartSample> chartSamples;
    chartSamples.reserve(qMin(static_cast<int>(m_connections.size()), 1500));

    QHash<QString, int> countryCounts;
    QHash<QString, int> blockedAppCounts;
    QHash<QString, QString> blockedAppNames;

    for (size_t i = 0; i < m_connections.size(); ++i) {
        const ConnRecord& c = m_connections[i];
        const QString& vd = c.verdict;
        const bool isBlk = (vd == QLatin1String("block") || vd == QLatin1String("drop") || vd == QLatin1String("failed"));
        const bool isInet = c.scope.contains(QLatin1String("internet"));
        const bool isActive = c.active;
        const bool isTun = c.tunneled;
        const qint64 rx = c.rx;
        const qint64 tx = c.tx;
        const QString& cc = c.country;
        const QString& profKey = c.profileKey;
        const QString& profName = c.profileName;

        if (isBlk) {
            ++blockedTotal;
            if (isInet) {
                ++internetBlockedTotal;
                ++inetBlockedTotal;
            }
            if (!profKey.isEmpty()) {
                blockedAppCounts[profKey] += 1;
                if (!profName.isEmpty()) blockedAppNames[profKey] = profName;
            }
        } else {
            ++allowedTotal;
            if (isInet) ++inetAllowedTotal;
        }
        if (isActive) ++activeTotal;
        if (isTun) ++tunneledTotal;
        totalRx += rx;
        totalTx += tx;
        if (isInet) {
            inetRx += rx;
            inetTx += tx;
        }

        if (!isBlk && !cc.isEmpty() && cc != QLatin1String("-99")) {
            countryCounts[cc] += 1;
        }

        if (chartSamples.size() < 5000) {
            const qint64 baseRx = qMax<qint64>(0, rx - c.accountedLiveRx);
            const qint64 baseTx = qMax<qint64>(0, tx - c.accountedLiveTx);
            chartSamples.append({c.startedTs, isBlk, isInet, baseRx, baseTx});
        }
    }

    m_connAllowedTotal = allowedTotal;
    m_connBlockedTotal = blockedTotal;
    m_connInetAllowedTotal = inetAllowedTotal;
    m_connInetBlockedTotal = inetBlockedTotal;
    m_connInternetBlockedTotal = internetBlockedTotal;
    m_connActiveTotal = activeTotal;
    m_connTunneledTotal = tunneledTotal;
    m_connRxStr = formatBytes(totalRx);
    m_connTxStr = formatBytes(totalTx);
    m_connInetRxStr = formatBytes(inetRx);
    m_connInetTxStr = formatBytes(inetTx);

    const auto& liveBw = m_dashboardChartInternetOnly ? m_liveInetBwBins : m_liveBwBins;
    m_connChartBins = buildChartBinsHelper(chartSamples, m_dashboardChartInternetOnly, true, nowSec, &liveBw);

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

    m_connModel->syncFrom(limit);
    emit connectionsChanged();
}

void PortmasterBackend::rebuildChartBinsOnly() {
    if (!m_uiWindowVisible || m_demoMode) return;
    const qint64 nowSec = QDateTime::currentSecsSinceEpoch();
    QVector<ChartSample> chartSamples;
    chartSamples.reserve(qMin(static_cast<int>(m_connections.size()), 5000));
    for (const auto& c : m_connections) {
        if (chartSamples.size() >= 5000) break;
        const QString& vd = c.verdict;
        const bool isBlk = (vd == QLatin1String("block") || vd == QLatin1String("drop") || vd == QLatin1String("failed"));
        const bool isInet = c.scope.contains(QLatin1String("internet"));
        const qint64 baseRx = qMax<qint64>(0, c.rx - c.accountedLiveRx);
        const qint64 baseTx = qMax<qint64>(0, c.tx - c.accountedLiveTx);
        chartSamples.append({c.startedTs, isBlk, isInet, baseRx, baseTx});
    }
    const auto& liveBw = m_dashboardChartInternetOnly ? m_liveInetBwBins : m_liveBwBins;
    m_connChartBins = buildChartBinsHelper(chartSamples, m_dashboardChartInternetOnly, true, nowSec, &liveBw);
    emit connectionsChanged();
    if (!m_selectedProfileKey.isEmpty()) rebuildSelectedAppCache();
}

QVariantMap PortmasterBackend::buildSettingMap(const QJsonObject& s, const QJsonObject* appCfg) const {
    const QString key = s.value(QStringLiteral("Key")).toString();
    const int optType = s.value(QStringLiteral("OptType")).toInt(1);
    const QJsonObject ann = s.value(QStringLiteral("Annotations")).toObject();
    const int order = ann.value(QStringLiteral("safing/portbase:ui:order")).toInt(9999);
    const QString category = ann.value(QStringLiteral("safing/portbase:ui:category")).toString();
    const bool settablePerApp = ann.value(QStringLiteral("safing/portmaster:settable-per-app")).toBool(false)
        || key == QLatin1String("dns/nameservers");

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
        if (it.key() == QLatin1String("core/releaseLevel") || it.key() == QLatin1String("core/metrics/comment")) {
            continue;
        }
        m_settingsCache.append(buildSettingMap(it.value(), nullptr));
    }
    sortSettingsByOrder(m_settingsCache);
    emit settingsChanged();
    updateTrayMenu();
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
    updateTrayIcon();
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
    if (!m_demoMode) m_api->sendPutFrame(QStringLiteral("update"), scopedKey, p);
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
    if (!m_demoMode) m_api->sendPutFrame(QStringLiteral("update"), scopedKey, p);
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
    const QString trimmed = rule.trimmed();
    if (!m_profiles.contains(scopedKey) || trimmed.isEmpty()) return;
    QJsonObject p = m_profiles.value(scopedKey);
    QJsonObject cfg = p.value(QStringLiteral("Config")).toObject();
    QJsonArray arr = readProfileCfg(cfg, QStringLiteral("filter/endpoints")).toArray();

    QJsonArray next;
    next.append(trimmed);

    QStringList dropList{trimmed};
    if (trimmed.startsWith(QLatin1String("+ ")) || trimmed.startsWith(QLatin1String("- "))) {
        const QString sign = trimmed.left(1);
        const QString oppSign = (sign == QLatin1String("+")) ? QStringLiteral("-") : QStringLiteral("+");
        const QString target = trimmed.mid(2).trimmed();
        if (!target.isEmpty()) {
            dropList.append(oppSign + QStringLiteral(" ") + target);
            const QString bare = target.startsWith('.') ? target.mid(1) : QString();
            const QString dotted = target.startsWith('.') ? QString() : (QStringLiteral(".") + target);
            if (!bare.isEmpty()) {
                dropList.append(sign + QStringLiteral(" ") + bare);
                dropList.append(oppSign + QStringLiteral(" ") + bare);
            }
            if (!dotted.isEmpty()) {
                dropList.append(sign + QStringLiteral(" ") + dotted);
                dropList.append(oppSign + QStringLiteral(" ") + dotted);
            }
        }
    }

    for (const auto& v : arr) {
        const QString cur = v.toString().trimmed();
        if (!cur.isEmpty() && !dropList.contains(cur)) {
            next.append(cur);
        }
    }
    setAppSetting(scopedKey, QStringLiteral("filter/endpoints"), next.toVariantList());
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
    for (auto& c : m_connections) {
        if (c.id != connId) continue;
        QString dbKey = !c.profileKey.isEmpty()
                            ? c.profileKey
                            : (c.profile.startsWith(QLatin1String("core:"))
                                   ? c.profile
                                   : QStringLiteral("core:profiles/%1").arg(c.profile.contains('/') ? c.profile : QStringLiteral("local/") + c.profile));
        QString dom = c.domain.trimmed();
        while (dom.startsWith('.')) dom.remove(0, 1);
        while (dom.endsWith('.')) dom.chop(1);
        const QString ip = c.ip.trimmed();
        const bool domIsIp = !dom.isEmpty() && !QHostAddress(dom).isNull();
        const QString pat = (dom.isEmpty() || domIsIp) ? (ip.isEmpty() ? dom : ip) : (QStringLiteral(".") + dom);
        if (pat.isEmpty()) return;

        const QString rule = QStringLiteral("%1 %2").arg(allow ? QStringLiteral("+") : QStringLiteral("-"), pat);
        if (m_demoMode) {
            c.verdict = allow ? QStringLiteral("permit") : QStringLiteral("block");
            c.reason = allow ? QStringLiteral("Permitted by demo rule") : QStringLiteral("Blocked by demo rule");
            addAppRule(dbKey, rule);
            rebuildConnectionsCache();
            rebuildAppsCache();
            if (!m_selectedProfileKey.isEmpty()) rebuildSelectedAppCache();
            return;
        }
        if (m_profiles.contains(dbKey)) {
            addAppRule(dbKey, rule);
            emit toastMessage(
                allow ? (m_ru ? QStringLiteral("Правило разрешено") : QStringLiteral("Rule permitted"))
                      : (m_ru ? QStringLiteral("Правило заблокировано") : QStringLiteral("Rule blocked")),
                rule,
                false);
        }
        return;
    }
}

void PortmasterBackend::clearHistory() {
    if (m_demoMode) {
        m_connections.clear();
        m_connSeqById.clear();
        m_liveBwBins.clear();
        m_liveInetBwBins.clear();
        m_connHeadSeq = 0;
        m_historyLoaded = true;
        rebuildConnectionsCache();
        rebuildAppsCache();
        return;
    }
    m_api->httpPost(QStringLiteral("/api/v1/netquery/history/clear"), "{}", QStringLiteral("application/json"), [this](int code, const QByteArray&, const QString&) {
        if (code >= 200 && code < 300) {
            m_connections.clear();
            m_connSeqById.clear();
            m_liveBwBins.clear();
            m_liveInetBwBins.clear();
            m_connHeadSeq = 0;
            m_historyLoaded = true;
            rebuildConnectionsCache();
            scheduleAppsTickRebuild();
        }
    });
}

void PortmasterBackend::setHistoryLimit(int limit) {
    const int clamped = qBound(500, limit, 50000);
    if (m_historyLimit == clamped) return;
    m_historyLimit = clamped;
    QSettings qs(QStringLiteral("portmaster"), QStringLiteral("qt-ui"));
    qs.setValue(QStringLiteral("historyLimit"), m_historyLimit);
    emit historyConfigChanged();
    if (static_cast<int>(m_connections.size()) > m_historyLimit) {
        m_connections.resize(static_cast<size_t>(m_historyLimit));
        m_connections.shrink_to_fit();
        m_connSeqById.clear();
        m_connHeadSeq = static_cast<qint64>(m_connections.size());
        m_connSeqById.reserve(static_cast<int>(m_connections.size()));
        for (size_t i = 0; i < m_connections.size(); ++i) {
            const QString& cid = m_connections[i].id;
            if (!cid.isEmpty()) m_connSeqById.insert(cid, m_connHeadSeq - static_cast<qint64>(i));
        }
        rebuildConnectionsCache();
    } else if (!m_demoMode) {
        loadHistoryFromDb(true);
    } else {
        rebuildConnectionsCache();
    }
}

void PortmasterBackend::refreshHistory() {
    if (!m_demoMode) loadHistoryFromDb(true);
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
    static const QHash<QString, QString> fallbackNames{
        {QStringLiteral("SG"), QStringLiteral("Singapore")},
        {QStringLiteral("HK"), QStringLiteral("Hong Kong")},
        {QStringLiteral("MO"), QStringLiteral("Macao")},
        {QStringLiteral("MC"), QStringLiteral("Monaco")},
        {QStringLiteral("MT"), QStringLiteral("Malta")},
        {QStringLiteral("BH"), QStringLiteral("Bahrain")},
        {QStringLiteral("LI"), QStringLiteral("Liechtenstein")},
        {QStringLiteral("AD"), QStringLiteral("Andorra")},
        {QStringLiteral("SM"), QStringLiteral("San Marino")},
        {QStringLiteral("GI"), QStringLiteral("Gibraltar")},
        {QStringLiteral("VA"), QStringLiteral("Vatican City")},
        {QStringLiteral("SC"), QStringLiteral("Seychelles")},
        {QStringLiteral("MU"), QStringLiteral("Mauritius")},
        {QStringLiteral("MV"), QStringLiteral("Maldives")},
    };
    return fallbackNames.value(up, up);
}

void PortmasterBackend::toggleGlobalVpn() {
    updateSetting(QStringLiteral("spn/use"), !globalVpn());
}

void PortmasterBackend::resetAllAppOverrides() {
    m_api->httpPost(QStringLiteral("/api/v1/profiles/reset-routing"), "{}", QStringLiteral("application/json"), nullptr);
}

void PortmasterBackend::toggleXrayConnect() {
    bool conn = m_xrayStatusMap.value(QStringLiteral("connected")).toBool();
    if (m_demoMode) {
        m_xrayStatusMap.insert(QStringLiteral("connected"), !conn);
        if (!conn && m_xrayStatusMap.value(QStringLiteral("nodeName")).toString().isEmpty() && !m_xrayNodesList.isEmpty()) {
            const QVariantMap first = m_xrayNodesList.first().toMap();
            m_xrayStatusMap.insert(QStringLiteral("activeNodeId"), first.value(QStringLiteral("id")));
            m_xrayStatusMap.insert(QStringLiteral("nodeName"), first.value(QStringLiteral("name")));
            m_xrayStatusMap.insert(QStringLiteral("exitCountry"), first.value(QStringLiteral("country")));
            m_xrayStatusMap.insert(QStringLiteral("latencyMs"), first.value(QStringLiteral("latencyMs")));
        }
        rebuildMapPins();
        emit xrayChanged();
        updateTrayIcon();
        return;
    }
    const QString ep = conn ? QStringLiteral("/api/v1/xray/disconnect") : QStringLiteral("/api/v1/xray/connect-last");
    m_api->httpPost(ep, "{}", QStringLiteral("application/json"), [this](int, const QByteArray&, const QString&) {
        refreshHttpData();
    });
}

void PortmasterBackend::connectNode(const QString& nodeId) {
    if (m_demoMode) {
        for (const QVariant& nv : m_xrayNodesList) {
            const QVariantMap m = nv.toMap();
            if (m.value(QStringLiteral("id")).toString() == nodeId) {
                m_xrayStatusMap.insert(QStringLiteral("connected"), true);
                m_xrayStatusMap.insert(QStringLiteral("activeNodeId"), nodeId);
                m_xrayStatusMap.insert(QStringLiteral("nodeName"), m.value(QStringLiteral("name")));
                m_xrayStatusMap.insert(QStringLiteral("exitCountry"), m.value(QStringLiteral("country")));
                m_xrayStatusMap.insert(QStringLiteral("latencyMs"), m.value(QStringLiteral("latencyMs")));
                break;
            }
        }
        rebuildMapPins();
        emit xrayChanged();
        updateTrayIcon();
        return;
    }
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
        emit toastMessage(QStringLiteral("Xray"),
            m_ru ? (code == 200 ? QStringLiteral("xray-rust скачан/обновлён.") : QStringLiteral("Не удалось скачать xray-rust."))
                 : (code == 200 ? QStringLiteral("xray-rust downloaded/updated.") : QStringLiteral("Failed to download xray-rust.")),
            code != 200);
        refreshHttpData();
    });
}

void PortmasterBackend::downloadGeodata() {
    m_api->httpPost(QStringLiteral("/api/v1/xray/download-geodata"), "{}", QStringLiteral("application/json"), [this](int code, const QByteArray&, const QString&) {
        emit toastMessage(QStringLiteral("GeoIP & GeoSite"),
            m_ru ? (code == 200 ? QStringLiteral("geoip.dat и geosite.dat обновлены.") : QStringLiteral("Не удалось обновить гео-базы."))
                 : (code == 200 ? QStringLiteral("geoip.dat & geosite.dat updated.") : QStringLiteral("Failed to update geodata.")),
            code != 200);
        fetchGeodataHints();
        refreshHttpData();
    });
}

void PortmasterBackend::fetchGeodataHints() {
    m_api->httpGet(QStringLiteral("/api/v1/xray/geodata/hints"), [this](int code, const QByteArray& body, const QString&) {
        if (code == 200) {
            m_geodataHints = QJsonDocument::fromJson(body).object().toVariantMap();
            emit geodataHintsChanged();
        }
    });
}

void PortmasterBackend::openNodeEditor(const QString& nodeId) {
    if (nodeId.isEmpty()) return;
    if (m_demoMode) {
        openDemoNodeEditor(nodeId);
        return;
    }
    const QString ep = QStringLiteral("/api/v1/xray/nodes/") + QString::fromUtf8(QUrl::toPercentEncoding(nodeId));
    m_api->httpGet(ep, [this](int code, const QByteArray& body, const QString& err) {
        if (code == 200) {
            m_editingNode = QJsonDocument::fromJson(body).object().toVariantMap();
            emit editingNodeChanged();
        } else {
            emit toastMessage(QStringLiteral("Xray"),
                err.isEmpty() ? (m_ru ? QStringLiteral("Не удалось открыть параметры узла") : QStringLiteral("Failed to open node settings")) : err,
                true);
        }
    });
}

void PortmasterBackend::clearEditingNode() {
    m_editingNode.clear();
    emit editingNodeChanged();
}

void PortmasterBackend::saveNodeEdit(const QString& nodeId, const QVariantMap& fields) {
    if (nodeId.isEmpty()) return;
    if (m_demoMode) {
        for (int i = 0; i < m_xrayNodesList.size(); ++i) {
            QVariantMap m = m_xrayNodesList[i].toMap();
            if (m.value(QStringLiteral("id")).toString() == nodeId) {
                for (auto it = fields.cbegin(); it != fields.cend(); ++it) {
                    m.insert(it.key(), it.value());
                }
                m_xrayNodesList[i] = m;
                break;
            }
        }
        m_editingNode.clear();
        emit editingNodeChanged();
        emit xrayChanged();
        emit toastMessage(QStringLiteral("Xray"), m_ru ? QStringLiteral("Узел и правила сохранены.") : QStringLiteral("Node updated."), false);
        return;
    }
    const QString ep = QStringLiteral("/api/v1/xray/nodes/") + QString::fromUtf8(QUrl::toPercentEncoding(nodeId));
    const QByteArray payload = QJsonDocument(QJsonObject::fromVariantMap(fields)).toJson(QJsonDocument::Compact);
    m_api->httpPut(ep, payload, QStringLiteral("application/json"), [this](int code, const QByteArray& body, const QString&) {
        if (code == 200) {
            m_editingNode.clear();
            emit editingNodeChanged();
            emit toastMessage(QStringLiteral("Xray"), m_ru ? QStringLiteral("Узел и правила сохранены.") : QStringLiteral("Node updated."), false);
            refreshHttpData();
        } else {
            const QString msg = QJsonDocument::fromJson(body).object().value(QStringLiteral("error")).toString(QString::fromUtf8(body));
            emit toastMessage(QStringLiteral("Xray"),
                msg.isEmpty() ? (m_ru ? QStringLiteral("Ошибка сохранения узла") : QStringLiteral("Failed to save node")) : msg,
                true);
        }
    });
}

void PortmasterBackend::deleteNode(const QString& nodeId) {
    if (nodeId.isEmpty()) return;
    const QString ep = QStringLiteral("/api/v1/xray/nodes/") + QString::fromUtf8(QUrl::toPercentEncoding(nodeId));
    m_api->httpDelete(ep, [this](int code, const QByteArray&, const QString&) {
        if (code == 200) {
            m_editingNode.clear();
            emit editingNodeChanged();
            emit toastMessage(QStringLiteral("Xray"), m_ru ? QStringLiteral("Узел удалён.") : QStringLiteral("Node deleted."), false);
            refreshHttpData();
        }
    });
}

void PortmasterBackend::addGlobalRule(const QString& rule) {
    const QString trimmed = rule.trimmed();
    if (trimmed.isEmpty()) return;
    applyPreset(QStringLiteral("xray/rules"), {trimmed}, true);
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
    if (!m_demoMode) m_api->sendPutFrame(QStringLiteral("update"), QStringLiteral("config:") + key, payload);
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
    if (!m_demoMode) m_api->sendPutFrame(QStringLiteral("update"), QStringLiteral("config:") + key, payload);
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

void PortmasterBackend::installUiFlavor(const QString& flavorId) {
    if (m_uiBusy) return;
    m_uiBusy = true;
    emit uiReportChanged();
    QByteArray body = QJsonDocument(QJsonObject{{QStringLiteral("flavor"), flavorId}}).toJson();
    m_api->httpPost(QStringLiteral("/api/v1/ui/install"), body, QStringLiteral("application/json"), [this, flavorId](int code, const QByteArray& resp, const QString&) {
        m_uiBusy = false;
        if (code == 200) {
            m_uiReportMap = QJsonDocument::fromJson(resp).object().toVariantMap();
            emit toastMessage(QStringLiteral("UI"), m_ru ? (flavorId + QStringLiteral(" установлен.")) : (flavorId + QStringLiteral(" installed.")), false);
        } else {
            const QString msg = QString::fromUtf8(resp).trimmed();
            emit toastMessage(QStringLiteral("UI"),
                msg.isEmpty() ? (m_ru ? QStringLiteral("Ошибка скачивания UI") : QStringLiteral("Failed to download UI")) : msg,
                true);
        }
        emit uiReportChanged();
    });
}

void PortmasterBackend::uninstallUiFlavor(const QString& flavorId) {
    if (m_uiBusy) return;
    m_uiBusy = true;
    emit uiReportChanged();
    QByteArray body = QJsonDocument(QJsonObject{{QStringLiteral("flavor"), flavorId}}).toJson();
    m_api->httpPost(QStringLiteral("/api/v1/ui/uninstall"), body, QStringLiteral("application/json"), [this, flavorId](int code, const QByteArray& resp, const QString&) {
        m_uiBusy = false;
        if (code == 200) {
            m_uiReportMap = QJsonDocument::fromJson(resp).object().toVariantMap();
            emit toastMessage(QStringLiteral("UI"), m_ru ? (flavorId + QStringLiteral(" удалён.")) : (flavorId + QStringLiteral(" uninstalled.")), false);
        }
        emit uiReportChanged();
    });
}

void PortmasterBackend::refreshAdminLock() {
    m_api->httpGet(QStringLiteral("/api/v1/auth/status"), [this](int code, const QByteArray& body, const QString&) {
        if (code == 200 && body != m_lastAdminLockBytes) {
            m_lastAdminLockBytes = body;
            m_adminLock = QJsonDocument::fromJson(body).object().toVariantMap();
            emit adminLockChanged();
        }
    });
}

void PortmasterBackend::unlockAdmin(const QString& password) {
    QByteArray body = QJsonDocument(QJsonObject{{QStringLiteral("password"), password}}).toJson(QJsonDocument::Compact);
    m_api->httpPost(QStringLiteral("/api/v1/auth/unlock"), body, QStringLiteral("application/json"), [this](int code, const QByteArray& resp, const QString&) {
        QJsonObject obj = QJsonDocument::fromJson(resp).object();
        if (code == 200) {
            m_api->setAdminToken(obj.value(QStringLiteral("token")).toString());
            refreshAdminLock();
            emit toastMessage(QStringLiteral("Admin Lock"), m_ru ? QStringLiteral("Разблокировано (сессия 1 ч).") : QStringLiteral("Unlocked for session."), false);
        } else {
            emit toastMessage(QStringLiteral("Admin Lock"), obj.value(QStringLiteral("error")).toString(m_ru ? QStringLiteral("Неверный пароль") : QStringLiteral("Incorrect password")), true);
        }
    });
}

void PortmasterBackend::lockAdmin() {
    m_api->setAdminToken(QString());
    m_api->httpPost(QStringLiteral("/api/v1/auth/lock"), "{}", QStringLiteral("application/json"), [this](int code, const QByteArray& body, const QString&) {
        if (code == 200) {
            m_lastAdminLockBytes = body;
            m_adminLock = QJsonDocument::fromJson(body).object().toVariantMap();
            emit adminLockChanged();
        } else {
            refreshAdminLock();
        }
    });
}

void PortmasterBackend::configureAdminLock(const QVariantMap& payload) {
    QByteArray body = QJsonDocument(QJsonObject::fromVariantMap(payload)).toJson(QJsonDocument::Compact);
    m_api->httpPost(QStringLiteral("/api/v1/auth/set-password"), body, QStringLiteral("application/json"), [this](int code, const QByteArray& resp, const QString&) {
        QJsonObject obj = QJsonDocument::fromJson(resp).object();
        if (code == 200) {
            const QString tok = obj.value(QStringLiteral("token")).toString();
            m_api->setAdminToken(tok);
            m_lastAdminLockBytes.clear();
            m_adminLock = obj.toVariantMap();
            emit adminLockChanged();
            emit toastMessage(QStringLiteral("Admin Lock"),
                obj.value(QStringLiteral("passwordSet")).toBool()
                    ? (m_ru ? QStringLiteral("Пароль администратора сохранён.") : QStringLiteral("Admin password saved."))
                    : (m_ru ? QStringLiteral("Пароль администратора снят.") : QStringLiteral("Admin password removed.")),
                false);
        } else {
            emit toastMessage(QStringLiteral("Admin Lock"), obj.value(QStringLiteral("error")).toString(QString::fromUtf8(resp)), true);
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
    if (m_demoMode) return;
    m_api->httpGet(QStringLiteral("/api/v1/sync/settings/export?from=core:config/global"), [this](int code, const QByteArray& body, const QString&) {
        if (code == 200) {
            m_modalTitle = m_ru ? QStringLiteral("Экспорт глобальных настроек") : QStringLiteral("Export Global Settings");
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

void PortmasterBackend::enableDemoMode() {
    if (m_demoMode) return;
    m_demoMode = true;
    loadDemoData();
    emit connectionChanged();
}

void PortmasterBackend::openDemoNodeEditor(const QString& nodeId) {
    QVariantMap found;
    for (const QVariant& nv : m_xrayNodesList) {
        const QVariantMap m = nv.toMap();
        if (m.value(QStringLiteral("id")).toString() == nodeId || (nodeId.isEmpty() && found.isEmpty())) {
            found = m;
            if (!nodeId.isEmpty()) break;
        }
    }
    if (found.isEmpty()) return;
    const QString proto = found.value(QStringLiteral("protocol"), QStringLiteral("vless")).toString();
    m_editingNode = QVariantMap{
        {QStringLiteral("id"), found.value(QStringLiteral("id"))},
        {QStringLiteral("name"), found.value(QStringLiteral("name"))},
        {QStringLiteral("protocol"), proto},
        {QStringLiteral("address"), QStringLiteral("203.0.113.10")},
        {QStringLiteral("port"), 443},
        {QStringLiteral("uuid"), QStringLiteral("00000000-0000-4000-8000-000000000001")},
        {QStringLiteral("password"), QString()},
        {QStringLiteral("method"), QStringLiteral("chacha20-ietf-poly1305")},
        {QStringLiteral("encryption"), QStringLiteral("none")},
        {QStringLiteral("security"), QStringLiteral("reality")},
        {QStringLiteral("network"), QStringLiteral("xhttp")},
        {QStringLiteral("sni"), QStringLiteral("cdn.example.org")},
        {QStringLiteral("host"), QStringLiteral("edge.example.org")},
        {QStringLiteral("path"), QStringLiteral("/xhttp-stream")},
        {QStringLiteral("flow"), QStringLiteral("xtls-rprx-vision")},
        {QStringLiteral("serviceName"), QStringLiteral("tunnel.rpc")},
        {QStringLiteral("headerType"), QStringLiteral("none")},
        {QStringLiteral("xhttpMode"), QStringLiteral("packet-up")},
        {QStringLiteral("xhttpExtra"), QStringLiteral("{\"noGRPCHeader\":false,\"scMaxEachPostBytes\":1000000}")},
        {QStringLiteral("grpcMultiMode"), false},
        {QStringLiteral("grpcAuthority"), QString()},
        {QStringLiteral("kcpSeed"), QString()},
        {QStringLiteral("wsHeartbeatPeriod"), 15},
        {QStringLiteral("fingerprint"), QStringLiteral("chrome")},
        {QStringLiteral("alpn"), QStringList{QStringLiteral("h2"), QStringLiteral("http/1.1")}},
        {QStringLiteral("realityPublicKey"), QStringLiteral("DemoPublicKey00000000000000000000000000000000")},
        {QStringLiteral("realityShortId"), QStringLiteral("0123abcd")},
        {QStringLiteral("realitySpiderX"), QStringLiteral("/")},
        {QStringLiteral("pinnedPeerCert"), QString()},
        {QStringLiteral("verifyPeerCertByName"), QString()},
        {QStringLiteral("echConfigList"), QString()},
        {QStringLiteral("muxEnabled"), false},
        {QStringLiteral("muxConcurrency"), 8},
        {QStringLiteral("fragmentEnabled"), true},
        {QStringLiteral("tcpFastOpen"), true},
        {QStringLiteral("tcpKeepAliveInterval"), 30},
        {QStringLiteral("headers"), QVariantMap{{QStringLiteral("User-Agent"), QStringLiteral("Mozilla/5.0")}}},
        {QStringLiteral("plugin"), QString()},
        {QStringLiteral("pluginOpts"), QString()},
        {QStringLiteral("hyObfs"), QStringLiteral("salamander")},
        {QStringLiteral("hyObfsPassword"), QStringLiteral("demo-obfs-key")},
        {QStringLiteral("hyPin"), QString()},
        {QStringLiteral("hyMport"), QStringLiteral("20000-30000")},
        {QStringLiteral("wgPublicKey"), QString()},
        {QStringLiteral("wgPreSharedKey"), QString()},
        {QStringLiteral("wgAddresses"), QStringList{QStringLiteral("10.0.0.2/32")}},
        {QStringLiteral("wgDns"), QStringList{QStringLiteral("1.1.1.1")}},
        {QStringLiteral("wgMtu"), 1420},
        {QStringLiteral("wgKeepAlive"), 25},
        {QStringLiteral("rulePreset"), QStringLiteral("split_ru")},
        {QStringLiteral("routingRules"), QStringList{
            QStringLiteral("direct:geosite:private"),
            QStringLiteral("direct:geoip:private"),
            QStringLiteral("block:geosite:category-ads-all"),
            QStringLiteral("tunnel:example.org"),
        }},
    };
    emit editingNodeChanged();
}

void PortmasterBackend::loadDemoData() {
    m_demoMode = true;
    m_historyLoaded = true;
    m_historyFetching = false;
    fetchWorldMap();

    m_settings.clear();
    auto addSetting = [this](const QString& key, const QString& name, const QString& desc, const QString& cat,
                             int order, int optType, int expertise, const QJsonValue& defVal, const QJsonValue& val,
                             bool perApp, const QJsonArray& choices = {}) {
        QJsonObject s{
            {QStringLiteral("Key"), key},
            {QStringLiteral("Name"), name},
            {QStringLiteral("Description"), desc},
            {QStringLiteral("OptType"), optType},
            {QStringLiteral("ExpertiseLevel"), expertise},
            {QStringLiteral("DefaultValue"), defVal},
            {QStringLiteral("Annotations"), QJsonObject{
                {QStringLiteral("safing/portbase:ui:order"), order},
                {QStringLiteral("safing/portbase:ui:category"), cat},
                {QStringLiteral("safing/portmaster:settable-per-app"), perApp},
            }},
        };
        if (!val.isUndefined() && !val.isNull()) s.insert(QStringLiteral("Value"), val);
        if (!choices.isEmpty()) s.insert(QStringLiteral("PossibleValues"), choices);
        m_settings.insert(key, s);
    };

    // OptType: 1=String, 2=StringArray, 3=Int, 4=Bool
    addSetting(QStringLiteral("filter/defaultAction"), QStringLiteral("Default Action"), QStringLiteral("Default action for unhandled network connections."), QStringLiteral("Filter"), 10, 1, 0, QStringLiteral("permit"), QStringLiteral("permit"), true,
               QJsonArray{QJsonObject{{QStringLiteral("Name"), QStringLiteral("Allow")}, {QStringLiteral("Value"), QStringLiteral("permit")}},
                          QJsonObject{{QStringLiteral("Name"), QStringLiteral("Block")}, {QStringLiteral("Value"), QStringLiteral("block")}}});
    addSetting(QStringLiteral("filter/blockInternet"), QStringLiteral("Block Internet"), QStringLiteral("Block outgoing Internet access globally or per app."), QStringLiteral("Filter"), 20, 4, 0, false, false, true);
    addSetting(QStringLiteral("filter/blockLAN"), QStringLiteral("Block LAN"), QStringLiteral("Block local area network traffic."), QStringLiteral("Filter"), 30, 4, 0, false, false, true);
    addSetting(QStringLiteral("filter/blockP2P"), QStringLiteral("Block P2P"), QStringLiteral("Block peer-to-peer connections."), QStringLiteral("Filter"), 40, 4, 0, false, false, true);
    addSetting(QStringLiteral("filter/blockInbound"), QStringLiteral("Block Inbound"), QStringLiteral("Block incoming unsolicited traffic."), QStringLiteral("Filter"), 50, 4, 0, true, true, true);
    addSetting(QStringLiteral("filter/lists"), QStringLiteral("Filter Lists"), QStringLiteral("Enabled threat and tracker blocklists."), QStringLiteral("Filter"), 60, 2, 1, QJsonArray{QStringLiteral("TRAC"), QStringLiteral("MAL"), QStringLiteral("BAD")}, QJsonArray{QStringLiteral("TRAC"), QStringLiteral("MAL")}, true);
    addSetting(QStringLiteral("spn/enable"), QStringLiteral("Enable Xray Tunnel"), QStringLiteral("Route selected or all internet traffic through Xray."), QStringLiteral("Tunnel"), 10, 4, 0, false, true, false);
    addSetting(QStringLiteral("spn/use"), QStringLiteral("Use Xray Routing"), QStringLiteral("Send traffic through active proxy tunnel by default."), QStringLiteral("Tunnel"), 20, 4, 0, false, true, true);
    addSetting(QStringLiteral("splittun/use"), QStringLiteral("Split Tunneling"), QStringLiteral("Allow bypassing tunnel for direct apps or routes."), QStringLiteral("Tunnel"), 30, 4, 1, true, true, true);
    addSetting(QStringLiteral("dns/nameservers"), QStringLiteral("DNS Servers"), QStringLiteral("Configured encrypted DNS upstream resolvers."), QStringLiteral("DNS"), 10, 2, 0, QJsonArray{QStringLiteral("dot://dns.quad9.net")}, QJsonArray{QStringLiteral("dot://dns.quad9.net"), QStringLiteral("doh://cloudflare-dns.com/dns-query")}, true);
    addSetting(QStringLiteral("core/locale"), QStringLiteral("Language"), QStringLiteral("Interface language."), QStringLiteral("General"), 5, 1, 0, QStringLiteral("en"), QStringLiteral("en"), false);
    addSetting(QStringLiteral("xray/rules"), QStringLiteral("Global Xray Rules"), QStringLiteral("Custom global routing directives for Xray."), QStringLiteral("Xray"), 40, 2, 1, QJsonArray{}, QJsonArray{QStringLiteral("direct:geosite:private"), QStringLiteral("block:geosite:category-ads-all")}, false);

    m_profiles.clear();
    struct DemoApp { const char* id; const char* name; const char* path; bool vpn; bool blkNet; bool internal; };
    const DemoApp demoApps[] = {
        {"browser-demo",    "Firefox Demo",      "/usr/lib/firefox/firefox",         true,  false, false},
        {"code-studio",     "Code Studio",       "/usr/bin/code-studio",             false, false, false},
        {"cloud-sync",      "Cloud Sync Client", "/usr/bin/cloud-sync",              true,  false, false},
        {"media-player",    "Media Streamer",    "/usr/bin/media-streamer",          false, false, false},
        {"terminal-ssh",    "Terminal Client",   "/usr/bin/terminal-client",         true,  false, false},
        {"package-updater", "System Updater",    "/usr/libexec/system-updater",      false, false, true},
    };
    for (const auto& a : demoApps) {
        const QString id = QString::fromLatin1(a.id);
        const QString key = QStringLiteral("core:profiles/local/") + id;
        QJsonObject cfg{
            {QStringLiteral("spn"), QJsonObject{{QStringLiteral("use"), a.vpn}}},
            {QStringLiteral("filter"), QJsonObject{
                {QStringLiteral("defaultAction"), QStringLiteral("permit")},
                {QStringLiteral("blockInternet"), a.blkNet},
                {QStringLiteral("endpoints"), id == QLatin1String("browser-demo")
                    ? QJsonArray{QStringLiteral("+ .example.org"), QStringLiteral("- .tracker.example")}
                    : QJsonArray{}},
            }},
        };
        m_profiles.insert(key, QJsonObject{
            {QStringLiteral("ID"), id},
            {QStringLiteral("Source"), QStringLiteral("local")},
            {QStringLiteral("Name"), QString::fromLatin1(a.name)},
            {QStringLiteral("LinkedPath"), QString::fromLatin1(a.path)},
            {QStringLiteral("Internal"), a.internal},
            {QStringLiteral("Config"), cfg},
            {QStringLiteral("Fingerprints"), QJsonArray{QStringLiteral("sha256:demo")}},
        });
    }

    m_xrayNodesList = QVariantList{
        QVariantMap{{QStringLiteral("id"), QStringLiteral("nl-01")}, {QStringLiteral("name"), QStringLiteral("Netherlands Amsterdam #1")}, {QStringLiteral("protocol"), QStringLiteral("vless")},       {QStringLiteral("address"), QStringLiteral("203.0.113.10")}, {QStringLiteral("port"), 443}, {QStringLiteral("country"), QStringLiteral("NL")}, {QStringLiteral("latencyMs"), 24},  {QStringLiteral("network"), QStringLiteral("xhttp")}, {QStringLiteral("security"), QStringLiteral("reality")}},
        QVariantMap{{QStringLiteral("id"), QStringLiteral("de-01")}, {QStringLiteral("name"), QStringLiteral("Germany Frankfurt #2")},   {QStringLiteral("protocol"), QStringLiteral("vless")},       {QStringLiteral("address"), QStringLiteral("203.0.113.11")}, {QStringLiteral("port"), 443}, {QStringLiteral("country"), QStringLiteral("DE")}, {QStringLiteral("latencyMs"), 31},  {QStringLiteral("network"), QStringLiteral("raw")},   {QStringLiteral("security"), QStringLiteral("reality")}},
        QVariantMap{{QStringLiteral("id"), QStringLiteral("fi-01")}, {QStringLiteral("name"), QStringLiteral("Finland Helsinki #1")},    {QStringLiteral("protocol"), QStringLiteral("hysteria2")},   {QStringLiteral("address"), QStringLiteral("198.51.100.12")}, {QStringLiteral("port"), 8443}, {QStringLiteral("country"), QStringLiteral("FI")}, {QStringLiteral("latencyMs"), 19},  {QStringLiteral("network"), QStringLiteral("raw")},   {QStringLiteral("security"), QStringLiteral("tls")}},
        QVariantMap{{QStringLiteral("id"), QStringLiteral("ch-01")}, {QStringLiteral("name"), QStringLiteral("Switzerland Zurich #1")},  {QStringLiteral("protocol"), QStringLiteral("trojan")},      {QStringLiteral("address"), QStringLiteral("198.51.100.13")}, {QStringLiteral("port"), 443}, {QStringLiteral("country"), QStringLiteral("CH")}, {QStringLiteral("latencyMs"), 38},  {QStringLiteral("network"), QStringLiteral("grpc")},  {QStringLiteral("security"), QStringLiteral("tls")}},
        QVariantMap{{QStringLiteral("id"), QStringLiteral("gb-01")}, {QStringLiteral("name"), QStringLiteral("UK London #1")},           {QStringLiteral("protocol"), QStringLiteral("vmess")},       {QStringLiteral("address"), QStringLiteral("203.0.113.14")}, {QStringLiteral("port"), 443}, {QStringLiteral("country"), QStringLiteral("GB")}, {QStringLiteral("latencyMs"), 44},  {QStringLiteral("network"), QStringLiteral("ws")},    {QStringLiteral("security"), QStringLiteral("tls")}},
        QVariantMap{{QStringLiteral("id"), QStringLiteral("us-01")}, {QStringLiteral("name"), QStringLiteral("USA New York #1")},        {QStringLiteral("protocol"), QStringLiteral("vless")},       {QStringLiteral("address"), QStringLiteral("192.0.2.15")},   {QStringLiteral("port"), 443}, {QStringLiteral("country"), QStringLiteral("US")}, {QStringLiteral("latencyMs"), 96},  {QStringLiteral("network"), QStringLiteral("xhttp")}, {QStringLiteral("security"), QStringLiteral("reality")}},
        QVariantMap{{QStringLiteral("id"), QStringLiteral("jp-01")}, {QStringLiteral("name"), QStringLiteral("Japan Tokyo #1")},         {QStringLiteral("protocol"), QStringLiteral("shadowsocks")}, {QStringLiteral("address"), QStringLiteral("192.0.2.16")},   {QStringLiteral("port"), 8388},{QStringLiteral("country"), QStringLiteral("JP")}, {QStringLiteral("latencyMs"), 118}, {QStringLiteral("network"), QStringLiteral("raw")},   {QStringLiteral("security"), QStringLiteral("none")}},
        QVariantMap{{QStringLiteral("id"), QStringLiteral("sg-01")}, {QStringLiteral("name"), QStringLiteral("Singapore #1")},           {QStringLiteral("protocol"), QStringLiteral("wireguard")},   {QStringLiteral("address"), QStringLiteral("198.51.100.17")}, {QStringLiteral("port"), 51820},{QStringLiteral("country"), QStringLiteral("SG")}, {QStringLiteral("latencyMs"), 142}, {QStringLiteral("network"), QStringLiteral("raw")},   {QStringLiteral("security"), QStringLiteral("none")}},
    };
    m_xraySubsList = QVariantList{
        QVariantMap{{QStringLiteral("id"), QStringLiteral("sub-demo-1")}, {QStringLiteral("name"), QStringLiteral("Example Edge Cluster")}, {QStringLiteral("url"), QStringLiteral("https://sub.example.org/demo-profile")}, {QStringLiteral("node_count"), 8}, {QStringLiteral("nodeCount"), 8}},
    };
    m_xrayStatusMap = QVariantMap{
        {QStringLiteral("connected"), true},
        {QStringLiteral("activeNodeId"), QStringLiteral("nl-01")},
        {QStringLiteral("nodeName"), QStringLiteral("Netherlands Amsterdam #1")},
        {QStringLiteral("exitCountry"), QStringLiteral("NL")},
        {QStringLiteral("exitIp"), QStringLiteral("203.0.113.10")},
        {QStringLiteral("latencyMs"), 24},
        {QStringLiteral("binaryInstalled"), true},
        {QStringLiteral("geoipAvailable"), true},
        {QStringLiteral("geositeAvailable"), true},
        {QStringLiteral("inboundAddress"), QStringLiteral("127.0.0.1:10808")},
    };
    m_geodataHints = QVariantMap{
        {QStringLiteral("installed"), true},
        {QStringLiteral("geoipCategories"), QStringList{QStringLiteral("private"), QStringLiteral("us"), QStringLiteral("nl"), QStringLiteral("de")}},
        {QStringLiteral("geositeCategories"), QStringList{QStringLiteral("private"), QStringLiteral("category-ads-all"), QStringLiteral("geolocation-!cn")}},
    };
    m_uiReportMap = QVariantMap{
        {QStringLiteral("activeFlavor"), QStringLiteral("qt")},
        {QStringLiteral("hwAccel"), m_hwAccel},
        {QStringLiteral("apiPortListening"), false},
        {QStringLiteral("keepApiPort"), false},
        {QStringLiteral("unixSocketPath"), QStringLiteral("/run/portmaster/api.sock")},
        {QStringLiteral("flavors"), QVariantList{
            QVariantMap{{QStringLiteral("id"), QStringLiteral("qt")},      {QStringLiteral("name"), QStringLiteral("Qt Native UI")},    {QStringLiteral("active"), true},  {QStringLiteral("running"), true},  {QStringLiteral("installed"), true}, {QStringLiteral("liveRamMb"), 42.4}, {QStringLiteral("processCount"), 1}},
            QVariantMap{{QStringLiteral("id"), QStringLiteral("angular")}, {QStringLiteral("name"), QStringLiteral("Tauri / Angular")}, {QStringLiteral("active"), false}, {QStringLiteral("running"), false}, {QStringLiteral("installed"), true}, {QStringLiteral("benchmarkRamMb"), 385.0}},
            QVariantMap{{QStringLiteral("id"), QStringLiteral("web")},     {QStringLiteral("name"), QStringLiteral("Web (:817)")},     {QStringLiteral("active"), false}, {QStringLiteral("running"), false}, {QStringLiteral("installed"), true}, {QStringLiteral("benchmarkRamMb"), 92.0}},
        }},
    };
    m_adminLock = QVariantMap{
        {QStringLiteral("passwordSet"), false},
        {QStringLiteral("locked"), false},
    };

    m_connections.clear();
    m_connSeqById.clear();
    m_liveBwBins.clear();
    m_liveInetBwBins.clear();
    const qint64 now = QDateTime::currentSecsSinceEpoch();
    struct DemoConnSpec {
        const char* prof;
        const char* profName;
        const char* domain;
        const char* ip;
        int port;
        const char* proto;
        const char* cc;
        const char* asOwner;
        const char* scope;
        const char* verdict;
        const char* reason;
        bool tun;
        bool active;
        bool internal;
        qint64 rx;
        qint64 tx;
        int ageSec;
    };
    const DemoConnSpec specs[] = {
        {"local/browser-demo",    "Firefox Demo",      "cdn.example.org",         "198.51.100.14",  443, "TCP", "NL", "Example Transit NL",       "internet", "permit", "Permitted via Xray NL-01",        true,  true,  false, 18425600, 1924100,   4},
        {"local/browser-demo",    "Firefox Demo",      "api.example.com",         "203.0.113.42",   443, "TCP", "DE", "Example Cloud GmbH",       "internet", "permit", "Permitted via Xray DE-01",        true,  true,  false,  4128900,  642120,  11},
        {"local/browser-demo",    "Firefox Demo",      "ads.tracker.example",     "192.0.2.88",     443, "TCP", "US", "Ad Telemetry Network",     "internet", "block",  "Blocked by filter list (TRAC)",   false, false, false,        0,       0,  18},
        {"local/browser-demo",    "Firefox Demo",      "video.example.org",       "198.51.100.91",  443, "UDP", "FI", "Nordic CDN Oy",            "internet", "permit", "Permitted (QUIC HTTP/3)",         true,  true,  false, 42190400, 1280400,  26},
        {"local/code-studio",     "Code Studio",       "git.example.org",         "203.0.113.77",   443, "TCP", "US", "Example Code Hosting",     "internet", "permit", "Direct connection permitted",     false, true,  false,  7854200, 2190400,  34},
        {"local/code-studio",     "Code Studio",       "metrics.example.org",     "192.0.2.120",    443, "TCP", "US", "Telemetry Collector",      "internet", "block",  "Blocked by filter list (TRAC)",   false, false, false,        0,       0,  49},
        {"local/cloud-sync",      "Cloud Sync Client", "sync.example.net",        "198.51.100.203", 443, "TCP", "CH", "Swiss Vault AG",           "internet", "permit", "Permitted via Xray CH-01",        true,  true,  false, 12450100, 5612300,  62},
        {"local/media-player",    "Media Streamer",    "stream.example.org",      "203.0.113.115",  443, "TCP", "GB", "UK Media Delivery",        "internet", "permit", "Direct connection permitted",     false, true,  false, 29310400,  892100,  85},
        {"local/terminal-ssh",    "Terminal Client",   "bastion.example.net",     "198.51.100.55",   22, "TCP", "JP", "Tokyo Edge Transit",       "internet", "permit", "Permitted via Xray JP-01",        true,  true,  false,  1294000,  418200, 105},
        {"local/browser-demo",    "Firefox Demo",      "pixel.tracker.example",   "192.0.2.91",     443, "TCP", "NL", "Analytics Edge",           "internet", "block",  "Blocked by per-app domain rule",  false, false, false,        0,       0, 130},
        {"local/code-studio",     "Code Studio",       "packages.example.org",    "203.0.113.18",   443, "TCP", "SG", "APAC Package Mirror",      "internet", "permit", "Direct connection permitted",     false, false, false,  9842100,  312400, 175},
        {"local/browser-demo",    "Firefox Demo",      "nas.lan",                 "192.168.10.20",  443, "TCP", "",   "",                         "lan",      "permit", "Permitted on local network",      false, true,  false,  3124000,  218000, 210},
        {"local/package-updater", "System Updater",    "updates.example.net",     "198.51.100.64",  443, "TCP", "DE", "Linux Mirror DE",          "internet", "permit", "Permitted (System Service)",      false, false, true,  15820100,  245100, 260},
        {"local/_portmaster",     "Portmaster Core Service", "",                  "9.9.9.9",        443, "TCP", "CH", "Quad9 DoH",                "internet", "permit", "Internal DoH resolver",           false, true,  true,     48200,   19400,  15},
        {"local/code-studio",     "Code Studio",       "localhost",               "127.0.0.1",     8080, "TCP", "",   "",                         "local",    "permit", "Loopback development server",     false, true,  false,   142000,   98400,  22},
    };

    const int nSpecs = static_cast<int>(sizeof(specs) / sizeof(specs[0]));
    for (int repeat = 0; repeat < 4; ++repeat) {
        for (int i = 0; i < nSpecs; ++i) {
            const auto& sp = specs[i];
            ConnRecord rec;
            rec.id = QStringLiteral("demo-%1-%2").arg(repeat).arg(i);
            rec.profile = QString::fromLatin1(sp.prof);
            rec.profileKey = QStringLiteral("core:profiles/") + rec.profile;
            rec.profileName = QString::fromLatin1(sp.profName);
            rec.domain = QString::fromLatin1(sp.domain);
            rec.ip = QString::fromLatin1(sp.ip);
            rec.port = sp.port;
            rec.proto = QString::fromLatin1(sp.proto);
            rec.direction = QStringLiteral("outbound");
            rec.scope = QString::fromLatin1(sp.scope);
            rec.country = QString::fromLatin1(sp.cc);
            rec.asOwner = QString::fromLatin1(sp.asOwner);
            rec.verdict = QString::fromLatin1(sp.verdict);
            rec.reason = QString::fromLatin1(sp.reason);
            rec.connType = qstrcmp(sp.ip, "9.9.9.9") == 0 ? QStringLiteral("ip") : QStringLiteral("domain");
            rec.rx = sp.rx / (repeat + 1);
            rec.tx = sp.tx / (repeat + 1);
            rec.bytesStr = formatBytesStatic(rec.rx + rec.tx);
            rec.startedTs = now - qBound(2, sp.ageSec + repeat * 120, 585);
            rec.tunneled = sp.tun;
            rec.active = (repeat == 0) && sp.active;
            rec.internal = sp.internal;
            rec.session = true;
            ++m_connHeadSeq;
            m_connSeqById.insert(rec.id, m_connHeadSeq);
            m_connections.push_back(std::move(rec));
        }
    }

    std::stable_sort(m_connections.begin(), m_connections.end(), [](const ConnRecord& a, const ConnRecord& b) {
        return a.startedTs > b.startedTs;
    });

    m_connSeqById.clear();
    m_connHeadSeq = static_cast<qint64>(m_connections.size());
    for (size_t i = 0; i < m_connections.size(); ++i) {
        m_connSeqById.insert(m_connections[i].id, m_connHeadSeq - static_cast<qint64>(i));
    }

    m_selectedProfileKey = QStringLiteral("core:profiles/local/browser-demo");
    rebuildSettingsCache();
    rebuildMapPins();
    rebuildAppsCache();
    rebuildConnectionsCache();
    rebuildSelectedAppCache();
    emit xrayChanged();
    emit geodataHintsChanged();
    emit uiReportChanged();
    emit adminLockChanged();
    updateTrayIcon();
}
