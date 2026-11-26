import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Item {
    id: root
    signal openApp(string key)

    property string searchQuery: ""
    property string scopeFilter: "all"   // "all" | "internet" | "lan" | "local" | "inbound"
    property string verdictFilter: "all" // "all" | "permit" | "xray" | "block"
    property string expandedId: ""

    function filteredConnections() {
        var list = pm.connections || []
        var q = searchQuery.toLowerCase().trim()
        var out = []
        for (var i = 0; i < list.length; i++) {
            var c = list[i]
            if (q !== "") {
                var hay = ((c.target || "") + " " + (c.ip || "") + " " + (c.app || "") + " " + (c.country || "") + " " + (c.asOwner || "")).toLowerCase()
                if (hay.indexOf(q) < 0) continue
            }
            if (scopeFilter !== "all") {
                var sc = (c.scope || "").toLowerCase()
                var dir = (c.direction || "").toLowerCase()
                if (scopeFilter === "inbound") {
                    if (dir !== "inbound") continue
                } else if (sc.indexOf(scopeFilter) < 0) {
                    continue
                }
            }
            if (verdictFilter !== "all") {
                var vd = (c.verdict || "").toLowerCase()
                var isBlk = (vd === "block" || vd === "drop" || vd === "failed")
                var isXray = !!c.tunneled || vd === "tunneled"
                if (verdictFilter === "block" && !isBlk) continue
                if (verdictFilter === "xray" && !isXray) continue
                if (verdictFilter === "permit" && isBlk) continue
            }
            out.push(c)
        }
        return out
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 24
        spacing: 14

        // Top Filter Toolbar matching original Portmaster Monitor
        RowLayout {
            Layout.fillWidth: true
            spacing: 12

            Label {
                text: pm.t("mon.title")
                color: "#ffffff"
                font.pixelSize: 20
                font.bold: true
            }

            Rectangle {
                height: 22; radius: 11; color: "#222328"
                implicitWidth: cntLbl.implicitWidth + 16
                Label {
                    id: cntLbl; anchors.centerIn: parent
                    text: root.filteredConnections().length + " / " + (pm.connections || []).length
                    color: "#9e9ea6"; font.pixelSize: 11
                }
            }

            Item { Layout.fillWidth: true }

            TextField {
                Layout.preferredWidth: 270
                height: 30
                placeholderText: pm.isRu ? "Фильтр по домену, IP, приложению, ASN..." : "Filter by domain, IP, app, ASN..."
                placeholderTextColor: "#66666e"
                color: "#ffffff"
                font.pixelSize: 12
                onTextChanged: root.searchQuery = text
                background: Rectangle {
                    color: "#1b1b1d"; radius: 4
                    border.color: parent.activeFocus ? "#4e97fa" : "#2c2c30"
                }
            }

            Rectangle {
                height: 30; radius: 4
                color: clrMa.containsMouse ? "#381c1c" : "#241919"
                border.color: "#522525"
                implicitWidth: clrLbl.implicitWidth + 18
                Label { id: clrLbl; anchors.centerIn: parent; text: pm.t("mon.clear"); color: "#e88b8b"; font.pixelSize: 11 }
                MouseArea {
                    id: clrMa; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor
                    onClicked: pm.clearHistory()
                }
            }
        }

        // Scope & Verdict Chips Bar
        RowLayout {
            Layout.fillWidth: true
            spacing: 16

            Row {
                spacing: 4
                Repeater {
                    model: [
                        { id: "all",      label: pm.t("mon.all") },
                        { id: "internet", label: pm.t("mon.internet") },
                        { id: "lan",      label: pm.t("mon.lan") },
                        { id: "local",    label: pm.t("mon.local") },
                        { id: "inbound",  label: pm.t("mon.inbound") }
                    ]
                    delegate: Rectangle {
                        height: 24; radius: 12
                        implicitWidth: scLbl.implicitWidth + 16
                        color: root.scopeFilter === modelData.id ? "#2d2f38" : "#1c1c1e"
                        border.color: root.scopeFilter === modelData.id ? "#4e97fa" : "#2a2a2e"
                        Label {
                            id: scLbl; anchors.centerIn: parent
                            text: modelData.label
                            color: root.scopeFilter === modelData.id ? "#ffffff" : "#8e8e94"
                            font.pixelSize: 11; font.bold: root.scopeFilter === modelData.id
                        }
                        MouseArea {
                            anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                            onClicked: root.scopeFilter = modelData.id
                        }
                    }
                }
            }

            Row {
                spacing: 4
                Repeater {
                    model: [
                        { id: "all",    label: pm.t("mon.all"),       accent: "#4e97fa" },
                        { id: "permit", label: pm.t("mon.permitted"), accent: "#1de966" },
                        { id: "xray",   label: "Xray VPN",            accent: "#4e97fa" },
                        { id: "block",  label: pm.t("mon.blocked"),   accent: "#e01d1d" }
                    ]
                    delegate: Rectangle {
                        height: 24; radius: 12
                        implicitWidth: vdLbl.implicitWidth + 16
                        color: root.verdictFilter === modelData.id ? "#26262b" : "#1c1c1e"
                        border.color: root.verdictFilter === modelData.id ? modelData.accent : "#2a2a2e"
                        Label {
                            id: vdLbl; anchors.centerIn: parent
                            text: modelData.label
                            color: root.verdictFilter === modelData.id ? modelData.accent : "#8e8e94"
                            font.pixelSize: 11; font.bold: root.verdictFilter === modelData.id
                        }
                        MouseArea {
                            anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                            onClicked: root.verdictFilter = modelData.id
                        }
                    }
                }
            }
        }

        // Borderless Portmaster Connection Feed
        Rectangle {
            Layout.fillWidth: true
            Layout.fillHeight: true
            color: "#161618"
            radius: 6
            border.color: "#242428"

            ColumnLayout {
                anchors.fill: parent
                spacing: 0

                // Table Column Labels
                Rectangle {
                    Layout.fillWidth: true
                    height: 30
                    color: "#1b1b1e"
                    RowLayout {
                        anchors.fill: parent
                        anchors.leftMargin: 16; anchors.rightMargin: 16
                        spacing: 10
                        Label { text: ""; Layout.preferredWidth: 20 }
                        Label { text: pm.isRu ? "Адрес / Домен" : "Target / Domain"; color: "#76767e"; font.pixelSize: 10; font.bold: true; Layout.fillWidth: true }
                        Label { text: pm.isRu ? "Страна" : "Country"; color: "#76767e"; font.pixelSize: 10; font.bold: true; Layout.preferredWidth: 56 }
                        Label { text: pm.isRu ? "Приложение" : "Application"; color: "#76767e"; font.pixelSize: 10; font.bold: true; Layout.preferredWidth: 165 }
                        Label { text: pm.isRu ? "Порт" : "Port"; color: "#76767e"; font.pixelSize: 10; font.bold: true; Layout.preferredWidth: 94 }
                        Label { text: pm.isRu ? "Маршрут / Вердикт" : "Route / Verdict"; color: "#76767e"; font.pixelSize: 10; font.bold: true; Layout.preferredWidth: 115 }
                        Label { text: pm.isRu ? "Трафик" : "Traffic"; color: "#76767e"; font.pixelSize: 10; font.bold: true; Layout.preferredWidth: 74; horizontalAlignment: Text.AlignRight }
                    }
                }

                ListView {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    clip: true
                    spacing: 1
                    model: root.filteredConnections()
                    ScrollBar.vertical: PmScrollBar {}

                    delegate: Rectangle {
                        id: rowItem
                        width: ListView.view.width
                        property bool expanded: root.expandedId === modelData.id
                        property bool isBlock: modelData.verdict === "block" || modelData.verdict === "drop" || modelData.verdict === "failed"
                        property bool isAsk: modelData.verdict === "ask"
                        property bool isXray: !!modelData.tunneled || modelData.verdict === "tunneled"
                        height: expanded ? 92 : 36
                        color: expanded ? "#232328" : (rowMa.containsMouse ? "#202024" : (index % 2 === 0 ? "#18181b" : "#151517"))

                        MouseArea {
                            id: rowMa
                            z: -1
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: root.expandedId = (root.expandedId === modelData.id) ? "" : modelData.id
                        }

                        ColumnLayout {
                            anchors.fill: parent
                            anchors.leftMargin: 16; anchors.rightMargin: 16
                            anchors.topMargin: 6; anchors.bottomMargin: 6
                            spacing: 6

                            RowLayout {
                                Layout.fillWidth: true
                                spacing: 10

                                Row {
                                    Layout.preferredWidth: 20
                                    spacing: 4
                                    Rectangle {
                                        width: 8; height: 8; radius: 4; anchors.verticalCenter: parent.verticalCenter
                                        color: isBlock ? "#e01d1d" : (isAsk ? "#e9d81d" : (isXray ? "#4e97fa" : "#1de966"))
                                    }
                                    Label {
                                        text: modelData.direction === "inbound" ? "↙" : "↗"
                                        color: "#76767e"; font.pixelSize: 10
                                    }
                                }

                                RowLayout {
                                    Layout.fillWidth: true
                                    spacing: 8
                                    Label {
                                        text: modelData.target || "-"
                                        color: "#ffffff"; font.pixelSize: 12
                                        elide: Text.ElideRight; Layout.maximumWidth: 320
                                    }
                                    Label {
                                        visible: modelData.domain !== "" && modelData.ip !== "" && modelData.domain !== modelData.ip
                                        text: modelData.ip
                                        color: "#6a6a72"; font.pixelSize: 11; font.family: "monospace"
                                    }
                                }

                                Rectangle {
                                    Layout.preferredWidth: 56; height: 18; radius: 3
                                    color: modelData.country !== "" ? "#26262b" : "transparent"
                                    Label { anchors.centerIn: parent; text: modelData.country || "-"; color: "#cacaca"; font.pixelSize: 10 }
                                }

                                Item {
                                    Layout.preferredWidth: 165; height: 20
                                    Label {
                                        anchors.fill: parent
                                        verticalAlignment: Text.AlignVCenter
                                        text: modelData.app || "-"
                                        color: appJumpMa.containsMouse && modelData.profileKey !== "" ? "#4e97fa" : "#cacaca"
                                        font.pixelSize: 12; font.underline: appJumpMa.containsMouse && modelData.profileKey !== ""
                                        elide: Text.ElideRight
                                    }
                                    MouseArea {
                                        id: appJumpMa
                                        anchors.fill: parent; hoverEnabled: true
                                        enabled: (modelData.profileKey || "") !== ""
                                        cursorShape: enabled ? Qt.PointingHandCursor : Qt.ArrowCursor
                                        onClicked: if (modelData.profileKey) root.openApp(modelData.profileKey)
                                    }
                                }

                                Label {
                                    text: (modelData.proto || "") + (modelData.port > 0 ? (":" + modelData.port) : "")
                                    color: "#8a8a94"; font.pixelSize: 11; font.family: "monospace"
                                    Layout.preferredWidth: 94
                                }

                                Row {
                                    Layout.preferredWidth: 115
                                    spacing: 4
                                    Rectangle {
                                        height: 18; radius: 3
                                        width: isXray ? 56 : 74
                                        color: isBlock ? "#3d1414" : (isAsk ? "#3b3612" : (isXray ? "#173256" : "#123821"))
                                        Label {
                                            anchors.centerIn: parent
                                            text: (isXray ? "XRAY" : (modelData.verdict || "accept")).toUpperCase()
                                            color: isBlock ? "#e01d1d" : (isAsk ? "#e9d81d" : (isXray ? "#4e97fa" : "#1de966"))
                                            font.pixelSize: 9; font.bold: true
                                        }
                                    }
                                    Rectangle {
                                        visible: isXray && isBlock
                                        height: 18; width: 42; radius: 3; color: "#3d1414"
                                        Label { anchors.centerIn: parent; text: "BLK"; color: "#e01d1d"; font.pixelSize: 9; font.bold: true }
                                    }
                                }

                                Label {
                                    text: modelData.bytes || "0 B"
                                    color: "#9e9ea6"; font.pixelSize: 11
                                    Layout.preferredWidth: 74; horizontalAlignment: Text.AlignRight
                                }
                            }

                            // Expanded Inline Action Bar
                            RowLayout {
                                visible: rowItem.expanded
                                Layout.fillWidth: true
                                spacing: 10

                                Label {
                                    text: (modelData.reason || (pm.isRu ? "По умолчанию" : "Default policy"))
                                          + (modelData.scope ? (" • " + modelData.scope) : "")
                                          + (modelData.asOwner ? (" • " + modelData.asOwner) : "")
                                    color: "#8e8e98"; font.pixelSize: 11
                                    elide: Text.ElideRight; Layout.fillWidth: true
                                }

                                Rectangle {
                                    height: 24; radius: 4; color: "#136734"
                                    implicitWidth: alLbl.implicitWidth + 14
                                    Label { id: alLbl; anchors.centerIn: parent; text: pm.isRu ? "+ Правило прил." : "+ Permit App"; color: "#ffffff"; font.pixelSize: 10; font.bold: true }
                                    MouseArea {
                                        anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                                        onClicked: pm.allowOrBlockConnection(modelData.id, true)
                                    }
                                }
                                Rectangle {
                                    height: 24; radius: 4; color: "#7a1515"
                                    implicitWidth: blLbl.implicitWidth + 14
                                    Label { id: blLbl; anchors.centerIn: parent; text: pm.isRu ? "- Блок прил." : "- Block App"; color: "#ffffff"; font.pixelSize: 10; font.bold: true }
                                    MouseArea {
                                        anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                                        onClicked: pm.allowOrBlockConnection(modelData.id, false)
                                    }
                                }
                                Rectangle {
                                    height: 24; radius: 4; color: "#262834"; border.color: "#4e97fa"
                                    implicitWidth: glLbl.implicitWidth + 14
                                    Label { id: glLbl; anchors.centerIn: parent; text: pm.isRu ? "- В глобальный блок" : "- Global Block"; color: "#4e97fa"; font.pixelSize: 10; font.bold: true }
                                    MouseArea {
                                        anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                                        onClicked: {
                                            var pat = modelData.domain !== "" ? ("- ." + modelData.domain) : ("- " + modelData.ip)
                                            pm.promoteRuleToGlobal("filter/endpoints", pat)
                                        }
                                    }
                                }
                            }
                        }
                    }

                    Label {
                        anchors.centerIn: parent
                        visible: root.filteredConnections().length === 0
                        text: pm.isRu ? "Сетевая активность по фильтру не найдена." : "No matching network connections."
                        color: "#76767e"; font.pixelSize: 13
                    }
                }
            }
        }
    }
}
