import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Item {
    id: root
    signal goBack()
    signal openNetworkSettings()
    signal openRuleBuilder(string appKey, string settingKey, string mode)

    property int activeTab: 0 // 0=Connections, 1=Rules, 2=Settings, 3=Details
    property var app: pm.selectedApp
    property string ruleDir: "+"
    property string testPathInput: ""
    property string testPathResult: ""
    property string appSetSearch: ""
    property string appSetFilter: "all" // "all" | "custom" | "filter" | "spn" | "history"
    property int appMaxExpertise: 2
    property bool confirmDelete: false

    function filteredAppSettings() {
        var list = pm.selectedAppSettings || []
        var q = appSetSearch.toLowerCase().trim()
        var out = []
        for (var i = 0; i < list.length; i++) {
            var st = list[i]
            if ((st.expertise || 0) > appMaxExpertise) continue
            if (appSetFilter === "custom" && !st.modified) continue
            if (appSetFilter === "filter" && st.subsystem !== "filter") continue
            if (appSetFilter === "spn" && st.subsystem !== "spn" && st.subsystem !== "splittun") continue
            if (appSetFilter === "history" && st.subsystem !== "history") continue
            if (q !== "") {
                var hay = ((st.key || "") + " " + (st.name || "") + " " + (st.description || "")).toLowerCase()
                if (hay.indexOf(q) < 0) continue
            }
            out.push(st)
        }
        return out
    }

    onAppChanged: {
        confirmDelete = false
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 24
        spacing: 16

        // Top Breadcrumb & Action Bar
        RowLayout {
            Layout.fillWidth: true
            spacing: 12

            Rectangle {
                height: 30
                radius: 4
                color: backMa.containsMouse ? "#2c2c2e" : "#202022"
                border.color: "#2e2e32"
                implicitWidth: backLbl.implicitWidth + 20
                Label {
                    id: backLbl
                    anchors.centerIn: parent
                    text: pm.t("app.back")
                    color: "#cacaca"
                    font.pixelSize: 12
                }
                MouseArea {
                    id: backMa
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.goBack()
                }
            }

            Item { Layout.fillWidth: true }

            Rectangle {
                height: 28; radius: 4
                color: expMa.containsMouse ? "#2c2c30" : "#202022"
                border.color: "#323236"
                implicitWidth: expLbl.implicitWidth + 18
                Label { id: expLbl; anchors.centerIn: parent; text: pm.t("app.export"); color: "#cacaca"; font.pixelSize: 11 }
                MouseArea {
                    id: expMa; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor
                    onClicked: if (app.key) pm.exportProfile(app.key)
                }
            }

            Rectangle {
                visible: app.source === "local"
                height: 28; radius: 4
                color: confirmDelete ? "#811818" : (delMa.containsMouse ? "#451717" : "#2b1616")
                border.color: "#e01d1d"
                implicitWidth: delLbl.implicitWidth + 18
                Label {
                    id: delLbl; anchors.centerIn: parent
                    text: confirmDelete ? (pm.isRu ? "Подтвердить удаление?" : "Confirm Delete?") : pm.t("app.delete")
                    color: "#ffffff"; font.pixelSize: 11; font.bold: confirmDelete
                }
                MouseArea {
                    id: delMa; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor
                    onClicked: {
                        if (!confirmDelete) { confirmDelete = true; return }
                        pm.deleteProfile(app.key)
                        root.goBack()
                    }
                }
            }
        }

        // App Header Card (1:1 Portmaster style)
        Rectangle {
            Layout.fillWidth: true
            height: 82
            radius: 6
            color: "#1b1b1d"
            border.color: "#28282c"

            RowLayout {
                anchors.fill: parent
                anchors.leftMargin: 16
                anchors.rightMargin: 16
                spacing: 14

                Rectangle {
                    width: 48; height: 48; radius: 10
                    property bool iconReady: detailIconImg.status === Image.Ready && detailIconImg.implicitWidth > 1
                    color: iconReady ? "transparent" : (app.avatarColor || "#2a2a2e")
                    border.color: (app.blockInternet || app.action === "block") ? "#e01d1d" : (iconReady ? "transparent" : "#3a3a40")
                    Image {
                        id: detailIconImg
                        anchors.fill: parent
                        fillMode: Image.PreserveAspectFit
                        smooth: true
                        source: (app.iconKey || "") !== "" ? ("image://appicon/" + app.iconKey + "?v=" + pm.iconVersion) : ""
                        visible: parent.iconReady
                    }
                    Label {
                        visible: !parent.iconReady
                        anchors.centerIn: parent
                        text: app.initials || "A"
                        color: "#ffffff"
                        font.pixelSize: 18; font.bold: true
                    }
                }

                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: 3
                    RowLayout {
                        spacing: 8
                        Label {
                            text: app.name || "-"
                            color: "#ffffff"; font.pixelSize: 18; font.bold: true
                            elide: Text.ElideRight; Layout.maximumWidth: 320
                        }
                        Rectangle {
                            height: 18; radius: 3; color: "#28282c"
                            implicitWidth: srcLbl.implicitWidth + 12
                            Label { id: srcLbl; anchors.centerIn: parent; text: app.source || "local"; color: "#9e9ea6"; font.pixelSize: 10 }
                        }
                    }
                    Label {
                        text: app.path || "-"
                        color: "#84848c"; font.pixelSize: 11; font.family: "monospace"
                        elide: Text.ElideMiddle; Layout.fillWidth: true
                    }
                }

                // Quick Block Internet Pill
                ColumnLayout {
                    spacing: 3
                    Label { text: pm.isRu ? "Интернет" : "Internet"; color: "#76767e"; font.pixelSize: 10 }
                    Rectangle {
                        height: 26; radius: 4
                        color: !!app.blockInternet ? "#7a1515" : "#136734"
                        border.color: !!app.blockInternet ? "#e01d1d" : "#1de966"
                        implicitWidth: qBlkAppLbl.implicitWidth + 18
                        Label {
                            id: qBlkAppLbl; anchors.centerIn: parent
                            text: !!app.blockInternet ? (pm.isRu ? "Заблокирован" : "Blocked") : (pm.isRu ? "Открыт" : "Allowed")
                            color: "#ffffff"; font.pixelSize: 11; font.bold: true
                        }
                        MouseArea {
                            anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                            onClicked: if (app.key) pm.setAppSetting(app.key, "filter/blockInternet", !app.blockInternet)
                        }
                    }
                }

                // 3-way Default Action Pills
                ColumnLayout {
                    spacing: 3
                    Label { text: pm.isRu ? "Доступ по умолчанию" : "Default Action"; color: "#76767e"; font.pixelSize: 10 }
                    Row {
                        spacing: 4
                        Repeater {
                            model: [
                                { id: "permit", label: pm.t("app.permit"), bg: "#136734", fg: "#1de966" },
                                { id: "ask",    label: pm.t("app.ask"),    bg: "#5c5210", fg: "#e9d81d" },
                                { id: "block",  label: pm.t("app.block"),  bg: "#7a1515", fg: "#e01d1d" }
                            ]
                            delegate: Rectangle {
                                height: 26; radius: 4
                                implicitWidth: actTxt.implicitWidth + 16
                                color: app.action === modelData.id ? modelData.bg : "#26262a"
                                border.color: app.action === modelData.id ? modelData.fg : "#34343a"
                                Label {
                                    id: actTxt; anchors.centerIn: parent
                                    text: modelData.label
                                    color: app.action === modelData.id ? "#ffffff" : "#9e9ea6"
                                    font.pixelSize: 11; font.bold: app.action === modelData.id
                                }
                                MouseArea {
                                    anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                                    onClicked: if (app.key) pm.setAppAction(app.key, modelData.id)
                                }
                            }
                        }
                    }
                }

                // 3-way Xray Routing Pills
                ColumnLayout {
                    spacing: 3
                    Label { text: "Xray VPN"; color: "#76767e"; font.pixelSize: 10 }
                    Row {
                        spacing: 4
                        Repeater {
                            model: [
                                { id: "default", label: pm.isRu ? "Авто" : "Auto",    bg: "#2d3038", fg: "#cacaca" },
                                { id: "vpn",     label: "Xray",                       bg: "#173b66", fg: "#4e97fa" },
                                { id: "direct",  label: pm.isRu ? "Прямо" : "Direct", bg: "#2e2e32", fg: "#cacaca" }
                            ]
                            delegate: Rectangle {
                                height: 26; radius: 4
                                implicitWidth: vpnTxt.implicitWidth + 14
                                color: app.vpnMode === modelData.id ? modelData.bg : "#26262a"
                                border.color: app.vpnMode === modelData.id ? modelData.fg : "#34343a"
                                Label {
                                    id: vpnTxt; anchors.centerIn: parent
                                    text: modelData.label
                                    color: app.vpnMode === modelData.id ? "#ffffff" : "#9e9ea6"
                                    font.pixelSize: 11; font.bold: app.vpnMode === modelData.id
                                }
                                MouseArea {
                                    anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                                    onClicked: if (app.key) pm.setAppVpnMode(app.key, modelData.id)
                                }
                            }
                        }
                    }
                }

                // 3-way Split Tunnel Pills
                ColumnLayout {
                    spacing: 3
                    Label { text: "Split Tunnel"; color: "#76767e"; font.pixelSize: 10 }
                    Row {
                        spacing: 4
                        Repeater {
                            model: [
                                { id: "default", label: pm.isRu ? "Авто" : "Auto", bg: "#2d3038", fg: "#cacaca" },
                                { id: "on",      label: "ON",                      bg: "#5c5210", fg: "#e9d81d" },
                                { id: "off",     label: "OFF",                     bg: "#2e2e32", fg: "#cacaca" }
                            ]
                            delegate: Rectangle {
                                height: 26; radius: 4
                                implicitWidth: stTxt.implicitWidth + 14
                                color: app.splitMode === modelData.id ? modelData.bg : "#26262a"
                                border.color: app.splitMode === modelData.id ? modelData.fg : "#34343a"
                                Label {
                                    id: stTxt; anchors.centerIn: parent
                                    text: modelData.label
                                    color: app.splitMode === modelData.id ? "#ffffff" : "#9e9ea6"
                                    font.pixelSize: 11; font.bold: app.splitMode === modelData.id
                                }
                                MouseArea {
                                    anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                                    onClicked: if (app.key) pm.setAppSplitMode(app.key, modelData.id)
                                }
                            }
                        }
                    }
                }
            }
        }

        // 4 Stat Boxes Row
        RowLayout {
            Layout.fillWidth: true
            spacing: 12
            Repeater {
                model: [
                    { title: pm.isRu ? "Активные" : "Active",    val: String(app.alive !== undefined ? app.alive : 0),    accent: "#1de966" },
                    { title: pm.isRu ? "Блокировано" : "Blocked", val: String(app.blocked !== undefined ? app.blocked : 0), accent: "#e01d1d" },
                    { title: pm.isRu ? "Получено" : "Received",  val: app.rxStr || "0 B", accent: "#4e97fa" },
                    { title: pm.isRu ? "Отправлено" : "Sent",    val: app.txStr || "0 B", accent: "#cacaca" }
                ]
                delegate: Rectangle {
                    Layout.fillWidth: true
                    height: 54
                    radius: 5
                    color: "#1b1b1d"
                    border.color: "#26262a"
                    Column {
                        anchors.centerIn: parent
                        spacing: 2
                        Label { anchors.horizontalCenter: parent.horizontalCenter; text: modelData.title; color: "#76767e"; font.pixelSize: 10 }
                        Label { anchors.horizontalCenter: parent.horizontalCenter; text: modelData.val; color: modelData.accent; font.pixelSize: 16; font.bold: true }
                    }
                }
            }
        }

        // Tab Switcher Row
        Row {
            spacing: 6
            Repeater {
                model: [
                    { idx: 0, label: pm.t("app.connections") + " (" + pm.selectedAppConnections.length + ")" },
                    { idx: 1, label: pm.t("app.rules") + " (" + (app.rules ? app.rules.length : 0) + ")" },
                    { idx: 2, label: pm.t("app.settings") },
                    { idx: 3, label: pm.t("app.details") }
                ]
                delegate: Rectangle {
                    height: 30; radius: 4
                    implicitWidth: tabLbl.implicitWidth + 24
                    color: root.activeTab === modelData.idx ? "#2e2e34" : "#1c1c1e"
                    border.color: root.activeTab === modelData.idx ? "#4e97fa" : "#2a2a2e"
                    Label {
                        id: tabLbl; anchors.centerIn: parent
                        text: modelData.label
                        color: root.activeTab === modelData.idx ? "#ffffff" : "#8e8e94"
                        font.pixelSize: 12; font.bold: root.activeTab === modelData.idx
                    }
                    MouseArea {
                        anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                        onClicked: root.activeTab = modelData.idx
                    }
                }
            }
        }

        // Tab Content Area
        StackLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            currentIndex: root.activeTab

            // TAB 0: APP CONNECTIONS + ACTIVITY GRAPH
            ColumnLayout {
                spacing: 10

                // Per-App Activity Graph Card (Connections or RX/TX Data)
                Rectangle {
                    Layout.fillWidth: true
                    height: 152
                    radius: 6
                    color: "#18181a"
                    border.color: "#26262a"

                    ColumnLayout {
                        anchors.fill: parent
                        anchors.margins: 10
                        spacing: 6

                        PmChartHeader {
                            title: pm.isRu
                                   ? (pm.chartDataMode ? "График трафика приложения (RX / TX)" : "График соединений приложения")
                                   : (pm.chartDataMode ? "App Traffic Timeline (RX / TX)" : "App Connections Graph")
                            showInternetToggle: true
                            internetOnly: pm.appChartInternetOnly
                            onToggleInternet: pm.setAppChartInternetOnly(!pm.appChartInternetOnly)
                            okCount: pm.appChartInternetOnly ? pm.selectedAppInetAllowed : pm.selectedAppAllAllowed
                            blockedCount: pm.appChartInternetOnly ? pm.selectedAppInetBlocked : pm.selectedAppAllBlocked
                            rxStr: pm.appChartInternetOnly ? (root.app.rxInetStr || "0 B") : (root.app.rxStr || "0 B")
                            txStr: pm.appChartInternetOnly ? (root.app.txInetStr || "0 B") : (root.app.txStr || "0 B")
                        }

                        ConnChartCanvas {
                            Layout.fillWidth: true
                            Layout.fillHeight: true
                            showAxes: true
                            dataMode: pm.chartDataMode
                            bins: pm.selectedAppChartBins
                        }
                    }
                }

                Rectangle {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    color: "#18181a"
                    radius: 6
                    border.color: "#26262a"

                    ListView {
                    anchors.fill: parent
                    anchors.margins: 8
                    clip: true
                    spacing: 4
                    model: pm.selectedAppConnections
                    ScrollBar.vertical: PmScrollBar {}
                    PmWheelHelper {}

                    delegate: Rectangle {
                        width: ListView.view.width
                        height: 40
                        radius: 4
                        color: index % 2 === 0 ? "#1d1d20" : "#19191b"

                        RowLayout {
                            anchors.fill: parent
                            anchors.leftMargin: 12; anchors.rightMargin: 12
                            spacing: 10

                            Rectangle {
                                width: 8; height: 8; radius: 4
                                color: (modelData.verdict === "block" || modelData.verdict === "drop" || modelData.verdict === "failed") ? "#e01d1d"
                                     : (modelData.verdict === "ask" ? "#e9d81d" : (modelData.tunneled ? "#4e97fa" : "#1de966"))
                            }
                            Label {
                                text: modelData.domain || modelData.ip || "-"
                                color: "#ffffff"; font.pixelSize: 12; font.bold: true
                                elide: Text.ElideRight; Layout.preferredWidth: 260
                            }
                            Label {
                                text: (modelData.ip || "") + ":" + (modelData.port || "")
                                color: "#888890"; font.pixelSize: 11; font.family: "monospace"
                                Layout.preferredWidth: 160
                            }
                            Label { text: modelData.proto || ""; color: "#76767e"; font.pixelSize: 11; Layout.preferredWidth: 46 }
                            Rectangle {
                                visible: (modelData.country || "") !== ""
                                height: 18; radius: 3; color: "#28282c"
                                implicitWidth: ccTxt.implicitWidth + 10
                                Label { id: ccTxt; anchors.centerIn: parent; text: modelData.country || ""; color: "#cacaca"; font.pixelSize: 10 }
                            }
                            Label { text: modelData.reason || ""; color: "#84848c"; font.pixelSize: 11; elide: Text.ElideRight; Layout.fillWidth: true }
                            Label { text: modelData.bytes || ""; color: "#cacaca"; font.pixelSize: 11 }

                            Rectangle {
                                height: 22; radius: 3; color: "#136734"
                                implicitWidth: pTxt.implicitWidth + 12
                                Label { id: pTxt; anchors.centerIn: parent; text: "+ " + pm.t("app.permit"); color: "#ffffff"; font.pixelSize: 10 }
                                MouseArea {
                                    anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                                    onClicked: pm.allowOrBlockConnection(modelData.id, true)
                                }
                            }
                            Rectangle {
                                height: 22; radius: 3; color: "#7a1515"
                                implicitWidth: bTxt.implicitWidth + 12
                                Label { id: bTxt; anchors.centerIn: parent; text: "- " + pm.t("app.block"); color: "#ffffff"; font.pixelSize: 10 }
                                MouseArea {
                                    anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                                    onClicked: pm.allowOrBlockConnection(modelData.id, false)
                                }
                            }
                        }
                    }

                    Label {
                        anchors.centerIn: parent
                        visible: pm.selectedAppConnections.length === 0
                        text: pm.isRu ? "У этого приложения пока нет записанных соединений." : "No recorded connections for this app yet."
                        color: "#76767e"; font.pixelSize: 13
                    }
                }
            }
            }

            // TAB 1: RULES & ROUTING
            ColumnLayout {
                spacing: 12

                Rectangle {
                    Layout.fillWidth: true
                    height: 86
                    radius: 6
                    color: "#1b1b1d"
                    border.color: "#28282c"

                    ColumnLayout {
                        anchors.fill: parent
                        anchors.margins: 12
                        spacing: 8

                        RowLayout {
                            Layout.fillWidth: true
                            spacing: 8

                            Row {
                                spacing: 4
                                Repeater {
                                    model: [
                                        { dir: "+", label: pm.isRu ? "+ Разрешить" : "+ Allow", bg: "#136734" },
                                        { dir: "-", label: pm.isRu ? "- Блокировать" : "- Block", bg: "#7a1515" }
                                    ]
                                    delegate: Rectangle {
                                        height: 30; radius: 4
                                        implicitWidth: dirLbl.implicitWidth + 18
                                        color: root.ruleDir === modelData.dir ? modelData.bg : "#26262a"
                                        border.color: root.ruleDir === modelData.dir ? "#ffffff" : "#34343a"
                                        Label { id: dirLbl; anchors.centerIn: parent; text: modelData.label; color: "#ffffff"; font.pixelSize: 11; font.bold: true }
                                        MouseArea {
                                            anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                                            onClicked: root.ruleDir = modelData.dir
                                        }
                                    }
                                }
                            }

                            TextField {
                                id: rulePatternInput
                                Layout.fillWidth: true
                                placeholderText: pm.isRu ? "Домен, IP, CIDR или GeoSite (напр. .google.com, 192.168.1.0/24, geosite:youtube)" : "Domain, IP, CIDR or GeoSite (e.g. .google.com, 192.168.1.0/24)"
                                placeholderTextColor: "#66666e"
                                color: "#ffffff"; font.pixelSize: 12; font.family: "monospace"
                                background: Rectangle { color: "#121213"; radius: 4; border.color: rulePatternInput.activeFocus ? "#4e97fa" : "#2e2e32" }
                                function submitRule() {
                                    var txt = rulePatternInput.text.trim()
                                    if (txt === "" || !app.key) return
                                    var rule = (txt.startsWith("+") || txt.startsWith("-")) ? txt : (root.ruleDir + " " + txt)
                                    pm.addAppRule(app.key, rule)
                                    rulePatternInput.text = ""
                                }
                                onAccepted: submitRule()
                            }

                            Rectangle {
                                height: 30; radius: 4; color: "#4e97fa"
                                implicitWidth: addRuleLbl.implicitWidth + 24
                                Label { id: addRuleLbl; anchors.centerIn: parent; text: pm.isRu ? "Добавить правило" : "Add Rule"; color: "#ffffff"; font.pixelSize: 12; font.bold: true }
                                MouseArea {
                                    id: addRuleBtnMa
                                    anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                                    onClicked: rulePatternInput.submitRule()
                                }
                            }

                            Rectangle {
                                height: 30; radius: 4; color: "#1c283a"; border.color: "#4e97fa"
                                implicitWidth: rbAppLbl.implicitWidth + 16
                                Label { id: rbAppLbl; anchors.centerIn: parent; text: pm.isRu ? "⚙ Конструктор" : "⚙ Builder"; color: "#89b7ff"; font.pixelSize: 11; font.bold: true }
                                MouseArea {
                                    anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                                    onClicked: if (app.key) root.openRuleBuilder(app.key, "endpoints", "firewall")
                                }
                            }
                        }

                        // Quick Preset Rule Chips
                        RowLayout {
                            spacing: 6
                            Label { text: pm.isRu ? "Шаблоны:" : "Quick presets:"; color: "#76767e"; font.pixelSize: 11 }
                            Repeater {
                                model: [
                                    { label: "+ HTTPS (443)", val: "+ * 443" },
                                    { label: "+ LAN",         val: "+ LAN" },
                                    { label: "- Ads / Trackers", val: "- geosite:category-ads-all" },
                                    { label: "+ KDE Connect LAN", val: "+ LAN 1714-1764" }
                                ]
                                delegate: Rectangle {
                                    height: 22; radius: 3; color: "#252529"; border.color: "#34343c"
                                    implicitWidth: preLbl.implicitWidth + 14
                                    Label { id: preLbl; anchors.centerIn: parent; text: modelData.label; color: "#cacaca"; font.pixelSize: 10 }
                                    MouseArea {
                                        anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                                        onClicked: if (app.key) pm.addAppRule(app.key, modelData.val)
                                    }
                                }
                            }
                        }
                    }
                }

                Rectangle {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    radius: 6
                    color: "#18181a"
                    border.color: "#26262a"

                    ListView {
                        anchors.fill: parent
                        anchors.margins: 8
                        clip: true
                        spacing: 4
                        model: app.rules || []
                        ScrollBar.vertical: PmScrollBar {}
                        PmWheelHelper {}

                        delegate: Rectangle {
                            width: ListView.view.width
                            height: 38
                            radius: 4
                            color: "#1e1e21"

                            RowLayout {
                                anchors.fill: parent
                                anchors.leftMargin: 12; anchors.rightMargin: 12
                                spacing: 10

                                Rectangle {
                                    width: 24; height: 22; radius: 3
                                    color: modelData.allow ? "#136734" : "#7a1515"
                                    Label {
                                        anchors.centerIn: parent
                                        text: modelData.allow ? "+" : "-"
                                        color: "#ffffff"; font.bold: true; font.pixelSize: 13
                                    }
                                }
                                Label {
                                    text: modelData.pattern || modelData.raw || ""
                                    color: "#ffffff"; font.pixelSize: 12; font.family: "monospace"
                                    Layout.fillWidth: true
                                }
                                Rectangle {
                                    height: 24; radius: 3; color: "#262834"; border.color: "#4e97fa"
                                    implicitWidth: promLbl.implicitWidth + 14
                                    Label { id: promLbl; anchors.centerIn: parent; text: pm.t("app.promote"); color: "#4e97fa"; font.pixelSize: 10; font.bold: true }
                                    MouseArea {
                                        anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                                        onClicked: pm.promoteRuleToGlobal("filter/endpoints", modelData.raw)
                                    }
                                }
                                Rectangle {
                                    width: 26; height: 24; radius: 3; color: "#341818"
                                    Label { anchors.centerIn: parent; text: "✕"; color: "#e01d1d"; font.pixelSize: 11 }
                                    MouseArea {
                                        anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                                        onClicked: if (app.key) pm.removeAppRule(app.key, modelData.raw)
                                    }
                                }
                            }
                        }

                        Label {
                            anchors.centerIn: parent
                            visible: !app.rules || app.rules.length === 0
                            text: pm.isRu ? "Персональных правил фильтрации пока нет." : "No per-app endpoint rules configured."
                            color: "#76767e"; font.pixelSize: 13
                        }
                    }
                }
            }

            // TAB 2: PER-APP SETTINGS (Full parity with Global Settings)
            ColumnLayout {
                spacing: 10

                // Quick Per-App Network Scope Card + Global DNS Link
                Rectangle {
                    Layout.fillWidth: true
                    implicitHeight: appQuickCol.implicitHeight + 20
                    radius: 6
                    color: !!app.blockInternet ? "#261618" : "#1b1b1d"
                    border.color: !!app.blockInternet ? "#e01d1d" : "#26262a"

                    ColumnLayout {
                        id: appQuickCol
                        anchors.fill: parent
                        anchors.margins: 10
                        spacing: 8

                        Flow {
                            Layout.fillWidth: true
                            spacing: 14

                            RowLayout {
                                spacing: 7
                                PmSwitch {
                                    checked: !!app.blockInternet
                                    onToggled: if (app.key) pm.setAppSetting(app.key, "filter/blockInternet", checked)
                                }
                                Label {
                                    text: pm.isRu ? "Блокировать Интернет" : "Block Internet"
                                    color: !!app.blockInternet ? "#ff8888" : "#ffffff"
                                    font.pixelSize: 12; font.bold: true
                                }
                                Rectangle {
                                    visible: !!app.overriddenInternet
                                    width: 18; height: 18; radius: 3; color: "#1c2838"; border.color: "#4e97fa"
                                    Label { anchors.centerIn: parent; text: "↺"; color: "#7bb5ff"; font.pixelSize: 11; font.bold: true }
                                    MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: if (app.key) pm.resetAppSetting(app.key, "filter/blockInternet") }
                                }
                            }

                            RowLayout {
                                spacing: 7
                                PmSwitch {
                                    checked: !!app.blockLan
                                    onToggled: if (app.key) pm.setAppSetting(app.key, "filter/blockLAN", checked)
                                }
                                Label { text: pm.isRu ? "Блокировать LAN" : "Block LAN"; color: "#dcdce2"; font.pixelSize: 12 }
                                Rectangle {
                                    visible: !!app.overriddenLan
                                    width: 18; height: 18; radius: 3; color: "#1c2838"; border.color: "#4e97fa"
                                    Label { anchors.centerIn: parent; text: "↺"; color: "#7bb5ff"; font.pixelSize: 11; font.bold: true }
                                    MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: if (app.key) pm.resetAppSetting(app.key, "filter/blockLAN") }
                                }
                            }

                            RowLayout {
                                spacing: 7
                                PmSwitch {
                                    checked: !!app.blockP2p
                                    onToggled: if (app.key) pm.setAppSetting(app.key, "filter/blockP2P", checked)
                                }
                                Label { text: pm.isRu ? "Блокировать P2P" : "Block P2P"; color: "#dcdce2"; font.pixelSize: 12 }
                                Rectangle {
                                    visible: !!app.overriddenP2p
                                    width: 18; height: 18; radius: 3; color: "#1c2838"; border.color: "#4e97fa"
                                    Label { anchors.centerIn: parent; text: "↺"; color: "#7bb5ff"; font.pixelSize: 11; font.bold: true }
                                    MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: if (app.key) pm.resetAppSetting(app.key, "filter/blockP2P") }
                                }
                            }

                            RowLayout {
                                spacing: 7
                                PmSwitch {
                                    checked: !!app.blockInbound
                                    onToggled: if (app.key) pm.setAppSetting(app.key, "filter/blockInbound", checked)
                                }
                                Label { text: pm.isRu ? "Блокировать входящие" : "Block Inbound"; color: "#dcdce2"; font.pixelSize: 12 }
                                Rectangle {
                                    visible: !!app.overriddenInbound
                                    width: 18; height: 18; radius: 3; color: "#1c2838"; border.color: "#4e97fa"
                                    Label { anchors.centerIn: parent; text: "↺"; color: "#7bb5ff"; font.pixelSize: 11; font.bold: true }
                                    MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: if (app.key) pm.resetAppSetting(app.key, "filter/blockInbound") }
                                }
                            }
                        }

                        RowLayout {
                            Layout.fillWidth: true
                            spacing: 8
                            Label {
                                text: (pm.isRu ? "Системный DNS (глобально для всех приложений): " : "Global System DNS: ") + (pm.dnsSummary || "-")
                                color: "#8e8e98"; font.pixelSize: 11
                                elide: Text.ElideRight; Layout.fillWidth: true
                            }
                            Rectangle {
                                height: 22; radius: 4; color: dnsLnkMa.containsMouse ? "#28354d" : "#212632"; border.color: "#3b5680"
                                implicitWidth: dnsLnkLbl.implicitWidth + 14
                                Label { id: dnsLnkLbl; anchors.centerIn: parent; text: pm.isRu ? "Настроить сеть и DNS →" : "Open Network & DNS →"; color: "#89b7ff"; font.pixelSize: 10; font.bold: true }
                                MouseArea { id: dnsLnkMa; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: root.openNetworkSettings() }
                            }
                        }
                    }
                }

                // Search, Category Pills & Expertise Toolbar
                RowLayout {
                    Layout.fillWidth: true
                    spacing: 8

                    TextField {
                        Layout.fillWidth: true
                        height: 30
                        placeholderText: pm.isRu ? "Поиск настроек приложения..." : "Search per-app settings..."
                        placeholderTextColor: "#66666e"
                        color: "#ffffff"; font.pixelSize: 12
                        onTextChanged: root.appSetSearch = text
                        background: Rectangle { color: "#1b1b1d"; radius: 4; border.color: parent.activeFocus ? "#4e97fa" : "#2c2c30" }
                    }

                    Repeater {
                        model: [
                            { id: "all",     label: pm.isRu ? "Все" : "All" },
                            { id: "custom",  label: (pm.isRu ? "Свои (" : "Custom (") + (app.customCount || 0) + ")" },
                            { id: "filter",  label: pm.isRu ? "Сеть" : "Filter" },
                            { id: "spn",     label: "Xray / Split" },
                            { id: "history", label: pm.isRu ? "История" : "History" }
                        ]
                        delegate: Rectangle {
                            height: 28; radius: 4
                            color: root.appSetFilter === modelData.id ? "#1e2c42" : "#1c1c1e"
                            border.color: root.appSetFilter === modelData.id ? "#4e97fa" : "#2a2a2e"
                            implicitWidth: afLbl.implicitWidth + 14
                            Label { id: afLbl; anchors.centerIn: parent; text: modelData.label; color: root.appSetFilter === modelData.id ? "#ffffff" : "#9e9ea6"; font.pixelSize: 10; font.bold: root.appSetFilter === modelData.id }
                            MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: root.appSetFilter = modelData.id }
                        }
                    }

                    Repeater {
                        model: [
                            { lvl: 0, label: pm.isRu ? "Простой" : "Simple" },
                            { lvl: 1, label: pm.isRu ? "Больше" : "More" },
                            { lvl: 2, label: pm.isRu ? "Все" : "All" }
                        ]
                        delegate: Rectangle {
                            height: 28; radius: 4
                            color: root.appMaxExpertise === modelData.lvl ? "#2c2f3a" : "#1c1c1e"
                            border.color: root.appMaxExpertise === modelData.lvl ? "#4e97fa" : "#2a2a2e"
                            implicitWidth: aeLbl.implicitWidth + 12
                            Label { id: aeLbl; anchors.centerIn: parent; text: modelData.label; color: "#ffffff"; font.pixelSize: 10 }
                            MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: root.appMaxExpertise = modelData.lvl }
                        }
                    }
                }

                // Full Per-App Setting Cards (mirrored 1:1 with Global Settings)
                ListView {
                    id: appSettingsList
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    clip: true
                    spacing: 8
                    model: root.filteredAppSettings()
                    ScrollBar.vertical: PmScrollBar {}
                    PmWheelHelper {}

                    delegate: SettingCard {
                        width: ListView.view.width - 10
                        st: modelData
                        isPerApp: true
                        appKey: app.key || ""
                        onRequestRuleBuilder: function(key, mode) {
                            root.openRuleBuilder(app.key || "", key, mode)
                        }
                    }

                    Label {
                        anchors.centerIn: parent
                        visible: appSettingsList.count === 0
                        text: root.appSetFilter === "custom"
                              ? (pm.isRu ? "У этого приложения пока все настройки наследуются глобально." : "All settings currently inherit global defaults.")
                              : (pm.isRu ? "Нет настроек под выбранный фильтр." : "No matching per-app settings.")
                        color: "#76767e"; font.pixelSize: 12
                    }
                }
            }

            // TAB 3: DETAILS & PATH TESTER (#2166)
            Rectangle {
                radius: 6
                color: "#1b1b1d"
                border.color: "#26262a"

                ColumnLayout {
                    anchors.left: parent.left; anchors.right: parent.right; anchors.top: parent.top
                    anchors.margins: 20
                    spacing: 16

                    GridLayout {
                        columns: 2
                        columnSpacing: 20
                        rowSpacing: 8
                        Label { text: "Profile Key:"; color: "#76767e"; font.pixelSize: 12 }
                        Label { text: app.key || "-"; color: "#ffffff"; font.pixelSize: 12; font.family: "monospace" }
                        Label { text: "Binary / Linked Path:"; color: "#76767e"; font.pixelSize: 12 }
                        Label { text: app.path || "-"; color: "#cacaca"; font.pixelSize: 12; font.family: "monospace" }
                        Label { text: "Fingerprints:"; color: "#76767e"; font.pixelSize: 12 }
                        Label { text: String(app.fpCount !== undefined ? app.fpCount : 0); color: "#cacaca"; font.pixelSize: 12 }
                    }

                    Rectangle { Layout.fillWidth: true; height: 1; color: "#28282c" }

                    ColumnLayout {
                        Layout.fillWidth: true
                        spacing: 8
                        Label { text: pm.isRu ? "Тестер пути профиля (#2166)" : "Profile Path Matcher Tester (#2166)"; color: "#ffffff"; font.pixelSize: 14; font.bold: true }
                        Label { text: pm.isRu ? "Проверьте, попадёт ли конкретный исполняемый файл под этот профиль." : "Check whether a binary path matches this profile's LinkedPath or Fingerprints."; color: "#888890"; font.pixelSize: 12 }
                        RowLayout {
                            Layout.fillWidth: true
                            spacing: 10
                            TextField {
                                Layout.fillWidth: true
                                text: root.testPathInput
                                onTextChanged: {
                                    root.testPathInput = text
                                    root.testPathResult = app.key ? pm.testAppPath(app.key, text) : ""
                                }
                                placeholderText: "/usr/bin/curl"
                                placeholderTextColor: "#66666e"
                                color: "#ffffff"; font.pixelSize: 12; font.family: "monospace"
                                background: Rectangle { color: "#121213"; radius: 4; border.color: "#2e2e32" }
                            }
                            Rectangle {
                                visible: root.testPathResult !== ""
                                height: 30; radius: 4
                                color: root.testPathResult.startsWith("MATCH") ? "#136734" : "#402020"
                                implicitWidth: resLbl.implicitWidth + 20
                                Label { id: resLbl; anchors.centerIn: parent; text: root.testPathResult; color: "#ffffff"; font.pixelSize: 11; font.bold: true }
                            }
                        }
                    }
                }
            }
        }
    }
}
