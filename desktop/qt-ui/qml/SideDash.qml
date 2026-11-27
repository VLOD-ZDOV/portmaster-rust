import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Rectangle {
    id: root
    color: "#121213"

    signal openDashboard()
    signal openApp(string key)
    signal openMonitor()
    signal openXray()
    signal openNetworkSettings()

    property string scoutSearch: ""

    Rectangle {
        anchors.right: parent.right; anchors.top: parent.top; anchors.bottom: parent.bottom
        width: 1; color: "#2c2c2c"
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 12
        spacing: 10

        // 1. 1:1 Angular Security Lock Shield & Clean Subtitle (Click opens Dashboard)
        Item {
            Layout.fillWidth: true
            height: 112

            ColumnLayout {
                anchors.centerIn: parent
                spacing: 0

                SecurityShield {
                    Layout.alignment: Qt.AlignHCenter
                    width: 88
                    height: 88
                    cutoutColor: "#121213"
                }

                Label {
                    Layout.alignment: Qt.AlignHCenter
                    text: !pm.connected ? pm.t("dash.offline") : (pm.paused ? pm.t("dash.paused") : pm.t("dash.protected"))
                    color: "#e8e8ee"
                    font.pixelSize: 14
                    font.weight: Font.Light
                }
            }

            MouseArea {
                anchors.fill: parent
                cursorShape: Qt.PointingHandCursor
                onClicked: root.openDashboard()
            }
        }

        // 2. Feature Scout Bar (History Book + Xray Mesh Icons + Node/Flag status like Angular!)
        ColumnLayout {
            Layout.fillWidth: true
            spacing: 5

            RowLayout {
                Layout.alignment: Qt.AlignHCenter
                spacing: 12

                // History book icon
                Item {
                    width: 24; height: 22
                    Canvas {
                        id: histIconCvs
                        anchors.centerIn: parent; width: 18; height: 18
                        Connections { target: pm; function onSettingsChanged() { histIconCvs.requestPaint() } }
                        onPaint: {
                            var ctx = getContext("2d")
                            ctx.clearRect(0, 0, width, height)
                            ctx.strokeStyle = pm.historyEnabled ? "#e4e4ec" : "#585862"
                            ctx.lineWidth = 1.4; ctx.lineCap = "round"; ctx.lineJoin = "round"
                            ctx.beginPath()
                            ctx.moveTo(9, 4.5); ctx.quadraticCurveTo(6, 2.5, 2.5, 3.2); ctx.lineTo(2.5, 13.8); ctx.quadraticCurveTo(6, 13.0, 9, 15.2)
                            ctx.moveTo(9, 4.5); ctx.quadraticCurveTo(12, 2.5, 15.5, 3.2); ctx.lineTo(15.5, 13.8); ctx.quadraticCurveTo(12, 13.0, 9, 15.2)
                            ctx.moveTo(9, 4.5); ctx.lineTo(9, 15.2)
                            ctx.stroke()
                        }
                        Component.onCompleted: requestPaint()
                    }
                    MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: root.openMonitor() }
                }

                // Xray 3-node circle mesh icon
                Item {
                    width: 24; height: 22
                    Canvas {
                        id: xrayIconCvs
                        anchors.centerIn: parent; width: 18; height: 18
                        Connections { target: pm; function onXrayChanged() { xrayIconCvs.requestPaint() } }
                        onPaint: {
                            var ctx = getContext("2d")
                            ctx.clearRect(0, 0, width, height)
                            ctx.strokeStyle = pm.xrayStatus.connected ? "#1de966" : "#888894"
                            ctx.lineWidth = 1.4; ctx.lineCap = "round"
                            ctx.beginPath(); ctx.arc(9, 9.5, 6.2, 0.25, 1.75); ctx.stroke()
                            ctx.beginPath(); ctx.arc(9, 9.5, 6.2, 2.25, 3.85); ctx.stroke()
                            ctx.beginPath(); ctx.arc(9, 9.5, 6.2, 4.5, 5.95); ctx.stroke()
                            ctx.fillStyle = pm.xrayStatus.connected ? "#1de966" : "#888894"
                            ctx.beginPath(); ctx.arc(9, 3.2, 1.6, 0, Math.PI * 2); ctx.fill()
                            ctx.beginPath(); ctx.arc(3.8, 12.6, 1.6, 0, Math.PI * 2); ctx.fill()
                            ctx.beginPath(); ctx.arc(14.2, 12.6, 1.6, 0, Math.PI * 2); ctx.fill()
                        }
                        Component.onCompleted: requestPaint()
                    }
                    MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: root.openXray() }
                }
            }

            // Xray ON/OFF + Exit Flag Pill
            Item {
                Layout.alignment: Qt.AlignHCenter
                implicitWidth: xrayPillRow.implicitWidth + 8
                implicitHeight: Math.max(20, xrayPillRow.implicitHeight)
                RowLayout {
                    id: xrayPillRow
                    anchors.centerIn: parent
                    spacing: 6
                    Rectangle {
                        width: 8; height: 8; radius: 4
                        color: pm.xrayStatus.connected ? "#1de966" : "#6c6c76"
                    }
                    Label {
                        text: pm.xrayStatus.connected ? ("Xray: ON") : "Xray: OFF"
                        color: pm.xrayStatus.connected ? "#1de966" : "#9696a2"
                        font.pixelSize: 11; font.bold: true
                    }
                    FlagBadge {
                        visible: !!pm.xrayStatus.connected && (pm.xrayStatus.exitCountry || "") !== ""
                        code: pm.xrayStatus.exitCountry || ""
                        badgeWidth: 18; badgeHeight: 12
                    }
                    Label {
                        visible: !!pm.xrayStatus.connected && (pm.xrayStatus.exitCountry || "") !== ""
                        text: pm.xrayStatus.exitCountry || ""
                        color: "#cacad4"; font.pixelSize: 11
                    }
                }
                MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: root.openXray() }
            }
        }

        // 3. Live Green (Allowed) / Red (Blocked) Connections Sparkline Graph
        Rectangle {
            Layout.fillWidth: true
            height: 68
            radius: 5
            color: "#18191d"
            border.color: chartHoverMa.containsMouse ? "#3e4252" : "#262830"

            ColumnLayout {
                anchors.fill: parent
                anchors.margins: 6
                spacing: 2

                RowLayout {
                    Layout.fillWidth: true
                    spacing: 6
                    Rectangle { width: 6; height: 6; radius: 3; color: "#2be876" }
                    Label {
                        text: pm.chartDataMode ? ("↓ " + pm.connRxStr) : pm.connAllowedTotal
                        color: "#2be876"; font.pixelSize: 10; font.bold: true
                    }
                    Rectangle { width: 6; height: 6; radius: 3; color: pm.chartDataMode ? "#4e97fa" : "#f03c3c" }
                    Label {
                        text: pm.chartDataMode ? ("↑ " + pm.connTxStr) : pm.connBlockedTotal
                        color: pm.chartDataMode ? "#4e97fa" : "#f03c3c"; font.pixelSize: 10; font.bold: true
                    }
                    Item { Layout.fillWidth: true }
                    Label { text: pm.isRu ? "Трафик →" : "Monitor →"; color: "#767884"; font.pixelSize: 9 }
                }

                ConnChartCanvas {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    compact: true
                    showAxes: false
                    dataMode: pm.chartDataMode
                    bins: pm.connChartBins
                }
            }

            MouseArea {
                id: chartHoverMa
                anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor
                onClicked: root.openMonitor()
            }
        }

        // Quick Network Kill-Switch & DNS Bar
        RowLayout {
            Layout.fillWidth: true
            spacing: 6
            Rectangle {
                Layout.fillWidth: true; height: 26; radius: 4
                color: pm.blockInternet ? "#3d1414" : "#1b1b1b"
                border.color: pm.blockInternet ? "#e01d1d" : "#2c2c2c"
                Label {
                    anchors.centerIn: parent
                    text: pm.blockInternet ? (pm.isRu ? "Инет: БЛОК" : "Net: BLOCK") : (pm.isRu ? "Инет: Вкл" : "Net: Open")
                    color: pm.blockInternet ? "#ff8888" : "#1de966"
                    font.pixelSize: 10; font.bold: true
                }
                MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: pm.updateSetting("filter/blockInternet", !pm.blockInternet) }
            }
            Rectangle {
                Layout.fillWidth: true; height: 26; radius: 4; color: "#1b1b1b"; border.color: "#2c2c2c"
                Label {
                    anchors.fill: parent; anchors.leftMargin: 6; anchors.rightMargin: 6
                    horizontalAlignment: Text.AlignHCenter; verticalAlignment: Text.AlignVCenter
                    text: "DNS: " + pm.dnsSummary
                    color: "#89b7ff"; font.pixelSize: 10; elide: Text.ElideRight
                }
                MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: root.openNetworkSettings() }
            }
        }

        // 3. Interactive Prompts (#2083 Allow All / Block All)
        Rectangle {
            Layout.fillWidth: true
            visible: pm.prompts.length > 0
            color: "#222222"
            border.color: "#e9d81d"
            radius: 4
            implicitHeight: promptCol.implicitHeight + 14

            ColumnLayout {
                id: promptCol
                anchors.left: parent.left; anchors.right: parent.right; anchors.top: parent.top
                anchors.margins: 7
                spacing: 6

                RowLayout {
                    Layout.fillWidth: true
                    Label {
                        text: pm.t("dash.prompts") + " (" + pm.prompts.length + ")"
                        color: "#e9d81d"; font.pixelSize: 10; font.bold: true; Layout.fillWidth: true
                    }
                    Button {
                        text: pm.t("dash.allowAll")
                        onClicked: pm.resolveAllPrompts(true)
                        background: Rectangle { color: "#143d24"; border.color: "#1de966"; radius: 3 }
                        contentItem: Text { text: parent.text; color: "#1de966"; font.pixelSize: 9; padding: 2 }
                    }
                    Button {
                        text: pm.t("dash.blockAll")
                        onClicked: pm.resolveAllPrompts(false)
                        background: Rectangle { color: "#3d1414"; border.color: "#e01d1d"; radius: 3 }
                        contentItem: Text { text: parent.text; color: "#e01d1d"; font.pixelSize: 9; padding: 2 }
                    }
                }

                Repeater {
                    model: pm.prompts
                    Rectangle {
                        Layout.fillWidth: true
                        height: 44
                        color: "#1b1b1b"; border.color: "#2c2c2c"; radius: 3
                        RowLayout {
                            anchors.fill: parent; anchors.margins: 5; spacing: 4
                            ColumnLayout {
                                Layout.fillWidth: true; spacing: 1
                                Label { text: modelData.title; color: "#ffffff"; font.pixelSize: 10; font.bold: true; elide: Text.ElideRight; Layout.fillWidth: true }
                                Label { text: modelData.message; color: "#ababab"; font.pixelSize: 9; elide: Text.ElideRight; Layout.fillWidth: true }
                            }
                            Button {
                                text: "✓"
                                onClicked: pm.resolvePrompt(modelData.id, true)
                                background: Rectangle { color: "#143d24"; radius: 2 }
                                contentItem: Text { text: parent.text; color: "#1de966"; font.pixelSize: 10 }
                            }
                            Button {
                                text: "✕"
                                onClicked: pm.resolvePrompt(modelData.id, false)
                                background: Rectangle { color: "#3d1414"; radius: 2 }
                                contentItem: Text { text: parent.text; color: "#e01d1d"; font.pixelSize: 10 }
                            }
                        }
                    }
                }
            }
        }

        // 4. Network Scout (Session / Live Active Apps)
        RowLayout {
            Layout.fillWidth: true
            spacing: 6
            Label { text: pm.t("apps.active").toUpperCase(); color: "#888888"; font.pixelSize: 10; font.bold: true; Layout.fillWidth: true }
            Rectangle {
                height: 18; radius: 3
                color: pm.onlyNowApps ? "#173252" : "#1d1d21"
                border.color: pm.onlyNowApps ? "#4e97fa" : "#2f3038"
                implicitWidth: sdNowLbl.implicitWidth + 10
                Label {
                    id: sdNowLbl; anchors.centerIn: parent
                    text: pm.isRu ? "Сейчас" : "Live"
                    color: pm.onlyNowApps ? "#89b7ff" : "#7e808a"
                    font.pixelSize: 9; font.bold: pm.onlyNowApps
                }
                MouseArea {
                    anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                    onClicked: pm.setOnlyNowApps(!pm.onlyNowApps)
                }
            }
            Label { text: pm.activeApps.length; color: "#1de966"; font.pixelSize: 10; font.bold: true }
        }

        TextField {
            Layout.fillWidth: true
            placeholderText: pm.t("apps.search")
            placeholderTextColor: "#666666"
            color: "#ffffff"
            font.pixelSize: 11
            onTextChanged: root.scoutSearch = text.toLowerCase()
            background: Rectangle { color: "#1b1b1b"; border.color: "#2c2c2c"; radius: 4 }
        }

        ListView {
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            cacheBuffer: 600
            spacing: 4
            ScrollBar.vertical: PmScrollBar {}
            PmWheelHelper {}
            model: {
                var q = root.scoutSearch
                var out = []
                for (var i = 0; i < pm.activeApps.length; i++) {
                    var a = pm.activeApps[i]
                    if (q === "" || a.name.toLowerCase().indexOf(q) >= 0) out.push(a)
                }
                return out
            }
            delegate: Rectangle {
                width: ListView.view.width
                height: 38
                radius: 4
                color: appScoutMa.containsMouse ? "#292929" : "#202020"
                RowLayout {
                    anchors.fill: parent; anchors.margins: 7; spacing: 8
                    Rectangle {
                        width: 24; height: 24; radius: 6
                        property bool iconReady: scoutIconImg.status === Image.Ready && scoutIconImg.implicitWidth > 1
                        color: iconReady ? "transparent" : (modelData.avatarColor || "#2c2c2c")
                        Image {
                            id: scoutIconImg
                            anchors.fill: parent
                            fillMode: Image.PreserveAspectFit
                            smooth: true
                            source: "image://appicon/" + modelData.iconKey + "?v=" + pm.iconVersion
                            visible: parent.iconReady
                        }
                        Label {
                            visible: !parent.iconReady
                            anchors.centerIn: parent; text: modelData.initials; color: "#ffffff"; font.pixelSize: 9; font.bold: true
                        }
                    }
                    ColumnLayout {
                        Layout.fillWidth: true; spacing: 0
                        Label { text: modelData.name; color: "#f0f0f5"; font.pixelSize: 11; font.bold: true; elide: Text.ElideRight; Layout.fillWidth: true }
                        Label { text: modelData.effectiveVpn ? "Xray" : ((modelData.blockInternet || modelData.action === "block") ? (pm.isRu ? "Блок" : "Blocked") : (pm.isRu ? "Прямо" : "Direct")); color: (modelData.blockInternet || modelData.action === "block") ? "#e01d1d" : (modelData.effectiveVpn ? "#4e97fa" : "#1de966"); font.pixelSize: 9 }
                    }
                    Rectangle {
                        width: 24; height: 18; radius: 3; color: "#2c2c2c"
                        Label { anchors.centerIn: parent; text: modelData.alive; color: "#ababab"; font.pixelSize: 10; font.family: "monospace" }
                    }
                }
                MouseArea {
                    id: appScoutMa
                    anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor
                    onClicked: root.openApp(modelData.key)
                }
            }
        }

        // 5. Alerts Strip
        ColumnLayout {
            Layout.fillWidth: true
            visible: pm.alerts.length > 0
            spacing: 4
            RowLayout {
                Layout.fillWidth: true
                Label { text: pm.t("dash.alerts") + " (" + pm.alerts.length + ")"; color: "#888888"; font.pixelSize: 10; Layout.fillWidth: true }
                Label {
                    text: pm.t("mon.clear"); color: "#4e97fa"; font.pixelSize: 10
                    MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: pm.clearNotifications() }
                }
            }
            Repeater {
                model: pm.alerts.slice(0, 3)
                Rectangle {
                    Layout.fillWidth: true; height: 34; radius: 3; color: "#1b1b1b"; border.color: "#2c2c2c"
                    ColumnLayout {
                        anchors.fill: parent; anchors.margins: 4; spacing: 0
                        Label { text: modelData.title; color: "#ffffff"; font.pixelSize: 10; font.bold: true; elide: Text.ElideRight; Layout.fillWidth: true }
                        Label { text: modelData.message; color: "#ababab"; font.pixelSize: 9; elide: Text.ElideRight; Layout.fillWidth: true }
                    }
                }
            }
        }
    }
}
