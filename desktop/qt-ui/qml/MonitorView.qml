import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Item {
    id: root
    signal openApp(string key)

    property string searchQuery: ""
    property bool scopeInternet: false
    property bool scopeLan: false
    property bool scopeLocal: false
    property string dirFilter: "all"     // "all" | "outbound" | "inbound"
    property string verdictFilter: "all" // "all" | "permit" | "xray" | "block"
    property bool hideDnsNoise: true
    property bool hideLocalhost: true
    property string expandedId: ""
    property var cachedFiltered: []

    function applySearch(q) {
        searchField.text = q || ""
        searchQuery = q || ""
    }

    function recompute() {
        var list = pm.connections || []
        var q = searchQuery.toLowerCase().trim()
        var anyScope = scopeInternet || scopeLan || scopeLocal
        var out = []
        for (var i = 0; i < list.length; i++) {
            var c = list[i]
            if (q !== "") {
                var hay = ((c.target || "") + " " + (c.ip || "") + " " + (c.app || "") + " " + (c.country || "") + " " + (c.asOwner || "")).toLowerCase()
                if (hay.indexOf(q) < 0) continue
            }
            if (dirFilter !== "all") {
                var dir = (c.direction || "outbound").toLowerCase()
                if (dir !== dirFilter) continue
            }
            if (anyScope) {
                var sc = (c.scope || "").toLowerCase()
                var hit = (scopeInternet && sc.indexOf("internet") >= 0) ||
                          (scopeLan && sc.indexOf("lan") >= 0) ||
                          (scopeLocal && sc.indexOf("local") >= 0)
                if (!hit) continue
            }
            if (verdictFilter !== "all") {
                var vd = (c.verdict || "").toLowerCase()
                var isBlk = (vd === "block" || vd === "drop" || vd === "failed")
                var isXray = !!c.tunneled || vd === "tunneled"
                if (verdictFilter === "block" && !isBlk) continue
                if (verdictFilter === "xray" && !isXray) continue
                if (verdictFilter === "permit" && isBlk) continue
            }
            if (hideDnsNoise) {
                var rport = c.port || 0
                if (rport === 53 || rport === 853) continue
            }
            if (hideLocalhost && !scopeLocal) {
                var scLo = (c.scope || "").toLowerCase()
                var cip = c.ip || ""
                if (scLo.indexOf("local") >= 0 || cip === "127.0.0.1" || cip === "::1" || cip.indexOf("127.") === 0 || cip === "::") continue
            }
            out.push(c)
        }
        cachedFiltered = out
    }

    onSearchQueryChanged: recompute()
    onScopeInternetChanged: recompute()
    onScopeLanChanged: recompute()
    onScopeLocalChanged: recompute()
    onDirFilterChanged: recompute()
    onVerdictFilterChanged: recompute()
    onHideDnsNoiseChanged: recompute()
    onHideLocalhostChanged: recompute()
    onVisibleChanged: if (visible) recompute()
    Component.onCompleted: recompute()
    Connections {
        target: pm
        function onConnectionsChanged() { if (root.visible) root.recompute() }
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
                    text: root.cachedFiltered.length + " / " + (pm.connections || []).length + (pm.historyLoading ? " ↻" : "")
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
                color: root.hideDnsNoise ? "#1c3354" : "#1c1c1e"
                border.color: root.hideDnsNoise ? "#4e97fa" : "#2a2a2e"
                implicitWidth: dnsHideLbl.implicitWidth + 12
                Label {
                    id: dnsHideLbl; anchors.centerIn: parent
                    text: pm.isRu ? "Скрыть DNS" : "Hide DNS"
                    color: root.hideDnsNoise ? "#89b7ff" : "#888894"
                    font.pixelSize: 10; font.bold: root.hideDnsNoise
                }
                MouseArea {
                    anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                    onClicked: root.hideDnsNoise = !root.hideDnsNoise
                }
            }

            Rectangle {
                height: 22; radius: 4
                color: root.hideLocalhost ? "#1c3354" : "#1c1c1e"
                border.color: root.hideLocalhost ? "#4e97fa" : "#2a2a2e"
                implicitWidth: localHideLbl.implicitWidth + 12
                Label {
                    id: localHideLbl; anchors.centerIn: parent
                    text: pm.isRu ? "Скрыть локалхост" : "Hide Localhost"
                    color: root.hideLocalhost ? "#89b7ff" : "#888894"
                    font.pixelSize: 10; font.bold: root.hideLocalhost
                }
                MouseArea {
                    anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                    onClicked: {
                        root.hideLocalhost = !root.hideLocalhost
                        if (root.hideLocalhost && root.scopeLocal) root.scopeLocal = false
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
                    act: !root.scopeInternet && !root.scopeLan && !root.scopeLocal
                    color: act ? "#2d2f38" : "#1c1c1e"
                    onClicked: { root.scopeInternet = false; root.scopeLan = false; root.scopeLocal = false }
                }
                ScopeChip {
                    label: pm.t("mon.internet")
                    act: root.scopeInternet
                    onClicked: root.scopeInternet = !root.scopeInternet
                }
                ScopeChip {
                    label: pm.t("mon.lan")
                    act: root.scopeLan
                    onClicked: root.scopeLan = !root.scopeLan
                }
                ScopeChip {
                    label: pm.t("mon.local")
                    act: root.scopeLocal
                    onClicked: {
                        root.scopeLocal = !root.scopeLocal
                        if (root.scopeLocal) root.hideLocalhost = false
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
                        color: root.dirFilter === modelData.id ? "#2d2f38" : "#1c1c1e"
                        border.color: root.dirFilter === modelData.id ? "#4e97fa" : "#2a2a2e"
                        Label { id: drLbl; anchors.centerIn: parent; text: modelData.label; color: root.dirFilter === modelData.id ? "#ffffff" : "#8e8e94"; font.pixelSize: 11; font.bold: root.dirFilter === modelData.id }
                        MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: root.dirFilter = modelData.id }
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

        // Green (Allowed) / Red (Blocked) Live Connections Graph Strip
        Rectangle {
            Layout.fillWidth: true
            height: 74
            radius: 6
            color: "#16171a"
            border.color: "#24262e"

            ColumnLayout {
                anchors.fill: parent
                anchors.leftMargin: 12; anchors.rightMargin: 12; anchors.topMargin: 6; anchors.bottomMargin: 6
                spacing: 2

                RowLayout {
                    Layout.fillWidth: true
                    spacing: 10
                    Label {
                        text: pm.isRu ? "График соединений (Зелёный = Разрешено • Красный = Блок)" : "Connections Graph (Green = Allowed • Red = Blocked)"
                        color: "#7e8290"; font.pixelSize: 10; font.bold: true
                    }
                    Item { Layout.fillWidth: true }
                    Rectangle { width: 7; height: 7; radius: 3.5; color: "#1de966" }
                    Label { text: pm.connAllowedTotal; color: "#1de966"; font.pixelSize: 10; font.bold: true }
                    Rectangle { width: 7; height: 7; radius: 3.5; color: "#f03c3c" }
                    Label { text: pm.connBlockedTotal; color: "#f03c3c"; font.pixelSize: 10; font.bold: true }
                }

                ConnChartCanvas {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    compact: true
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
                    model: root.cachedFiltered
                    ScrollBar.vertical: PmScrollBar {}
                    PmWheelHelper {}

                    delegate: Rectangle {
                        id: rowItem
                        width: ListView.view.width
                        property bool expanded: root.expandedId === modelData.id
                        property bool isBlock: modelData.verdict === "block" || modelData.verdict === "drop" || modelData.verdict === "failed"
                        property bool isAsk: modelData.verdict === "ask"
                        property bool isXray: !!modelData.tunneled || modelData.verdict === "tunneled"
                        height: expanded ? 92 : 34
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
                                        text: modelData.direction === "inbound" ? "↙" : "↗"
                                        color: "#76767e"; font.pixelSize: 10
                                    }
                                }

                                // Flex Col 2: Domain / Target
                                Label {
                                    text: modelData.target || "-"
                                    color: "#ffffff"; font.pixelSize: 12
                                    elide: Text.ElideRight
                                    Layout.fillWidth: true
                                    Layout.minimumWidth: 140
                                }

                                // Fixed Col 3: IP Address (126px)
                                Label {
                                    text: modelData.ip || "-"
                                    color: "#80828c"; font.pixelSize: 11; font.family: "monospace"
                                    elide: Text.ElideRight
                                    Layout.preferredWidth: 126; Layout.minimumWidth: 126; Layout.maximumWidth: 126
                                }

                                // Fixed Col 4: Country Flag + ISO (68px)
                                RowLayout {
                                    Layout.preferredWidth: 68; Layout.minimumWidth: 68; Layout.maximumWidth: 68
                                    spacing: 5
                                    FlagBadge {
                                        visible: (modelData.country || "") !== ""
                                        code: modelData.country || ""
                                        badgeWidth: 18; badgeHeight: 12
                                    }
                                    Label {
                                        text: modelData.country || "-"
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

                                // Fixed Col 6: Proto:Port (92px)
                                Label {
                                    text: (modelData.proto || "") + (modelData.port > 0 ? (":" + modelData.port) : "")
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

                                // Fixed Col 8: Traffic Bytes (76px)
                                Label {
                                    text: modelData.bytes || "0 B"
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
                        visible: root.cachedFiltered.length === 0
                        text: pm.isRu ? "Сетевая активность по фильтру не найдена." : "No matching network connections."
                        color: "#76767e"; font.pixelSize: 13
                    }
                }
            }
        }
    }
}
