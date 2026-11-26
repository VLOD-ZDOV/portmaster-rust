import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Item {
    id: root
    signal openMonitorPreset(bool internetOnly, string verdict, string query)
    signal openAppsActive()
    signal openApp(string key)
    signal openXray()
    signal openSettingSub(string sub)

    property string hoveredCountry: ""

    function boolSetting(key, fallback) {
        var list = pm.settingsList || []
        for (var i = 0; i < list.length; i++) {
            if (list[i].key === key) return !!list[i].boolVal
        }
        return fallback
    }

    Connections {
        target: pm
        function onWorldMapReady() { dashMapCanvas.requestPaint() }
        function onConnectionsChanged() { dashMapCanvas.requestPaint() }
        function onXrayChanged() { dashMapCanvas.requestPaint() }
    }
    onHoveredCountryChanged: dashMapHover.requestPaint()

    ScrollView {
        anchors.fill: parent
        contentWidth: availableWidth
        clip: true
        ScrollBar.vertical: PmScrollBar {}

        ColumnLayout {
            width: parent.width - 48
            x: 24; y: 20
            spacing: 16

            // 1. Header Bar + Xray Gateway Quick Controls
            RowLayout {
                Layout.fillWidth: true
                spacing: 14

                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: 2
                    Label {
                        text: pm.isRu ? "Панель управления" : "Dashboard"
                        color: "#ffffff"
                        font.pixelSize: 22
                        font.bold: true
                    }
                    Label {
                        text: pm.isRu
                              ? "Быстрые переключатели защиты, статистика активности и география подключений"
                              : "Quick protection switches, activity summary, and connection geography"
                        color: "#8e8e98"
                        font.pixelSize: 12
                    }
                }

                // Xray Status Pill
                RowLayout {
                    spacing: 10
                    ColumnLayout {
                        spacing: 1
                        RowLayout {
                            Layout.alignment: Qt.AlignRight
                            spacing: 6
                            Rectangle {
                                width: 8; height: 8; radius: 4
                                color: pm.xrayStatus.connected ? "#1de966" : "#76767e"
                            }
                            Label {
                                text: pm.xrayStatus.connected
                                      ? (pm.isRu ? "Xray: Подключен" : "Xray: Connected")
                                      : (pm.isRu ? "Xray: Отключен" : "Xray: Disconnected")
                                color: pm.xrayStatus.connected ? "#1de966" : "#ababab"
                                font.pixelSize: 12; font.bold: true
                            }
                        }
                        RowLayout {
                            Layout.alignment: Qt.AlignRight
                            spacing: 6
                            visible: !!pm.xrayStatus.connected
                            FlagBadge {
                                visible: (pm.xrayStatus.exitCountry || "") !== ""
                                code: pm.xrayStatus.exitCountry || ""
                                badgeWidth: 18; badgeHeight: 12
                            }
                            Label {
                                text: (pm.xrayStatus.nodeName || "") + ((pm.xrayStatus.exitIp || "") !== "" ? (" • " + pm.xrayStatus.exitIp) : "")
                                color: "#9e9ea6"; font.pixelSize: 11
                            }
                        }
                    }

                    Rectangle {
                        height: 30; radius: 4
                        color: pm.xrayStatus.connected ? "#7a1515" : "#154428"
                        border.color: pm.xrayStatus.connected ? "#e01d1d" : "#1de966"
                        implicitWidth: xBtnLbl.implicitWidth + 20
                        Label {
                            id: xBtnLbl; anchors.centerIn: parent
                            text: pm.xrayStatus.connected ? pm.t("xray.disconnect") : pm.t("xray.connect")
                            color: "#ffffff"; font.pixelSize: 11; font.bold: true
                        }
                        MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: pm.toggleXrayConnect() }
                    }

                    Rectangle {
                        height: 30; radius: 4; color: "#22242c"; border.color: "#363846"
                        implicitWidth: mngLbl.implicitWidth + 18
                        Label { id: mngLbl; anchors.centerIn: parent; text: pm.isRu ? "Карта Xray →" : "Xray Map →"; color: "#89b7ff"; font.pixelSize: 11; font.bold: true }
                        MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: root.openXray() }
                    }
                }
            }

            // 2. Features & Quick Switches Grid
            Label {
                text: pm.isRu ? "ФУНКЦИИ И ПЕРЕКЛЮЧАТЕЛИ ЗАЩИТЫ" : "PROTECTION FEATURES & SWITCHES"
                color: "#888890"; font.pixelSize: 10; font.bold: true
            }

            GridLayout {
                Layout.fillWidth: true
                columns: Math.max(2, Math.floor((parent.width + 10) / 275))
                columnSpacing: 10
                rowSpacing: 10

                component DashFeatureCard: Rectangle {
                    id: fCard
                    property string title: ""
                    property string subtitle: ""
                    property string accent: "#4e97fa"
                    property bool active: false
                    property bool hasToggle: true
                    signal toggled(bool checked)
                    signal cardClicked()

                    Layout.fillWidth: true
                    height: 76
                    radius: 6
                    color: cardMa.containsMouse ? "#212228" : "#1b1b1f"
                    border.color: active ? accent : "#2a2a32"

                    RowLayout {
                        anchors.fill: parent
                        anchors.margins: 12
                        spacing: 10

                        ColumnLayout {
                            Layout.fillWidth: true
                            spacing: 3
                            Label { text: fCard.title; color: "#ffffff"; font.pixelSize: 12; font.bold: true; elide: Text.ElideRight; Layout.fillWidth: true }
                            Label { text: fCard.subtitle; color: "#8e8e98"; font.pixelSize: 10; elide: Text.ElideRight; Layout.fillWidth: true }
                        }

                        PmSwitch {
                            visible: fCard.hasToggle
                            checked: fCard.active
                            onToggled: fCard.toggled(checked)
                        }
                    }
                    MouseArea {
                        id: cardMa
                        z: -1
                        anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor
                        onClicked: fCard.cardClicked()
                    }
                }

                DashFeatureCard {
                    title: pm.isRu ? "Xray VPN для всех приложений" : "Xray VPN Global Routing"
                    subtitle: pm.globalVpn ? (pm.isRu ? "Всё идёт через туннель по умолч." : "Routing all apps by default") : (pm.isRu ? "Выборочно по приложениям" : "Selective per-app routing")
                    active: pm.globalVpn
                    accent: "#1de966"
                    onToggled: function(ch) { pm.updateSetting("spn/use", ch) }
                    onCardClicked: root.openXray()
                }

                DashFeatureCard {
                    title: pm.isRu ? "Только защищённый DNS" : "Secure DNS Only (DoT/DoH)"
                    subtitle: pm.dnsSummary
                    active: pm.dnsNoInsecure
                    accent: "#4e97fa"
                    onToggled: function(ch) { pm.updateSetting("dns/noInsecureProtocols", ch) }
                    onCardClicked: root.openSettingSub("network")
                }

                DashFeatureCard {
                    title: pm.isRu ? "Блокировать входящие" : "Block Inbound Connections"
                    subtitle: pm.isRu ? "Скрывает незапрошенные порты" : "Blocks unsolicited inbound traffic"
                    active: pm.blockInbound
                    accent: "#4e97fa"
                    onToggled: function(ch) { pm.updateSetting("filter/blockInbound", ch) }
                    onCardClicked: root.openSettingSub("network")
                }

                DashFeatureCard {
                    title: pm.isRu ? "История соединений" : "Network Activity History"
                    subtitle: pm.isRu ? "Сохранение истории в SQLite БД" : "Persist connection history in SQLite"
                    active: pm.historyEnabled
                    accent: "#1de966"
                    onToggled: function(ch) { pm.updateSetting("history/enable", ch) }
                    onCardClicked: root.openMonitorPreset(false, "all", "")
                }

                DashFeatureCard {
                    title: pm.isRu ? "Блокировать весь Интернет" : "Force Block Internet"
                    subtitle: pm.blockInternet ? (pm.isRu ? "Аварийный рубильник ВКЛ" : "Kill-switch active") : (pm.isRu ? "Выход в интернет разрешён" : "Internet access allowed")
                    active: pm.blockInternet
                    accent: "#e01d1d"
                    onToggled: function(ch) { pm.updateSetting("filter/blockInternet", ch) }
                    onCardClicked: root.openSettingSub("network")
                }

                DashFeatureCard {
                    title: pm.isRu ? "Блокировать LAN и P2P" : "Block LAN Traffic"
                    subtitle: pm.isRu ? "Изоляция локальной подсети" : "Isolate local subnet access"
                    active: pm.blockLan
                    accent: "#e9d81d"
                    onToggled: function(ch) { pm.updateSetting("filter/blockLAN", ch) }
                    onCardClicked: root.openSettingSub("network")
                }
            }

            // 3. Recent Activity Clickable Mini Stats (1:1 Angular Dashboard!)
            Label {
                text: pm.isRu ? "НЕДАВНЯЯ АКТИВНОСТЬ (НАЖМИТЕ ДЛЯ ПЕРЕХОДА К ФИЛЬТРУ)" : "RECENT ACTIVITY (CLICK TO FILTER MONITOR)"
                color: "#888890"; font.pixelSize: 10; font.bold: true
            }

            RowLayout {
                Layout.fillWidth: true
                spacing: 10

                component MiniStatCard: Rectangle {
                    property string label: ""
                    property string val: "0"
                    property string hint: ""
                    property string accent: "#ffffff"
                    signal statClicked()

                    Layout.fillWidth: true
                    height: 82
                    radius: 6
                    color: msMa.containsMouse ? "#23252d" : "#1b1b1f"
                    border.color: msMa.containsMouse ? accent : "#282830"

                    ColumnLayout {
                        anchors.fill: parent
                        anchors.margins: 12
                        spacing: 3
                        Label { text: label; color: "#9e9ea6"; font.pixelSize: 10; elide: Text.ElideRight; Layout.fillWidth: true }
                        Label { text: val; color: accent; font.pixelSize: 22; font.bold: true }
                        Label { visible: hint !== ""; text: hint; color: "#686e7a"; font.pixelSize: 9; elide: Text.ElideRight; Layout.fillWidth: true }
                    }
                    MouseArea {
                        id: msMa; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor
                        onClicked: parent.statClicked()
                    }
                }

                MiniStatCard {
                    label: pm.isRu ? "Заблокировано (Инет / Всего)" : "Blocked (Internet / Total)"
                    val: pm.connInternetBlockedTotal + " / " + pm.connBlockedTotal
                    hint: pm.isRu ? "Открыть: Интернет + Блок" : "Open: Internet + Blocked"
                    accent: "#f03c3c"
                    onStatClicked: root.openMonitorPreset(true, "block", "")
                }

                MiniStatCard {
                    label: pm.isRu ? "Активные соединения" : "Active Connections"
                    val: "" + pm.connActiveTotal
                    hint: pm.isRu ? "Открыть монитор" : "Open network monitor"
                    accent: "#1de966"
                    onStatClicked: root.openMonitorPreset(false, "all", "")
                }

                MiniStatCard {
                    label: pm.isRu ? "Активные приложения" : "Active Apps"
                    val: "" + pm.activeApps.length
                    hint: pm.isRu ? "Фильтр активных программ" : "View active apps"
                    accent: "#4e97fa"
                    onStatClicked: root.openAppsActive()
                }

                MiniStatCard {
                    label: pm.isRu ? "Туннель Xray (соединений)" : "Tunneled via Xray"
                    val: "" + pm.connTunneledTotal
                    hint: pm.isRu ? "Показать трафик Xray" : "Filter Xray traffic"
                    accent: "#4e97fa"
                    onStatClicked: root.openMonitorPreset(false, "xray", "")
                }

                MiniStatCard {
                    label: pm.isRu ? "Объём (Получено / Отдано)" : "Traffic (RX / TX)"
                    val: pm.connRxStr
                    hint: "↑ " + pm.connTxStr
                    accent: "#9fd0ff"
                    onStatClicked: root.openMonitorPreset(false, "all", "")
                }
            }

            // 4. Activity Graph Card (Green Allowed / Red Blocked)
            Rectangle {
                Layout.fillWidth: true
                height: 146
                radius: 6
                color: "#18191d"
                border.color: "#272932"

                ColumnLayout {
                    anchors.fill: parent
                    anchors.margins: 12
                    spacing: 6

                    RowLayout {
                        Layout.fillWidth: true
                        Label {
                            text: pm.isRu ? "График активности: Разрешённые и Заблокированные соединения" : "Activity Graph: Permitted & Blocked Connections"
                            color: "#dcdce4"; font.pixelSize: 12; font.bold: true
                        }
                        Item { Layout.fillWidth: true }
                        Rectangle { width: 8; height: 8; radius: 4; color: "#1de966" }
                        Label { text: (pm.isRu ? "OK: " : "OK: ") + pm.connAllowedTotal; color: "#1de966"; font.pixelSize: 11; font.bold: true }
                        Item { width: 10 }
                        Rectangle { width: 8; height: 8; radius: 4; color: "#e01d1d" }
                        Label { text: (pm.isRu ? "Блок: " : "Blocked: ") + pm.connBlockedTotal; color: "#f03c3c"; font.pixelSize: 11; font.bold: true }
                    }

                    ConnChartCanvas {
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        bins: pm.connChartBins
                    }
                }
                MouseArea {
                    anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                    onClicked: root.openMonitorPreset(false, "all", "")
                }
            }

            // 5. Recent Countries & Recently Blocked Apps + Recent Connected Countries Map
            RowLayout {
                Layout.fillWidth: true
                spacing: 14

                // Left: Recent Countries + Recently Blocked Apps
                ColumnLayout {
                    Layout.fillWidth: true
                    Layout.preferredWidth: 1
                    spacing: 12

                    Rectangle {
                        Layout.fillWidth: true
                        implicitHeight: cCol.implicitHeight + 24
                        radius: 6; color: "#18191d"; border.color: "#272932"

                        ColumnLayout {
                            id: cCol
                            anchors.fill: parent; anchors.margins: 12; spacing: 8
                            Label {
                                text: pm.isRu ? "Недавние подключения по странам" : "Recent Connections by Country"
                                color: "#dcdce4"; font.pixelSize: 12; font.bold: true
                            }
                            Label {
                                visible: (pm.connCountriesSummary || []).length === 0
                                text: pm.isRu ? "Пока нет внешних подключений по странам." : "No external country connections recorded yet."
                                color: "#686c78"; font.pixelSize: 11
                            }
                            Flow {
                                Layout.fillWidth: true
                                spacing: 6
                                Repeater {
                                    model: pm.connCountriesSummary || []
                                    delegate: Rectangle {
                                        height: 30; radius: 5
                                        color: cMa.containsMouse ? "#282b36" : "#20222a"
                                        border.color: cMa.containsMouse ? "#4e97fa" : "#2c2f3a"
                                        implicitWidth: crRow.implicitWidth + 16
                                        RowLayout {
                                            id: crRow; anchors.centerIn: parent; spacing: 6
                                            FlagBadge { code: modelData.code; badgeWidth: 20; badgeHeight: 14 }
                                            Label { text: modelData.name || modelData.code; color: "#ffffff"; font.pixelSize: 11 }
                                            Label { text: modelData.count; color: "#4e97fa"; font.pixelSize: 11; font.bold: true }
                                        }
                                        MouseArea {
                                            id: cMa; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor
                                            onEntered: root.hoveredCountry = modelData.code
                                            onExited: root.hoveredCountry = ""
                                            onClicked: root.openMonitorPreset(true, "all", modelData.code)
                                        }
                                    }
                                }
                            }
                        }
                    }

                    Rectangle {
                        Layout.fillWidth: true
                        implicitHeight: bCol.implicitHeight + 24
                        radius: 6; color: "#18191d"; border.color: "#272932"

                        ColumnLayout {
                            id: bCol
                            anchors.fill: parent; anchors.margins: 12; spacing: 8
                            Label {
                                text: pm.isRu ? "Недавно заблокированные приложения" : "Recently Blocked Applications"
                                color: "#dcdce4"; font.pixelSize: 12; font.bold: true
                            }
                            Label {
                                visible: (pm.blockedAppsSummary || []).length === 0
                                text: pm.isRu ? "Ни одно приложение не было заблокировано." : "No recently blocked applications."
                                color: "#686c78"; font.pixelSize: 11
                            }
                            Flow {
                                Layout.fillWidth: true
                                spacing: 6
                                Repeater {
                                    model: pm.blockedAppsSummary || []
                                    delegate: Rectangle {
                                        height: 34; radius: 5
                                        color: baMa.containsMouse ? "#2e2024" : "#221a1c"
                                        border.color: baMa.containsMouse ? "#e01d1d" : "#3b2426"
                                        implicitWidth: baRow.implicitWidth + 16
                                        RowLayout {
                                            id: baRow; anchors.centerIn: parent; spacing: 6
                                            Rectangle {
                                                width: 20; height: 20; radius: 5
                                                property bool ready: baIcon.status === Image.Ready && baIcon.implicitWidth > 1
                                                color: ready ? "transparent" : (modelData.avatarColor || "#383842")
                                                Image {
                                                    id: baIcon; anchors.fill: parent; fillMode: Image.PreserveAspectFit; smooth: true
                                                    source: "image://appicon/" + modelData.iconKey + "?v=" + pm.iconVersion
                                                    visible: parent.ready
                                                }
                                                Label { visible: !parent.ready; anchors.centerIn: parent; text: modelData.initials; color: "#fff"; font.pixelSize: 8; font.bold: true }
                                            }
                                            Label { text: modelData.name; color: "#f5dcdc"; font.pixelSize: 11 }
                                            Label { text: modelData.count; color: "#f03c3c"; font.pixelSize: 11; font.bold: true }
                                        }
                                        MouseArea {
                                            id: baMa; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor
                                            onClicked: root.openApp(modelData.profileKey)
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Right: Connected Countries World Map (borderless unless hovered!)
                Rectangle {
                    id: dashMapRect
                    Layout.fillWidth: true
                    Layout.preferredWidth: 1.25
                    height: 290
                    radius: 6
                    color: "#111112"
                    border.color: "#272932"
                    clip: true

                    function drawCountryFlat(ctx, c, w, h) {
                        var cpolys = c.polygons || []
                        for (var k = 0; k < cpolys.length; k++) {
                            var pts = cpolys[k]
                            if (pts.length < 6) continue
                            ctx.beginPath()
                            ctx.moveTo(pts[0] * w, pts[1] * h)
                            for (var j = 2; j < pts.length; j += 2) {
                                ctx.lineTo(pts[j] * w, pts[j + 1] * h)
                            }
                            ctx.closePath()
                            ctx.fill()
                            ctx.stroke()
                        }
                    }

                    Canvas {
                        id: dashMapCanvas
                        anchors.fill: parent
                        onPaint: {
                            var ctx = getContext("2d")
                            ctx.clearRect(0, 0, width, height)
                            var cs = pm.worldCountries || []
                            var ccList = pm.connCountriesSummary || []
                            var activeSet = {}
                            for (var ci = 0; ci < ccList.length; ci++) {
                                activeSet[(ccList[ci].code || "").toUpperCase()] = true
                            }
                            for (var i = 0; i < cs.length; i++) {
                                var c = cs[i]
                                var code = c.code || ""
                                var hasConn = !!activeSet[code]
                                var fill = hasConn ? "#1f4d36" : "#1e1e1e"
                                var stroke = hasConn ? "#1de966" : "#1e1e1e"
                                ctx.fillStyle = fill
                                ctx.strokeStyle = stroke
                                ctx.lineWidth = hasConn ? 1.1 : 1.5
                                dashMapRect.drawCountryFlat(ctx, c, width, height)
                            }
                        }
                        Component.onCompleted: requestPaint()
                    }

                    Canvas {
                        id: dashMapHover
                        anchors.fill: parent
                        onPaint: {
                            var ctx = getContext("2d")
                            ctx.clearRect(0, 0, width, height)
                            if (root.hoveredCountry === "") return
                            var cs = pm.worldCountries || []
                            for (var i = 0; i < cs.length; i++) {
                                if ((cs[i].code || "") === root.hoveredCountry) {
                                    ctx.fillStyle = "#315b96"
                                    ctx.strokeStyle = "#85b5ff"
                                    ctx.lineWidth = 1.6
                                    dashMapRect.drawCountryFlat(ctx, cs[i], width, height)
                                    break
                                }
                            }
                        }
                    }

                    Label {
                        anchors.left: parent.left; anchors.top: parent.top; anchors.margins: 10
                        text: pm.isRu ? "Карта стран подключений" : "Connected Countries Map"
                        color: "#9e9ea6"; font.pixelSize: 11; font.bold: true
                    }
                }
            }

            Item { height: 24 }
        }
    }
}
