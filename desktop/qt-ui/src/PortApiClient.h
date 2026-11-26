#pragma once

#include <QByteArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <QJsonValue>
#include <QLocalSocket>
#include <QMap>
#include <QNetworkAccessManager>
#include <QObject>
#include <QTimer>
#include <QWebSocket>
#include <functional>

class PortApiClient : public QObject {
    Q_OBJECT
public:
    using HttpCallback = std::function<void(int statusCode, const QByteArray& body, const QString& err)>;
    using SubCallback = std::function<void(const QString& msgType, const QString& key, const QJsonValue& val)>;

    explicit PortApiClient(QObject* parent = nullptr);

    void start();
    QString discoveredSocketPath() const;
    QString transportLabel() const;
    bool isConnected() const { return m_wsConnected; }

    void httpGet(const QString& path, HttpCallback cb);
    void httpPost(const QString& path, const QByteArray& body, const QString& contentType, HttpCallback cb);
    void httpPut(const QString& path, const QByteArray& body, const QString& contentType, HttpCallback cb);
    void httpDelete(const QString& path, HttpCallback cb);

    QString qsub(const QString& query, SubCallback cb);
    void sendPutFrame(const QString& msgType, const QString& key, const QJsonValue& val);
    void sendDeleteFrame(const QString& key);

signals:
    void connectionChanged(bool connected, const QString& transport);
    void pingMeasured(int ms);

private slots:
    void tryConnect();
    void onLocalWsReadyRead();
    void onLocalWsDisconnected();
    void onTcpWsConnected();
    void onTcpWsDisconnected();
    void onTcpWsTextReceived(const QString& msg);
    void onTcpWsBinaryReceived(const QByteArray& msg);

private:
    void httpRequest(const QString& method, const QString& path, const QByteArray& body, const QString& contentType, HttpCallback cb);
    void httpOverUnixSocket(const QString& sockPath, const QString& method, const QString& path, const QByteArray& body, const QString& contentType, HttpCallback cb);
    void sendRawWsText(const QString& text);
    void handleWireMessage(const QString& line);
    void replaySubscriptions();
    void processWsBytes();
    void sendMaskedUnixFrame(quint8 opcode, const QByteArray& payload);
    QString nextOpId(const QString& prefix);
    QString baseHttpUrl() const;

    QNetworkAccessManager* m_nam = nullptr;
    QLocalSocket* m_localWs = nullptr;
    QWebSocket* m_tcpWs = nullptr;
    QTimer* m_reconnectTimer = nullptr;

    bool m_wsConnected = false;
    bool m_usingUnixWs = false;
    bool m_localHttpDone = false;
    QByteArray m_localRxBuf;
    QByteArray m_localFragBuf;
    quint8 m_fragOpcode = 0;
    QString m_activeTransport;

    quint64 m_counter = 0;
    struct SubEntry {
        QString wire;
        SubCallback cb;
    };
    QMap<QString, SubEntry> m_subs;
};
