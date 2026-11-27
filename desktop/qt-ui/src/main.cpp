#include <cstdio>
#include <functional>
#include <memory>

#include <QApplication>
#include <QDir>
#include <QImage>
#include <QJsonDocument>
#include <QJsonObject>
#include <QLocalServer>
#include <QLocalSocket>
#include <QQmlApplicationEngine>
#include <QQmlContext>
#include <QQuickStyle>
#include <QQuickWindow>
#include <QSettings>
#include <QSurfaceFormat>
#include <QTimer>
#include <QVector>

#include "PortmasterBackend.h"

int main(int argc, char* argv[]) {
    bool background = false;
    bool noTray = false;
    bool selfcheck = false;
    bool demo = false;
    QString screenshotDir;
    QString demoLang;
    for (int i = 1; i < argc; ++i) {
        if (qstrcmp(argv[i], "--background") == 0) {
            background = true;
        } else if (qstrcmp(argv[i], "--no-tray") == 0) {
            noTray = true;
        } else if (qstrcmp(argv[i], "--demo") == 0) {
            demo = true;
        } else if (qstrcmp(argv[i], "--lang") == 0 && i + 1 < argc) {
            demoLang = QString::fromLocal8Bit(argv[++i]).toLower();
        } else if (qstrcmp(argv[i], "--screenshot") == 0 && i + 1 < argc) {
            screenshotDir = QString::fromLocal8Bit(argv[++i]);
            demo = true;
            noTray = true;
            background = false;
        } else if (qstrcmp(argv[i], "--selfcheck") == 0) {
            selfcheck = true;
            noTray = true;
            background = true;
        }
    }

    QQuickStyle::setStyle(QStringLiteral("Basic"));
    const bool hwAccel = QSettings(QStringLiteral("portmaster"), QStringLiteral("qt-ui")).value(QStringLiteral("hwAccel"), true).toBool();
    if (!hwAccel || !screenshotDir.isEmpty()) {
        QQuickWindow::setGraphicsApi(QSGRendererInterface::Software);
    } else {
        // Request native display VSync pacing (e.g. 165 Hz) when using GPU SceneGraph.
        QSurfaceFormat fmt = QSurfaceFormat::defaultFormat();
        fmt.setSwapInterval(1);
        QSurfaceFormat::setDefaultFormat(fmt);
    }
    QApplication app(argc, argv);
    app.setApplicationName(QStringLiteral("portmaster-qt"));
    app.setOrganizationName(QStringLiteral("Portmaster"));
    app.setQuitOnLastWindowClosed(false);

    const bool isolatedMode = selfcheck || demo || !screenshotDir.isEmpty();
    QString rtDir = qEnvironmentVariable("XDG_RUNTIME_DIR").trimmed();
    if (rtDir.isEmpty()) rtDir = QDir::tempPath();
    const QString ipcName = rtDir + QStringLiteral("/portmaster-qt-ui.sock");
    if (!isolatedMode) {
        QLocalSocket probe;
        probe.connectToServer(ipcName);
        if (probe.waitForConnected(250)) {
            if (!background) {
                probe.write("show\n");
                probe.waitForBytesWritten(250);
            }
            return 0;
        }
    }

    QLocalServer ipcServer;
    if (!isolatedMode) {
        QLocalServer::removeServer(ipcName);
        ipcServer.listen(ipcName);
    }

    PortmasterBackend backend(background, !noTray, demo);
    if ((demo || selfcheck) && (demoLang == QLatin1String("ru") || demoLang == QLatin1String("en"))) {
        backend.setLanguage(demoLang);
    }
    if (!isolatedMode) {
        QObject::connect(&ipcServer, &QLocalServer::newConnection, &backend, [&ipcServer, &backend]() {
            while (auto* conn = ipcServer.nextPendingConnection()) {
                conn->readAll();
                conn->deleteLater();
                emit backend.showWindowRequested();
            }
        });
    }

    QQmlApplicationEngine engine;
    engine.addImageProvider(QStringLiteral("appicon"), backend.createIconProvider());
    engine.rootContext()->setContextProperty(QStringLiteral("pm"), &backend);
    engine.rootContext()->setContextProperty(QStringLiteral("startHidden"), background);
    engine.load(QUrl(QStringLiteral("qrc:/qt/qml/Portmaster/Main.qml")));

    if (engine.rootObjects().isEmpty()) {
        return -1;
    }

    if (!screenshotDir.isEmpty()) {
        QDir().mkpath(screenshotDir);
        QObject* rootObj = engine.rootObjects().first();
        auto* win = qobject_cast<QQuickWindow*>(rootObj);
        if (!win) return -1;
        backend.setUiWindowVisible(true);
        win->setWidth(1280);
        win->setHeight(820);
        win->show();

        struct ShotStep {
            QString page;
            QString appKey;
            QString nodeEdit;
            bool dataMode;
            QString file;
        };
        const QVector<ShotStep> steps{
            {QStringLiteral("dashboard"), QString(), QString(), false, QStringLiteral("qt-dashboard.png")},
            {QStringLiteral("dashboard"), QString(), QString(), true,  QStringLiteral("qt-dashboard-data.png")},
            {QStringLiteral("monitor"),   QString(), QString(), false, QStringLiteral("qt-monitor.png")},
            {QStringLiteral("apps"),      QString(), QString(), false, QStringLiteral("qt-apps.png")},
            {QStringLiteral("appDetail"), QStringLiteral("core:profiles/local/browser-demo"), QString(), true, QStringLiteral("qt-app-detail.png")},
            {QStringLiteral("xray"),      QString(), QString(), false, QStringLiteral("qt-xray.png")},
            {QStringLiteral("xray"),      QString(), QStringLiteral("nl-01"), false, QStringLiteral("qt-node-editor.png")},
            {QStringLiteral("settings"),  QString(), QString(), false, QStringLiteral("qt-settings.png")},
        };
        auto runStep = std::make_shared<std::function<void(int)>>();
        *runStep = [&app, &backend, rootObj, win, screenshotDir, steps, runStep](int idx) {
            if (idx >= steps.size()) {
                app.quit();
                return;
            }
            const ShotStep& st = steps[idx];
            backend.setChartDataMode(st.dataMode);
            if (!st.appKey.isEmpty()) backend.selectApp(st.appKey);
            rootObj->setProperty("currentPage", st.page);
            if (!st.nodeEdit.isEmpty()) backend.openDemoNodeEditor(st.nodeEdit);
            else backend.clearEditingNode();
            QTimer::singleShot(450, win, [&backend, win, screenshotDir, st, idx, runStep]() {
                const QImage img = win->grabWindow();
                img.save(screenshotDir + QLatin1Char('/') + st.file);
                if (!st.nodeEdit.isEmpty()) backend.clearEditingNode();
                (*runStep)(idx + 1);
            });
        };
        QTimer::singleShot(300, win, [runStep]() { (*runStep)(0); });
        return app.exec();
    }

    if (selfcheck) {
        backend.setUiWindowVisible(true);
        QTimer::singleShot(2500, &app, [&backend, &app]() {
            int boolOpts = 0;
            for (const auto& sv : backend.settingsList()) {
                if (sv.toMap().value(QStringLiteral("optType")).toInt() == 4) ++boolOpts;
            }
            int nonAutoApps = 0;
            int resolvedIcons = 0;
            for (const auto& av : backend.allApps()) {
                const auto m = av.toMap();
                if (m.value(QStringLiteral("vpnMode")).toString() != QLatin1String("default")) ++nonAutoApps;
                if (m.value(QStringLiteral("hasIcon")).toBool()) ++resolvedIcons;
            }
            int perAppOpts = 0;
            if (!backend.allApps().isEmpty()) {
                backend.selectApp(backend.allApps().first().toMap().value(QStringLiteral("key")).toString());
                perAppOpts = backend.selectedAppSettings().size();
            }
            QJsonObject out{
                {QStringLiteral("connected"), backend.connected()},
                {QStringLiteral("transport"), backend.transport()},
                {QStringLiteral("apps"), backend.allApps().size()},
                {QStringLiteral("activeApps"), backend.activeApps().size()},
                {QStringLiteral("nonAutoApps"), nonAutoApps},
                {QStringLiteral("connections"), backend.connectionsTotal()},
                {QStringLiteral("settings"), backend.settingsList().size()},
                {QStringLiteral("boolSettings"), boolOpts},
                {QStringLiteral("perAppSettings"), perAppOpts},
                {QStringLiteral("blockInternet"), backend.blockInternet()},
                {QStringLiteral("dnsSummary"), backend.dnsSummary()},
                {QStringLiteral("xrayConnected"), backend.xrayStatus().value(QStringLiteral("connected")).toBool()},
                {QStringLiteral("xrayNodeName"), backend.xrayStatus().value(QStringLiteral("nodeName")).toString()},
                {QStringLiteral("xrayNodes"), backend.xrayNodes().size()},
                {QStringLiteral("mapPins"), backend.mapPins().size()},
                {QStringLiteral("worldPolygons"), backend.worldPolygons().size()},
                {QStringLiteral("worldCountries"), backend.worldCountries().size()},
                {QStringLiteral("resolvedIcons"), resolvedIcons},
                {QStringLiteral("hwAccel"), backend.hwAccel()},
                {QStringLiteral("historyLimit"), backend.historyLimit()},
                {QStringLiteral("connChartBins"), backend.connChartBins().size()},
                {QStringLiteral("selectedAppChartBins"), backend.selectedAppChartBins().size()},
                {QStringLiteral("connAllowedTotal"), backend.connAllowedTotal()},
                {QStringLiteral("connBlockedTotal"), backend.connBlockedTotal()},
            };
            std::printf("%s\n", QJsonDocument(out).toJson(QJsonDocument::Compact).constData());
            std::fflush(stdout);
            app.quit();
        });
    }

    return app.exec();
}
