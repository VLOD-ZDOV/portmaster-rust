#include "PortApiClient.h"

#include <memory>
#include <QDir>
#include <QFileInfo>
#include <QJsonArray>
#include <QNetworkReply>
#include <QNetworkRequest>
#include <QRandomGenerator>
#include <QUrl>

namespace {
QByteArray decodeChunkedBody(const QByteArray& raw) {
    QByteArray out;
    int pos = 0;
    while (pos < raw.size()) {
        int lineEnd = raw.indexOf("\r\n", pos);
        if (lineEnd < 0) break;
        bool ok = false;
        int len = QString::fromLatin1(raw.mid(pos, lineEnd - pos)).section(';', 0, 0).trimmed().toInt(&ok, 16);
        if (!ok || len < 0) break;
        pos = lineEnd + 2;
        if (len == 0) break;
        if (pos + len > raw.size()) {
            out.append(raw.mid(pos));
            break;
        }
        out.append(raw.mid(pos, len));
        pos += len + 2;
    }
    return out;
}
} // namespace

PortApiClient::PortApiClient(QObject* parent)
    : QObject(parent),
      m_nam(new QNetworkAccessManager(this)),
      m_reconnectTimer(new QTimer(this)) {
    m_reconnectTimer->setInterval(2000);
    connect(m_reconnectTimer, &QTimer::timeout, this, &PortApiClient::tryConnect);
}

QString PortApiClient::baseHttpUrl() const {
    QString url = qEnvironmentVariable("PORTMASTER_API_URL", QStringLiteral("http://127.0.0.1:817")).trimmed();
    while (url.endsWith('/')) url.chop(1);
    return url.isEmpty() ? QStringLiteral("http://127.0.0.1:817") : url;
}

QString PortApiClient::discoveredSocketPath() const {
#ifdef Q_OS_UNIX
    if (qEnvironmentVariableIsSet("PORTMASTER_FORCE_TCP")) {
        return {};
    }
    const QString envSock = qEnvironmentVariable("PORTMASTER_API_SOCKET").trimmed();
    if (!envSock.isEmpty() && QFileInfo::exists(envSock)) {
        return envSock;
    }
    const QString xdg = qEnvironmentVariable("XDG_DATA_HOME").trimmed();
    QStringList candidates{
        QStringLiteral("/run/portmaster/api.sock"),
        QStringLiteral("/var/lib/portmaster/api.sock"),
        QStringLiteral("/opt/safing/portmaster/api.sock"),
    };
    if (!xdg.isEmpty()) {
        candidates << xdg + QStringLiteral("/portmaster/api.sock");
    }
    candidates << QDir::homePath() + QStringLiteral("/.local/share/portmaster/api.sock");
    for (const QString& c : candidates) {
        if (QFileInfo::exists(c)) {
            return c;
        }
    }
#endif
    return {};
}

QString PortApiClient::transportLabel() const {
    return m_activeTransport;
}

void PortApiClient::start() {
    tryConnect();
    m_reconnectTimer->start();
}

QString PortApiClient::nextOpId(const QString& prefix) {
    return QStringLiteral("%1-%2").arg(prefix).arg(++m_counter);
}

void PortApiClient::tryConnect() {
    if (m_wsConnected) return;
    if (m_localWs && m_localWs->state() != QLocalSocket::UnconnectedState) return;
    if (m_tcpWs && m_tcpWs->state() != QAbstractSocket::UnconnectedState) return;

#ifdef Q_OS_UNIX
    const QString sock = discoveredSocketPath();
    if (!sock.isEmpty()) {
        if (m_localWs) {
            m_localWs->deleteLater();
            m_localWs = nullptr;
        }
        m_localWs = new QLocalSocket(this);
        m_localHttpDone = false;
        m_localRxBuf.clear();
        m_usingUnixWs = true;

        connect(m_localWs, &QLocalSocket::connected, this, [this, sock]() {
            QByteArray keyBytes(16, Qt::Uninitialized);
            for (int i = 0; i < 16; ++i) {
                keyBytes[i] = static_cast<char>(QRandomGenerator::global()->bounded(256));
            }
            const QByteArray b64Key = keyBytes.toBase64();
            const QByteArray req =
                "GET /api/database/v1 HTTP/1.1\r\n"
                "Host: 127.0.0.1:817\r\n"
                "Upgrade: websocket\r\n"
                "Connection: Upgrade\r\n"
                "Sec-WebSocket-Key: " + b64Key + "\r\n"
                "Sec-WebSocket-Version: 13\r\n\r\n";
            m_localWs->write(req);
            m_activeTransport = QStringLiteral("Unix IPC (%1)").arg(sock);
        });
        connect(m_localWs, &QLocalSocket::readyRead, this, &PortApiClient::onLocalWsReadyRead);
        connect(m_localWs, &QLocalSocket::disconnected, this, &PortApiClient::onLocalWsDisconnected);
        connect(m_localWs, &QLocalSocket::errorOccurred, this, [this](QLocalSocket::LocalSocketError) {
            if (m_localWs) {
                m_localWs->abort();
            }
        });
        m_localWs->connectToServer(sock);
        return;
    }
#endif

    // TCP fallback (Windows or when Unix socket absent)
    if (!m_tcpWs) {
        m_tcpWs = new QWebSocket(QString(), QWebSocketProtocol::VersionLatest, this);
        connect(m_tcpWs, &QWebSocket::connected, this, &PortApiClient::onTcpWsConnected);
        connect(m_tcpWs, &QWebSocket::disconnected, this, &PortApiClient::onTcpWsDisconnected);
        connect(m_tcpWs, &QWebSocket::textMessageReceived, this, &PortApiClient::onTcpWsTextReceived);
        connect(m_tcpWs, &QWebSocket::binaryMessageReceived, this, &PortApiClient::onTcpWsBinaryReceived);
    }
    m_usingUnixWs = false;
    const QString httpBase = baseHttpUrl();
    QString wsUrl = httpBase;
    if (wsUrl.startsWith(QLatin1String("https://"))) wsUrl.replace(0, 8, QStringLiteral("wss://"));
    else if (wsUrl.startsWith(QLatin1String("http://"))) wsUrl.replace(0, 7, QStringLiteral("ws://"));
    m_activeTransport = QStringLiteral("TCP ") + QUrl(httpBase).authority();
    m_tcpWs->open(QUrl(wsUrl + QStringLiteral("/api/database/v1")));
}

void PortApiClient::onLocalWsReadyRead() {
    if (!m_localWs) return;
    m_localRxBuf.append(m_localWs->readAll());

    if (!m_localHttpDone) {
        int hdrEnd = m_localRxBuf.indexOf("\r\n\r\n");
        if (hdrEnd < 0) return;
        const QByteArray hdr = m_localRxBuf.left(hdrEnd);
        m_localRxBuf.remove(0, hdrEnd + 4);
        if (!hdr.contains("101")) {
            m_localWs->abort();
            return;
        }
        m_localHttpDone = true;
        m_wsConnected = true;
        replaySubscriptions();
        emit connectionChanged(true, m_activeTransport);
    }
    processWsBytes();
}

void PortApiClient::processWsBytes() {
    while (m_localRxBuf.size() >= 2) {
        const quint8 b0 = static_cast<quint8>(m_localRxBuf[0]);
        const quint8 b1 = static_cast<quint8>(m_localRxBuf[1]);
        const bool fin = (b0 & 0x80) != 0;
        const quint8 opcode = b0 & 0x0f;
        const bool masked = (b1 & 0x80) != 0;
        quint64 payloadLen = b1 & 0x7f;
        int offset = 2;

        if (payloadLen == 126) {
            if (m_localRxBuf.size() < 4) return;
            payloadLen = (static_cast<quint8>(m_localRxBuf[2]) << 8) | static_cast<quint8>(m_localRxBuf[3]);
            offset = 4;
        } else if (payloadLen == 127) {
            if (m_localRxBuf.size() < 10) return;
            payloadLen = 0;
            for (int i = 0; i < 8; ++i) {
                payloadLen = (payloadLen << 8) | static_cast<quint8>(m_localRxBuf[2 + i]);
            }
            offset = 10;
        }
        int maskOff = offset;
        if (masked) offset += 4;
        if (static_cast<quint64>(m_localRxBuf.size()) < static_cast<quint64>(offset) + payloadLen) return;

        QByteArray payload = m_localRxBuf.mid(offset, static_cast<int>(payloadLen));
        if (masked) {
            for (int i = 0; i < payload.size(); ++i) {
                payload[i] = payload[i] ^ m_localRxBuf[maskOff + (i % 4)];
            }
        }
        m_localRxBuf.remove(0, offset + static_cast<int>(payloadLen));

        if (opcode == 0x8) {
            // Close frame
            m_localWs->disconnectFromServer();
            return;
        } else if (opcode == 0x9) {
            // Ping -> Pong
            sendMaskedUnixFrame(0xA, payload);
        } else if (opcode == 0xA) {
            // Pong
        } else if (opcode == 0x1 || opcode == 0x2 || opcode == 0x0) {
            if (opcode != 0x0) {
                m_fragOpcode = opcode;
                m_localFragBuf = payload;
            } else {
                m_localFragBuf.append(payload);
            }
            if (fin && (m_fragOpcode == 0x1 || m_fragOpcode == 0x2)) {
                handleWireMessage(QString::fromUtf8(m_localFragBuf));
                m_localFragBuf.clear();
                m_fragOpcode = 0;
            } else if (fin) {
                m_localFragBuf.clear();
                m_fragOpcode = 0;
            }
        }
    }
}

void PortApiClient::sendMaskedUnixFrame(quint8 opcode, const QByteArray& payload) {
    if (!m_localWs || m_localWs->state() != QLocalSocket::ConnectedState) return;
    QByteArray frame;
    frame.append(static_cast<char>(0x80 | (opcode & 0x0f)));
    const quint64 len = static_cast<quint64>(payload.size());
    if (len < 126) {
        frame.append(static_cast<char>(0x80 | static_cast<quint8>(len)));
    } else if (len <= 0xffff) {
        frame.append(static_cast<char>(0x80 | 126));
        frame.append(static_cast<char>((len >> 8) & 0xff));
        frame.append(static_cast<char>(len & 0xff));
    } else {
        frame.append(static_cast<char>(0x80 | 127));
        for (int i = 7; i >= 0; --i) {
            frame.append(static_cast<char>((len >> (i * 8)) & 0xff));
        }
    }
    quint32 mask = QRandomGenerator::global()->generate();
    char maskBytes[4] = {
        static_cast<char>((mask >> 24) & 0xff),
        static_cast<char>((mask >> 16) & 0xff),
        static_cast<char>((mask >> 8) & 0xff),
        static_cast<char>(mask & 0xff),
    };
    frame.append(maskBytes, 4);
    int oldSize = frame.size();
    frame.resize(oldSize + payload.size());
    for (int i = 0; i < payload.size(); ++i) {
        frame[oldSize + i] = payload[i] ^ maskBytes[i % 4];
    }
    m_localWs->write(frame);
}

void PortApiClient::onLocalWsDisconnected() {
    bool was = m_wsConnected;
    m_wsConnected = false;
    m_localHttpDone = false;
    m_localRxBuf.clear();
    m_localFragBuf.clear();
    m_fragOpcode = 0;
    if (was) {
        emit connectionChanged(false, m_activeTransport);
    }
}

void PortApiClient::onTcpWsConnected() {
    m_wsConnected = true;
    replaySubscriptions();
    emit connectionChanged(true, m_activeTransport);
}

void PortApiClient::onTcpWsDisconnected() {
    bool was = m_wsConnected;
    m_wsConnected = false;
    if (was) {
        emit connectionChanged(false, m_activeTransport);
    }
}

void PortApiClient::onTcpWsTextReceived(const QString& msg) {
    handleWireMessage(msg);
}

void PortApiClient::onTcpWsBinaryReceived(const QByteArray& msg) {
    handleWireMessage(QString::fromUtf8(msg));
}

void PortApiClient::sendRawWsText(const QString& text) {
    if (!m_wsConnected) return;
    if (m_usingUnixWs) {
        sendMaskedUnixFrame(0x1, text.toUtf8());
    } else if (m_tcpWs) {
        m_tcpWs->sendTextMessage(text);
    }
}

void PortApiClient::replaySubscriptions() {
    for (auto it = m_subs.cbegin(); it != m_subs.cend(); ++it) {
        sendRawWsText(it.value().wire);
    }
}

QString PortApiClient::qsub(const QString& query, SubCallback cb) {
    const QString opId = nextOpId(QStringLiteral("qsub"));
    const QString cleanQ = query.startsWith(QLatin1String("query ")) ? query : (QStringLiteral("query ") + query);
    const QString wire = QStringLiteral("%1|qsub|%2").arg(opId, cleanQ);
    m_subs.insert(opId, SubEntry{wire, std::move(cb)});
    if (m_wsConnected) {
        sendRawWsText(wire);
    }
    return opId;
}

void PortApiClient::sendPutFrame(const QString& msgType, const QString& key, const QJsonValue& val) {
    const QString opId = nextOpId(msgType);
    QByteArray dataBytes;
    if (val.isObject()) {
        dataBytes = QJsonDocument(val.toObject()).toJson(QJsonDocument::Compact);
    } else if (val.isArray()) {
        dataBytes = QJsonDocument(val.toArray()).toJson(QJsonDocument::Compact);
    } else {
        dataBytes = "{}";
    }
    // PortAPI create/update records require DSD format byte 'J' before JSON payload.
    const QString payloadPrefix = (msgType == QLatin1String("create") || msgType == QLatin1String("update"))
                                      ? QStringLiteral("J")
                                      : QString();
    const QString wire = QStringLiteral("%1|%2|%3|%4%5")
                             .arg(opId, msgType, key, payloadPrefix, QString::fromUtf8(dataBytes));
    sendRawWsText(wire);
}

void PortApiClient::sendDeleteFrame(const QString& key) {
    const QString opId = nextOpId(QStringLiteral("delete"));
    const QString wire = QStringLiteral("%1|delete|%2").arg(opId, key);
    sendRawWsText(wire);
}

void PortApiClient::handleWireMessage(const QString& line) {
    const int p1 = line.indexOf('|');
    if (p1 <= 0) return;
    const QString opId = line.left(p1);
    const int p2 = line.indexOf('|', p1 + 1);
    if (p2 < 0) return;
    const QString msgType = line.mid(p1 + 1, p2 - p1 - 1);
    if (msgType == QLatin1String("done") || msgType == QLatin1String("warning")) return;

    const int p3 = line.indexOf('|', p2 + 1);
    const QString key = p3 >= 0 ? line.mid(p2 + 1, p3 - p2 - 1) : line.mid(p2 + 1);
    QJsonValue val;
    if (p3 >= 0) {
        QByteArray jsonRaw = line.mid(p3 + 1).toUtf8();
        if (jsonRaw.startsWith('J')) {
            jsonRaw.remove(0, 1);
        }
        const QJsonDocument doc = QJsonDocument::fromJson(jsonRaw);
        if (doc.isObject()) val = doc.object();
        else if (doc.isArray()) val = doc.array();
    }

    auto it = m_subs.find(opId);
    if (it != m_subs.end() && it.value().cb) {
        it.value().cb(msgType, key, val);
    }
}

void PortApiClient::httpGet(const QString& path, HttpCallback cb) {
    httpRequest(QStringLiteral("GET"), path, {}, {}, std::move(cb));
}

void PortApiClient::httpPost(const QString& path, const QByteArray& body, const QString& contentType, HttpCallback cb) {
    httpRequest(QStringLiteral("POST"), path, body, contentType, std::move(cb));
}

void PortApiClient::httpPut(const QString& path, const QByteArray& body, const QString& contentType, HttpCallback cb) {
    httpRequest(QStringLiteral("PUT"), path, body, contentType, std::move(cb));
}

void PortApiClient::httpDelete(const QString& path, HttpCallback cb) {
    httpRequest(QStringLiteral("DELETE"), path, {}, {}, std::move(cb));
}

void PortApiClient::httpRequest(const QString& method, const QString& path, const QByteArray& body, const QString& contentType, HttpCallback cb) {
    const QString sock = discoveredSocketPath();
    if (!sock.isEmpty()) {
        httpOverUnixSocket(sock, method, path, body, contentType, std::move(cb));
        return;
    }

    QUrl url(baseHttpUrl() + path);
    QNetworkRequest req(url);
    if (!contentType.isEmpty()) {
        req.setHeader(QNetworkRequest::ContentTypeHeader, contentType);
    }
    QNetworkReply* reply = nullptr;
    if (method == QLatin1String("GET")) reply = m_nam->get(req);
    else if (method == QLatin1String("POST")) reply = m_nam->post(req, body);
    else if (method == QLatin1String("PUT")) reply = m_nam->put(req, body);
    else if (method == QLatin1String("DELETE")) reply = m_nam->deleteResource(req);

    if (!reply) {
        cb(0, {}, QStringLiteral("Unsupported HTTP method"));
        return;
    }
    connect(reply, &QNetworkReply::finished, this, [reply, cb = std::move(cb)]() {
        int code = reply->attribute(QNetworkRequest::HttpStatusCodeAttribute).toInt();
        QByteArray data = reply->readAll();
        QString err = reply->error() == QNetworkReply::NoError ? QString() : reply->errorString();
        reply->deleteLater();
        cb(code, data, err);
    });
}

void PortApiClient::httpOverUnixSocket(const QString& sockPath, const QString& method, const QString& path, const QByteArray& body, const QString& contentType, HttpCallback cb) {
    auto* sock = new QLocalSocket(this);
    auto buf = std::make_shared<QByteArray>();
    auto cbShared = std::make_shared<HttpCallback>(std::move(cb));
    auto finished = std::make_shared<bool>(false);

    auto finish = [sock, cbShared, finished](int code, const QByteArray& respBody, const QString& err) {
        if (*finished) return;
        *finished = true;
        sock->deleteLater();
        if (*cbShared) (*cbShared)(code, respBody, err);
    };

    connect(sock, &QLocalSocket::connected, sock, [sock, method, path, body, contentType, finished]() {
        if (*finished) return;
        QByteArray raw = method.toUtf8() + " " + path.toUtf8() + " HTTP/1.1\r\n"
            "Host: 127.0.0.1:817\r\n"
            "Connection: close\r\n";
        if (!contentType.isEmpty()) {
            raw += "Content-Type: " + contentType.toUtf8() + "\r\n";
        }
        if (!body.isEmpty() || method == QLatin1String("POST") || method == QLatin1String("PUT")) {
            raw += "Content-Length: " + QByteArray::number(body.size()) + "\r\n";
        }
        raw += "\r\n" + body;
        sock->write(raw);
    });

    connect(sock, &QLocalSocket::readyRead, sock, [sock, buf, finished]() {
        if (*finished) return;
        buf->append(sock->readAll());
    });

    connect(sock, &QLocalSocket::disconnected, sock, [sock, buf, finished, finish]() {
        if (*finished) return;
        buf->append(sock->readAll());
        int hdrEnd = buf->indexOf("\r\n\r\n");
        if (hdrEnd < 0) {
            finish(0, {}, QStringLiteral("Invalid HTTP response over Unix socket"));
            return;
        }
        const QByteArray hdr = buf->left(hdrEnd);
        QByteArray bodyBytes = buf->mid(hdrEnd + 4);
        int code = 0;
        int firstSp = hdr.indexOf(' ');
        if (firstSp > 0) {
            code = hdr.mid(firstSp + 1, 3).toInt();
        }
        if (hdr.toLower().contains("transfer-encoding: chunked")) {
            bodyBytes = decodeChunkedBody(bodyBytes);
        }
        finish(code, bodyBytes, {});
    });

    connect(sock, &QLocalSocket::errorOccurred, sock, [sock, finished, finish](QLocalSocket::LocalSocketError err) {
        if (*finished) return;
        if (err == QLocalSocket::PeerClosedError) {
            // Clean HTTP EOF triggers PeerClosedError right before disconnected().
            return;
        }
        finish(0, {}, sock->errorString());
    });

    sock->connectToServer(sockPath);
}
