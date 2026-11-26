import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Item {
    id: root
    signal openApp(string key)

    property string searchQuery: ""
    property string routingFilter: "all" // "all" | "active" | "offline" | "vpn" | "blocked"
    property bool showCreateDrawer: false
    property bool showImportDrawer: false

    function matchesFilter(app) {
        if (searchQuery !== "") {
            var q = searchQuery.toLowerCase()
            if (app.name.toLowerCase().indexOf(q) < 0 && app.path.toLowerCase().indexOf(q) < 0) return false
        }
        if (routingFilter === "active") return app.alive > 0
        if (routingFilter === "offline") return app.alive <= 0
        if (routingFilter === "vpn") return app.effectiveVpn
        if (routingFilter === "blocked") return app.action === "block"
        return true
    }

    function filteredList(src) {
        var res = []
        for (var i = 0; i < src.length; i++) {
            if (matchesFilter(src[i])) res.push(src[i])
        }
        return res
    }

    component AppCard: Rectangle {
        property var app: ({})
        width: cardGrid.cellW
        height: 54
        radius: 4
        color: cardMa.containsMouse ? "#292929" : "#202020"

        // Top-right blue dot when customized (1:1 Angular card-header)
        Rectangle {
            visible: app.hasCustom
            width: 7; height: 7; radius: 3.5
            color: "#4e97fa"
            anchors.top: parent.top; anchors.right: parent.right; anchors.margins: 5
        }

        RowLayout {
            anchors.fill: parent
            anchors.leftMargin: 10
            anchors.rightMargin: 12
            spacing: 9

            // 22x22 Monogram / Blocked Icon matching overview.html
            Rectangle {
                width: 26; height: 26; radius: 5
                color: app.action === "block" ? "#3d1414" : "#2c2c2c"
                border.color: app.action === "block" ? "#e01d1d" : "transparent"
                Label {
                    anchors.centerIn: parent
                    text: app.initials || "A"
                    color: app.action === "block" ? "#e01d1d" : "#ffffff"
                    font.pixelSize: 10; font.bold: true
                }
            }

            // App Name + Path + Visual Status Badges (#2251)
            ColumnLayout {
                Layout.fillWidth: true
                spacing: 1
                RowLayout {
                    Layout.fillWidth: true
                    spacing: 5
                    Label {
                        text: app.name || ""
                        color: cardMa.containsMouse ? "#ffffff" : "#cacaca"
                        font.pixelSize: 12; font.bold: true
                        elide: Text.ElideRight
                        Layout.maximumWidth: 165
                    }
                    Rectangle {
                        visible: app.action === "block"
                        height: 15; radius: 2; color: "#3d1414"; border.color: "#811818"
                        implicitWidth: blkLbl.implicitWidth + 8
                        Label { id: blkLbl; anchors.centerIn: parent; text: "BLK"; color: "#e01d1d"; font.pixelSize: 8; font.bold: true }
                    }
                    Rectangle {
                        visible: app.effectiveSplit
                        height: 15; radius: 2; color: "#3d3a14"; border.color: "#e9d81d"
                        implicitWidth: stLbl.implicitWidth + 8
                        Label { id: stLbl; anchors.centerIn: parent; text: "ST"; color: "#e9d81d"; font.pixelSize: 8; font.bold: true }
                    }
                }
                Label {
                    text: app.alive > 0 ? (app.alive + " conn • ↓" + app.rxStr + " ↑" + app.txStr) : (app.path || app.source || "")
                    color: app.alive > 0 ? "#1de966" : "#888888"
                    font.pixelSize: 10
                    elide: Text.ElideMiddle
                    Layout.fillWidth: true
                }
            }

            // Exact 3-segment Auto / Xray / Direct toggle matching Angular overview.html line 195
            Rectangle {
                height: 22; radius: 4; color: "#222222"; border.color: "#474747"
                implicitWidth: segRow.implicitWidth + 2
                Row {
                    id: segRow
                    anchors.centerIn: parent
                    spacing: 0

                    Rectangle {
                        height: 20; width: 42; radius: 3
                        color: app.vpnMode === "default" ? (app.effectiveVpn ? "#4e97fa" : "#474747") : "transparent"
                        Label {
                            anchors.centerIn: parent; text: "Auto"
                            color: app.vpnMode === "default" ? "#ffffff" : "#ababab"; font.pixelSize: 10; font.bold: app.vpnMode === "default"
                        }
                        MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: pm.setAppVpnMode(app.key, "default") }
                    }
                    Rectangle { width: 1; height: 18; color: "#474747" }
                    Rectangle {
                        height: 20; width: 40; radius: 3
                        color: app.vpnMode === "vpn" ? "#4e97fa" : "transparent"
                        Label {
                            anchors.centerIn: parent; text: "Xray"
                            color: app.vpnMode === "vpn" ? "#ffffff" : "#ababab"; font.pixelSize: 10; font.bold: app.vpnMode === "vpn"
                        }
                        MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: pm.setAppVpnMode(app.key, "vpn") }
                    }
                    Rectangle { width: 1; height: 18; color: "#474747" }
                    Rectangle {
                        height: 20; width: 44; radius: 3
                        color: app.vpnMode === "direct" ? "#18823d" : "transparent"
                        Label {
                            anchors.centerIn: parent; text: "Direct"
                            color: app.vpnMode === "direct" ? "#ffffff" : "#ababab"; font.pixelSize: 10; font.bold: app.vpnMode === "direct"
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
        anchors.leftMargin: 32
        anchors.rightMargin: 28
        anchors.topMargin: 20
        anchors.bottomMargin: 14
        spacing: 10

        // Search Bar
        RowLayout {
            Layout.fillWidth: true
            TextField {
                Layout.preferredWidth: 260
                placeholderText: pm.t("apps.search")
                placeholderTextColor: "#666666"
                color: "#ffffff"
                font.pixelSize: 12
                onTextChanged: root.searchQuery = text
                background: Rectangle { color: "#1b1b1b"; border.color: "#2c2c2c"; radius: 4 }
            }
            Item { Layout.fillWidth: true }
            Button {
                text: pm.t("apps.newProfile")
                onClicked: { root.showCreateDrawer = !root.showCreateDrawer; root.showImportDrawer = false }
                background: Rectangle { color: "#222222"; border.color: "#2c2c2c"; radius: 4 }
                contentItem: Text { text: parent.text; color: "#cacaca"; font.pixelSize: 11; padding: 5 }
            }
            Button {
                text: pm.t("apps.import")
                onClicked: { root.showImportDrawer = !root.showImportDrawer; root.showCreateDrawer = false }
                background: Rectangle { color: "#222222"; border.color: "#2c2c2c"; radius: 4 }
                contentItem: Text { text: parent.text; color: "#4e97fa"; font.pixelSize: 11; padding: 5 }
            }
        }

        // Page Header Title
        Label {
            text: pm.t("apps.title")
            color: "#ffffff"
            font.pixelSize: 20
            font.bold: true
        }

        // Global Routing Strip matching overview.html line 88 1:1
        Rectangle {
            Layout.fillWidth: true
            implicitHeight: gRow.implicitHeight + 14
            radius: 4
            color: "#1b1b1b"
            border.color: "#2c2c2c"

            Flow {
                id: gRow
                anchors.fill: parent
                anchors.margins: 7
                spacing: 8

                Label { text: pm.t("apps.globalRouting"); color: "#ababab"; font.pixelSize: 11; height: 24; verticalAlignment: Text.AlignVCenter }

                Rectangle {
                    height: 24; radius: 4; color: "#222222"; border.color: "#474747"
                    implicitWidth: gSeg.implicitWidth + 2
                    Row {
                        id: gSeg
                        anchors.centerIn: parent
                        Rectangle {
                            height: 22; width: 92; radius: 3
                            color: !pm.globalVpn ? "#4e97fa" : "transparent"
                            Label { anchors.centerIn: parent; text: pm.t("apps.selective"); color: !pm.globalVpn ? "#ffffff" : "#ababab"; font.pixelSize: 10; font.bold: !pm.globalVpn }
                            MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: if (pm.globalVpn) pm.toggleGlobalVpn() }
                        }
                        Rectangle {
                            height: 22; width: 108; radius: 3
                            color: pm.globalVpn ? "#4e97fa" : "transparent"
                            Label { anchors.centerIn: parent; text: pm.t("apps.allVpn"); color: pm.globalVpn ? "#ffffff" : "#ababab"; font.pixelSize: 10; font.bold: pm.globalVpn }
                            MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: if (!pm.globalVpn) pm.toggleGlobalVpn() }
                        }
                    }
                }

                Button {
                    height: 24
                    text: pm.t("apps.resetDirect")
                    onClicked: pm.resetAllAppOverrides()
                    background: Rectangle { color: "#222222"; border.color: "#474747"; radius: 4 }
                    contentItem: Text { text: parent.text; color: "#ffffff"; font.pixelSize: 10; padding: 4; verticalAlignment: Text.AlignVCenter }
                }

                // Filter Pills
                Repeater {
                    model: [
                        {id: "all", key: "apps.filterAll"},
                        {id: "active", key: "apps.filterActive"},
                        {id: "offline", key: "apps.filterOffline"},
                        {id: "vpn", key: "apps.filterVpn"},
                        {id: "blocked", key: "apps.filterBlocked"}
                    ]
                    Rectangle {
                        height: 24; radius: 4
                        color: root.routingFilter === modelData.id ? "#4e97fa" : "#222222"
                        border.color: root.routingFilter === modelData.id ? "#4e97fa" : "#474747"
                        implicitWidth: pfLbl.implicitWidth + 16
                        Label {
                            id: pfLbl; anchors.centerIn: parent; text: pm.t(modelData.key)
                            color: root.routingFilter === modelData.id ? "#ffffff" : "#ababab"; font.pixelSize: 10; font.bold: root.routingFilter === modelData.id
                        }
                        MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: root.routingFilter = modelData.id }
                    }
                }
            }
        }

        // Inline Create Profile Drawer
        Rectangle {
            Layout.fillWidth: true; height: 46; radius: 4; color: "#1b1b1b"; border.color: "#4e97fa"
            visible: root.showCreateDrawer
            RowLayout {
                anchors.fill: parent; anchors.margins: 8; spacing: 8
                TextField { id: newPName; Layout.preferredWidth: 180; placeholderText: "Name (e.g. Firefox)"; color: "#ffffff"; background: Rectangle { color: "#121213"; border.color: "#2c2c2c"; radius: 3 } }
                TextField { id: newPPath; Layout.fillWidth: true; placeholderText: "/usr/bin/firefox"; color: "#ffffff"; background: Rectangle { color: "#121213"; border.color: "#2c2c2c"; radius: 3 } }
                Button {
                    text: "Create"
                    onClicked: { pm.createProfile(newPName.text, newPPath.text); newPName.text = ""; newPPath.text = ""; root.showCreateDrawer = false }
                    background: Rectangle { color: "#4e97fa"; radius: 4 }
                    contentItem: Text { text: parent.text; color: "#ffffff"; font.bold: true; font.pixelSize: 11; padding: 5 }
                }
            }
        }

        // Inline Import Profile Drawer
        Rectangle {
            Layout.fillWidth: true; height: 104; radius: 4; color: "#1b1b1b"; border.color: "#4e97fa"
            visible: root.showImportDrawer
            RowLayout {
                anchors.fill: parent; anchors.margins: 8; spacing: 8
                TextArea { id: impArea; Layout.fillWidth: true; Layout.fillHeight: true; placeholderText: "Paste exported profile JSON/YAML..."; color: "#ffffff"; background: Rectangle { color: "#121213"; border.color: "#2c2c2c"; radius: 3 } }
                Button {
                    text: pm.t("apps.import")
                    onClicked: { pm.importProfile(impArea.text); impArea.text = ""; root.showImportDrawer = false }
                    background: Rectangle { color: "#4e97fa"; radius: 4 }
                    contentItem: Text { text: parent.text; color: "#ffffff"; font.bold: true; font.pixelSize: 11; padding: 6 }
                }
            }
        }

        // Responsive Multi-Column Scrollable Cards Grid (Active Section + All Profiles Section)
        ScrollView {
            id: cardGrid
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            property int cols: Math.max(1, Math.floor((width + 14) / 360))
            property real cellW: (width - (cols - 1) * 14 - 8) / cols

            ColumnLayout {
                width: cardGrid.width - 8
                spacing: 12

                // ACTIVE APPLICATIONS HEADER
                Label {
                    visible: root.filteredList(pm.activeApps).length > 0 && root.routingFilter === "all"
                    text: pm.t("apps.active") + " (" + root.filteredList(pm.activeApps).length + ")"
                    color: "#ababab"
                    font.pixelSize: 13
                    font.bold: true
                }

                Flow {
                    Layout.fillWidth: true
                    spacing: 14
                    visible: root.routingFilter === "all" && root.filteredList(pm.activeApps).length > 0
                    Repeater {
                        model: root.filteredList(pm.activeApps)
                        AppCard { app: modelData }
                    }
                }

                // ALL / FILTERED PROFILES HEADER
                Label {
                    text: (root.routingFilter === "all" ? pm.t("apps.all") : pm.t("apps.title")) + " (" + (root.routingFilter === "all" ? root.filteredList(pm.offlineApps).length : root.filteredList(pm.allApps).length) + ")"
                    color: "#ababab"
                    font.pixelSize: 13
                    font.bold: true
                }

                Flow {
                    Layout.fillWidth: true
                    spacing: 14
                    Repeater {
                        model: root.routingFilter === "all" ? root.filteredList(pm.offlineApps) : root.filteredList(pm.allApps)
                        AppCard { app: modelData }
                    }
                }
            }
        }
    }
}
