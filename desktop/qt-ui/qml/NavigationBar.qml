import QtQuick
import QtQuick.Controls
import QtQuick.Controls.Basic as B
import QtQuick.Layouts

Rectangle {
    id: root
    color: "#121213"

    property string currentPage: "apps"
    property bool sideDashOpen: false
    signal navigate(string page)
    signal toggleSideDash()

    component PmMenu: B.Menu {
        background: Rectangle {
            implicitWidth: 210
            color: "#1b1b1e"
            border.color: "#2e2e36"
            radius: 6
        }
        delegate: B.MenuItem {
            id: mi
            implicitWidth: 210
            implicitHeight: 34
            contentItem: Label {
                text: mi.text
                font.pixelSize: 12
                color: mi.enabled ? (mi.highlighted ? "#ffffff" : "#d4d4dc") : "#5e5e66"
                verticalAlignment: Text.AlignVCenter
                leftPadding: 10
            }
            background: Rectangle {
                radius: 4
                color: mi.highlighted && mi.enabled ? "#2a2d38" : "transparent"
            }
        }
    }

    Rectangle {
        anchors.right: parent.right
        anchors.top: parent.top
        anchors.bottom: parent.bottom
        width: 1
        color: "#2c2c2c"
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.topMargin: 6
        anchors.bottomMargin: 16
        spacing: 4

        // TOP: Security Shield (collapsed) OR Portmaster Ring Logo (expanded) -> Opens Dashboard
        Rectangle {
            Layout.alignment: Qt.AlignHCenter
            width: 52
            height: 52
            radius: 12
            color: (root.currentPage === "dashboard" && root.sideDashOpen) ? "#23252c" : (topMa.containsMouse ? "#1e1f24" : "transparent")
            ToolTip.visible: topMa.containsMouse
            ToolTip.text: pm.isRu ? "Панель управления (Dashboard)" : "Dashboard"
            ToolTip.delay: 300

            SecurityShield {
                anchors.centerIn: parent
                width: 40
                height: 40
                cutoutColor: "#121213"
                visible: !root.sideDashOpen
            }

            // Static crisp vector Portmaster Ring mark when SideDash is open (zero idle Canvas repaints!)
            Canvas {
                id: topRingCanvas
                anchors.centerIn: parent
                width: 30
                height: 30
                visible: root.sideDashOpen
                Connections {
                    target: pm
                    function onConnectionChanged() { topRingCanvas.requestPaint() }
                }
                onPaint: {
                    var ctx = getContext("2d")
                    ctx.clearRect(0, 0, width, height)
                    ctx.translate(width / 2, height / 2)
                    ctx.strokeStyle = "#ffffff"
                    ctx.lineWidth = 2.4
                    ctx.beginPath(); ctx.arc(0, 0, 12, 0, Math.PI * 2); ctx.stroke()
                    ctx.strokeStyle = pm.connected ? "#1de966" : "#d12e2e"
                    ctx.lineWidth = 2.0
                    ctx.beginPath(); ctx.arc(0, 0, 8, 0.3, 2.4); ctx.stroke()
                    ctx.beginPath(); ctx.arc(0, 0, 8, 3.4, 5.5); ctx.stroke()
                    ctx.strokeStyle = "#ffffff"
                    ctx.globalAlpha = 0.7
                    ctx.lineWidth = 1.8
                    ctx.beginPath(); ctx.arc(0, 0, 4.5, 0.5, 4.0); ctx.stroke()
                }
                Component.onCompleted: requestPaint()
            }

            MouseArea {
                id: topMa
                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: root.navigate("dashboard")
            }
        }

        // Chevron Toggle SideDash
        Rectangle {
            Layout.alignment: Qt.AlignHCenter
            width: 28; height: 28; radius: 8
            color: root.sideDashOpen ? "#2c2c2c" : "#222222"
            Canvas {
                anchors.centerIn: parent; width: 12; height: 12
                onPaint: {
                    var ctx = getContext("2d")
                    ctx.clearRect(0, 0, width, height)
                    ctx.strokeStyle = "#ffffff"; ctx.lineWidth = 2; ctx.lineCap = "round"; ctx.lineJoin = "round"
                    ctx.beginPath()
                    if (root.sideDashOpen) {
                        ctx.moveTo(8, 2); ctx.lineTo(4, 6); ctx.lineTo(8, 10)
                    } else {
                        ctx.moveTo(4, 2); ctx.lineTo(8, 6); ctx.lineTo(4, 10)
                    }
                    ctx.stroke()
                }
                Connections {
                    target: root
                    function onSideDashOpenChanged() { parent.requestPaint() }
                }
            }
            MouseArea {
                anchors.fill: parent
                cursorShape: Qt.PointingHandCursor
                onClicked: root.toggleSideDash()
            }
        }

        Rectangle { Layout.alignment: Qt.AlignHCenter; width: 32; height: 1; color: "#2c2c2c" }

        // Bell / Prompt indicator
        Rectangle {
            Layout.alignment: Qt.AlignHCenter
            width: 34; height: 34; radius: 10
            color: bellMa.containsMouse ? "#222222" : "transparent"
            Canvas {
                id: bellCanvas
                anchors.centerIn: parent; width: 18; height: 18
                onPaint: {
                    var ctx = getContext("2d")
                    ctx.clearRect(0, 0, width, height)
                    ctx.strokeStyle = pm.prompts.length > 0 ? "#e9d81d" : (pm.alerts.length > 0 ? "#4e97fa" : "#888888")
                    ctx.lineWidth = 1.6; ctx.lineCap = "round"; ctx.lineJoin = "round"
                    ctx.beginPath()
                    ctx.arc(9, 8, 4.5, Math.PI, 0)
                    ctx.lineTo(14.5, 13.5); ctx.lineTo(3.5, 13.5); ctx.closePath()
                    ctx.stroke()
                    ctx.beginPath(); ctx.arc(9, 15, 1.3, 0, Math.PI); ctx.stroke()
                }
                Connections {
                    target: pm
                    function onNotificationsChanged() { bellCanvas.requestPaint() }
                }
            }
            Rectangle {
                visible: pm.prompts.length > 0 || pm.alerts.length > 0
                width: 6; height: 6; radius: 3
                anchors.top: parent.top; anchors.right: parent.right; anchors.margins: 4
                color: pm.prompts.length > 0 ? "#e9d81d" : "#4e97fa"
            }
            MouseArea {
                id: bellMa; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor
                onClicked: root.toggleSideDash()
            }
        }

        Item { Layout.fillHeight: true }

        // MAIN NAVIGATION LINKS (Monitor, Apps, Xray, Settings) matching exact Angular SVGs
        component NavLinkBtn: Rectangle {
            property string pageId: ""
            property string tip: ""
            property int iconKind: 0
            property bool isActive: root.currentPage === pageId || (pageId === "apps" && root.currentPage === "appDetail")
            Layout.alignment: Qt.AlignHCenter
            width: 36; height: 36; radius: 10
            color: isActive ? "#2c2c2c" : (ma.containsMouse ? "#222222" : "transparent")
            ToolTip.visible: ma.containsMouse
            ToolTip.text: tip
            ToolTip.delay: 350

            Canvas {
                anchors.centerIn: parent; width: 18; height: 18
                opacity: isActive || ma.containsMouse ? 1.0 : 0.55
                onOpacityChanged: requestPaint()
                onPaint: {
                    var ctx = getContext("2d")
                    ctx.clearRect(0, 0, width, height)
                    ctx.strokeStyle = isActive && pageId === "xray" && pm.xrayStatus.connected ? "#4e97fa" : "#ffffff"
                    ctx.fillStyle = "#ffffff"
                    ctx.lineWidth = 1.75
                    ctx.lineCap = "round"
                    ctx.lineJoin = "round"

                    if (iconKind === 0) {
                        // Radar / Monitor
                        ctx.beginPath(); ctx.arc(9, 9, 6.8, 0.9, 5.2); ctx.stroke()
                        ctx.beginPath(); ctx.arc(9, 9, 3.6, 1.3, 4.7); ctx.stroke()
                        ctx.beginPath(); ctx.moveTo(4.2, 4.2); ctx.lineTo(9, 9); ctx.stroke()
                    } else if (iconKind === 1) {
                        // 2x2 Apps grid (bottom-right filled like original)
                        ctx.strokeRect(2.5, 2.5, 5, 5)
                        ctx.strokeRect(10.5, 2.5, 5, 5)
                        ctx.strokeRect(2.5, 10.5, 5, 5)
                        ctx.fillRect(10.5, 10.5, 5.2, 5.2)
                    } else if (iconKind === 2) {
                        // Xray Globe
                        ctx.beginPath(); ctx.arc(9, 9, 6.8, 0, Math.PI * 2); ctx.stroke()
                        ctx.beginPath(); ctx.moveTo(2.2, 9); ctx.lineTo(15.8, 9); ctx.stroke()
                        ctx.beginPath(); ctx.ellipse(5.5, 2.2, 7, 13.6); ctx.stroke()
                    } else if (iconKind === 3) {
                        // Settings Gear
                        ctx.beginPath(); ctx.arc(9, 9, 2.6, 0, Math.PI * 2); ctx.stroke()
                        ctx.beginPath(); ctx.arc(9, 9, 6.0, 0, Math.PI * 2); ctx.stroke()
                        for (var a = 0; a < 8; a++) {
                            var rad = a * Math.PI / 4
                            ctx.beginPath()
                            ctx.moveTo(9 + Math.cos(rad) * 5.8, 9 + Math.sin(rad) * 5.8)
                            ctx.lineTo(9 + Math.cos(rad) * 7.8, 9 + Math.sin(rad) * 7.8)
                            ctx.stroke()
                        }
                    }
                }
            }

            MouseArea {
                id: ma
                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: root.navigate(pageId)
            }
        }

        NavLinkBtn { pageId: "monitor"; tip: pm.t("nav.monitor"); iconKind: 0 }
        NavLinkBtn { pageId: "apps"; tip: pm.t("nav.apps"); iconKind: 1 }
        NavLinkBtn { pageId: "xray"; tip: pm.t("nav.xray"); iconKind: 2 }
        NavLinkBtn { pageId: "settings"; tip: pm.t("nav.settings"); iconKind: 3 }

        Item { Layout.fillHeight: true }

        // Transport pill at bottom of rail
        Rectangle {
            Layout.alignment: Qt.AlignHCenter
            width: 36; height: 16; radius: 3
            color: pm.transport.indexOf("Unix") >= 0 ? "#17253a" : "#152a1e"
            border.color: pm.transport.indexOf("Unix") >= 0 ? "#2e4e7a" : "#245435"
            Label {
                anchors.centerIn: parent
                text: pm.transport.indexOf("Unix") >= 0 ? "IPC" : "TCP"
                color: pm.transport.indexOf("Unix") >= 0 ? "#4e97fa" : "#1de966"
                font.pixelSize: 9; font.bold: true; font.family: "monospace"
            }
        }

        // Vector Tools Icon + Dark Menu
        Rectangle {
            Layout.alignment: Qt.AlignHCenter
            width: 34; height: 34; radius: 10
            color: (toolsMenu.visible || toolsMa.containsMouse) ? "#222222" : "transparent"
            Canvas {
                anchors.centerIn: parent; width: 16; height: 16
                onPaint: {
                    var ctx = getContext("2d")
                    ctx.clearRect(0, 0, width, height)
                    ctx.strokeStyle = "#ababab"; ctx.lineWidth = 1.6; ctx.lineCap = "round"; ctx.lineJoin = "round"
                    ctx.beginPath(); ctx.moveTo(11.5, 2.5); ctx.arc(10.5, 5.5, 3.2, -0.5, 3.5); ctx.lineTo(4, 13.5); ctx.lineTo(2.5, 12); ctx.lineTo(9, 5.5); ctx.stroke()
                }
            }
            MouseArea { id: toolsMa; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: toolsMenu.open() }
            PmMenu {
                id: toolsMenu
                x: 44; y: -115
                MenuItem { text: pm.isRu ? "Очистить кэш DNS" : "Clear DNS Cache"; onTriggered: pm.clearDnsCache() }
                MenuItem { text: pm.isRu ? "Очистить историю" : "Cleanup History"; onTriggered: pm.clearHistory() }
                MenuItem { text: pm.isRu ? "Проверить обновления" : "Check Updates"; onTriggered: pm.checkUpdates() }
                MenuItem { text: pm.isRu ? "Язык: RU / EN" : "Language: EN / RU"; onTriggered: pm.setLanguage(pm.isRu ? "en" : "ru") }
            }
        }

        // Vector Pause Icon + Dark Menu
        Rectangle {
            Layout.alignment: Qt.AlignHCenter
            width: 34; height: 34; radius: 10
            color: (pauseMenu.visible || pm.paused || pauseMa.containsMouse) ? "#222222" : "transparent"
            Canvas {
                id: pauseCanvas
                anchors.centerIn: parent; width: 16; height: 16
                onPaint: {
                    var ctx = getContext("2d")
                    ctx.clearRect(0, 0, width, height)
                    ctx.fillStyle = pm.paused ? "#e9d81d" : "#ababab"
                    ctx.fillRect(4, 3, 3, 10)
                    ctx.fillRect(9.5, 3, 3, 10)
                }
                Connections { target: pm; function onNotificationsChanged() { pauseCanvas.requestPaint() } }
            }
            MouseArea { id: pauseMa; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: pauseMenu.open() }
            PmMenu {
                id: pauseMenu
                x: 44; y: -155
                MenuItem { text: pm.isRu ? "Пауза Xray (5 мин)" : "Pause Xray 5m"; onTriggered: pm.pauseProtection(5, true) }
                MenuItem { text: pm.isRu ? "Пауза Xray (15 мин)" : "Pause Xray 15m"; onTriggered: pm.pauseProtection(15, true) }
                MenuItem { text: pm.isRu ? "Пауза защиты (5 мин)" : "Pause All 5m"; onTriggered: pm.pauseProtection(5, false) }
                MenuItem { text: pm.isRu ? "Пауза защиты (15 мин)" : "Pause All 15m"; onTriggered: pm.pauseProtection(15, false) }
                MenuSeparator {}
                MenuItem { text: pm.isRu ? "Возобновить защиту" : "Resume Protection"; enabled: pm.paused; onTriggered: pm.resumeProtection() }
            }
        }

        // Vector Power Icon + Dark Menu
        Rectangle {
            Layout.alignment: Qt.AlignHCenter
            width: 34; height: 34; radius: 10
            color: (powerMenu.visible || powerMa.containsMouse) ? "#222222" : "transparent"
            Canvas {
                anchors.centerIn: parent; width: 16; height: 16
                onPaint: {
                    var ctx = getContext("2d")
                    ctx.clearRect(0, 0, width, height)
                    ctx.strokeStyle = "#ababab"; ctx.lineWidth = 1.6; ctx.lineCap = "round"
                    ctx.beginPath(); ctx.arc(8, 8.5, 5.2, -0.9, Math.PI + 0.9); ctx.stroke()
                    ctx.beginPath(); ctx.moveTo(8, 2); ctx.lineTo(8, 8); ctx.stroke()
                }
            }
            MouseArea { id: powerMa; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: powerMenu.open() }
            PmMenu {
                id: powerMenu
                x: 44; y: -90
                MenuItem { text: pm.isRu ? "Перезапустить ядро" : "Restart Core Service"; onTriggered: pm.restartCore() }
                MenuItem { text: pm.isRu ? "Остановить Portmaster" : "Shut Down Portmaster"; onTriggered: pm.shutdownCore() }
                MenuSeparator {}
                MenuItem { text: pm.isRu ? "Выйти из Qt UI" : "Quit Qt UI"; onTriggered: pm.quitApp() }
            }
        }
    }
}
