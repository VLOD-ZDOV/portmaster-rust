import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Item {
    id: root
    signal openApp(string key)

    property string expandedId: ""

    function applySearch(q) {
        searchField.text = q || ""
        pm.connectionsModel.searchQuery = q || ""
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 24
        spacing: 12

        // Top Toolbar: Title, Count, History Depth (500 / 2000 / 5000), Search, Refresh, Clear
        RowLayout {
            Layout.fillWidth: true
            spacing: 10

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
                    text: pm.connectionsModel.count + " / " + pm.connectionsTotal + (pm.historyLoading ? " ↻" : "")
                    color: pm.historyLoading ? "#89b7ff" : "#9e9ea6"; font.pixelSize: 11
                }
            }

            // History Retention Depth Selector (500 .. 50000)
            Row {
                spacing: 3
                Repeater {
                    model: [ 500, 2000, 5000, 20000, 50000 ]
                    delegate: Rectangle {
                        height: 22; radius: 4
                        implicitWidth: limLbl.implicitWidth + 10
                        color: pm.historyLimit === modelData ? "#1c3354" : "#1c1c1e"
                        border.color: pm.historyLimit === modelData ? "#4e97fa" : "#2a2a2e"
                        Label {
                            id: limLbl; anchors.centerIn: parent
                            text: modelData >= 10000 ? (modelData / 1000) + "k" : modelData
                            color: pm.historyLimit === modelData ? "#ffffff" : "#888894"
                            font.pixelSize: 10; font.bold: pm.historyLimit === modelData
                        }
                        MouseArea {
                            anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                            onClicked: pm.setHistoryLimit(modelData)
                        }
                    }
                }
            }

            Rectangle {
                height: 22; radius: 4
                color: pm.connectionsModel.hideDnsNoise ? "#1c3354" : "#1c1c1e"
                border.color: pm.connectionsModel.hideDnsNoise ? "#4e97fa" : "#2a2a2e"
                implicitWidth: dnsHideLbl.implicitWidth + 12
                Label {
                    id: dnsHideLbl; anchors.centerIn: parent
                    text: pm.isRu ? "Скрыть DNS" : "Hide DNS"
                    color: pm.connectionsModel.hideDnsNoise ? "#89b7ff" : "#888894"
                    font.pixelSize: 10; font.bold: pm.connectionsModel.hideDnsNoise
                }
                MouseArea {
                    anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                    onClicked: pm.connectionsModel.hideDnsNoise = !pm.connectionsModel.hideDnsNoise
                }
            }

            Rectangle {
                height: 22; radius: 4
                color: pm.connectionsModel.hideLocalhost ? "#1c3354" : "#1c1c1e"
                border.color: pm.connectionsModel.hideLocalhost ? "#4e97fa" : "#2a2a2e"
                implicitWidth: localHideLbl.implicitWidth + 12
                Label {
                    id: localHideLbl; anchors.centerIn: parent
                    text: pm.isRu ? "Скрыть локалхост" : "Hide Localhost"
                    color: pm.connectionsModel.hideLocalhost ? "#89b7ff" : "#888894"
                    font.pixelSize: 10; font.bold: pm.connectionsModel.hideLocalhost
                }
                MouseArea {
                    anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                    onClicked: {
                        pm.connectionsModel.hideLocalhost = !pm.connectionsModel.hideLocalhost
                        if (pm.connectionsModel.hideLocalhost && pm.connectionsModel.scopeLocal) {
                            pm.connectionsModel.scopeLocal = false
                        }
                    }
                }
            }

            Rectangle {
                width: 24; height: 22; radius: 4; color: "#1c1d22"; border.color: "#2c2e38"
                Label { anchors.centerIn: parent; text: "↻"; color: "#89b7ff"; font.pixelSize: 12; font.bold: true }
                MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: pm.refreshHistory() }
            }

            Item { Layout.fillWidth: true }

            TextField {
                id: searchField
                Layout.preferredWidth: 250
                height: 30
                placeholderText: pm.isRu ? "Фильтр по домену, IP, приложению, стране..." : "Filter by domain, IP, app, country..."
                placeholderTextColor: "#66666e"
                color: "#ffffff"
                font.pixelSize: 12
                onTextChanged: pm.connectionsModel.searchQuery = text
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

        // Independent Multi-Select Scope + Direction + Verdict Chips Bar
        Flow {
            Layout.fillWidth: true
            spacing: 12

            Row {
                spacing: 4
                component ScopeChip: Rectangle {
                    id: chip
                    property string label: ""
                    property bool act: false
                    signal clicked()
                    height: 24; radius: 12
                    implicitWidth: scLbl.implicitWidth + 16
                    color: chip.act ? "#1b3352" : "#1c1c1e"
                    border.color: chip.act ? "#4e97fa" : "#2a2a2e"
                    Label { id: scLbl; anchors.centerIn: parent; text: chip.label; color: chip.act ? "#ffffff" : "#8e8e94"; font.pixelSize: 11; font.bold: chip.act }
                    MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: chip.clicked() }
                }
                ScopeChip {
                    label: pm.t("mon.all")
                    act: !pm.connectionsModel.scopeInternet && !pm.connectionsModel.scopeLan && !pm.connectionsModel.scopeLocal
                    color: act ? "#2d2f38" : "#1c1c1e"
                    onClicked: {
                        pm.connectionsModel.scopeInternet = false
                        pm.connectionsModel.scopeLan = false
                        pm.connectionsModel.scopeLocal = false
                    }
                }
                ScopeChip {
                    label: pm.t("mon.internet")
                    act: pm.connectionsModel.scopeInternet
                    onClicked: pm.connectionsModel.scopeInternet = !pm.connectionsModel.scopeInternet
                }
                ScopeChip {
                    label: pm.t("mon.lan")
                    act: pm.connectionsModel.scopeLan
                    onClicked: pm.connectionsModel.scopeLan = !pm.connectionsModel.scopeLan
                }
                ScopeChip {
                    label: pm.t("mon.local")
                    act: pm.connectionsModel.scopeLocal
                    onClicked: {
                        pm.connectionsModel.scopeLocal = !pm.connectionsModel.scopeLocal
                        if (pm.connectionsModel.scopeLocal) pm.connectionsModel.hideLocalhost = false
                    }
                }
            }

            Row {
                spacing: 4
                Repeater {
                    model: [
                        { id: "all",      label: pm.isRu ? "Все напр." : "Any Dir" },
                        { id: "outbound", label: pm.isRu ? "↗ Исходящие" : "↗ Outbound" },
                        { id: "inbound",  label: "↙ " + pm.t("mon.inbound") }
                    ]
                    delegate: Rectangle {
                        height: 24; radius: 12
                        implicitWidth: drLbl.implicitWidth + 16
                        color: pm.connectionsModel.dirFilter === modelData.id ? "#2d2f38" : "#1c1c1e"
                        border.color: pm.connectionsModel.dirFilter === modelData.id ? "#4e97fa" : "#2a2a2e"
                        Label { id: drLbl; anchors.centerIn: parent; text: modelData.label; color: pm.connectionsModel.dirFilter === modelData.id ? "#ffffff" : "#8e8e94"; font.pixelSize: 11; font.bold: pm.connectionsModel.dirFilter === modelData.id }
                        MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: pm.connectionsModel.dirFilter = modelData.id }
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
                        color: pm.connectionsModel.verdictFilter === modelData.id ? "#26262b" : "#1c1c1e"
                        border.color: pm.connectionsModel.verdictFilter === modelData.id ? modelData.accent : "#2a2a2e"
                        Label {
                            id: vdLbl; anchors.centerIn: parent
                            text: modelData.label
                            color: pm.connectionsModel.verdictFilter === modelData.id ? modelData.accent : "#8e8e94"
                            font.pixelSize: 11; font.bold: pm.connectionsModel.verdictFilter === modelData.id
                        }
                        MouseArea {
                            anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                            onClicked: pm.connectionsModel.verdictFilter = modelData.id
                        }
                    }
                }
            }
        }

        // Green (Allowed) / Red (Blocked) Live Connections or Traffic Graph Strip
        Rectangle {
            Layout.fillWidth: true
            height: 104
            radius: 6
            color: "#16171a"
            border.color: "#24262e"

            ColumnLayout {
                anchors.fill: parent
                anchors.leftMargin: 12; anchors.rightMargin: 12; anchors.topMargin: 6; anchors.bottomMargin: 6
                spacing: 4

                PmChartHeader {
                    compact: true
                    showInternetToggle: true
                    internetOnly: pm.dashboardChartInternetOnly
                    onToggleInternet: pm.setDashboardChartInternetOnly(!pm.dashboardChartInternetOnly)
                    title: pm.isRu
                           ? (pm.chartDataMode ? "График данных (RX / TX)" : "График соединений (Разрешено / Блок)")
                           : (pm.chartDataMode ? "Data Volume Graph (RX / TX)" : "Connections Graph (Allowed / Blocked)")
                    okCount: pm.dashboardChartInternetOnly ? pm.connInetAllowedTotal : pm.connAllowedTotal
                    blockedCount: pm.dashboardChartInternetOnly ? pm.connInetBlockedTotal : pm.connBlockedTotal
                    rxStr: pm.dashboardChartInternetOnly ? pm.connInetRxStr : pm.connRxStr
                    txStr: pm.dashboardChartInternetOnly ? pm.connInetTxStr : pm.connTxStr
                }

                ConnChartCanvas {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    compact: true
                    showAxes: true
                    dataMode: pm.chartDataMode
                    bins: pm.connChartBins
                }
            }
        }

        // Strictly Aligned Portmaster Connection Table
        Rectangle {
            Layout.fillWidth: true
            Layout.fillHeight: true
            color: "#161618"
            radius: 6
            border.color: "#242428"

            ColumnLayout {
                anchors.fill: parent
                spacing: 0

                // Rigid Locked Header Columns
                Rectangle {
                    Layout.fillWidth: true
                    height: 30
                    color: "#1b1b1e"
                    RowLayout {
                        anchors.fill: parent
                        anchors.leftMargin: 14; anchors.rightMargin: 14
                        spacing: 10
                        Label { text: ""; Layout.preferredWidth: 26; Layout.minimumWidth: 26; Layout.maximumWidth: 26 }
                        Label { text: pm.isRu ? "Адрес / Домен" : "Target / Domain"; color: "#76767e"; font.pixelSize: 10; font.bold: true; Layout.fillWidth: true; Layout.minimumWidth: 140 }
                        Label { text: "IP"; color: "#76767e"; font.pixelSize: 10; font.bold: true; Layout.preferredWidth: 126; Layout.minimumWidth: 126; Layout.maximumWidth: 126 }
                        Label { text: pm.isRu ? "Страна" : "Country"; color: "#76767e"; font.pixelSize: 10; font.bold: true; Layout.preferredWidth: 68; Layout.minimumWidth: 68; Layout.maximumWidth: 68 }
                        Label { text: pm.isRu ? "Приложение" : "Application"; color: "#76767e"; font.pixelSize: 10; font.bold: true; Layout.preferredWidth: 170; Layout.minimumWidth: 170; Layout.maximumWidth: 170 }
                        Label { text: pm.isRu ? "Порт" : "Port"; color: "#76767e"; font.pixelSize: 10; font.bold: true; Layout.preferredWidth: 92; Layout.minimumWidth: 92; Layout.maximumWidth: 92 }
                        Label { text: pm.isRu ? "Вердикт" : "Verdict"; color: "#76767e"; font.pixelSize: 10; font.bold: true; Layout.preferredWidth: 112; Layout.minimumWidth: 112; Layout.maximumWidth: 112 }
                        Label { text: pm.isRu ? "Трафик" : "Traffic"; color: "#76767e"; font.pixelSize: 10; font.bold: true; Layout.preferredWidth: 76; Layout.minimumWidth: 76; Layout.maximumWidth: 76; horizontalAlignment: Text.AlignRight }
                    }
                }

                ListView {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    clip: true
                    cacheBuffer: 1200
                    spacing: 1
                    model: pm.connectionsModel
                    ScrollBar.vertical: PmScrollBar {}
                    PmWheelHelper {}

                    delegate: Rectangle {
                        id: rowItem
                        width: ListView.view.width
                        property bool expanded: root.expandedId === model.connId
                        property bool isBlock: model.verdict === "block" || model.verdict === "drop" || model.verdict === "failed"
                        property bool isAsk: model.verdict === "ask"
                        property bool isXray: !!model.tunneled || model.verdict === "tunneled"
                        height: expanded ? 92 : 34
                        color: expanded ? "#232328" : (rowMa.containsMouse ? "#202024" : (index % 2 === 0 ? "#18181b" : "#151517"))

                        MouseArea {
                            id: rowMa
                            z: -1
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: root.expandedId = (root.expandedId === model.connId) ? "" : model.connId
                        }

                        ColumnLayout {
                            anchors.fill: parent
                            anchors.leftMargin: 14; anchors.rightMargin: 14
                            anchors.topMargin: 5; anchors.bottomMargin: 5
                            spacing: 6

                            RowLayout {
                                Layout.fillWidth: true
                                spacing: 10

                                // Fixed Col 1: Status + Direction (26px)
                                Row {
                                    Layout.preferredWidth: 26; Layout.minimumWidth: 26; Layout.maximumWidth: 26
                                    spacing: 5
                                    Rectangle {
                                        width: 8; height: 8; radius: 4; anchors.verticalCenter: parent.verticalCenter
                                        color: isBlock ? "#e01d1d" : (isAsk ? "#e9d81d" : (isXray ? "#4e97fa" : "#1de966"))
                                    }
                                    Label {
                                        text: model.direction === "inbound" ? "↙" : "↗"
                                        color: "#76767e"; font.pixelSize: 10
                                    }
                                }

                                // Flex Col 2: Domain / Target
                                Label {
                                    text: model.target || "-"
                                    color: "#ffffff"; font.pixelSize: 12
                                    elide: Text.ElideRight
                                    Layout.fillWidth: true
                                    Layout.minimumWidth: 140
                                }

                                // Fixed Col 3: IP Address (126px)
                                Label {
                                    text: model.ip || "-"
                                    color: "#80828c"; font.pixelSize: 11; font.family: "monospace"
                                    elide: Text.ElideRight
                                    Layout.preferredWidth: 126; Layout.minimumWidth: 126; Layout.maximumWidth: 126
                                }

                                // Fixed Col 4: Country Flag + ISO (68px)
                                RowLayout {
                                    Layout.preferredWidth: 68; Layout.minimumWidth: 68; Layout.maximumWidth: 68
                                    spacing: 5
                                    FlagBadge {
                                        visible: (model.country || "") !== ""
                                        code: model.country || ""
                                        badgeWidth: 18; badgeHeight: 12
                                    }
                                    Label {
                                        text: model.country || "-"
                                        color: "#cacaca"; font.pixelSize: 10; font.family: "monospace"
                                        elide: Text.ElideRight
                                        Layout.fillWidth: true
                                    }
                                }

                                // Fixed Col 5: Application Name (170px)
                                Item {
                                    Layout.preferredWidth: 170; Layout.minimumWidth: 170; Layout.maximumWidth: 170
                                    height: 20
                                    Label {
                                        anchors.fill: parent
                                        verticalAlignment: Text.AlignVCenter
                                        text: model.app || "-"
                                        color: appJumpMa.containsMouse && model.profileKey !== "" ? "#4e97fa" : "#cacaca"
                                        font.pixelSize: 12; font.underline: appJumpMa.containsMouse && model.profileKey !== ""
                                        elide: Text.ElideRight
                                    }
                                    MouseArea {
                                        id: appJumpMa
                                        anchors.fill: parent; hoverEnabled: true
                                        enabled: (model.profileKey || "") !== ""
                                        cursorShape: enabled ? Qt.PointingHandCursor : Qt.ArrowCursor
                                        onClicked: if (model.profileKey) root.openApp(model.profileKey)
                                    }
                                }

                                // Fixed Col 6: Proto:Port (92px)
                                Label {
                                    text: (model.proto || "") + (model.port > 0 ? (":" + model.port) : "")
                                    color: "#8a8a94"; font.pixelSize: 11; font.family: "monospace"
                                    elide: Text.ElideRight
                                    Layout.preferredWidth: 92; Layout.minimumWidth: 92; Layout.maximumWidth: 92
                                }

                                // Fixed Col 7: Verdict / Route Badge (112px)
                                Row {
                                    Layout.preferredWidth: 112; Layout.minimumWidth: 112; Layout.maximumWidth: 112
                                    spacing: 4
                                    Rectangle {
                                        height: 18; radius: 3
                                        width: isXray ? 56 : 74
                                        color: isBlock ? "#3d1414" : (isAsk ? "#3b3612" : (isXray ? "#173256" : "#123821"))
                                        Label {
                                            anchors.centerIn: parent
                                            text: (isXray ? "XRAY" : (model.verdict || "accept")).toUpperCase()
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

                                // Fixed Col 8: Traffic Bytes (76px)
                                Label {
                                    text: model.bytes || "0 B"
                                    color: "#9e9ea6"; font.pixelSize: 11
                                    Layout.preferredWidth: 76; Layout.minimumWidth: 76; Layout.maximumWidth: 76
                                    horizontalAlignment: Text.AlignRight
                                }
                            }

                            // Expanded Inline Action Bar
                            RowLayout {
                                visible: rowItem.expanded
                                Layout.fillWidth: true
                                spacing: 10

                                Label {
                                    text: (model.reason || (pm.isRu ? "По умолчанию" : "Default policy"))
                                          + (model.scope ? (" • " + model.scope) : "")
                                          + (model.asOwner ? (" • " + model.asOwner) : "")
                                    color: "#8e8e98"; font.pixelSize: 11
                                    elide: Text.ElideRight; Layout.fillWidth: true
                                }

                                Rectangle {
                                    height: 24; radius: 4; color: "#136734"
                                    implicitWidth: alLbl.implicitWidth + 14
                                    Label { id: alLbl; anchors.centerIn: parent; text: pm.isRu ? "+ Правило прил." : "+ Permit App"; color: "#ffffff"; font.pixelSize: 10; font.bold: true }
                                    MouseArea {
                                        anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                                        onClicked: pm.allowOrBlockConnection(model.connId, true)
                                    }
                                }
                                Rectangle {
                                    height: 24; radius: 4; color: "#7a1515"
                                    implicitWidth: blLbl.implicitWidth + 14
                                    Label { id: blLbl; anchors.centerIn: parent; text: pm.isRu ? "- Блок прил." : "- Block App"; color: "#ffffff"; font.pixelSize: 10; font.bold: true }
                                    MouseArea {
                                        anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                                        onClicked: pm.allowOrBlockConnection(model.connId, false)
                                    }
                                }
                                Rectangle {
                                    height: 24; radius: 4; color: "#262834"; border.color: "#4e97fa"
                                    implicitWidth: glLbl.implicitWidth + 14
                                    Label { id: glLbl; anchors.centerIn: parent; text: pm.isRu ? "- В глобальный блок" : "- Global Block"; color: "#4e97fa"; font.pixelSize: 10; font.bold: true }
                                    MouseArea {
                                        anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                                        onClicked: {
                                            var pat = model.domain !== "" ? ("- ." + model.domain) : ("- " + model.ip)
                                            pm.promoteRuleToGlobal("filter/endpoints", pat)
                                        }
                                    }
                                }
                            }
                        }
                    }

                    Label {
                        anchors.centerIn: parent
                        visible: pm.connectionsModel.count === 0
                        text: pm.isRu ? "Сетевая активность по фильтру не найдена." : "No matching network connections."
                        color: "#76767e"; font.pixelSize: 13
                    }
                }
            }
        }
    }
}
