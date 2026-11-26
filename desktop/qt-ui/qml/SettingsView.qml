import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Item {
    id: root

    property string searchQuery: ""
    property string selectedSub: "all"
    property int maxExpertise: 2 // 0=Simple, 1=Advanced, 2=Developer
    property bool appendDns: true
    property bool showImportDrawer: false

    // pm-config OptType values:
    // 1 = String, 2 = StringArray, 3 = Int, 4 = Bool
    function subsystemList() {
        var seen = { "all": true, "ui": true }
        var out = ["all", "ui"]
        var list = pm.settingsList || []
        for (var i = 0; i < list.length; i++) {
            var s = list[i].subsystem || "core"
            if (!seen[s]) { seen[s] = true; out.push(s) }
        }
        return out
    }

    function filteredSettings() {
        if (selectedSub === "ui") return []
        var list = pm.settingsList || []
        var q = searchQuery.toLowerCase().trim()
        var out = []
        for (var i = 0; i < list.length; i++) {
            var st = list[i]
            if ((st.expertise || 0) > maxExpertise) continue
            if (selectedSub !== "all" && st.subsystem !== selectedSub) continue
            if (q !== "") {
                var hay = ((st.key || "") + " " + (st.name || "") + " " + (st.description || "")).toLowerCase()
                if (hay.indexOf(q) < 0) continue
            }
            out.push(st)
        }
        return out
    }

    function isArraySetting(st) {
        return st.optType === 2 ||
               (st.arrayItems && st.arrayItems.length > 0) ||
               st.key === "dns/nameservers" ||
               st.key === "xray/rules" ||
               st.key === "spn/usagePolicy" ||
               st.key === "filter/endpoints"
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 24
        spacing: 14

        // Settings Page Title + Search + Expertise + Language + Export/Import
        RowLayout {
            Layout.fillWidth: true
            spacing: 10

            Label {
                text: pm.t("set.title")
                color: "#ffffff"; font.pixelSize: 20; font.bold: true
            }

            Item { Layout.preferredWidth: 8 }

            TextField {
                Layout.fillWidth: true
                height: 32
                placeholderText: pm.t("set.search")
                placeholderTextColor: "#66666e"
                color: "#ffffff"; font.pixelSize: 12
                onTextChanged: root.searchQuery = text
                background: Rectangle { color: "#1b1b1d"; radius: 4; border.color: parent.activeFocus ? "#4e97fa" : "#2c2c30" }
            }

            Row {
                spacing: 4
                Repeater {
                    model: [
                        { lvl: 0, label: pm.isRu ? "Простой" : "Simple" },
                        { lvl: 1, label: pm.isRu ? "Продвин." : "Advanced" },
                        { lvl: 2, label: pm.isRu ? "Разраб." : "Developer" }
                    ]
                    delegate: Rectangle {
                        height: 30; radius: 4
                        implicitWidth: expLvlLbl.implicitWidth + 14
                        color: root.maxExpertise === modelData.lvl ? "#2c2f3a" : "#1c1c1e"
                        border.color: root.maxExpertise === modelData.lvl ? "#4e97fa" : "#2a2a2e"
                        Label { id: expLvlLbl; anchors.centerIn: parent; text: modelData.label; color: "#ffffff"; font.pixelSize: 11 }
                        MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: root.maxExpertise = modelData.lvl }
                    }
                }
            }

            Row {
                spacing: 4
                Repeater {
                    model: [ "ru", "en" ]
                    delegate: Rectangle {
                        height: 30; radius: 4; width: 36
                        color: pm.lang === modelData ? "#4e97fa" : "#1c1c1e"
                        border.color: "#2e2e34"
                        Label { anchors.centerIn: parent; text: modelData.toUpperCase(); color: "#ffffff"; font.pixelSize: 11; font.bold: true }
                        MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: pm.setLanguage(modelData) }
                    }
                }
            }

            Rectangle {
                height: 30; radius: 4; color: "#222328"; border.color: "#32343d"
                implicitWidth: expS.implicitWidth + 16
                Label { id: expS; anchors.centerIn: parent; text: pm.t("set.export"); color: "#cacaca"; font.pixelSize: 11 }
                MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: pm.exportSettings() }
            }

            Rectangle {
                height: 30; radius: 4; color: root.showImportDrawer ? "#4e97fa" : "#222328"; border.color: "#32343d"
                implicitWidth: impS.implicitWidth + 16
                Label { id: impS; anchors.centerIn: parent; text: pm.t("set.import"); color: "#ffffff"; font.pixelSize: 11 }
                MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: root.showImportDrawer = !root.showImportDrawer }
            }
        }

        // Optional Settings Import Drawer
        Rectangle {
            visible: root.showImportDrawer
            Layout.fillWidth: true
            height: 60; radius: 6; color: "#1b1c22"; border.color: "#4e97fa"
            RowLayout {
                anchors.fill: parent; anchors.margins: 12; spacing: 10
                TextField {
                    id: impSetField
                    Layout.fillWidth: true
                    placeholderText: pm.isRu ? "Вставьте экспортированный конфиг Portmaster..." : "Paste exported Portmaster config payload..."
                    placeholderTextColor: "#66666e"
                    color: "#ffffff"; font.pixelSize: 11; font.family: "monospace"
                    background: Rectangle { color: "#121213"; radius: 4; border.color: "#2e2e32" }
                }
                Rectangle {
                    height: 32; radius: 4; color: "#4e97fa"
                    implicitWidth: applyImpLbl.implicitWidth + 20
                    Label { id: applyImpLbl; anchors.centerIn: parent; text: pm.isRu ? "Применить" : "Import"; color: "#ffffff"; font.pixelSize: 11; font.bold: true }
                    MouseArea {
                        anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                        onClicked: {
                            if (impSetField.text.trim() !== "") {
                                pm.importSettings(impSetField.text.trim())
                                impSetField.text = ""
                                root.showImportDrawer = false
                            }
                        }
                    }
                }
            }
        }

        // Main Split: Subsystems Left + Scrollable Settings Cards Right
        RowLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            spacing: 14

            // Subsystems List
            Rectangle {
                Layout.preferredWidth: 172
                Layout.fillHeight: true
                radius: 6
                color: "#161618"
                border.color: "#242428"

                ListView {
                    anchors.fill: parent
                    anchors.margins: 6
                    clip: true
                    spacing: 2
                    model: root.subsystemList()
                    ScrollBar.vertical: PmScrollBar {}
                    delegate: Rectangle {
                        width: ListView.view.width
                        height: 32
                        radius: 4
                        color: root.selectedSub === modelData ? "#242732" : (subMa.containsMouse ? "#1e1e22" : "transparent")
                        Label {
                            anchors.verticalCenter: parent.verticalCenter
                            anchors.left: parent.left; anchors.leftMargin: 12
                            text: modelData === "all"
                                  ? (pm.isRu ? "Все разделы" : "All Sections")
                                  : (modelData === "ui" ? (pm.isRu ? "UI и Порт :817" : "UI & Port :817") : modelData.toUpperCase())
                            color: root.selectedSub === modelData ? "#4e97fa" : "#b8b8c0"
                            font.pixelSize: 11; font.bold: root.selectedSub === modelData
                        }
                        MouseArea {
                            id: subMa; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor
                            onClicked: root.selectedSub = modelData
                        }
                    }
                }
            }

            // Settings Cards Feed (UI/RAM/Port Card scrolls smoothly at top of All / UI / Core!)
            ListView {
                id: settingsListView
                Layout.fillWidth: true
                Layout.fillHeight: true
                clip: true
                spacing: 8
                model: root.filteredSettings()
                ScrollBar.vertical: PmScrollBar {}

                header: Item {
                    width: settingsListView.width - 10
                    property bool showUiCard: root.selectedSub === "all" || root.selectedSub === "ui" || root.selectedSub === "core"
                    height: showUiCard ? (uiCardRect.implicitHeight + 10) : 0
                    visible: showUiCard

                    Rectangle {
                        id: uiCardRect
                        width: parent.width
                        implicitHeight: uiCardCol.implicitHeight + 24
                        radius: 6
                        color: "#1b1b1e"
                        border.color: "#2c2e38"

                        ColumnLayout {
                            id: uiCardCol
                            anchors.fill: parent
                            anchors.margins: 12
                            spacing: 10

                            RowLayout {
                                Layout.fillWidth: true
                                spacing: 10

                                Label {
                                    text: pm.t("set.uiTitle")
                                    color: "#ffffff"; font.pixelSize: 14; font.bold: true
                                }
                                Item { Layout.fillWidth: true }

                                Rectangle {
                                    height: 22; radius: 4
                                    color: pm.uiReport.apiPortListening ? "#134729" : "#24262e"
                                    border.color: pm.uiReport.apiPortListening ? "#1de966" : "#4e97fa"
                                    implicitWidth: portLbl.implicitWidth + 14
                                    Label {
                                        id: portLbl; anchors.centerIn: parent
                                        text: pm.uiReport.apiPortListening
                                              ? (":817 " + pm.t("set.portOpen"))
                                              : (":817 " + (pm.isRu ? "Закрыт (Unix-сокет)" : "Closed (Unix Socket)"))
                                        color: pm.uiReport.apiPortListening ? "#1de966" : "#89b7ff"
                                        font.pixelSize: 10; font.bold: true
                                    }
                                }

                                Rectangle {
                                    visible: (pm.uiReport.unixSocketPath || "") !== ""
                                    height: 22; radius: 4; color: "#202127"; border.color: "#32343e"
                                    implicitWidth: sockLbl.implicitWidth + 14
                                    Label {
                                        id: sockLbl; anchors.centerIn: parent
                                        text: pm.uiReport.unixSocketPath || ""
                                        color: "#8e8e98"; font.pixelSize: 10; font.family: "monospace"
                                    }
                                }
                            }

                            RowLayout {
                                Layout.fillWidth: true
                                spacing: 10

                                Repeater {
                                    model: pm.uiReport.flavors || []
                                    delegate: Rectangle {
                                        Layout.fillWidth: true
                                        height: 78
                                        radius: 5
                                        color: modelData.active ? "#192538" : "#141416"
                                        border.color: modelData.active ? "#4e97fa" : "#272830"

                                        ColumnLayout {
                                            anchors.fill: parent
                                            anchors.margins: 10
                                            spacing: 4

                                            RowLayout {
                                                Layout.fillWidth: true
                                                spacing: 6
                                                Label {
                                                    text: modelData.name || modelData.id
                                                    color: "#ffffff"; font.pixelSize: 12; font.bold: true
                                                    elide: Text.ElideRight; Layout.fillWidth: true
                                                }
                                                Rectangle {
                                                    visible: !!modelData.active
                                                    height: 16; radius: 3; color: "#4e97fa"
                                                    implicitWidth: actB.implicitWidth + 10
                                                    Label { id: actB; anchors.centerIn: parent; text: pm.isRu ? "Активен" : "Active"; color: "#ffffff"; font.pixelSize: 9; font.bold: true }
                                                }
                                                Rectangle {
                                                    visible: !modelData.active && !!modelData.running
                                                    height: 16; radius: 3; color: "#134729"
                                                    implicitWidth: runB.implicitWidth + 10
                                                    Label { id: runB; anchors.centerIn: parent; text: "ON"; color: "#1de966"; font.pixelSize: 9; font.bold: true }
                                                }
                                            }

                                            RowLayout {
                                                Layout.fillWidth: true
                                                Label {
                                                    text: {
                                                        var live = (modelData.liveRamMb !== null && modelData.liveRamMb !== undefined)
                                                                   ? (modelData.liveRamMb.toFixed(1) + " MB PSS")
                                                                   : (pm.isRu ? "не запущен" : "idle")
                                                        var base = (modelData.benchmarkRamMb !== null && modelData.benchmarkRamMb !== undefined)
                                                                   ? (" • " + (pm.isRu ? "эталон: " : "base: ") + modelData.benchmarkRamMb.toFixed(1) + " MB")
                                                                   : ""
                                                        return live + base
                                                    }
                                                    color: "#9fd0ff"; font.pixelSize: 11; font.bold: true
                                                    Layout.fillWidth: true
                                                }
                                                Rectangle {
                                                    visible: !modelData.active && !!modelData.installed
                                                    height: 22; radius: 3; color: "#282b34"; border.color: "#4e97fa"
                                                    implicitWidth: swLbl.implicitWidth + 14
                                                    Label { id: swLbl; anchors.centerIn: parent; text: pm.isRu ? "Выбрать" : "Activate"; color: "#ffffff"; font.pixelSize: 10 }
                                                    MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: pm.switchUiFlavor(modelData.id) }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            RowLayout {
                                Layout.fillWidth: true
                                spacing: 10
                                PmSwitch {
                                    checked: !!pm.uiReport.keepApiPortOpen
                                    onToggled: pm.setKeepApiPortOpen(checked)
                                }
                                Label {
                                    text: pm.t("set.keepPort")
                                    color: "#ffffff"; font.pixelSize: 12; font.bold: true
                                }
                                Label {
                                    text: pm.uiReport.keepApiPortOpen ? "" : ("(" + pm.t("set.portClosed") + ")")
                                    color: "#89b7ff"; font.pixelSize: 11
                                    elide: Text.ElideRight; Layout.fillWidth: true
                                }
                            }
                        }
                    }
                }

                delegate: Rectangle {
                    id: stCard
                    property var st: modelData
                    property bool isArr: root.isArraySetting(st)
                    width: ListView.view.width - 10
                    implicitHeight: stCol.implicitHeight + 24
                    radius: 5
                    color: "#1a1a1d"
                    border.color: "#26262b"

                    // Modified blue bar on left (1:1 Portmaster Angular card look)
                    Rectangle {
                        visible: !!st.modified
                        width: 3
                        anchors.left: parent.left; anchors.top: parent.top; anchors.bottom: parent.bottom
                        color: "#4e97fa"
                    }

                    ColumnLayout {
                        id: stCol
                        anchors.left: parent.left; anchors.right: parent.right; anchors.top: parent.top
                        anchors.margins: 12; anchors.leftMargin: 16
                        spacing: 8

                        RowLayout {
                            Layout.fillWidth: true
                            spacing: 10

                            ColumnLayout {
                                Layout.fillWidth: true
                                spacing: 2
                                RowLayout {
                                    spacing: 8
                                    Label { text: st.name || st.key; color: "#ffffff"; font.pixelSize: 13; font.bold: true }
                                    Rectangle {
                                        height: 16; radius: 3; color: "#25262b"
                                        implicitWidth: kLbl.implicitWidth + 10
                                        Label { id: kLbl; anchors.centerIn: parent; text: st.key; color: "#888894"; font.pixelSize: 10; font.family: "monospace" }
                                    }
                                    Label {
                                        visible: !!st.modified
                                        text: pm.isRu ? "↺ Сбросить" : "↺ Reset"
                                        color: "#4e97fa"; font.pixelSize: 11
                                        MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: pm.resetSetting(st.key) }
                                    }
                                }
                                Label {
                                    visible: (st.description || "") !== ""
                                    text: st.description || ""
                                    color: "#84848e"; font.pixelSize: 11; wrapMode: Text.WordWrap
                                    Layout.fillWidth: true
                                }
                            }

                            // Bool Toggle Switch (optType == 4 in pm-config!)
                            PmSwitch {
                                visible: st.optType === 4 && !stCard.isArr
                                checked: !!st.boolVal
                                onToggled: pm.updateSetting(st.key, checked)
                            }
                        }

                        // Choice Selector (choices.length > 0 for non-Bool)
                        RowLayout {
                            visible: st.choices && st.choices.length > 0 && st.optType !== 4 && !stCard.isArr
                            Layout.fillWidth: true
                            spacing: 6
                            Flow {
                                Layout.fillWidth: true
                                spacing: 6
                                Repeater {
                                    model: st.choices || []
                                    delegate: Rectangle {
                                        property bool sel: String(modelData.Value) === String(st.valueStr)
                                        height: 26; radius: 4
                                        color: sel ? "#173b66" : "#242429"
                                        border.color: sel ? "#4e97fa" : "#32323a"
                                        implicitWidth: chLbl.implicitWidth + 16
                                        Label {
                                            id: chLbl; anchors.centerIn: parent
                                            text: modelData.Name || String(modelData.Value)
                                            color: sel ? "#ffffff" : "#cacaca"; font.pixelSize: 11
                                        }
                                        MouseArea {
                                            anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                                            onClicked: pm.updateSetting(st.key, modelData.Value)
                                        }
                                    }
                                }
                            }
                        }

                        // Array Editor + DNS/Xray/Endpoint Presets (optType == 2 or known array keys)
                        ColumnLayout {
                            visible: stCard.isArr
                            Layout.fillWidth: true
                            spacing: 6

                            Flow {
                                Layout.fillWidth: true
                                spacing: 6
                                Repeater {
                                    model: st.arrayItems || []
                                    delegate: Rectangle {
                                        height: 24; radius: 4; color: "#23252c"; border.color: "#343742"
                                        implicitWidth: chipRow.implicitWidth + 14
                                        Row {
                                            id: chipRow; anchors.centerIn: parent; spacing: 6
                                            Label { text: modelData; color: "#ffffff"; font.pixelSize: 11; font.family: "monospace" }
                                            Label {
                                                text: "✕"; color: "#e01d1d"; font.pixelSize: 10
                                                MouseArea {
                                                    anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                                                    onClicked: {
                                                        var next = []
                                                        var cur = st.arrayItems || []
                                                        for (var k = 0; k < cur.length; k++) if (cur[k] !== modelData) next.push(cur[k])
                                                        pm.updateSetting(st.key, next)
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            RowLayout {
                                Layout.fillWidth: true
                                spacing: 8
                                TextField {
                                    id: arrInput
                                    Layout.fillWidth: true
                                    placeholderText: pm.isRu ? "Добавить значение..." : "Add value..."
                                    placeholderTextColor: "#66666e"
                                    color: "#ffffff"; font.pixelSize: 11; font.family: "monospace"
                                    background: Rectangle { color: "#121213"; radius: 4; border.color: "#2e2e32" }
                                }
                                Rectangle {
                                    height: 28; radius: 4; color: "#4e97fa"
                                    implicitWidth: arrAddLbl.implicitWidth + 16
                                    Label { id: arrAddLbl; anchors.centerIn: parent; text: "+"; color: "#ffffff"; font.pixelSize: 13; font.bold: true }
                                    MouseArea {
                                        anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                                        onClicked: {
                                            var v = arrInput.text.trim()
                                            if (v === "") return
                                            pm.applyPreset(st.key, [v], true)
                                            arrInput.text = ""
                                        }
                                    }
                                }
                            }

                            // DNS Append Checkbox + Presets
                            PmCheckBox {
                                visible: st.key === "dns/nameservers"
                                text: pm.t("set.appendDns")
                                checked: root.appendDns
                                onToggled: root.appendDns = checked
                            }

                            Flow {
                                visible: st.key === "dns/nameservers" || st.key === "xray/rules" || st.key === "spn/usagePolicy" || st.key === "filter/endpoints"
                                Layout.fillWidth: true
                                spacing: 6
                                Repeater {
                                    model: {
                                        if (st.key === "dns/nameservers") return [
                                            { label: "Cloudflare DoH", vals: ["https://cloudflare-dns.com/dns-query", "https://1.1.1.1/dns-query"] },
                                            { label: "Quad9 DoH",      vals: ["https://dns.quad9.net/dns-query"] },
                                            { label: "AdGuard DoH",    vals: ["https://dns.adguard-dns.com/dns-query"] },
                                            { label: "Mullvad DoH",    vals: ["https://dns.mullvad.net/dns-query"] }
                                        ]
                                        if (st.key === "xray/rules" || st.key === "spn/usagePolicy") return [
                                            { label: pm.isRu ? "RU напрямую (geoip:ru)" : "RU Direct", vals: ["direct:.ru", "direct:.xn--p1ai", "direct:geoip:ru", "direct:geosite:category-ru"] },
                                            { label: "YouTube + Discord -> Xray", vals: ["proxy:geosite:youtube", "proxy:geosite:discord"] },
                                            { label: "OpenAI / AI -> Xray", vals: ["proxy:geosite:openai", "proxy:geosite:anthropic"] }
                                        ]
                                        return [
                                            { label: "+ KDE Connect LAN (1714-1764) #2324", vals: ["+ LAN 1714-1764"] },
                                            { label: "- Ads (geosite)", vals: ["- geosite:category-ads-all"] }
                                        ]
                                    }
                                    delegate: Rectangle {
                                        height: 22; radius: 3; color: "#23262f"; border.color: "#384152"
                                        implicitWidth: pL.implicitWidth + 12
                                        Label { id: pL; anchors.centerIn: parent; text: modelData.label; color: "#89b7ff"; font.pixelSize: 10 }
                                        MouseArea {
                                            anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                                            onClicked: pm.applyPreset(st.key, modelData.vals, st.key === "dns/nameservers" ? root.appendDns : true)
                                        }
                                    }
                                }
                            }
                        }

                        // Int Editor (optType == 3)
                        RowLayout {
                            visible: st.optType === 3 && (!st.choices || st.choices.length === 0) && !stCard.isArr
                            Layout.fillWidth: true
                            spacing: 8
                            Rectangle {
                                width: 28; height: 28; radius: 4; color: "#22242a"; border.color: "#343640"
                                Label { anchors.centerIn: parent; text: "−"; color: "#ffffff"; font.pixelSize: 14 }
                                MouseArea {
                                    anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                                    onClicked: {
                                        var v = parseInt(intInput.text)
                                        if (!isNaN(v)) pm.updateSetting(st.key, v - 1)
                                    }
                                }
                            }
                            TextField {
                                id: intInput
                                Layout.preferredWidth: 130
                                text: String(st.intVal)
                                color: "#ffffff"; font.pixelSize: 12; font.family: "monospace"
                                background: Rectangle { color: "#121213"; radius: 4; border.color: "#2e2e32" }
                            }
                            Rectangle {
                                width: 28; height: 28; radius: 4; color: "#22242a"; border.color: "#343640"
                                Label { anchors.centerIn: parent; text: "+"; color: "#ffffff"; font.pixelSize: 14 }
                                MouseArea {
                                    anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                                    onClicked: {
                                        var v = parseInt(intInput.text)
                                        if (!isNaN(v)) pm.updateSetting(st.key, v + 1)
                                    }
                                }
                            }
                            Rectangle {
                                height: 28; radius: 4; color: "#282c36"; border.color: "#4e97fa"
                                implicitWidth: intApLbl.implicitWidth + 18
                                Label { id: intApLbl; anchors.centerIn: parent; text: pm.isRu ? "Применить" : "Apply"; color: "#ffffff"; font.pixelSize: 11 }
                                MouseArea {
                                    anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                                    onClicked: {
                                        var v = parseInt(intInput.text)
                                        if (!isNaN(v)) pm.updateSetting(st.key, v)
                                    }
                                }
                            }
                        }

                        // Plain String Editor (optType == 1)
                        RowLayout {
                            visible: st.optType === 1 && (!st.choices || st.choices.length === 0) && !stCard.isArr
                            Layout.fillWidth: true
                            spacing: 8
                            TextField {
                                id: scalarInput
                                Layout.fillWidth: true
                                text: st.valueStr || ""
                                color: "#ffffff"; font.pixelSize: 12; font.family: "monospace"
                                background: Rectangle { color: "#121213"; radius: 4; border.color: "#2e2e32" }
                            }
                            Rectangle {
                                height: 28; radius: 4; color: "#282c36"; border.color: "#4e97fa"
                                implicitWidth: apLbl.implicitWidth + 18
                                Label { id: apLbl; anchors.centerIn: parent; text: pm.isRu ? "Применить" : "Apply"; color: "#ffffff"; font.pixelSize: 11 }
                                MouseArea {
                                    anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                                    onClicked: pm.updateSetting(st.key, scalarInput.text)
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
