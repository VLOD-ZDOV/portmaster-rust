import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

ApplicationWindow {
    id: win
    visible: !startHidden
    width: 1280
    height: 760
    minimumWidth: 880
    minimumHeight: 580
    title: "Portmaster"
    color: "#121213"

    property string currentPage: "apps" // "apps" | "appDetail" | "monitor" | "xray" | "settings"
    property string returnPage: "apps"
    property bool sideDashOpen: false
    property string toastBanner: ""
    property bool toastErr: false

    onClosing: function(close) {
        if (!pm.hasTray) {
            close.accepted = true
            pm.quitApp()
            return
        }
        close.accepted = false
        win.hide()
    }

    Connections {
        target: pm
        function onShowWindowRequested() {
            win.show()
            win.raise()
            win.requestActivate()
        }
        function onToastMessage(title, body, isErr) {
            win.toastBanner = title + ": " + body
            win.toastErr = isErr
            toastTimer.restart()
        }
    }

    Timer {
        id: toastTimer
        interval: 4000
        onTriggered: win.toastBanner = ""
    }

    RowLayout {
        anchors.fill: parent
        spacing: 0

        NavigationBar {
            Layout.fillHeight: true
            Layout.preferredWidth: 64
            currentPage: win.currentPage
            sideDashOpen: win.sideDashOpen
            onNavigate: function(page) {
                if (page !== "appDetail") pm.selectApp("")
                win.currentPage = page
            }
            onToggleSideDash: win.sideDashOpen = !win.sideDashOpen
        }

        SideDash {
            Layout.fillHeight: true
            Layout.preferredWidth: win.sideDashOpen ? 290 : 0
            visible: win.sideDashOpen
            onOpenDashboard: {
                pm.selectApp("")
                win.currentPage = "dashboard"
            }
            onOpenApp: function(key) {
                win.returnPage = (win.currentPage !== "appDetail") ? win.currentPage : win.returnPage
                pm.selectApp(key)
                win.currentPage = "appDetail"
            }
            onOpenMonitor: {
                win.currentPage = "monitor"
            }
            onOpenXray: {
                win.currentPage = "xray"
            }
            onOpenNetworkSettings: {
                pm.selectApp("")
                settingsView.selectedSub = "network"
                win.currentPage = "settings"
            }
        }

        // Main Workspace
        Rectangle {
            Layout.fillWidth: true
            Layout.fillHeight: true
            color: "#121213"

            ColumnLayout {
                anchors.fill: parent
                spacing: 0

                // Thin top transport / toast status bar (only shows details subtly like original)
                Rectangle {
                    Layout.fillWidth: true
                    height: 28
                    color: "#121213"
                    visible: win.toastBanner !== "" || !pm.connected

                    RowLayout {
                        anchors.fill: parent
                        anchors.leftMargin: 24
                        anchors.rightMargin: 24
                        Label {
                            text: win.toastBanner !== "" ? win.toastBanner : pm.t("dash.offline")
                            color: (win.toastErr || !pm.connected) ? "#e01d1d" : "#1de966"
                            font.pixelSize: 11
                            font.bold: true
                        }
                    }
                }

                // Page container matching exact original Portmaster padding
                StackLayout {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    currentIndex: {
                        if (win.currentPage === "dashboard") return 0
                        if (win.currentPage === "monitor") return 1
                        if (win.currentPage === "apps") return 2
                        if (win.currentPage === "appDetail") return 3
                        if (win.currentPage === "xray") return 4
                        return 5
                    }

                    DashboardView {
                        onOpenMonitorPreset: function(internetOnly, verdict, query) {
                            monitorView.scopeInternet = internetOnly
                            monitorView.scopeLan = false
                            monitorView.scopeLocal = false
                            monitorView.dirFilter = "all"
                            monitorView.verdictFilter = verdict
                            monitorView.applySearch(query)
                            win.currentPage = "monitor"
                        }
                        onOpenAppsActive: {
                            appsView.routingFilter = "active"
                            win.currentPage = "apps"
                        }
                        onOpenApp: function(key) {
                            win.returnPage = "dashboard"
                            pm.selectApp(key)
                            win.currentPage = "appDetail"
                        }
                        onOpenXray: {
                            pm.selectApp("")
                            win.currentPage = "xray"
                        }
                        onOpenSettingSub: function(sub) {
                            pm.selectApp("")
                            settingsView.selectedSub = sub
                            win.currentPage = "settings"
                        }
                    }

                    MonitorView {
                        id: monitorView
                        onOpenApp: function(key) {
                            win.returnPage = "monitor"
                            pm.selectApp(key)
                            win.currentPage = "appDetail"
                        }
                    }

                    AppsView {
                        id: appsView
                        onOpenApp: function(key) {
                            win.returnPage = "apps"
                            pm.selectApp(key)
                            win.currentPage = "appDetail"
                        }
                        onOpenNetworkSettings: {
                            pm.selectApp("")
                            settingsView.selectedSub = "network"
                            win.currentPage = "settings"
                        }
                    }

                    AppDetailView {
                        onGoBack: {
                            pm.selectApp("")
                            win.currentPage = win.returnPage
                        }
                        onOpenNetworkSettings: {
                            pm.selectApp("")
                            settingsView.selectedSub = "network"
                            win.currentPage = "settings"
                        }
                    }

                    XrayView {}

                    SettingsView { id: settingsView }
                }
            }
        }
    }

    // Connecting Overlay matching Angular app.component.html exact rotating rings when offline
    Rectangle {
        anchors.fill: parent
        anchors.leftMargin: 64
        color: "#e6121213"
        visible: !pm.connected
        z: 50

        ColumnLayout {
            anchors.centerIn: parent
            spacing: 14

            Canvas {
                id: overlayLogo
                Layout.alignment: Qt.AlignHCenter
                width: 80
                height: 80
                property real angle: 0
                NumberAnimation on angle {
                    from: 0; to: 360; duration: 3500; loops: Animation.Infinite; running: !pm.connected
                }
                onAngleChanged: requestPaint()
                onPaint: {
                    var ctx = getContext("2d")
                    ctx.clearRect(0, 0, width, height)
                    ctx.translate(width / 2, height / 2)
                    ctx.strokeStyle = "#ffffff"
                    ctx.lineWidth = 5
                    ctx.beginPath(); ctx.arc(0, 0, 32, 0, Math.PI * 2); ctx.stroke()
                    ctx.save(); ctx.rotate(-angle * Math.PI / 180)
                    ctx.globalAlpha = 0.8; ctx.strokeStyle = "#d12e2e"; ctx.lineWidth = 4
                    ctx.beginPath(); ctx.arc(0, 0, 22, 0.2, 2.2); ctx.stroke()
                    ctx.beginPath(); ctx.arc(0, 0, 22, 3.3, 5.3); ctx.stroke()
                    ctx.restore()
                    ctx.save(); ctx.rotate(angle * Math.PI / 180)
                    ctx.globalAlpha = 0.6; ctx.strokeStyle = "#d12e2e"; ctx.lineWidth = 3.5
                    ctx.beginPath(); ctx.arc(0, 0, 13, 0.4, 3.8); ctx.stroke()
                    ctx.restore()
                    ctx.setTransform(1, 0, 0, 1, 0, 0)
                }
            }

            Label {
                Layout.alignment: Qt.AlignHCenter
                text: pm.t("dash.offline")
                color: "#ffffff"
                font.pixelSize: 18
                font.weight: Font.Medium
            }

            Label {
                Layout.alignment: Qt.AlignHCenter
                text: pm.transport !== "" ? pm.transport : "/run/portmaster/api.sock / 127.0.0.1:817"
                color: "#888888"
                font.pixelSize: 11
                font.family: "monospace"
            }

            RowLayout {
                Layout.alignment: Qt.AlignHCenter
                spacing: 10
                Button {
                    text: "Close UI"
                    onClicked: pm.quitApp()
                    background: Rectangle { color: "#222222"; border.color: "#474747"; radius: 4 }
                    contentItem: Text { text: parent.text; color: "#ffffff"; font.pixelSize: 12; horizontalAlignment: Text.AlignHCenter; verticalAlignment: Text.AlignVCenter; padding: 6 }
                }
                Button {
                    text: "Shut Down Portmaster"
                    onClicked: pm.shutdownCore()
                    background: Rectangle { color: "#3d1414"; border.color: "#811818"; radius: 4 }
                    contentItem: Text { text: parent.text; color: "#ff8888"; font.pixelSize: 12; horizontalAlignment: Text.AlignHCenter; verticalAlignment: Text.AlignVCenter; padding: 6 }
                }
            }
        }
    }

    // Export Modal Overlay
    Rectangle {
        anchors.fill: parent
        color: "#aa000000"
        visible: pm.modalTitle !== ""
        z: 100
        MouseArea { anchors.fill: parent; onClicked: pm.closeModal() }

        Rectangle {
            anchors.centerIn: parent
            width: Math.min(parent.width - 80, 680)
            height: Math.min(parent.height - 80, 480)
            color: "#1b1b1b"
            border.color: "#2c2c2c"
            radius: 6
            MouseArea { anchors.fill: parent }

            ColumnLayout {
                anchors.fill: parent
                anchors.margins: 16
                spacing: 10

                RowLayout {
                    Layout.fillWidth: true
                    Label {
                        text: pm.modalTitle
                        color: "#ffffff"
                        font.pixelSize: 14
                        font.bold: true
                        Layout.fillWidth: true
                    }
                    Button {
                        text: "✕"
                        onClicked: pm.closeModal()
                        background: Rectangle { color: "#222222"; radius: 4 }
                        contentItem: Text { text: parent.text; color: "#ababab"; font.pixelSize: 12; horizontalAlignment: Text.AlignHCenter; verticalAlignment: Text.AlignVCenter }
                    }
                }

                ScrollView {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    TextArea {
                        readOnly: true
                        selectByMouse: true
                        text: pm.modalBody
                        color: "#e5e7eb"
                        font.family: "monospace"
                        font.pixelSize: 11
                        background: Rectangle { color: "#121213"; border.color: "#2c2c2c"; radius: 4 }
                    }
                }
            }
        }
    }
}
