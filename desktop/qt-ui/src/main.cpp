#include <QApplication>
#include <QQmlApplicationEngine>
#include <QQmlContext>
#include <QQuickStyle>

#include "PortmasterBackend.h"

int main(int argc, char* argv[]) {
    QQuickStyle::setStyle(QStringLiteral("Basic"));
    QApplication app(argc, argv);
    app.setApplicationName(QStringLiteral("portmaster-qt"));
    app.setOrganizationName(QStringLiteral("Portmaster"));
    app.setQuitOnLastWindowClosed(false);

    bool background = false;
    for (int i = 1; i < argc; ++i) {
        if (qstrcmp(argv[i], "--background") == 0) {
            background = true;
        }
    }

    PortmasterBackend backend(background);
    QQmlApplicationEngine engine;
    engine.rootContext()->setContextProperty(QStringLiteral("pm"), &backend);
    engine.rootContext()->setContextProperty(QStringLiteral("startHidden"), background);
    engine.load(QUrl(QStringLiteral("qrc:/qt/qml/Portmaster/Main.qml")));

    if (engine.rootObjects().isEmpty()) {
        return -1;
    }
    return app.exec();
}
