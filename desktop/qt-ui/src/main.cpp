#include <cstdio>

#include <QApplication>
#include <QDir>
#include <QJsonDocument>
#include <QJsonObject>
#include <QLocalServer>
#include <QLocalSocket>
#include <QQmlApplicationEngine>
#include <QQmlContext>
#include <QQuickStyle>
#include <QTimer>

#include "PortmasterBackend.h"

int main(int argc, char* argv[]) {
    QQuickStyle::setStyle(QStringLiteral("Basic"));
    QApplication app(argc, argv);
    app.setApplicationName(QStringLiteral("portmaster-qt"));
    app.setOrganizationName(QStringLiteral("Portmaster"));
    app.setQuitOnLastWindowClosed(false);

    bool background = false;
    bool noTray = false;
    bool selfcheck = false;
    for (int i = 1; i < argc; ++i) {
        if (qstrcmp(argv[i], "--background") == 0) {
            background = true;
        } else if (qstrcmp(argv[i], "--no-tray") == 0) {
            noTray = true;
        } else if (qstrcmp(argv[i], "--selfcheck") == 0) {
            selfcheck = true;
            noTray = true;
            background = true;
        }
    }

    QString rtDir = qEnvironmentVariable("XDG_RUNTIME_DIR").trimmed();
    if (rtDir.isEmpty()) rtDir = QDir::tempPath();
    const QString ipcName = rtDir + QStringLiteral("/portmaster-qt-ui.sock");
    if (!selfcheck) {
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
    if (!selfcheck) {
        QLocalServer::removeServer(ipcName);
        ipcServer.listen(ipcName);
    }

    PortmasterBackend backend(background, !noTray);
    if (!selfcheck) {
        QObject::connect(&ipcServer, &QLocalServer::newConnection, &backend, [&ipcServer, &backend]() {
            while (auto* conn = ipcServer.nextPendingConnection()) {
                conn->readAll();
                conn->deleteLater();
                emit backend.showWindowRequested();
            }
        });
    }

    QQmlApplicationEngine engine;
    engine.rootContext()->setContextProperty(QStringLiteral("pm"), &backend);
    engine.rootContext()->setContextProperty(QStringLiteral("startHidden"), background);
    engine.load(QUrl(QStringLiteral("qrc:/qt/qml/Portmaster/Main.qml")));

    if (engine.rootObjects().isEmpty()) {
        return -1;
    }

    if (selfcheck) {
        QTimer::singleShot(2500, &app, [&backend, &app]() {
            int boolOpts = 0;
            for (const auto& sv : backend.settingsList()) {
                if (sv.toMap().value(QStringLiteral("optType")).toInt() == 4) ++boolOpts;
            }
            int nonAutoApps = 0;
            for (const auto& av : backend.allApps()) {
                if (av.toMap().value(QStringLiteral("vpnMode")).toString() != QLatin1String("default")) ++nonAutoApps;
            }
            QJsonObject out{
                {QStringLiteral("connected"), backend.connected()},
                {QStringLiteral("transport"), backend.transport()},
                {QStringLiteral("apps"), backend.allApps().size()},
                {QStringLiteral("activeApps"), backend.activeApps().size()},
                {QStringLiteral("nonAutoApps"), nonAutoApps},
                {QStringLiteral("connections"), backend.connections().size()},
                {QStringLiteral("settings"), backend.settingsList().size()},
                {QStringLiteral("boolSettings"), boolOpts},
                {QStringLiteral("xrayConnected"), backend.xrayStatus().value(QStringLiteral("connected")).toBool()},
                {QStringLiteral("xrayNodeName"), backend.xrayStatus().value(QStringLiteral("nodeName")).toString()},
                {QStringLiteral("xrayNodes"), backend.xrayNodes().size()},
                {QStringLiteral("mapPins"), backend.mapPins().size()},
                {QStringLiteral("worldPolygons"), backend.worldPolygons().size()},
            };
            std::printf("%s\n", QJsonDocument(out).toJson(QJsonDocument::Compact).constData());
            std::fflush(stdout);
            app.quit();
        });
    }

    return app.exec();
}
