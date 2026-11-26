import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Item {
    id: root
    signal goBack()

    property int activeTab: 0 // 0=Connections, 1=Rules, 2=Settings, 3=Details
    property var app: pm.selectedApp
    property string ruleDir: "+"
    property string testPathInput: ""
    property string testPathResult: ""
    property string customDnsDraft: ""
    property bool confirmDelete: false

    onAppChanged: {
        customDnsDraft = app.customDns || ""
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
                    width: 48; height: 48; radius: 8
                    color: app.action === "block" ? "#3d1414" : "#2a2a2e"
                    border.color: app.action === "block" ? "#e01d1d" : "#3a3a40"
                    Label {
                        anchors.centerIn: parent
                        text: app.initials || "A"
                        color: app.action === "block" ? "#e01d1d" : "#ffffff"
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

            // TAB 0: APP CONNECTIONS
            Rectangle {
                color: "#18181a"
                radius: 6
                border.color: "#26262a"

                ListView {
                    anchors.fill: parent
                    anchors.margins: 8
                    clip: true
                    spacing: 4
                    model: pm.selectedAppConnections

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
                                color: (modelData.verdict === "block" || modelData.verdict === "drop") ? "#e01d1d"
                                     : (modelData.verdict === "ask" ? "#e9d81d" : "#1de966")
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
                                onAccepted: addRuleBtnMa.clicked(null)
                            }

                            Rectangle {
                                height: 30; radius: 4; color: "#4e97fa"
                                implicitWidth: addRuleLbl.implicitWidth + 24
                                Label { id: addRuleLbl; anchors.centerIn: parent; text: pm.isRu ? "Добавить правило" : "Add Rule"; color: "#ffffff"; font.pixelSize: 12; font.bold: true }
                                MouseArea {
                                    id: addRuleBtnMa
                                    anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                                    onClicked: {
                                        var txt = rulePatternInput.text.trim()
                                        if (txt === "" || !app.key) return
                                        var rule = (txt.startsWith("+") || txt.startsWith("-")) ? txt : (root.ruleDir + " " + txt)
                                        pm.addAppRule(app.key, rule)
                                        rulePatternInput.text = ""
                                    }
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

            // TAB 2: PER-APP SETTINGS
            Rectangle {
                radius: 6
                color: "#1b1b1d"
                border.color: "#26262a"

                ColumnLayout {
                    anchors.left: parent.left; anchors.right: parent.right; anchors.top: parent.top
                    anchors.margins: 20
                    spacing: 18

                    RowLayout {
                        Layout.fillWidth: true
                        ColumnLayout {
                            Layout.fillWidth: true
                            Label { text: pm.isRu ? "Блокировать локальную сеть (LAN)" : "Block Local Network (LAN)"; color: "#ffffff"; font.pixelSize: 14; font.bold: true }
                            Label { text: pm.isRu ? "Запрещает приложению обращаться к 192.168.x.x / 10.x.x.x и локальным устройствам." : "Prevent this app from reaching LAN / private subnets."; color: "#888890"; font.pixelSize: 12 }
                        }
                        Switch {
                            checked: !!app.blockLan
                            onToggled: if (app.key) pm.setAppBlockLan(app.key, checked)
                        }
                    }

                    Rectangle { Layout.fillWidth: true; height: 1; color: "#28282c" }

                    ColumnLayout {
                        Layout.fillWidth: true
                        spacing: 8
                        Label { text: pm.isRu ? "Персональные DNS-серверы (через запятую)" : "Custom DNS Servers (comma-separated)"; color: "#ffffff"; font.pixelSize: 14; font.bold: true }
                        RowLayout {
                            Layout.fillWidth: true
                            spacing: 10
                            TextField {
                                Layout.fillWidth: true
                                text: root.customDnsDraft
                                onTextChanged: root.customDnsDraft = text
                                placeholderText: "https://cloudflare-dns.com/dns-query, 9.9.9.9"
                                placeholderTextColor: "#66666e"
                                color: "#ffffff"; font.pixelSize: 12; font.family: "monospace"
                                background: Rectangle { color: "#121213"; radius: 4; border.color: "#2e2e32" }
                            }
                            Rectangle {
                                height: 32; radius: 4; color: "#4e97fa"
                                implicitWidth: saveDnsLbl.implicitWidth + 24
                                Label { id: saveDnsLbl; anchors.centerIn: parent; text: pm.isRu ? "Сохранить DNS" : "Save DNS"; color: "#ffffff"; font.pixelSize: 12; font.bold: true }
                                MouseArea {
                                    anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                                    onClicked: if (app.key) pm.setAppCustomDns(app.key, root.customDnsDraft)
                                }
                            }
                        }
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
