import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Rectangle {
    id: root
    color: "#121213"

    signal openApp(string key)
    signal openMonitor()
    signal openXray()

    property string scoutSearch: ""

    Rectangle {
        anchors.right: parent.right; anchors.top: parent.top; anchors.bottom: parent.bottom
        width: 1; color: "#2c2c2c"
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 12
        spacing: 12

        // 1. Large Security Lock Concentric Shields
        Rectangle {
            Layout.fillWidth: true
            height: 124
            radius: 4
            color: "#1b1b1b"
            border.color: "#2c2c2c"

            ColumnLayout {
                anchors.centerIn: parent
                spacing: 4

                Canvas {
                    id: shieldCanvas
                    Layout.alignment: Qt.AlignHCenter
                    width: 60; height: 60
                    Connections {
                        target: pm
                        function onConnectionChanged() { shieldCanvas.requestPaint() }
                        function onXrayChanged() { shieldCanvas.requestPaint() }
                        function onNotificationsChanged() { shieldCanvas.requestPaint() }
                    }
                    onPaint: {
                        var ctx = getContext("2d")
                        ctx.clearRect(0, 0, width, height)
                        var c1 = !pm.connected ? "#e01d1d" : (pm.paused ? "#e9d81d" : (pm.xrayStatus.connected ? "#4e97fa" : "#1de966"))
                        var c2 = !pm.connected ? "#811818" : (pm.paused ? "#827918" : (pm.xrayStatus.connected ? "#1e4b8f" : "#18823d"))
                        var c3 = !pm.connected ? "#3d1414" : (pm.paused ? "#3d3a14" : (pm.xrayStatus.connected ? "#16345c" : "#143d24"))

                        function drawS(sc, fl) {
                            ctx.save()
                            ctx.translate(width / 2, height / 2)
                            ctx.scale(sc, sc)
                            ctx.beginPath()
                            ctx.moveTo(0, -24); ctx.lineTo(20, -15)
                            ctx.quadraticCurveTo(20, 9, 0, 24)
                            ctx.quadraticCurveTo(-20, 9, -20, -15)
                            ctx.closePath()
                            ctx.fillStyle = fl; ctx.fill()
                            ctx.restore()
                        }
                        drawS(1.1, c3); drawS(0.84, c2); drawS(0.62, c1)
                        ctx.strokeStyle = "#ffffff"; ctx.lineWidth = 2.6; ctx.lineCap = "round"; ctx.lineJoin = "round"
                        ctx.beginPath(); ctx.moveTo(22, 31); ctx.lineTo(27.5, 36.5); ctx.lineTo(39, 24); ctx.stroke()
                    }
                }

                Label {
                    Layout.alignment: Qt.AlignHCenter
                    text: !pm.connected ? pm.t("dash.offline") : (pm.paused ? pm.t("dash.paused") : (pm.xrayStatus.connected ? "Xray VPN" : pm.t("dash.protected")))
                    color: "#ffffff"
                    font.pixelSize: 13
                    font.bold: true
                }
            }
        }

        // 2. Feature Scout Bar
        RowLayout {
            Layout.fillWidth: true
            spacing: 6
            Rectangle {
                Layout.fillWidth: true; height: 28; radius: 4; color: "#1b1b1b"; border.color: "#2c2c2c"
                Label { anchors.centerIn: parent; text: "History"; color: pm.historyEnabled ? "#4e97fa" : "#888888"; font.pixelSize: 10; font.bold: true }
                MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: root.openMonitor() }
            }
            Rectangle {
                Layout.fillWidth: true; height: 28; radius: 4; color: "#1b1b1b"; border.color: pm.xrayStatus.connected ? "#4e97fa" : "#2c2c2c"
                Label {
                    anchors.centerIn: parent
                    text: pm.xrayStatus.connected ? ("Xray: " + (pm.xrayStatus.exitCountry || "ON")) : "Xray VPN"
                    color: pm.xrayStatus.connected ? "#1de966" : "#888888"
                    font.pixelSize: 10; font.bold: true
                }
                MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: root.openXray() }
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

        // 4. Network Scout (Live Active Apps)
        RowLayout {
            Layout.fillWidth: true
            Label { text: pm.t("apps.active").toUpperCase(); color: "#888888"; font.pixelSize: 10; font.bold: true; Layout.fillWidth: true }
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
            spacing: 4
            ScrollBar.vertical: PmScrollBar {}
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
                        width: 22; height: 22; radius: 4; color: "#2c2c2c"
                        Label { anchors.centerIn: parent; text: modelData.initials; color: "#ffffff"; font.pixelSize: 9; font.bold: true }
                    }
                    ColumnLayout {
                        Layout.fillWidth: true; spacing: 0
                        Label { text: modelData.name; color: "#cacaca"; font.pixelSize: 11; font.bold: true; elide: Text.ElideRight; Layout.fillWidth: true }
                        Label { text: modelData.effectiveVpn ? "Xray VPN" : (modelData.action === "block" ? "Blocked" : "Direct"); color: modelData.action === "block" ? "#e01d1d" : (modelData.effectiveVpn ? "#4e97fa" : "#1de966"); font.pixelSize: 9 }
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
