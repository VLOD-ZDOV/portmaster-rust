import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Item {
    id: root

    property bool showSubDrawer: false
    property bool showLinkDrawer: false

    Connections {
        target: pm
        function onWorldMapReady() { worldCanvas.requestPaint() }
        function onXrayChanged() { worldCanvas.requestPaint() }
    }

    ScrollView {
        anchors.fill: parent
        contentWidth: availableWidth
        clip: true

        ColumnLayout {
            width: parent.width - 48
            x: 24; y: 20
            spacing: 16

            // Gateway Hero Card (matching xray-page.html)
            Rectangle {
                Layout.fillWidth: true
                height: 84
                radius: 6
                color: "#1b1b1d"
                border.color: pm.xrayStatus.connected ? "#195a33" : "#2a2a2e"

                RowLayout {
                    anchors.fill: parent
                    anchors.leftMargin: 18; anchors.rightMargin: 18
                    spacing: 14

                    Rectangle {
                        width: 12; height: 12; radius: 6
                        color: pm.xrayStatus.connected ? "#1de966" : "#6a6a72"
                    }

                    ColumnLayout {
                        Layout.fillWidth: true
                        spacing: 3
                        RowLayout {
                            spacing: 10
                            Label {
                                text: pm.t("xray.title")
                                color: "#ffffff"; font.pixelSize: 18; font.bold: true
                            }
                            Rectangle {
                                height: 20; radius: 4
                                color: pm.xrayStatus.connected ? "#136734" : "#26262a"
                                implicitWidth: stLbl.implicitWidth + 14
                                Label {
                                    id: stLbl; anchors.centerIn: parent
                                    text: pm.xrayStatus.connected ? (pm.isRu ? "Подключено" : "Connected") : (pm.isRu ? "Отключено" : "Disconnected")
                                    color: pm.xrayStatus.connected ? "#1de966" : "#9e9ea6"
                                    font.pixelSize: 10; font.bold: true
                                }
                            }
                        }
                        Label {
                            text: {
                                var node = pm.xrayStatus.nodeName || (pm.isRu ? "Узел не выбран" : "No node selected")
                                var exitIp = pm.xrayStatus.exitIp || ""
                                var cc = pm.xrayStatus.exitCountry || pm.xrayStatus.country || ""
                                var bin = pm.xrayStatus.binary_available ? "xray-rust: OK" : "xray-rust: missing"
                                return node + (exitIp !== "" ? ("  •  Exit IP: " + exitIp + (cc !== "" ? " (" + cc + ")" : "")) : "") + "  •  " + bin
                            }
                            color: "#9e9ea6"; font.pixelSize: 12
                        }
                    }

                    Rectangle {
                        height: 32; radius: 4; color: "#22242a"; border.color: "#363842"
                        implicitWidth: geoLbl.implicitWidth + 18
                        Label { id: geoLbl; anchors.centerIn: parent; text: pm.t("xray.dlGeo"); color: "#cacaca"; font.pixelSize: 11 }
                        MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: pm.downloadGeodata() }
                    }

                    Rectangle {
                        height: 32; radius: 4; color: "#22242a"; border.color: "#363842"
                        implicitWidth: binLbl.implicitWidth + 18
                        Label { id: binLbl; anchors.centerIn: parent; text: pm.t("xray.dlBin"); color: "#cacaca"; font.pixelSize: 11 }
                        MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: pm.downloadXrayBinary() }
                    }

                    Rectangle {
                        height: 34; radius: 4
                        color: pm.xrayStatus.connected ? "#7a1515" : "#4e97fa"
                        implicitWidth: connLbl.implicitWidth + 28
                        Label {
                            id: connLbl; anchors.centerIn: parent
                            text: pm.xrayStatus.connected ? pm.t("xray.disconnect") : pm.t("xray.connect")
                            color: "#ffffff"; font.pixelSize: 12; font.bold: true
                        }
                        MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: pm.toggleXrayConnect() }
                    }
                }
            }

            // Routing Quick Controls Row
            Rectangle {
                Layout.fillWidth: true
                height: 48
                radius: 6
                color: "#18181a"
                border.color: "#26262a"

                RowLayout {
                    anchors.fill: parent
                    anchors.leftMargin: 16; anchors.rightMargin: 16
                    spacing: 16

                    RowLayout {
                        spacing: 8
                        Switch { checked: pm.globalVpn; onToggled: pm.toggleGlobalVpn() }
                        Label { text: pm.t("xray.defaultVpn"); color: "#cacaca"; font.pixelSize: 12 }
                    }

                    RowLayout {
                        spacing: 8
                        Switch { checked: pm.autoConnectXray; onToggled: pm.updateSetting("xray/autoConnect", checked) }
                        Label { text: pm.t("xray.autoConnect"); color: "#cacaca"; font.pixelSize: 12 }
                    }

                    Item { Layout.fillWidth: true }

                    Rectangle {
                        height: 28; radius: 4; color: "#242428"; border.color: "#34343a"
                        implicitWidth: rstLbl.implicitWidth + 16
                        Label { id: rstLbl; anchors.centerIn: parent; text: pm.t("apps.resetDirect"); color: "#cacaca"; font.pixelSize: 11 }
                        MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: pm.resetAllAppOverrides() }
                    }

                    Rectangle {
                        height: 28; radius: 4; color: root.showSubDrawer ? "#4e97fa" : "#22242a"; border.color: "#363842"
                        implicitWidth: subBtnLbl.implicitWidth + 16
                        Label { id: subBtnLbl; anchors.centerIn: parent; text: "+ " + pm.t("xray.addSub"); color: "#ffffff"; font.pixelSize: 11 }
                        MouseArea {
                            anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                            onClicked: { root.showSubDrawer = !root.showSubDrawer; if (root.showSubDrawer) root.showLinkDrawer = false }
                        }
                    }

                    Rectangle {
                        height: 28; radius: 4; color: root.showLinkDrawer ? "#4e97fa" : "#22242a"; border.color: "#363842"
                        implicitWidth: lnkBtnLbl.implicitWidth + 16
                        Label { id: lnkBtnLbl; anchors.centerIn: parent; text: "+ " + pm.t("xray.addManual"); color: "#ffffff"; font.pixelSize: 11 }
                        MouseArea {
                            anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                            onClicked: { root.showLinkDrawer = !root.showLinkDrawer; if (root.showLinkDrawer) root.showSubDrawer = false }
                        }
                    }
                }
            }

            // Inline Subscription Drawer
            Rectangle {
                visible: root.showSubDrawer
                Layout.fillWidth: true
                height: 56; radius: 6; color: "#1d1e24"; border.color: "#4e97fa"
                RowLayout {
                    anchors.fill: parent; anchors.margins: 12; spacing: 10
                    TextField {
                        id: subUrlField
                        Layout.fillWidth: true
                        placeholderText: "https://example.com/subscription"
                        placeholderTextColor: "#66666e"
                        color: "#ffffff"; font.pixelSize: 12
                        background: Rectangle { color: "#121213"; radius: 4; border.color: "#2e2e32" }
                    }
                    Rectangle {
                        height: 30; radius: 4; color: "#4e97fa"
                        implicitWidth: saveSubLbl.implicitWidth + 20
                        Label { id: saveSubLbl; anchors.centerIn: parent; text: pm.isRu ? "Импорт подписки" : "Import"; color: "#ffffff"; font.pixelSize: 12; font.bold: true }
                        MouseArea {
                            anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                            onClicked: {
                                if (subUrlField.text.trim() !== "") {
                                    pm.addSubscription(subUrlField.text.trim())
                                    subUrlField.text = ""
                                    root.showSubDrawer = false
                                }
                            }
                        }
                    }
                }
            }

            // Inline Manual Link Drawer
            Rectangle {
                visible: root.showLinkDrawer
                Layout.fillWidth: true
                height: 56; radius: 6; color: "#1d1e24"; border.color: "#4e97fa"
                RowLayout {
                    anchors.fill: parent; anchors.margins: 12; spacing: 10
                    TextField {
                        id: linkField
                        Layout.fillWidth: true
                        placeholderText: "vless://... or trojan://... or ss://..."
                        placeholderTextColor: "#66666e"
                        color: "#ffffff"; font.pixelSize: 12; font.family: "monospace"
                        background: Rectangle { color: "#121213"; radius: 4; border.color: "#2e2e32" }
                    }
                    Rectangle {
                        height: 30; radius: 4; color: "#4e97fa"
                        implicitWidth: saveLnkLbl.implicitWidth + 20
                        Label { id: saveLnkLbl; anchors.centerIn: parent; text: pm.isRu ? "Добавить" : "Add Node"; color: "#ffffff"; font.pixelSize: 12; font.bold: true }
                        MouseArea {
                            anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                            onClicked: {
                                if (linkField.text.trim() !== "") {
                                    pm.addManualLink(linkField.text.trim())
                                    linkField.text = ""
                                    root.showLinkDrawer = false
                                }
                            }
                        }
                    }
                }
            }

            // Interactive TopoJSON World Map
            Rectangle {
                Layout.fillWidth: true
                height: 245
                radius: 6
                color: "#141519"
                border.color: "#24252b"
                clip: true

                Canvas {
                    id: worldCanvas
                    anchors.fill: parent
                    onPaint: {
                        var ctx = getContext("2d")
                        ctx.clearRect(0, 0, width, height)

                        // Subtle graticule lines
                        ctx.strokeStyle = "#1b1d23"
                        ctx.lineWidth = 1
                        for (var gx = 1; gx < 6; gx++) {
                            var x = width * gx / 6
                            ctx.beginPath(); ctx.moveTo(x, 0); ctx.lineTo(x, height); ctx.stroke()
                        }
                        for (var gy = 1; gy < 4; gy++) {
                            var y = height * gy / 4
                            ctx.beginPath(); ctx.moveTo(0, y); ctx.lineTo(width, y); ctx.stroke()
                        }

                        var polys = pm.worldPolygons || []
                        ctx.fillStyle = "#242730"
                        ctx.strokeStyle = "#323644"
                        ctx.lineWidth = 0.8
                        for (var i = 0; i < polys.length; i++) {
                            var pts = polys[i]
                            if (pts.length < 3) continue
                            ctx.beginPath()
                            ctx.moveTo(pts[0].x * width, pts[0].y * height)
                            for (var j = 1; j < pts.length; j++) {
                                ctx.lineTo(pts[j].x * width, pts[j].y * height)
                            }
                            ctx.closePath()
                            ctx.fill()
                            ctx.stroke()
                        }
                    }
                    Component.onCompleted: requestPaint()
                }

                // Glowing Interactive Country Node Pins
                Repeater {
                    model: pm.mapPins || []
                    delegate: Item {
                        x: modelData.nx * worldCanvas.width - width / 2
                        y: modelData.ny * worldCanvas.height - height / 2
                        width: 34; height: 22

                        Rectangle {
                            anchors.fill: parent
                            radius: 11
                            color: modelData.active ? "#166838" : (pinMa.containsMouse ? "#25589e" : "#1c3354")
                            border.color: modelData.active ? "#1de966" : "#4e97fa"
                            border.width: modelData.active ? 2 : 1

                            Label {
                                anchors.centerIn: parent
                                text: modelData.country + (modelData.count > 1 ? ("·" + modelData.count) : "")
                                color: "#ffffff"
                                font.pixelSize: 9; font.bold: true
                            }
                            MouseArea {
                                id: pinMa; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor
                                onClicked: pm.connectNode(modelData.firstNodeId)
                            }
                        }
                    }
                }

                Label {
                    anchors.left: parent.left; anchors.bottom: parent.bottom; anchors.margins: 10
                    text: pm.worldPolygons.length > 0
                          ? (pm.isRu ? "Клик по метке страны подключает сервер" : "Click a country marker to connect")
                          : (pm.isRu ? "Загрузка карты мира..." : "Loading world map...")
                    color: "#6a6b75"; font.pixelSize: 10
                }
            }

            // Active Subscriptions Bar
            RowLayout {
                visible: pm.xraySubs.length > 0
                Layout.fillWidth: true
                spacing: 8
                Repeater {
                    model: pm.xraySubs || []
                    delegate: Rectangle {
                        height: 36; radius: 5; color: "#1c1d22"; border.color: "#2c2e38"
                        implicitWidth: subRow.implicitWidth + 20
                        RowLayout {
                            id: subRow; anchors.centerIn: parent; spacing: 8
                            Label { text: (modelData.name || "Sub") + " (" + (modelData.node_count || 0) + ")"; color: "#cacaca"; font.pixelSize: 11; font.bold: true }
                            Label {
                                text: "↻"; color: "#4e97fa"; font.pixelSize: 14
                                MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: pm.refreshSubscription(modelData.id) }
                            }
                            Label {
                                text: "✕"; color: "#e01d1d"; font.pixelSize: 12
                                MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: pm.deleteSubscription(modelData.id) }
                            }
                        }
                    }
                }
            }

            // Server Cards 2-Column Grid
            GridLayout {
                Layout.fillWidth: true
                columns: 2
                columnSpacing: 10
                rowSpacing: 10

                Repeater {
                    model: pm.xrayNodes || []
                    delegate: Rectangle {
                        property bool isActiveNode: pm.xrayStatus.connected && pm.xrayStatus.nodeId === modelData.id
                        Layout.fillWidth: true
                        height: 62
                        radius: 5
                        color: isActiveNode ? "#142a1d" : (srvMa.containsMouse ? "#26262b" : "#1d1d20")
                        border.color: isActiveNode ? "#1de966" : (srvMa.containsMouse ? "#4e97fa" : "#28282c")

                        RowLayout {
                            anchors.fill: parent
                            anchors.leftMargin: 12; anchors.rightMargin: 14
                            spacing: 12

                            Rectangle {
                                width: 38; height: 28; radius: 4
                                color: isActiveNode ? "#136734" : "#282a32"
                                Label {
                                    anchors.centerIn: parent
                                    text: (modelData.country || "??").toUpperCase()
                                    color: isActiveNode ? "#1de966" : "#ffffff"
                                    font.pixelSize: 11; font.bold: true
                                }
                            }

                            ColumnLayout {
                                Layout.fillWidth: true
                                spacing: 2
                                Label {
                                    text: modelData.name || modelData.address || "Node"
                                    color: "#ffffff"; font.pixelSize: 13; font.bold: true
                                    elide: Text.ElideRight; Layout.fillWidth: true
                                }
                                RowLayout {
                                    spacing: 8
                                    Label {
                                        text: (modelData.protocol || "vless").toUpperCase()
                                        color: "#4e97fa"; font.pixelSize: 10; font.bold: true
                                    }
                                    Label {
                                        text: (modelData.address || "") + ":" + (modelData.port || "")
                                        color: "#76767e"; font.pixelSize: 11; font.family: "monospace"
                                        elide: Text.ElideRight
                                    }
                                }
                            }

                            Rectangle {
                                visible: (modelData.pingMs || 0) > 0
                                height: 20; radius: 4; color: "#222429"
                                implicitWidth: pngLbl.implicitWidth + 12
                                Label {
                                    id: pngLbl; anchors.centerIn: parent
                                    text: modelData.pingMs + " ms"
                                    color: modelData.pingMs < 120 ? "#1de966" : "#e9d81d"
                                    font.pixelSize: 10
                                }
                            }
                        }

                        MouseArea {
                            id: srvMa; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor
                            onClicked: pm.connectNode(modelData.id)
                        }
                    }
                }
            }

            Item { height: 24 }
        }
    }
}
