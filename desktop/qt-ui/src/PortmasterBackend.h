#pragma once

#include <QJsonArray>
#include <QJsonObject>
#include <QMap>
#include <QObject>
#include <QSystemTrayIcon>
#include <QTimer>
#include <QVariantList>
#include <QVariantMap>

#include "PortApiClient.h"

class PortmasterBackend : public QObject {
    Q_OBJECT

    Q_PROPERTY(bool connected READ connected NOTIFY connectionChanged)
    Q_PROPERTY(bool hasTray READ hasTray CONSTANT)
    Q_PROPERTY(QString transport READ transport NOTIFY connectionChanged)
    Q_PROPERTY(QString lang READ lang NOTIFY langChanged)
    Q_PROPERTY(bool isRu READ isRu NOTIFY langChanged)
    Q_PROPERTY(bool globalVpn READ globalVpn NOTIFY settingsChanged)
    Q_PROPERTY(bool globalSplit READ globalSplit NOTIFY settingsChanged)
    Q_PROPERTY(bool autoConnectXray READ autoConnectXray NOTIFY settingsChanged)
    Q_PROPERTY(bool historyEnabled READ historyEnabled NOTIFY settingsChanged)
    Q_PROPERTY(bool paused READ paused NOTIFY notificationsChanged)
    Q_PROPERTY(bool vpnPaused READ vpnPaused NOTIFY notificationsChanged)
    Q_PROPERTY(QString pauseUntil READ pauseUntil NOTIFY notificationsChanged)

    Q_PROPERTY(QVariantList activeApps READ activeApps NOTIFY appsChanged)
    Q_PROPERTY(QVariantList offlineApps READ offlineApps NOTIFY appsChanged)
    Q_PROPERTY(QVariantList allApps READ allApps NOTIFY appsChanged)
    Q_PROPERTY(QVariantMap selectedApp READ selectedApp NOTIFY selectedAppChanged)
    Q_PROPERTY(QVariantList selectedAppConnections READ selectedAppConnections NOTIFY selectedAppChanged)

    Q_PROPERTY(QVariantList connections READ connections NOTIFY connectionsChanged)
    Q_PROPERTY(QVariantList settingsList READ settingsList NOTIFY settingsChanged)
    Q_PROPERTY(QVariantList alerts READ alerts NOTIFY notificationsChanged)
    Q_PROPERTY(QVariantList prompts READ prompts NOTIFY notificationsChanged)

    Q_PROPERTY(QVariantMap xrayStatus READ xrayStatus NOTIFY xrayChanged)
    Q_PROPERTY(QVariantList xrayNodes READ xrayNodes NOTIFY xrayChanged)
    Q_PROPERTY(QVariantList xraySubs READ xraySubs NOTIFY xrayChanged)
    Q_PROPERTY(QVariantList worldPolygons READ worldPolygons NOTIFY worldMapReady)
    Q_PROPERTY(QVariantList mapPins READ mapPins NOTIFY xrayChanged)
    Q_PROPERTY(QVariantMap uiReport READ uiReport NOTIFY uiReportChanged)

    Q_PROPERTY(QString modalTitle READ modalTitle NOTIFY modalChanged)
    Q_PROPERTY(QString modalBody READ modalBody NOTIFY modalChanged)

public:
    explicit PortmasterBackend(bool startBackground = false, bool withTray = true, QObject* parent = nullptr);

    bool connected() const { return m_api->isConnected(); }
    bool hasTray() const { return m_withTray && QSystemTrayIcon::isSystemTrayAvailable(); }
    QString transport() const { return m_api->transportLabel(); }
    QString lang() const { return m_ru ? QStringLiteral("ru") : QStringLiteral("en"); }
    bool isRu() const { return m_ru; }

    bool globalVpn() const;
    bool globalSplit() const;
    bool autoConnectXray() const;
    bool historyEnabled() const;
    bool paused() const { return m_paused; }
    bool vpnPaused() const { return m_vpnPaused; }
    QString pauseUntil() const { return m_pauseUntil; }

    QVariantList activeApps() const { return m_activeAppsCache; }
    QVariantList offlineApps() const { return m_offlineAppsCache; }
    QVariantList allApps() const { return m_allAppsCache; }
    QVariantMap selectedApp() const { return m_selectedAppMap; }
    QVariantList selectedAppConnections() const { return m_selectedAppConns; }

    QVariantList connections() const { return m_connectionsCache; }
    QVariantList settingsList() const { return m_settingsCache; }
    QVariantList alerts() const { return m_alertsCache; }
    QVariantList prompts() const { return m_promptsCache; }

    QVariantMap xrayStatus() const { return m_xrayStatusMap; }
    QVariantList xrayNodes() const { return m_xrayNodesList; }
    QVariantList xraySubs() const { return m_xraySubsList; }
    QVariantList worldPolygons() const { return m_worldPolygons; }
    QVariantList mapPins() const { return m_mapPinsCache; }
    QVariantMap uiReport() const { return m_uiReportMap; }

    QString modalTitle() const { return m_modalTitle; }
    QString modalBody() const { return m_modalBody; }

    Q_INVOKABLE QString t(const QString& key) const;
    Q_INVOKABLE void setLanguage(const QString& code);
    Q_INVOKABLE void selectApp(const QString& scopedKey);

    Q_INVOKABLE void setAppAction(const QString& scopedKey, const QString& action);
    Q_INVOKABLE void setAppVpnMode(const QString& scopedKey, const QString& mode); // default, vpn, direct
    Q_INVOKABLE void setAppSplitMode(const QString& scopedKey, const QString& mode); // default, on, off
    Q_INVOKABLE void setAppBlockLan(const QString& scopedKey, bool blocked);
    Q_INVOKABLE void setAppCustomDns(const QString& scopedKey, const QString& serversCsv);
    Q_INVOKABLE void addAppRule(const QString& scopedKey, const QString& rule);
    Q_INVOKABLE void removeAppRule(const QString& scopedKey, const QString& rule);
    Q_INVOKABLE void promoteRuleToGlobal(const QString& cfgKey, const QString& rule);
    Q_INVOKABLE QString testAppPath(const QString& scopedKey, const QString& testPath) const;
    Q_INVOKABLE void createProfile(const QString& name, const QString& path);
    Q_INVOKABLE void deleteProfile(const QString& scopedKey);
    Q_INVOKABLE void exportProfile(const QString& scopedKey);
    Q_INVOKABLE void importProfile(const QString& rawPayload);

    Q_INVOKABLE void allowOrBlockConnection(const QString& connId, bool allow);
    Q_INVOKABLE void clearHistory();

    Q_INVOKABLE void toggleGlobalVpn();
    Q_INVOKABLE void resetAllAppOverrides();
    Q_INVOKABLE void toggleXrayConnect();
    Q_INVOKABLE void connectNode(const QString& nodeId);
    Q_INVOKABLE void addSubscription(const QString& url);
    Q_INVOKABLE void addManualLink(const QString& links);
    Q_INVOKABLE void refreshSubscription(const QString& id);
    Q_INVOKABLE void deleteSubscription(const QString& id);
    Q_INVOKABLE void downloadXrayBinary();
    Q_INVOKABLE void downloadGeodata();

    Q_INVOKABLE void updateSetting(const QString& key, const QVariant& val);
    Q_INVOKABLE void resetSetting(const QString& key);
    Q_INVOKABLE void applyPreset(const QString& key, const QStringList& values, bool append);
    Q_INVOKABLE void switchUiFlavor(const QString& flavorId);
    Q_INVOKABLE void setKeepApiPortOpen(bool keep);
    Q_INVOKABLE void exportSettings();
    Q_INVOKABLE void importSettings(const QString& rawPayload);
    Q_INVOKABLE void closeModal();

    Q_INVOKABLE void resolvePrompt(const QString& promptId, bool allow);
    Q_INVOKABLE void resolveAllPrompts(bool allow);
    Q_INVOKABLE void clearNotifications();

    Q_INVOKABLE void pauseProtection(int minutes, bool vpnOnly);
    Q_INVOKABLE void resumeProtection();
    Q_INVOKABLE void clearDnsCache();
    Q_INVOKABLE void checkUpdates();
    Q_INVOKABLE void restartCore();
    Q_INVOKABLE void shutdownCore();
    Q_INVOKABLE void quitApp();

signals:
    void connectionChanged();
    void langChanged();
    void appsChanged();
    void selectedAppChanged();
    void connectionsChanged();
    void settingsChanged();
    void notificationsChanged();
    void xrayChanged();
    void worldMapReady();
    void uiReportChanged();
    void modalChanged();
    void showWindowRequested();
    void toastMessage(const QString& title, const QString& body, bool isError);

private slots:
    void onConnectionChanged(bool connected, const QString& transport);
    void refreshHttpData();

private:
    void startSubscriptions();
    void fetchWorldMap();
    void scheduleAppsRebuild();
    void scheduleSettingsRebuild();
    void scheduleConnsRebuild();
    void rebuildAppsCache();
    void rebuildSelectedAppCache();
    void rebuildConnectionsCache();
    void rebuildSettingsCache();
    void rebuildNotificationsCache();
    void rebuildMapPins();
    void setupTray();
    void updateTrayIcon();
    QIcon buildShieldIcon(const QColor& c) const;
    QJsonValue effectiveValue(const QString& key) const;
    QString formatBytes(qint64 bytes) const;

    PortApiClient* m_api = nullptr;
    QSystemTrayIcon* m_tray = nullptr;
    QTimer* m_appsTimer = nullptr;
    QTimer* m_settingsTimer = nullptr;
    QTimer* m_connsTimer = nullptr;
    bool m_withTray = true;
    bool m_ru = true;
    bool m_subscribed = false;
    bool m_worldLoaded = false;

    QMap<QString, QJsonObject> m_profiles;
    QMap<QString, QJsonObject> m_settings;
    QMap<QString, QJsonObject> m_notifications;
    QVector<QJsonObject> m_connections;

    QString m_selectedProfileKey;
    bool m_paused = false;
    bool m_vpnPaused = false;
    QString m_pauseUntil;

    QVariantList m_activeAppsCache;
    QVariantList m_offlineAppsCache;
    QVariantList m_allAppsCache;
    QVariantMap m_selectedAppMap;
    QVariantList m_selectedAppConns;
    QVariantList m_connectionsCache;
    QVariantList m_settingsCache;
    QVariantList m_alertsCache;
    QVariantList m_promptsCache;

    QVariantMap m_xrayStatusMap;
    QVariantList m_xrayNodesList;
    QVariantList m_xraySubsList;
    QVariantList m_worldPolygons;
    QVariantList m_mapPinsCache;
    QVariantMap m_uiReportMap;

    QString m_modalTitle;
    QString m_modalBody;
};
