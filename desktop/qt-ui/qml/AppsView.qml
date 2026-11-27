import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Item {
    id: root
    signal openApp(string key)
    signal openNetworkSettings()

    property string searchQuery: ""
    property string routingFilter: "all" // "all" | "active" | "offline" | "vpn" | "blocked"
    property bool showCreateDrawer: false
    property bool showImportDrawer: false
    property var cachedFilteredMain: []
    property var cachedFilteredActive: []

    function matchesFilter(app) {
        if (searchQuery !== "") {
            var q = searchQuery.toLowerCase()
            var n = (app.name || "").toLowerCase()
            var p = (app.path || "").toLowerCase()
            if (n.indexOf(q) < 0 && p.indexOf(q) < 0) return false
        }
        if (routingFilter === "active") return pm.onlyNowApps ? (app.alive > 0) : (app.alive > 0 || (app.sessionConns || 0) > 0)
        if (routingFilter === "offline") return app.alive <= 0
        if (routingFilter === "vpn") return app.effectiveVpn
        if (routingFilter === "blocked") return app.action === "block"
        return true
    }

    function recompute() {
        var act = []
        var main = []
        var srcActive = pm.activeApps || []
        var srcMain = (routingFilter === "all") ? (pm.offlineApps || []) : (pm.allApps || [])

        for (var i = 0; i < srcActive.length; i++) {
            if (matchesFilter(srcActive[i])) act.push(srcActive[i])
        }
        for (var j = 0; j < srcMain.length; j++) {
            if (matchesFilter(srcMain[j])) main.push(srcMain[j])
        }
        cachedFilteredActive = act
        cachedFilteredMain = main
    }

    onSearchQueryChanged: recompute()
    onRoutingFilterChanged: recompute()
    Component.onCompleted: recompute()
    Connections {
        target: pm
        function onAppsChanged() { root.recompute() }
        function onLangChanged() { root.recompute() }
    }

    component AppCard: Rectangle {
        property var app: ({})
        width: 340
        height: 62
        radius: 5
        color: cardMa.containsMouse ? "#27272c" : "#1d1d20"
        border.color: cardMa.containsMouse ? "#3b3b42" : "#27272c"

        // Top-right blue dot when customized (1:1 Angular card-header)
        Rectangle {
            visible: !!app.hasCustom
            width: 7; height: 7; radius: 3.5
            color: "#4e97fa"
            anchors.top: parent.top; anchors.right: parent.right; anchors.margins: 6
        }

        RowLayout {
            anchors.fill: parent
            anchors.leftMargin: 12
            anchors.rightMargin: 12
            spacing: 10

            // App Icon (Real OS/.desktop/API icon or colorful Portmaster monogram fallback)
            Rectangle {
                width: 34; height: 34; radius: 7
                property bool iconReady: cardIconImg.status === Image.Ready && cardIconImg.implicitWidth > 1
                color: iconReady ? "transparent" : (app.avatarColor || "#2a2a30")
                border.color: (app.blockInternet || app.action === "block") ? "#e01d1d" : (iconReady ? "transparent" : "#3a3a42")
                border.width: (app.blockInternet || app.action === "block") ? 1 : 0
                Image {
                    id: cardIconImg
                    anchors.fill: parent
                    fillMode: Image.PreserveAspectFit
                    smooth: true
                    source: "image://appicon/" + app.iconKey + "?v=" + pm.iconVersion
                    visible: parent.iconReady
                }
                Label {
                    visible: !parent.iconReady
                    anchors.centerIn: parent
                    text: app.initials || "A"
                    color: "#ffffff"
                    font.pixelSize: 12; font.bold: true
                }
            }

            // App Name + Readable Path + Visual Status Badges
            ColumnLayout {
                Layout.fillWidth: true
                spacing: 2
                RowLayout {
                    Layout.fillWidth: true
                    spacing: 5
                    Label {
                        text: app.name || ""
                        color: cardMa.containsMouse ? "#ffffff" : "#f2f2f7"
                        font.pixelSize: 13; font.bold: true
                        elide: Text.ElideRight
                        Layout.fillWidth: true
                    }
                    Rectangle {
                        visible: !!app.effectiveSplit
                        height: 15; radius: 2; color: "#3d3a14"; border.color: "#e9d81d"
                        implicitWidth: stLbl.implicitWidth + 8
                        Label { id: stLbl; anchors.centerIn: parent; text: "ST"; color: "#e9d81d"; font.pixelSize: 8; font.bold: true }
                    }
                }
                Label {
                    text: app.alive > 0
                          ? (app.alive + (pm.isRu ? " акт. • ↓" : " active • ↓") + app.rxStr + " ↑" + app.txStr)
                          : (app.path || app.source || "-")
                    color: app.alive > 0 ? "#1de966" : "#a0a2ae"
                    font.pixelSize: 10
                    elide: Text.ElideMiddle
                    Layout.fillWidth: true
                }
            }

            // 1-Click Per-App Block/Allow Internet Button (without opening app settings)
            Rectangle {
                height: 24; radius: 4
                color: (app.blockInternet || app.action === "block") ? "#7a1515" : "#154428"
                border.color: (app.blockInternet || app.action === "block") ? "#e01d1d" : "#226b3f"
                implicitWidth: qNetLbl.implicitWidth + 12
                Label {
                    id: qNetLbl; anchors.centerIn: parent
                    text: (app.blockInternet || app.action === "block") ? (pm.isRu ? "Блок" : "Block") : (pm.isRu ? "Инет" : "Net")
                    color: (app.blockInternet || app.action === "block") ? "#ffd0d0" : "#8cf5b0"
                    font.pixelSize: 9; font.bold: true
                }
                MouseArea {
                    anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                    onClicked: {
                        if (app.action === "block" && !app.blockInternet) {
                            pm.setAppAction(app.key, "permit")
                        } else {
                            pm.setAppSetting(app.key, "filter/blockInternet", !app.blockInternet)
                        }
                    }
                }
            }

            // Compact 3-segment Auto | Xray | Direct pill
            Rectangle {
                height: 24; radius: 4; color: "#151518"; border.color: "#33333a"
                implicitWidth: segRow.implicitWidth + 4
                Row {
                    id: segRow
                    anchors.centerIn: parent
                    spacing: 1

                    Rectangle {
                        height: 20; width: 38; radius: 3
                        color: app.vpnMode === "default" ? (app.effectiveVpn ? "#1f4e8c" : "#3a3a42") : "transparent"
                        Label {
                            anchors.centerIn: parent; text: pm.isRu ? "Авто" : "Auto"
                            color: app.vpnMode === "default" ? "#ffffff" : "#8c8c96"; font.pixelSize: 9; font.bold: app.vpnMode === "default"
                        }
                        MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: pm.setAppVpnMode(app.key, "default") }
                    }
                    Rectangle {
                        height: 20; width: 34; radius: 3
                        color: app.vpnMode === "vpn" ? "#4e97fa" : "transparent"
                        Label {
                            anchors.centerIn: parent; text: "Xray"
                            color: app.vpnMode === "vpn" ? "#ffffff" : "#8c8c96"; font.pixelSize: 9; font.bold: app.vpnMode === "vpn"
                        }
                        MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: pm.setAppVpnMode(app.key, "vpn") }
                    }
                    Rectangle {
                        height: 20; width: 42; radius: 3
                        color: app.vpnMode === "direct" ? "#166838" : "transparent"
                        Label {
                            anchors.centerIn: parent; text: pm.isRu ? "Прямо" : "Direct"
                            color: app.vpnMode === "direct" ? "#ffffff" : "#8c8c96"; font.pixelSize: 9; font.bold: app.vpnMode === "direct"
                        }
                        MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: pm.setAppVpnMode(app.key, "direct") }
                    }
                }
            }
        }

        MouseArea {
            id: cardMa
            z: -1
            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: root.openApp(app.key)
        }
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 24
        spacing: 12

        // Fixed Top Bar: Title + Search + Create/Import + Global Routing Strip
        RowLayout {
            Layout.fillWidth: true
            spacing: 12

            Label {
                text: pm.t("apps.title")
                color: "#ffffff"
                font.pixelSize: 20
                font.bold: true
            }

            Rectangle {
                height: 22; radius: 11; color: "#222328"
                implicitWidth: totalLbl.implicitWidth + 16
                Label {
                    id: totalLbl; anchors.centerIn: parent
                    text: (pm.allApps || []).length
                    color: "#9e9ea6"; font.pixelSize: 11
                }
            }

            Item { Layout.fillWidth: true }

            TextField {
                Layout.preferredWidth: 260
                height: 32
                placeholderText: pm.t("apps.search")
                placeholderTextColor: "#66666e"
                color: "#ffffff"
                font.pixelSize: 12
                onTextChanged: root.searchQuery = text
                background: Rectangle { color: "#1b1b1d"; border.color: parent.activeFocus ? "#4e97fa" : "#2c2c30"; radius: 4 }
            }

            Rectangle {
                height: 32; radius: 4; color: "#1e1e22"; border.color: "#323238"
                implicitWidth: npLbl.implicitWidth + 18
                Label { id: npLbl; anchors.centerIn: parent; text: pm.t("apps.newProfile"); color: "#dcdce2"; font.pixelSize: 11 }
                MouseArea {
                    anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                    onClicked: { root.showCreateDrawer = !root.showCreateDrawer; root.showImportDrawer = false }
                }
            }

            Rectangle {
                height: 32; radius: 4; color: "#1a2230"; border.color: "#354b6e"
                implicitWidth: impLbl.implicitWidth + 18
                Label { id: impLbl; anchors.centerIn: parent; text: pm.t("apps.import"); color: "#89b7ff"; font.pixelSize: 11; font.bold: true }
                MouseArea {
                    anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                    onClicked: { root.showImportDrawer = !root.showImportDrawer; root.showCreateDrawer = false }
                }
            }
        }

        // Global Routing Strip matching overview.html line 88 1:1
        Rectangle {
            Layout.fillWidth: true
            implicitHeight: gRow.implicitHeight + 14
            radius: 5
            color: "#1b1b1d"
            border.color: "#28282c"

            Flow {
                id: gRow
                anchors.fill: parent
                anchors.margins: 7
                spacing: 8

                Label { text: pm.t("apps.globalRouting"); color: "#ababab"; font.pixelSize: 11; height: 26; verticalAlignment: Text.AlignVCenter }

                Rectangle {
                    height: 26; radius: 4; color: "#141416"; border.color: "#33333a"
                    implicitWidth: gSeg.implicitWidth + 4
                    Row {
                        id: gSeg
                        anchors.centerIn: parent
                        spacing: 2
                        Rectangle {
                            height: 22; width: 100; radius: 3
                            color: !pm.globalVpn ? "#4e97fa" : "transparent"
                            Label { anchors.centerIn: parent; text: pm.t("apps.selective"); color: !pm.globalVpn ? "#ffffff" : "#9e9ea6"; font.pixelSize: 10; font.bold: !pm.globalVpn }
                            MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: if (pm.globalVpn) pm.toggleGlobalVpn() }
                        }
                        Rectangle {
                            height: 22; width: 118; radius: 3
                            color: pm.globalVpn ? "#4e97fa" : "transparent"
                            Label { anchors.centerIn: parent; text: pm.t("apps.allVpn"); color: pm.globalVpn ? "#ffffff" : "#9e9ea6"; font.pixelSize: 10; font.bold: pm.globalVpn }
                            MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: if (!pm.globalVpn) pm.toggleGlobalVpn() }
                        }
                    }
                }

                Rectangle {
                    height: 26; radius: 4; color: "#222226"; border.color: "#36363e"
                    implicitWidth: rstAllLbl.implicitWidth + 16
                    Label { id: rstAllLbl; anchors.centerIn: parent; text: pm.t("apps.resetDirect"); color: "#dcdce2"; font.pixelSize: 10 }
                    MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: pm.resetAllAppOverrides() }
                }

                Item { width: 8; height: 1 }

                // Filter Pills
                Repeater {
                    model: [
                        {id: "all", key: "apps.filterAll"},
                        {id: "active", key: "apps.filterActive"},
                        {id: "offline", key: "apps.filterOffline"},
                        {id: "vpn", key: "apps.filterVpn"},
                        {id: "blocked", key: "apps.filterBlocked"}
                    ]
                    delegate: Rectangle {
                        height: 26; radius: 13
                        color: root.routingFilter === modelData.id ? "#222938" : "#151518"
                        border.color: root.routingFilter === modelData.id ? "#4e97fa" : "#2c2c32"
                        implicitWidth: pfLbl.implicitWidth + 18
                        Label {
                            id: pfLbl; anchors.centerIn: parent; text: pm.t(modelData.key)
                            color: root.routingFilter === modelData.id ? "#ffffff" : "#9e9ea6"; font.pixelSize: 10; font.bold: root.routingFilter === modelData.id
                        }
                        MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: root.routingFilter = modelData.id }
                    }
                }

                Item { width: 8; height: 1 }

                // Quick Global Block Internet + DNS pill
                Rectangle {
                    height: 26; radius: 4
                    color: pm.blockInternet ? "#4d1212" : "#202025"
                    border.color: pm.blockInternet ? "#e01d1d" : "#32323a"
                    implicitWidth: gBlkLbl.implicitWidth + 16
                    Label {
                        id: gBlkLbl; anchors.centerIn: parent
                        text: pm.blockInternet ? (pm.isRu ? "Интернет: БЛОК" : "Internet: BLOCKED") : (pm.isRu ? "Блок Инета" : "Block Internet")
                        color: pm.blockInternet ? "#ff8888" : "#cacaca"; font.pixelSize: 10; font.bold: pm.blockInternet
                    }
                    MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: pm.updateSetting("filter/blockInternet", !pm.blockInternet) }
                }

                Rectangle {
                    height: 26; radius: 4; color: "#1c222e"; border.color: "#334666"
                    implicitWidth: gDnsLbl.implicitWidth + 16
                    Label {
                        id: gDnsLbl; anchors.centerIn: parent
                        text: "DNS: " + pm.dnsSummary
                        color: "#89b7ff"; font.pixelSize: 10; font.bold: true
                    }
                    MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: root.openNetworkSettings() }
                }
            }
        }

        // Inline Create Profile Drawer
        Rectangle {
            Layout.fillWidth: true; height: 50; radius: 5; color: "#1b1c22"; border.color: "#4e97fa"
            visible: root.showCreateDrawer
            RowLayout {
                anchors.fill: parent; anchors.margins: 9; spacing: 8
                TextField {
                    id: newPName; Layout.preferredWidth: 190
                    placeholderText: pm.isRu ? "Имя (напр. Firefox)" : "Name (e.g. Firefox)"
                    placeholderTextColor: "#66666e"; color: "#ffffff"; font.pixelSize: 12
                    background: Rectangle { color: "#121213"; border.color: "#2c2c32"; radius: 4 }
                }
                TextField {
                    id: newPPath; Layout.fillWidth: true
                    placeholderText: "/usr/bin/firefox"
                    placeholderTextColor: "#66666e"; color: "#ffffff"; font.pixelSize: 12; font.family: "monospace"
                    background: Rectangle { color: "#121213"; border.color: "#2c2c32"; radius: 4 }
                }
                Rectangle {
                    height: 30; radius: 4; color: "#4e97fa"
                    implicitWidth: crLbl.implicitWidth + 22
                    Label { id: crLbl; anchors.centerIn: parent; text: pm.isRu ? "Создать" : "Create"; color: "#ffffff"; font.bold: true; font.pixelSize: 11 }
                    MouseArea {
                        anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                        onClicked: { pm.createProfile(newPName.text, newPPath.text); newPName.text = ""; newPPath.text = ""; root.showCreateDrawer = false }
                    }
                }
            }
        }

        // Inline Import Profile Drawer
        Rectangle {
            Layout.fillWidth: true; height: 104; radius: 5; color: "#1b1c22"; border.color: "#4e97fa"
            visible: root.showImportDrawer
            RowLayout {
                anchors.fill: parent; anchors.margins: 9; spacing: 8
                TextArea {
                    id: impArea; Layout.fillWidth: true; Layout.fillHeight: true
                    placeholderText: pm.isRu ? "Вставьте экспортированный профиль JSON/YAML..." : "Paste exported profile JSON/YAML..."
                    placeholderTextColor: "#66666e"; color: "#ffffff"; font.pixelSize: 11; font.family: "monospace"
                    background: Rectangle { color: "#121213"; border.color: "#2c2c32"; radius: 4 }
                }
                Rectangle {
                    height: 32; radius: 4; color: "#4e97fa"
                    implicitWidth: doImpLbl.implicitWidth + 22
                    Label { id: doImpLbl; anchors.centerIn: parent; text: pm.t("apps.import"); color: "#ffffff"; font.bold: true; font.pixelSize: 11 }
                    MouseArea {
                        anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                        onClicked: { pm.importProfile(impArea.text); impArea.text = ""; root.showImportDrawer = false }
                    }
                }
            }
        }

        // Unified Virtualized GridView: Active Apps inside scrollable header + 1500+ All Apps virtualized!
        GridView {
            id: mainGrid
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            cacheBuffer: 1200
            property int cols: Math.max(1, Math.floor((width - 12) / 400))
            cellWidth: Math.floor((width - 12) / cols)
            cellHeight: 72
            model: root.cachedFilteredMain
            ScrollBar.vertical: PmScrollBar {}
            PmWheelHelper {}

            header: Item {
                width: mainGrid.width - 12
                height: hdrCol.implicitHeight + 8

                ColumnLayout {
                    id: hdrCol
                    width: parent.width
                    spacing: 10

                    RowLayout {
                        visible: root.routingFilter === "all" && (root.cachedFilteredActive.length > 0 || pm.onlyNowApps)
                        Layout.fillWidth: true
                        spacing: 10
                        Label {
                            text: pm.t("apps.active") + " (" + root.cachedFilteredActive.length + ")"
                            color: "#1de966"
                            font.pixelSize: 12
                            font.bold: true
                        }
                        Rectangle {
                            height: 22; radius: 4
                            color: pm.onlyNowApps ? "#173252" : "#1c1c20"
                            border.color: pm.onlyNowApps ? "#4e97fa" : "#2f3038"
                            implicitWidth: onlyNowLbl.implicitWidth + 14
                            Label {
                                id: onlyNowLbl; anchors.centerIn: parent
                                text: pm.isRu ? "Только сейчас" : "Now only"
                                color: pm.onlyNowApps ? "#89b7ff" : "#8e8e98"
                                font.pixelSize: 10; font.bold: pm.onlyNowApps
                            }
                            MouseArea {
                                anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                                onClicked: pm.setOnlyNowApps(!pm.onlyNowApps)
                            }
                        }
                        Item { Layout.fillWidth: true }
                    }

                    Flow {
                        id: activeFlow
                        Layout.fillWidth: true
                        spacing: 10
                        visible: root.routingFilter === "all" && root.cachedFilteredActive.length > 0
                        property int cols: Math.max(1, Math.floor((width + 10) / 400))
                        property real cw: Math.floor((width - (cols - 1) * 10) / cols)
                        Repeater {
                            model: root.cachedFilteredActive
                            AppCard { app: modelData; width: activeFlow.cw }
                        }
                    }

                    Label {
                        topPadding: (root.cachedFilteredActive.length > 0 && root.routingFilter === "all") ? 6 : 0
                        text: (root.routingFilter === "all" ? pm.t("apps.all") : pm.t("apps.title")) + " (" + root.cachedFilteredMain.length + ")"
                        color: "#9e9ea6"
                        font.pixelSize: 12
                        font.bold: true
                    }
                }
            }

            delegate: Item {
                width: mainGrid.cellWidth
                height: mainGrid.cellHeight
                AppCard {
                    anchors.fill: parent
                    anchors.rightMargin: 10
                    anchors.bottomMargin: 10
                    app: modelData
                }
            }

            Label {
                anchors.centerIn: parent
                visible: root.cachedFilteredMain.length === 0 && (root.routingFilter !== "all" || root.cachedFilteredActive.length === 0)
                text: pm.isRu ? "Профили приложений не найдены." : "No matching application profiles."
                color: "#76767e"; font.pixelSize: 13
            }
        }
    }
}
