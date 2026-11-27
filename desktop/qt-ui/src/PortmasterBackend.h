#pragma once

#include <deque>
#include <QAbstractListModel>
#include <QHash>
#include <QImage>
#include <QJsonArray>
#include <QJsonObject>
#include <QMap>
#include <QMutex>
#include <QObject>
#include <QQuickImageProvider>
#include <QSet>
#include <QSharedPointer>
#include <QSystemTrayIcon>
#include <QTimer>
#include <QVariantList>
#include <QVariantMap>

#include "PortApiClient.h"

class QAction;
class QMenu;

struct ConnRecord {
    QString id;
    QString profile;
    QString profileKey;
    QString profileName;
    QString domain;
    QString ip;
    QString proto;
    QString direction;
    QString scope;
    QString country;
    QString asOwner;
    QString verdict;
    QString reason;
    QString connType;
    QString bytesStr;
    int port = 0;
    qint64 rx = 0;
    qint64 tx = 0;
    qint64 accountedLiveRx = 0;
    qint64 accountedLiveTx = 0;
    qint64 startedTs = 0;
    bool tunneled = false;
    bool active = false;
    bool internal = false;
    bool session = false;
};

class ConnectionListModel : public QAbstractListModel {
    Q_OBJECT
    Q_PROPERTY(int count READ rowCount NOTIFY countChanged)
    Q_PROPERTY(QString searchQuery READ searchQuery WRITE setSearchQuery NOTIFY filterChanged)
    Q_PROPERTY(bool scopeInternet READ scopeInternet WRITE setScopeInternet NOTIFY filterChanged)
    Q_PROPERTY(bool scopeLan READ scopeLan WRITE setScopeLan NOTIFY filterChanged)
    Q_PROPERTY(bool scopeLocal READ scopeLocal WRITE setScopeLocal NOTIFY filterChanged)
    Q_PROPERTY(QString dirFilter READ dirFilter WRITE setDirFilter NOTIFY filterChanged)
    Q_PROPERTY(QString verdictFilter READ verdictFilter WRITE setVerdictFilter NOTIFY filterChanged)
    Q_PROPERTY(bool hideDnsNoise READ hideDnsNoise WRITE setHideDnsNoise NOTIFY filterChanged)
    Q_PROPERTY(bool hideLocalhost READ hideLocalhost WRITE setHideLocalhost NOTIFY filterChanged)

public:
    enum Roles {
        ConnIdRole = Qt::UserRole + 1,
        ProfileKeyRole,
        TargetRole,
        DomainRole,
        IpRole,
        PortRole,
        ProtoRole,
        CountryRole,
        AppRole,
        ScopeRole,
        DirectionRole,
        VerdictRole,
        TunneledRole,
        ReasonRole,
        AsOwnerRole,
        BytesRole,
        ActiveRole,
        InternalRole,
        ConnTypeRole,
    };

    explicit ConnectionListModel(const std::deque<ConnRecord>* source, QObject* parent = nullptr);

    int rowCount(const QModelIndex& parent = QModelIndex()) const override;
    QVariant data(const QModelIndex& index, int role = Qt::DisplayRole) const override;
    QHash<int, QByteArray> roleNames() const override;

    QString searchQuery() const { return m_searchQuery; }
    bool scopeInternet() const { return m_scopeInternet; }
    bool scopeLan() const { return m_scopeLan; }
    bool scopeLocal() const { return m_scopeLocal; }
    QString dirFilter() const { return m_dirFilter; }
    QString verdictFilter() const { return m_verdictFilter; }
    bool hideDnsNoise() const { return m_hideDnsNoise; }
    bool hideLocalhost() const { return m_hideLocalhost; }

    void setSearchQuery(const QString& q);
    void setScopeInternet(bool v);
    void setScopeLan(bool v);
    void setScopeLocal(bool v);
    void setDirFilter(const QString& v);
    void setVerdictFilter(const QString& v);
    void setHideDnsNoise(bool v);
    void setHideLocalhost(bool v);

    void syncFrom(int limit);

signals:
    void countChanged();
    void filterChanged();

private:
    void refilter(bool preserveOnSameShape = true);

    const std::deque<ConnRecord>* m_source = nullptr;
    int m_limit = 2000;
    QVector<int> m_filtered;
    QString m_searchQuery;
    bool m_scopeInternet = false;
    bool m_scopeLan = false;
    bool m_scopeLocal = false;
    QString m_dirFilter = QStringLiteral("all");
    QString m_verdictFilter = QStringLiteral("all");
    bool m_hideDnsNoise = true;
    bool m_hideLocalhost = true;
};

struct PortmasterIconStore {
    QMutex mutex;
    QHash<QString, QImage> images;
};

class PortmasterIconProvider : public QQuickImageProvider {
public:
    explicit PortmasterIconProvider(QSharedPointer<PortmasterIconStore> store);
    QImage requestImage(const QString& id, QSize* size, const QSize& requestedSize) override;

private:
    QSharedPointer<PortmasterIconStore> m_store;
};

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
    Q_PROPERTY(bool blockInternet READ blockInternet NOTIFY settingsChanged)
    Q_PROPERTY(bool blockLan READ blockLan NOTIFY settingsChanged)
    Q_PROPERTY(bool blockP2p READ blockP2p NOTIFY settingsChanged)
    Q_PROPERTY(bool blockInbound READ blockInbound NOTIFY settingsChanged)
    Q_PROPERTY(bool preventDnsBypass READ preventDnsBypass NOTIFY settingsChanged)
    Q_PROPERTY(bool dnsNoInsecure READ dnsNoInsecure NOTIFY settingsChanged)
    Q_PROPERTY(bool dnsIgnoreSystem READ dnsIgnoreSystem NOTIFY settingsChanged)
    Q_PROPERTY(bool dnsUseStaleCache READ dnsUseStaleCache NOTIFY settingsChanged)
    Q_PROPERTY(QStringList dnsServers READ dnsServers NOTIFY settingsChanged)
    Q_PROPERTY(QString dnsSummary READ dnsSummary NOTIFY settingsChanged)
    Q_PROPERTY(bool paused READ paused NOTIFY notificationsChanged)
    Q_PROPERTY(bool vpnPaused READ vpnPaused NOTIFY notificationsChanged)
    Q_PROPERTY(QString pauseUntil READ pauseUntil NOTIFY notificationsChanged)

    Q_PROPERTY(QVariantList activeApps READ activeApps NOTIFY appsChanged)
    Q_PROPERTY(bool onlyNowApps READ onlyNowApps WRITE setOnlyNowApps NOTIFY appsChanged)
    Q_PROPERTY(QVariantList offlineApps READ offlineApps NOTIFY appsChanged)
    Q_PROPERTY(QVariantList allApps READ allApps NOTIFY appsChanged)
    Q_PROPERTY(QVariantMap selectedApp READ selectedApp NOTIFY selectedAppChanged)
    Q_PROPERTY(QVariantList selectedAppConnections READ selectedAppConnections NOTIFY selectedAppChanged)
    Q_PROPERTY(QVariantList selectedAppSettings READ selectedAppSettings NOTIFY selectedAppChanged)
    Q_PROPERTY(QVariantList selectedAppChartBins READ selectedAppChartBins NOTIFY selectedAppChanged)
    Q_PROPERTY(bool appChartInternetOnly READ appChartInternetOnly WRITE setAppChartInternetOnly NOTIFY selectedAppChanged)
    Q_PROPERTY(int selectedAppInetAllowed READ selectedAppInetAllowed NOTIFY selectedAppChanged)
    Q_PROPERTY(int selectedAppInetBlocked READ selectedAppInetBlocked NOTIFY selectedAppChanged)
    Q_PROPERTY(int selectedAppAllAllowed READ selectedAppAllAllowed NOTIFY selectedAppChanged)
    Q_PROPERTY(int selectedAppAllBlocked READ selectedAppAllBlocked NOTIFY selectedAppChanged)

    Q_PROPERTY(QObject* connectionsModel READ connectionsModel CONSTANT)
    Q_PROPERTY(int connectionsTotal READ connectionsTotal NOTIFY connectionsChanged)
    Q_PROPERTY(QVariantList settingsList READ settingsList NOTIFY settingsChanged)
    Q_PROPERTY(QVariantList alerts READ alerts NOTIFY notificationsChanged)
    Q_PROPERTY(QVariantList prompts READ prompts NOTIFY notificationsChanged)

    Q_PROPERTY(QVariantMap xrayStatus READ xrayStatus NOTIFY xrayChanged)
    Q_PROPERTY(QVariantList xrayNodes READ xrayNodes NOTIFY xrayChanged)
    Q_PROPERTY(QVariantList xraySubs READ xraySubs NOTIFY xrayChanged)
    Q_PROPERTY(QVariantList worldPolygons READ worldPolygons NOTIFY worldMapReady)
    Q_PROPERTY(QVariantList worldCountries READ worldCountries NOTIFY worldMapReady)
    Q_PROPERTY(QVariantList mapPins READ mapPins NOTIFY xrayChanged)
    Q_PROPERTY(QVariantMap uiReport READ uiReport NOTIFY uiReportChanged)
    Q_PROPERTY(bool uiBusy READ uiBusy NOTIFY uiReportChanged)
    Q_PROPERTY(QVariantMap geodataHints READ geodataHints NOTIFY geodataHintsChanged)
    Q_PROPERTY(QVariantMap editingNode READ editingNode NOTIFY editingNodeChanged)
    Q_PROPERTY(QVariantMap adminLock READ adminLock NOTIFY adminLockChanged)
    Q_PROPERTY(int iconVersion READ iconVersion NOTIFY iconsChanged)
    Q_PROPERTY(bool hwAccel READ hwAccel NOTIFY uiReportChanged)
    Q_PROPERTY(int historyLimit READ historyLimit NOTIFY historyConfigChanged)
    Q_PROPERTY(bool historyLoading READ historyLoading NOTIFY historyConfigChanged)
    Q_PROPERTY(QVariantList connChartBins READ connChartBins NOTIFY connectionsChanged)
    Q_PROPERTY(bool dashboardChartInternetOnly READ dashboardChartInternetOnly WRITE setDashboardChartInternetOnly NOTIFY connectionsChanged)
    Q_PROPERTY(bool chartDataMode READ chartDataMode WRITE setChartDataMode NOTIFY chartDataModeChanged)
    Q_PROPERTY(int connAllowedTotal READ connAllowedTotal NOTIFY connectionsChanged)
    Q_PROPERTY(int connBlockedTotal READ connBlockedTotal NOTIFY connectionsChanged)
    Q_PROPERTY(int connInetAllowedTotal READ connInetAllowedTotal NOTIFY connectionsChanged)
    Q_PROPERTY(int connInetBlockedTotal READ connInetBlockedTotal NOTIFY connectionsChanged)
    Q_PROPERTY(int connInternetBlockedTotal READ connInternetBlockedTotal NOTIFY connectionsChanged)
    Q_PROPERTY(int connActiveTotal READ connActiveTotal NOTIFY connectionsChanged)
    Q_PROPERTY(int connTunneledTotal READ connTunneledTotal NOTIFY connectionsChanged)
    Q_PROPERTY(QString connRxStr READ connRxStr NOTIFY connectionsChanged)
    Q_PROPERTY(QString connTxStr READ connTxStr NOTIFY connectionsChanged)
    Q_PROPERTY(QString connInetRxStr READ connInetRxStr NOTIFY connectionsChanged)
    Q_PROPERTY(QString connInetTxStr READ connInetTxStr NOTIFY connectionsChanged)
    Q_PROPERTY(QVariantList connCountriesSummary READ connCountriesSummary NOTIFY connectionsChanged)
    Q_PROPERTY(QVariantList blockedAppsSummary READ blockedAppsSummary NOTIFY connectionsChanged)

    Q_PROPERTY(QString modalTitle READ modalTitle NOTIFY modalChanged)
    Q_PROPERTY(QString modalBody READ modalBody NOTIFY modalChanged)

public:
    explicit PortmasterBackend(bool startBackground = false, bool withTray = true, bool demoMode = false, QObject* parent = nullptr);
    ~PortmasterBackend() override;

    bool connected() const { return m_demoMode || m_api->isConnected(); }
    bool hasTray() const { return m_withTray && QSystemTrayIcon::isSystemTrayAvailable(); }
    QString transport() const { return m_demoMode ? QStringLiteral("Demo Mode (Offline)") : m_api->transportLabel(); }
    QString lang() const { return m_ru ? QStringLiteral("ru") : QStringLiteral("en"); }
    bool isRu() const { return m_ru; }

    bool globalVpn() const;
    bool globalSplit() const;
    bool autoConnectXray() const;
    bool historyEnabled() const;
    bool blockInternet() const;
    bool blockLan() const;
    bool blockP2p() const;
    bool blockInbound() const;
    bool preventDnsBypass() const;
    bool dnsNoInsecure() const;
    bool dnsIgnoreSystem() const;
    bool dnsUseStaleCache() const;
    QStringList dnsServers() const;
    QString dnsSummary() const;
    bool paused() const { return m_paused; }
    bool vpnPaused() const { return m_vpnPaused; }
    QString pauseUntil() const { return m_pauseUntil; }

    QVariantList activeApps() const { return m_activeAppsCache; }
    bool onlyNowApps() const { return m_onlyNowApps; }
    Q_INVOKABLE void setOnlyNowApps(bool onlyNow);
    QVariantList offlineApps() const { return m_offlineAppsCache; }
    QVariantList allApps() const { return m_allAppsCache; }
    QVariantMap selectedApp() const { return m_selectedAppMap; }
    QVariantList selectedAppConnections() const { return m_selectedAppConns; }
    QVariantList selectedAppSettings() const { return m_selectedAppSettings; }
    QVariantList selectedAppChartBins() const { return m_selectedAppChartBins; }
    bool appChartInternetOnly() const { return m_appChartInternetOnly; }
    int selectedAppInetAllowed() const { return m_selectedAppInetAllowed; }
    int selectedAppInetBlocked() const { return m_selectedAppInetBlocked; }
    int selectedAppAllAllowed() const { return m_selectedAppAllAllowed; }
    int selectedAppAllBlocked() const { return m_selectedAppAllBlocked; }
    Q_INVOKABLE void setAppChartInternetOnly(bool internetOnly);
    Q_INVOKABLE void setUiWindowVisible(bool visible);
    Q_INVOKABLE void enableDemoMode();
    Q_INVOKABLE void openDemoNodeEditor(const QString& nodeId);
    Q_INVOKABLE QVariantMap hitTestMapCountry(double nx, double ny) const;

    QObject* connectionsModel() const { return m_connModel; }
    int connectionsTotal() const { return static_cast<int>(m_connections.size()); }
    QVariantList settingsList() const { return m_settingsCache; }
    QVariantList alerts() const { return m_alertsCache; }
    QVariantList prompts() const { return m_promptsCache; }

    QVariantMap xrayStatus() const { return m_xrayStatusMap; }
    QVariantList xrayNodes() const { return m_xrayNodesList; }
    QVariantList xraySubs() const { return m_xraySubsList; }
    QVariantList worldPolygons() const { return m_worldPolygons; }
    QVariantList worldCountries() const { return m_worldCountries; }
    QVariantList mapPins() const { return m_mapPinsCache; }
    QVariantMap uiReport() const { return m_uiReportMap; }
    bool uiBusy() const { return m_uiBusy; }
    QVariantMap geodataHints() const { return m_geodataHints; }
    QVariantMap editingNode() const { return m_editingNode; }
    QVariantMap adminLock() const { return m_adminLock; }
    int iconVersion() const { return m_iconVersion; }
    bool hwAccel() const { return m_hwAccel; }
    int historyLimit() const { return m_historyLimit; }
    bool historyLoading() const { return m_historyFetching; }
    QVariantList connChartBins() const { return m_connChartBins; }
    bool dashboardChartInternetOnly() const { return m_dashboardChartInternetOnly; }
    Q_INVOKABLE void setDashboardChartInternetOnly(bool internetOnly);
    bool chartDataMode() const { return m_chartDataMode; }
    Q_INVOKABLE void setChartDataMode(bool dataMode);
    int connAllowedTotal() const { return m_connAllowedTotal; }
    int connBlockedTotal() const { return m_connBlockedTotal; }
    int connInetAllowedTotal() const { return m_connInetAllowedTotal; }
    int connInetBlockedTotal() const { return m_connInetBlockedTotal; }
    int connInternetBlockedTotal() const { return m_connInternetBlockedTotal; }
    int connActiveTotal() const { return m_connActiveTotal; }
    int connTunneledTotal() const { return m_connTunneledTotal; }
    QString connRxStr() const { return m_connRxStr; }
    QString connTxStr() const { return m_connTxStr; }
    QString connInetRxStr() const { return m_connInetRxStr; }
    QString connInetTxStr() const { return m_connInetTxStr; }
    QVariantList connCountriesSummary() const { return m_connCountriesSummary; }
    QVariantList blockedAppsSummary() const { return m_blockedAppsSummary; }

    QQuickImageProvider* createIconProvider() const;

    QString modalTitle() const { return m_modalTitle; }
    QString modalBody() const { return m_modalBody; }

    Q_INVOKABLE QString t(const QString& key) const;
    Q_INVOKABLE QString cfgText(const QString& key) const;
    Q_INVOKABLE void setLanguage(const QString& code);
    Q_INVOKABLE void selectApp(const QString& scopedKey);

    Q_INVOKABLE void setAppAction(const QString& scopedKey, const QString& action);
    Q_INVOKABLE void setAppVpnMode(const QString& scopedKey, const QString& mode); // default, vpn, direct
    Q_INVOKABLE void setAppSplitMode(const QString& scopedKey, const QString& mode); // default, on, off
    Q_INVOKABLE void setAppBlockLan(const QString& scopedKey, bool blocked);
    Q_INVOKABLE void setAppSetting(const QString& scopedKey, const QString& cfgKey, const QVariant& val);
    Q_INVOKABLE void resetAppSetting(const QString& scopedKey, const QString& cfgKey);
    Q_INVOKABLE void applyAppPreset(const QString& scopedKey, const QString& cfgKey, const QStringList& values, bool append);
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
    Q_INVOKABLE void setHistoryLimit(int limit);
    Q_INVOKABLE void refreshHistory();
    Q_INVOKABLE QString countryFlagUrl(const QString& code) const;
    Q_INVOKABLE QString countryName(const QString& code) const;

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
    Q_INVOKABLE void fetchGeodataHints();
    Q_INVOKABLE void openNodeEditor(const QString& nodeId);
    Q_INVOKABLE void clearEditingNode();
    Q_INVOKABLE void saveNodeEdit(const QString& nodeId, const QVariantMap& fields);
    Q_INVOKABLE void deleteNode(const QString& nodeId);
    Q_INVOKABLE void addGlobalRule(const QString& rule);

    Q_INVOKABLE void updateSetting(const QString& key, const QVariant& val);
    Q_INVOKABLE void resetSetting(const QString& key);
    Q_INVOKABLE void applyPreset(const QString& key, const QStringList& values, bool append);
    Q_INVOKABLE void moveSettingArrayItem(const QString& key, int index, int delta);
    Q_INVOKABLE void moveAppArrayItem(const QString& scopedKey, const QString& cfgKey, int index, int delta);
    Q_INVOKABLE void switchUiFlavor(const QString& flavorId);
    Q_INVOKABLE void installUiFlavor(const QString& flavorId);
    Q_INVOKABLE void uninstallUiFlavor(const QString& flavorId);
    Q_INVOKABLE void setKeepApiPortOpen(bool keep);
    Q_INVOKABLE void setHwAccel(bool enabled);
    Q_INVOKABLE void refreshAdminLock();
    Q_INVOKABLE void unlockAdmin(const QString& password);
    Q_INVOKABLE void lockAdmin();
    Q_INVOKABLE void configureAdminLock(const QVariantMap& payload);
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
    void chartDataModeChanged();
    void settingsChanged();
    void notificationsChanged();
    void xrayChanged();
    void worldMapReady();
    void uiReportChanged();
    void geodataHintsChanged();
    void editingNodeChanged();
    void adminLockChanged();
    void historyConfigChanged();
    void iconsChanged();
    void modalChanged();
    void showWindowRequested();
    void toastMessage(const QString& title, const QString& body, bool isError);

private slots:
    void onConnectionChanged(bool connected, const QString& transport);
    void refreshHttpData();

private:
    void startSubscriptions();
    void fetchWorldMap();
    void loadHistoryFromDb(bool forceReload = false);
    void fetchHistoryPage(int gen, const QString& db, int page, int targetLimit, QVector<ConnRecord> acc);
    void mergeDbConnections(const QVector<ConnRecord>& rows);
    void parseWorldMapJson(const QByteArray& body);
    void loadDemoData();
    void scheduleAppsRebuild();
    void scheduleAppsTickRebuild();
    void scheduleSettingsRebuild();
    void scheduleConnsRebuild();
    void rebuildAppsCache();
    void rebuildSelectedAppCache();
    void rebuildConnectionsCache();
    void rebuildChartBinsOnly();
    void rebuildSettingsCache();
    void rebuildNotificationsCache();
    void rebuildMapPins();
    struct WorldCountryHit {
        QString code;
        QString hitKey;
        QString name;
        double minX = 0.0;
        double maxX = 0.0;
        double minY = 0.0;
        double maxY = 0.0;
        QVector<QVector<QVector<double>>> polygons;
    };

    void setupTray();
    void updateTrayIcon();
    void updateTrayMenu();
    QIcon buildShieldIcon(const QColor& c) const;
    QIcon loadTrayShieldIcon(const QString& colorName) const;
    QJsonValue effectiveValue(const QString& key) const;
    QVariantMap buildSettingMap(const QJsonObject& option, const QJsonObject* appCfg) const;
    void loadI18nCatalog();
    void buildDesktopIconIndex();
    bool hasCachedIcon(const QString& profileKey) const;
    void queueIconResolve(const QString& profileKey, bool prioritize = false);
    void processNextIconBatch();
    QString formatBytes(qint64 bytes) const;

    bool m_demoMode = false;
    bool m_uiWindowVisible = true;
    bool m_appsDirty = false;
    bool m_connsDirty = false;
    bool m_appChartInternetOnly = false;
    bool m_chartDataMode = false;
    QString m_trayTheme = QStringLiteral("system");
    QMenu* m_trayMenu = nullptr;
    QAction* m_trayStatusAct = nullptr;
    QAction* m_trayPauseInfoAct = nullptr;
    QAction* m_trayResumeAct = nullptr;
    QMenu* m_trayPauseSubmenu = nullptr;
    QAction* m_trayPauseVpn5Act = nullptr;
    QAction* m_trayPauseVpn15Act = nullptr;
    QAction* m_trayPauseVpn60Act = nullptr;
    QAction* m_trayVpnStatusAct = nullptr;
    QAction* m_trayVpnToggleAct = nullptr;
    QAction* m_trayVpnAllAppsAct = nullptr;
    QAction* m_trayThemeSysAct = nullptr;
    QAction* m_trayThemeLightAct = nullptr;
    QAction* m_trayThemeDarkAct = nullptr;
    PortApiClient* m_api = nullptr;
    QSystemTrayIcon* m_tray = nullptr;
    QTimer* m_appsTimer = nullptr;
    QTimer* m_appsTickTimer = nullptr;
    QTimer* m_settingsTimer = nullptr;
    QTimer* m_connsTimer = nullptr;
    QTimer* m_chartTickTimer = nullptr;
    bool m_withTray = true;
    bool m_ru = true;
    bool m_hwAccel = true;
    bool m_onlyNowApps = false;
    bool m_dashboardChartInternetOnly = true;
    bool m_uiBusy = false;
    int m_historyLimit = 2000;
    bool m_historyLoaded = false;
    bool m_historyFetching = false;
    int m_historyFetchGen = 0;
    bool m_subscribed = false;
    bool m_worldLoaded = false;
    int m_uiPollTick = 0;

    QMap<QString, QJsonObject> m_profiles;
    QMap<QString, QJsonObject> m_settings;
    QMap<QString, QJsonObject> m_notifications;
    std::deque<ConnRecord> m_connections;
    ConnectionListModel* m_connModel = nullptr;
    qint64 m_connHeadSeq = 0;
    QHash<QString, qint64> m_connSeqById;
    QHash<qint64, QPair<qint64, qint64>> m_liveBwBins;
    QHash<qint64, QPair<qint64, qint64>> m_liveInetBwBins;
    QVector<WorldCountryHit> m_worldHitIndex;
    QByteArray m_lastXrayStatusBytes;
    QByteArray m_lastXrayNodesBytes;
    QByteArray m_lastXraySubsBytes;
    QByteArray m_lastUiReportBytes;
    QByteArray m_lastAdminLockBytes;

    QString m_selectedProfileKey;
    bool m_paused = false;
    bool m_vpnPaused = false;
    QString m_pauseUntil;

    QVariantList m_activeAppsCache;
    QVariantList m_offlineAppsCache;
    QVariantList m_allAppsCache;
    QVariantMap m_selectedAppMap;
    QVariantList m_selectedAppConns;
    QVariantList m_selectedAppSettings;
    QVariantList m_selectedAppChartBins;
    int m_selectedAppInetAllowed = 0;
    int m_selectedAppInetBlocked = 0;
    int m_selectedAppAllAllowed = 0;
    int m_selectedAppAllBlocked = 0;
    QVariantList m_connChartBins;
    int m_connAllowedTotal = 0;
    int m_connBlockedTotal = 0;
    int m_connInetAllowedTotal = 0;
    int m_connInetBlockedTotal = 0;
    int m_connInternetBlockedTotal = 0;
    int m_connActiveTotal = 0;
    int m_connTunneledTotal = 0;
    QString m_connRxStr = QStringLiteral("0 B");
    QString m_connTxStr = QStringLiteral("0 B");
    QString m_connInetRxStr = QStringLiteral("0 B");
    QString m_connInetTxStr = QStringLiteral("0 B");
    QVariantList m_connCountriesSummary;
    QVariantList m_blockedAppsSummary;
    QHash<QString, QString> m_countryNames;
    QVariantList m_settingsCache;
    QVariantList m_alertsCache;
    QVariantList m_promptsCache;

    QVariantMap m_xrayStatusMap;
    QVariantList m_xrayNodesList;
    QVariantList m_xraySubsList;
    QVariantList m_worldPolygons;
    QVariantList m_worldCountries;
    QVariantList m_mapPinsCache;
    QVariantMap m_uiReportMap;
    QVariantMap m_geodataHints;
    QVariantMap m_editingNode;
    QVariantMap m_adminLock;

    QSharedPointer<PortmasterIconStore> m_iconStore;
    QHash<QString, QString> m_desktopIcons;
    QSet<QString> m_iconPending;
    QSet<QString> m_iconQueued;
    QStringList m_iconQueue;
    QSet<QString> m_iconAttempted;
    QTimer* m_iconBatchTimer = nullptr;
    QTimer* m_iconNotifyTimer = nullptr;
    int m_iconVersion = 0;

    QMap<QString, QString> m_i18nRu;
    QMap<QString, QString> m_i18nEn;

    QString m_modalTitle;
    QString m_modalBody;
};
