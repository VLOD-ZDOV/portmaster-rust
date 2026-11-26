import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Rectangle {
    id: root
    color: "#121213"

    property string currentPage: "apps"
    property bool sideDashOpen: false
    signal navigate(string page)
    signal toggleSideDash()

    // Subtle right divider matching Portmaster sticky navigation
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

        // TOP: Concentric Shields (collapsed) OR Rotating Portmaster Ring Logo (expanded)
        Item {
            Layout.alignment: Qt.AlignHCenter
            width: 56
            height: 56

            Canvas {
                id: topBadgeCanvas
                anchors.centerIn: parent
                width: 34
                height: 34
                property real spinAngle: 0
                NumberAnimation on spinAngle {
                    from: 0; to: 360; duration: 3500; loops: Animation.Infinite; running: pm.connected && root.sideDashOpen
                }
                onSpinAngleChanged: requestPaint()
                Connections {
                    target: pm
                    function onConnectionChanged() { topBadgeCanvas.requestPaint() }
                    function onXrayChanged() { topBadgeCanvas.requestPaint() }
                    function onNotificationsChanged() { topBadgeCanvas.requestPaint() }
                }
                Connections {
                    target: root
                    function onSideDashOpenChanged() { topBadgeCanvas.requestPaint() }
                }

                onPaint: {
                    var ctx = getContext("2d")
                    ctx.clearRect(0, 0, width, height)
                    var c1 = !pm.connected ? "#e01d1d" : (pm.paused ? "#e9d81d" : (pm.xrayStatus.connected ? "#4e97fa" : "#1de966"))
                    var c2 = !pm.connected ? "#811818" : (pm.paused ? "#827918" : (pm.xrayStatus.connected ? "#1e4b8f" : "#18823d"))
                    var c3 = !pm.connected ? "#3d1414" : (pm.paused ? "#3d3a14" : (pm.xrayStatus.connected ? "#16345c" : "#143d24"))

                    if (!root.sideDashOpen) {
                        // Exact 3 concentric shields like collapsed Angular nav
                        function drawShield(scale, fill) {
                            ctx.save()
                            ctx.translate(width / 2, height / 2)
                            ctx.scale(scale, scale)
                            ctx.beginPath()
                            ctx.moveTo(0, -13)
                            ctx.lineTo(11, -8)
                            ctx.quadraticCurveTo(11, 5, 0, 13)
                            ctx.quadraticCurveTo(-11, 5, -11, -8)
                            ctx.closePath()
                            ctx.fillStyle = fill
                            ctx.fill()
                            ctx.restore()
                        }
                        drawShield(1.15, c3)
                        drawShield(0.88, c2)
                        drawShield(0.65, c1)
                        ctx.strokeStyle = "#ffffff"
                        ctx.lineWidth = 2.0
                        ctx.lineCap = "round"
                        ctx.lineJoin = "round"
                        ctx.beginPath()
                        ctx.moveTo(12, 18)
                        ctx.lineTo(15.5, 21.5)
                        ctx.lineTo(22.5, 13.5)
                        ctx.stroke()
                    } else {
                        // Spinning rings when SideDash expanded
                        ctx.save()
                        ctx.translate(width / 2, height / 2)
                        ctx.strokeStyle = "#ffffff"
                        ctx.lineWidth = 2.6
                        ctx.beginPath(); ctx.arc(0, 0, 14, 0, Math.PI * 2); ctx.stroke()
                        ctx.save(); ctx.rotate(-spinAngle * Math.PI / 180)
                        ctx.strokeStyle = pm.connected ? "#ffffff" : "#d12e2e"
                        ctx.globalAlpha = 0.8; ctx.lineWidth = 2.2
                        ctx.beginPath(); ctx.arc(0, 0, 9.5, 0.2, 2.2); ctx.stroke()
                        ctx.beginPath(); ctx.arc(0, 0, 9.5, 3.3, 5.3); ctx.stroke()
                        ctx.restore()
                        ctx.save(); ctx.rotate(spinAngle * Math.PI / 180)
                        ctx.globalAlpha = 0.6; ctx.lineWidth = 2
                        ctx.beginPath(); ctx.arc(0, 0, 5.5, 0.4, 3.8); ctx.stroke()
                        ctx.restore()
                        ctx.restore()
                    }
                }
            }

            MouseArea {
                anchors.fill: parent
                cursorShape: Qt.PointingHandCursor
                onClicked: root.toggleSideDash()
            }
        }

        // Chevron Toggle SideDash
        Rectangle {
            Layout.alignment: Qt.AlignHCenter
            width: 28; height: 28; radius: 8
            color: root.sideDashOpen ? "#2c2c2c" : "#222222"
            Text {
                anchors.centerIn: parent
                text: root.sideDashOpen ? "‹" : "›"
                color: "#ffffff"
                font.pixelSize: 16
                font.bold: true
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
            width: 32; height: 32; radius: 10
            color: "transparent"
            Canvas {
                anchors.centerIn: parent; width: 18; height: 18
                onPaint: {
                    var ctx = getContext("2d")
                    ctx.clearRect(0, 0, width, height)
                    ctx.strokeStyle = pm.prompts.length > 0 ? "#e9d81d" : (pm.alerts.length > 0 ? "#4e97fa" : "#888888")
                    ctx.lineWidth = 1.6
                    ctx.beginPath()
                    ctx.arc(9, 8, 4.5, Math.PI, 0)
                    ctx.lineTo(14.5, 13.5); ctx.lineTo(3.5, 13.5); ctx.closePath()
                    ctx.stroke()
                    ctx.beginPath(); ctx.arc(9, 15, 1.3, 0, Math.PI); ctx.stroke()
                }
            }
            Rectangle {
                visible: pm.prompts.length > 0 || pm.alerts.length > 0
                width: 6; height: 6; radius: 3
                anchors.top: parent.top; anchors.right: parent.right; anchors.margins: 4
                color: pm.prompts.length > 0 ? "#e9d81d" : "#4e97fa"
            }
            MouseArea {
                anchors.fill: parent
                cursorShape: Qt.PointingHandCursor
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
            width: 34; height: 34; radius: 10
            color: isActive ? "#2c2c2c" : (ma.containsMouse ? "#222222" : "transparent")
            ToolTip.visible: ma.containsMouse
            ToolTip.text: tip
            ToolTip.delay: 400

            Canvas {
                anchors.centerIn: parent; width: 18; height: 18
                opacity: isActive || ma.containsMouse ? 1.0 : 0.55
                onOpacityChanged: requestPaint()
                onPaint: {
                    var ctx = getContext("2d")
                    ctx.clearRect(0, 0, width, height)
                    ctx.strokeStyle = "#ffffff"
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
                        ctx.beginPath(); ctx.arc(9, 9, 6.2, 0, Math.PI * 2); ctx.stroke()
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
        Label {
            Layout.alignment: Qt.AlignHCenter
            text: pm.transport.indexOf("Unix") >= 0 ? "IPC" : "TCP"
            color: pm.transport.indexOf("Unix") >= 0 ? "#4e97fa" : "#1de966"
            font.pixelSize: 9
            font.bold: true
            font.family: "monospace"
        }

        // Lower Tools / Pause / Power Menus matching Navigation 1:1
        Rectangle {
            Layout.alignment: Qt.AlignHCenter
            width: 32; height: 32; radius: 10
            color: toolsMenu.visible ? "#2c2c2c" : "transparent"
            Text { anchors.centerIn: parent; text: "🔧"; font.pixelSize: 13; color: "#ababab" }
            MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: toolsMenu.open() }
            Menu {
                id: toolsMenu
                x: 42; y: -110
                MenuItem { text: "Clear DNS Cache"; onTriggered: pm.clearDnsCache() }
                MenuItem { text: "Cleanup History"; onTriggered: pm.clearHistory() }
                MenuItem { text: "Check Updates"; onTriggered: pm.checkUpdates() }
                MenuItem { text: "RU / EN"; onTriggered: pm.setLanguage(pm.isRu ? "en" : "ru") }
            }
        }

        Rectangle {
            Layout.alignment: Qt.AlignHCenter
            width: 32; height: 32; radius: 10
            color: (pauseMenu.visible || pm.paused) ? "#2c2c2c" : "transparent"
            Text { anchors.centerIn: parent; text: "⏸"; font.pixelSize: 13; color: pm.paused ? "#e9d81d" : "#ababab" }
            MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: pauseMenu.open() }
            Menu {
                id: pauseMenu
                x: 42; y: -140
                MenuItem { text: "Pause Xray 5m"; onTriggered: pm.pauseProtection(5, true) }
                MenuItem { text: "Pause Xray 15m"; onTriggered: pm.pauseProtection(15, true) }
                MenuItem { text: "Pause All 5m"; onTriggered: pm.pauseProtection(5, false) }
                MenuItem { text: "Pause All 15m"; onTriggered: pm.pauseProtection(15, false) }
                MenuSeparator {}
                MenuItem { text: "Resume Protection"; enabled: pm.paused; onTriggered: pm.resumeProtection() }
            }
        }

        Rectangle {
            Layout.alignment: Qt.AlignHCenter
            width: 32; height: 32; radius: 10
            color: powerMenu.visible ? "#2c2c2c" : "transparent"
            Text { anchors.centerIn: parent; text: "⏻"; font.pixelSize: 14; color: "#ababab" }
            MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: powerMenu.open() }
            Menu {
                id: powerMenu
                x: 42; y: -80
                MenuItem { text: "Restart Core Service"; onTriggered: pm.restartCore() }
                MenuItem { text: "Shut Down Portmaster"; onTriggered: pm.shutdownCore() }
                MenuSeparator {}
                MenuItem { text: "Quit Qt UI"; onTriggered: pm.quitApp() }
            }
        }
    }
}
