import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Item {
    id: root
    signal openRuleBuilder(string key, string mode)

    property string searchQuery: ""
    property string selectedSub: "all"
    property int maxExpertise: 2 // 0=Simple, 1=Advanced, 2=Developer
    property bool appendDns: true
    property bool showImportDrawer: false

    // pm-config OptType values:
    // 1 = String, 2 = StringArray, 3 = Int, 4 = Bool
    function subsystemList() {
        var seen = { "all": true, "network": true, "ui": true }
        var out = ["all", "network", "ui"]
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
            if (selectedSub === "network") {
                if (st.subsystem !== "filter" && st.subsystem !== "dns") continue
            } else if (selectedSub !== "all" && st.subsystem !== selectedSub) {
                continue
            }
            if (q !== "") {
                var hay = ((st.key || "") + " " + (st.name || "") + " " + (st.description || "")).toLowerCase()
                if (hay.indexOf(q) < 0) continue
            }
            out.push(st)
        }
        return out
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
                        { lvl: 1, label: pm.isRu ? "Больше" : "More" },
                        { lvl: 2, label: pm.isRu ? "Все" : "All" }
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
                    PmWheelHelper {}
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
                                  : (modelData === "network"
                                     ? (pm.isRu ? "Сеть и DNS" : "Network & DNS")
                                     : (modelData === "ui" ? (pm.isRu ? "UI и Порт :817" : "UI & Port :817") : modelData.toUpperCase()))
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
                cacheBuffer: 1200
                spacing: 8
                model: root.filteredSettings()
                ScrollBar.vertical: PmScrollBar {}
                PmWheelHelper {}
                property bool userScrolled: false
                onMovementStarted: userScrolled = true
                onOriginYChanged: if (!userScrolled) positionViewAtBeginning()
                onModelChanged: {
                    userScrolled = false
                    Qt.callLater(function() { settingsListView.positionViewAtBeginning() })
                }
                Connections {
                    target: root
                    function onVisibleChanged() {
                        if (root.visible) {
                            settingsListView.userScrolled = false
                            Qt.callLater(function() { settingsListView.positionViewAtBeginning() })
                        }
                    }
                    function onSelectedSubChanged() {
                        settingsListView.userScrolled = false
                        Qt.callLater(function() { settingsListView.positionViewAtBeginning() })
                    }
                }

                header: ColumnLayout {
                    width: settingsListView.width - 10
                    height: implicitHeight
                    spacing: 10
                    property bool showUiCard: root.selectedSub === "all" || root.selectedSub === "ui" || root.selectedSub === "core"
                    property bool showNetCard: root.selectedSub === "all" || root.selectedSub === "network" || root.selectedSub === "filter" || root.selectedSub === "dns"

                    Rectangle {
                        id: uiCardRect
                        visible: parent.showUiCard
                        Layout.fillWidth: true
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
                                                        if (modelData.liveRamMb !== null && modelData.liveRamMb !== undefined) {
                                                            var cnt = modelData.processCount || 1
                                                            var suffix = modelData.id === "qt"
                                                                         ? (pm.isRu ? " (Qt активен)" : " (Qt active)")
                                                                         : (" (" + cnt + (pm.isRu ? " проц. в фоне)" : " bg procs)"))
                                                            return modelData.liveRamMb.toFixed(1) + " MB PSS" + suffix
                                                        }
                                                        var est = (modelData.benchmarkRamMb !== null && modelData.benchmarkRamMb !== undefined)
                                                                  ? modelData.benchmarkRamMb.toFixed(1) + " MB"
                                                                  : "-"
                                                        return (pm.isRu ? "не запущен • оценка ~" : "not running • est. ~") + est
                                                    }
                                                    color: (modelData.liveRamMb !== null && modelData.liveRamMb !== undefined)
                                                           ? (modelData.id === "angular" ? "#f5b864" : "#1de966")
                                                           : "#8e8e98"
                                                    font.pixelSize: 10; font.bold: true
                                                    elide: Text.ElideRight
                                                    Layout.fillWidth: true
                                                }
                                                Row {
                                                    spacing: 5
                                                    Rectangle {
                                                        visible: !modelData.active && !!modelData.installed
                                                        height: 22; radius: 3; color: "#282b34"; border.color: "#4e97fa"
                                                        implicitWidth: swLbl.implicitWidth + 12
                                                        Label { id: swLbl; anchors.centerIn: parent; text: pm.isRu ? "Выбрать" : "Activate"; color: "#ffffff"; font.pixelSize: 10 }
                                                        MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: pm.switchUiFlavor(modelData.id) }
                                                    }
                                                    Rectangle {
                                                        visible: !modelData.installed && modelData.id !== "qt"
                                                        height: 22; radius: 3; color: pm.uiBusy ? "#1e2430" : "#183354"; border.color: "#4e97fa"
                                                        implicitWidth: dlLbl.implicitWidth + 12
                                                        Label { id: dlLbl; anchors.centerIn: parent; text: pm.uiBusy ? "..." : (pm.isRu ? "Скачать" : "Download"); color: "#89b7ff"; font.pixelSize: 10; font.bold: true }
                                                        MouseArea { anchors.fill: parent; enabled: !pm.uiBusy; cursorShape: Qt.PointingHandCursor; onClicked: pm.installUiFlavor(modelData.id) }
                                                    }
                                                    Rectangle {
                                                        visible: !!modelData.installed && modelData.id !== "qt" && !modelData.active
                                                        height: 22; radius: 3; color: "#2e1818"; border.color: "#7a2a2a"
                                                        implicitWidth: rmUiLbl.implicitWidth + 10
                                                        Label { id: rmUiLbl; anchors.centerIn: parent; text: pm.isRu ? "Удалить" : "Remove"; color: "#ff8f8f"; font.pixelSize: 10 }
                                                        MouseArea { anchors.fill: parent; enabled: !pm.uiBusy; cursorShape: Qt.PointingHandCursor; onClicked: pm.uninstallUiFlavor(modelData.id) }
                                                    }
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

                            RowLayout {
                                Layout.fillWidth: true
                                spacing: 10
                                PmSwitch {
                                    checked: pm.hwAccel
                                    onToggled: pm.setHwAccel(checked)
                                }
                                Label {
                                    text: pm.isRu ? "Аппаратное ускорение GPU (Qt SceneGraph)" : "Hardware GPU Acceleration (Qt SceneGraph)"
                                    color: "#ffffff"; font.pixelSize: 12; font.bold: true
                                }
                                Label {
                                    text: pm.hwAccel
                                          ? (pm.isRu ? "(Вкл — плавная прокрутка; выкл. уберёт VRAM, применить после перезапуска)" : "(ON — smooth GPU rendering; turn OFF for 0 VRAM on restart)")
                                          : (pm.isRu ? "(Выкл — программный рендер CPU, 0 VRAM)" : "(OFF — CPU software renderer, 0 VRAM)")
                                    color: pm.hwAccel ? "#1de966" : "#e9d81d"
                                    font.pixelSize: 11
                                    elide: Text.ElideRight; Layout.fillWidth: true
                                }
                            }

                            Label {
                                visible: !!(pm.uiReport.coreMemory && pm.uiReport.coreMemory.livePssMb !== null && pm.uiReport.coreMemory.livePssMb !== undefined)
                                text: (pm.isRu ? "Фоновая служба ядра (portmaster-core-rs): " : "Core daemon (portmaster-core-rs): ")
                                      + ((pm.uiReport.coreMemory && pm.uiReport.coreMemory.livePssMb) ? pm.uiReport.coreMemory.livePssMb.toFixed(1) : "0") + " MB PSS  ("
                                      + (pm.isRu ? "базы GeoIP 102 МБ + списки фильтрации + SQLite в ОЗУ" : "102 MB GeoIP DBs + filterlists + SQLite in RAM") + ")"
                                color: "#787c88"; font.pixelSize: 10
                            }
                        }
                    }

                    // Optional Admin Password / PIN Lock Card (Session-cached 1h + Owner Filter)
                    Rectangle {
                        id: adminLockCard
                        visible: parent.showUiCard
                        Layout.fillWidth: true
                        implicitHeight: lockCol.implicitHeight + 24
                        radius: 6
                        color: !!pm.adminLock.locked ? "#241918" : "#181a20"
                        border.color: !!pm.adminLock.locked ? "#e05a1d" : (!!pm.adminLock.passwordSet ? "#2e4b73" : "#272932")

                        ColumnLayout {
                            id: lockCol
                            anchors.fill: parent
                            anchors.margins: 12
                            spacing: 10

                            RowLayout {
                                Layout.fillWidth: true
                                spacing: 10
                                Label {
                                    text: pm.isRu ? "Защита настроек паролем (необязательно)" : "Admin Password Lock (Optional)"
                                    color: "#ffffff"; font.pixelSize: 14; font.bold: true
                                }
                                Rectangle {
                                    height: 20; radius: 4
                                    color: !pm.adminLock.passwordSet ? "#23252c" : (!!pm.adminLock.locked ? "#6e2116" : "#134729")
                                    border.color: !pm.adminLock.passwordSet ? "#343742" : (!!pm.adminLock.locked ? "#e05a1d" : "#1de966")
                                    implicitWidth: lkStatL.implicitWidth + 12
                                    Label {
                                        id: lkStatL; anchors.centerIn: parent
                                        text: !pm.adminLock.passwordSet
                                              ? (pm.isRu ? "Выкл (пароль не задан)" : "Disabled (No password)")
                                              : (!!pm.adminLock.locked
                                                 ? (pm.isRu ? "Заблокировано (гость)" : "Locked (Guest mode)")
                                                 : (pm.isRu ? "Открыто (сессия 1 ч)" : "Unlocked (1h session)"))
                                        color: "#ffffff"; font.pixelSize: 10; font.bold: true
                                    }
                                }
                                Item { Layout.fillWidth: true }
                                Rectangle {
                                    visible: !!pm.adminLock.passwordSet && !pm.adminLock.locked
                                    height: 24; radius: 4; color: "#2c221b"; border.color: "#7a5328"
                                    implicitWidth: lkNowL.implicitWidth + 12
                                    Label { id: lkNowL; anchors.centerIn: parent; text: pm.isRu ? "🔒 Заблокировать" : "🔒 Lock"; color: "#ffd28a"; font.pixelSize: 10; font.bold: true }
                                    MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: pm.lockAdmin() }
                                }
                                Rectangle {
                                    visible: !!pm.adminLock.passwordSet && !pm.adminLock.locked
                                    height: 24; radius: 4; color: "#361818"; border.color: "#7a2a2a"
                                    implicitWidth: clrPwL.implicitWidth + 12
                                    Label { id: clrPwL; anchors.centerIn: parent; text: pm.isRu ? "Снять пароль" : "Remove Password"; color: "#ff8c8c"; font.pixelSize: 10 }
                                    MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: pm.configureAdminLock({ "newPassword": "" }) }
                                }
                            }

                            // Unlock row if locked
                            RowLayout {
                                visible: !!pm.adminLock.locked
                                Layout.fillWidth: true; spacing: 8
                                TextField {
                                    id: unlockSettingsPw
                                    Layout.fillWidth: true
                                    echoMode: TextInput.Password
                                    placeholderText: pm.isRu ? "Введите пароль администратора для разблокировки на 1 час..." : "Enter admin password to unlock for 1 hour..."
                                    placeholderTextColor: "#66666e"; color: "#ffffff"; font.pixelSize: 12
                                    background: Rectangle { color: "#121214"; radius: 4; border.color: "#383b45" }
                                    onAccepted: { pm.unlockAdmin(text); text = "" }
                                }
                                Rectangle {
                                    height: 30; radius: 4; color: "#4e97fa"
                                    implicitWidth: ulBtnL.implicitWidth + 18
                                    Label { id: ulBtnL; anchors.centerIn: parent; text: pm.isRu ? "Разблокировать" : "Unlock"; color: "#ffffff"; font.pixelSize: 11; font.bold: true }
                                    MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: { pm.unlockAdmin(unlockSettingsPw.text); unlockSettingsPw.text = "" } }
                                }
                            }

                            // Set / Change password + Owner & Guest Options when unlocked (or disabled)
                            RowLayout {
                                visible: !pm.adminLock.locked
                                Layout.fillWidth: true; spacing: 8
                                TextField {
                                    id: setAdminPwField
                                    Layout.fillWidth: true
                                    echoMode: TextInput.Password
                                    placeholderText: !!pm.adminLock.passwordSet
                                                     ? (pm.isRu ? "Новый пароль администратора..." : "New admin password...")
                                                     : (pm.isRu ? "Задать необязательный пароль / PIN-код..." : "Set optional admin password / PIN...")
                                    placeholderTextColor: "#66666e"; color: "#ffffff"; font.pixelSize: 12
                                    background: Rectangle { color: "#121214"; radius: 4; border.color: "#30333c" }
                                    onAccepted: if (text !== "") { pm.configureAdminLock({ "newPassword": text }); text = "" }
                                }
                                Rectangle {
                                    height: 30; radius: 4; color: "#28364d"; border.color: "#4e97fa"
                                    implicitWidth: svPwL.implicitWidth + 16
                                    Label { id: svPwL; anchors.centerIn: parent; text: !!pm.adminLock.passwordSet ? (pm.isRu ? "Сменить" : "Change") : (pm.isRu ? "Задать пароль" : "Set Password"); color: "#ffffff"; font.pixelSize: 11; font.bold: true }
                                    MouseArea {
                                        anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                                        onClicked: if (setAdminPwField.text !== "") { pm.configureAdminLock({ "newPassword": setAdminPwField.text }); setAdminPwField.text = "" }
                                    }
                                }
                            }

                            Flow {
                                visible: !!pm.adminLock.passwordSet && !pm.adminLock.locked
                                Layout.fillWidth: true
                                spacing: 14
                                RowLayout {
                                    spacing: 8
                                    PmSwitch {
                                        checked: !!pm.adminLock.autoApproveOwner
                                        onToggled: pm.configureAdminLock({ "autoApproveOwner": checked })
                                    }
                                    Label {
                                        text: pm.isRu
                                              ? ("Не спрашивать на учётке владельца" + ((pm.adminLock.createdBy || "") !== "" ? (" (" + pm.adminLock.createdBy + ")") : ""))
                                              : ("Trust owner account" + ((pm.adminLock.createdBy || "") !== "" ? (" (" + pm.adminLock.createdBy + ")") : ""))
                                        color: "#dcdce4"; font.pixelSize: 11
                                    }
                                }
                                RowLayout {
                                    spacing: 8
                                    PmSwitch {
                                        checked: !!pm.adminLock.allowPromptsWhenLocked
                                        onToggled: pm.configureAdminLock({ "allowPromptsWhenLocked": checked })
                                    }
                                    Label {
                                        text: pm.isRu ? "Разрешить запросы (Prompts) при блокировке" : "Allow interactive prompts when locked"
                                        color: "#dcdce4"; font.pixelSize: 11
                                    }
                                }
                                RowLayout {
                                    spacing: 8
                                    PmSwitch {
                                        checked: !!pm.adminLock.allowXrayWhenLocked
                                        onToggled: pm.configureAdminLock({ "allowXrayWhenLocked": checked })
                                    }
                                    Label {
                                        text: pm.isRu ? "Разрешить вкл/выкл Xray при блокировке" : "Allow Xray toggle when locked"
                                        color: "#dcdce4"; font.pixelSize: 11
                                    }
                                }
                            }
                        }
                    }

                    // Quick Network Kill-Switches & DNS Card (Global)
                    Rectangle {
                        id: networkCardRect
                        visible: parent.showNetCard
                        Layout.fillWidth: true
                        implicitHeight: netCardCol.implicitHeight + 24
                        radius: 6
                        color: pm.blockInternet ? "#261618" : "#191a1f"
                        border.color: pm.blockInternet ? "#e01d1d" : "#2c3140"

                        ColumnLayout {
                            id: netCardCol
                            anchors.fill: parent
                            anchors.margins: 12
                            spacing: 10

                            RowLayout {
                                Layout.fillWidth: true
                                spacing: 10
                                Label {
                                    text: pm.isRu ? "Быстрые настройки сети и DNS (глобально)" : "Quick Network, Blocking & DNS (Global)"
                                    color: "#ffffff"; font.pixelSize: 14; font.bold: true
                                }
                                Rectangle {
                                    visible: pm.blockInternet
                                    height: 20; radius: 4; color: "#7a1515"; border.color: "#e01d1d"
                                    implicitWidth: blkWarnLbl.implicitWidth + 14
                                    Label { id: blkWarnLbl; anchors.centerIn: parent; text: pm.isRu ? "ИНТЕРНЕТ ЗАБЛОКИРОВАН" : "INTERNET BLOCKED"; color: "#ffffff"; font.pixelSize: 9; font.bold: true }
                                }
                                Item { Layout.fillWidth: true }
                                Rectangle {
                                    height: 26; radius: 4; color: clrDnsMa.containsMouse ? "#283142" : "#22242c"; border.color: "#384152"
                                    implicitWidth: clrDnsLbl.implicitWidth + 16
                                    Label { id: clrDnsLbl; anchors.centerIn: parent; text: pm.isRu ? "Очистить кэш DNS" : "Clear DNS Cache"; color: "#89b7ff"; font.pixelSize: 11; font.bold: true }
                                    MouseArea { id: clrDnsMa; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: pm.clearDnsCache() }
                                }
                            }

                            // Network Blocking Quick Toggles
                            Flow {
                                Layout.fillWidth: true
                                spacing: 14
                                RowLayout {
                                    spacing: 8
                                    PmSwitch { checked: pm.blockInternet; onToggled: pm.updateSetting("filter/blockInternet", checked) }
                                    Label { text: pm.isRu ? "Блокировать Интернет" : "Force Block Internet"; color: pm.blockInternet ? "#ff8888" : "#ffffff"; font.pixelSize: 12; font.bold: true }
                                }
                                RowLayout {
                                    spacing: 8
                                    PmSwitch { checked: pm.blockLan; onToggled: pm.updateSetting("filter/blockLAN", checked) }
                                    Label { text: pm.isRu ? "Блокировать LAN" : "Force Block LAN"; color: "#dcdce2"; font.pixelSize: 12 }
                                }
                                RowLayout {
                                    spacing: 8
                                    PmSwitch { checked: pm.blockP2p; onToggled: pm.updateSetting("filter/blockP2P", checked) }
                                    Label { text: pm.isRu ? "Блокировать P2P" : "Force Block P2P"; color: "#dcdce2"; font.pixelSize: 12 }
                                }
                                RowLayout {
                                    spacing: 8
                                    PmSwitch { checked: pm.blockInbound; onToggled: pm.updateSetting("filter/blockInbound", checked) }
                                    Label { text: pm.isRu ? "Блокировать входящие" : "Block Inbound"; color: "#dcdce2"; font.pixelSize: 12 }
                                }
                                RowLayout {
                                    spacing: 8
                                    PmSwitch { checked: pm.preventDnsBypass; onToggled: pm.updateSetting("filter/preventBypassing", checked) }
                                    Label { text: pm.isRu ? "Запретить обход DNS" : "Block DNS Bypassing"; color: "#dcdce2"; font.pixelSize: 12 }
                                }
                            }

                            Rectangle { Layout.fillWidth: true; height: 1; color: "#272932" }

                            // Active Ordered DNS List + Custom Add + Rich Provider Presets
                            RowLayout {
                                Layout.fillWidth: true
                                spacing: 8
                                Label { text: pm.isRu ? "Активные DNS-серверы (по приоритету):" : "Active DNS Servers (Priority Order):"; color: "#cacaca"; font.pixelSize: 12; font.bold: true }
                                Item { Layout.fillWidth: true }
                                PmCheckBox {
                                    text: pm.t("set.appendDns")
                                    checked: root.appendDns
                                    onToggled: root.appendDns = checked
                                }
                            }

                            ColumnLayout {
                                Layout.fillWidth: true
                                spacing: 4
                                Repeater {
                                    model: pm.dnsServers
                                    delegate: Rectangle {
                                        Layout.fillWidth: true
                                        height: 28; radius: 4; color: "#20222a"; border.color: index === 0 ? "#3c5882" : "#2f323d"
                                        RowLayout {
                                            anchors.fill: parent; anchors.leftMargin: 8; anchors.rightMargin: 8; spacing: 8
                                            Label { text: "#" + (index + 1); color: index === 0 ? "#4e97fa" : "#76767e"; font.pixelSize: 10; font.bold: true }
                                            Label { text: modelData; color: "#9fd0ff"; font.pixelSize: 10; font.family: "monospace"; elide: Text.ElideMiddle; Layout.fillWidth: true }
                                            Label {
                                                visible: index > 0
                                                text: "↑"; color: "#89b7ff"; font.pixelSize: 12; font.bold: true
                                                MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: pm.moveSettingArrayItem("dns/nameservers", index, -1) }
                                            }
                                            Label {
                                                visible: index < (pm.dnsServers.length - 1)
                                                text: "↓"; color: "#89b7ff"; font.pixelSize: 12; font.bold: true
                                                MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: pm.moveSettingArrayItem("dns/nameservers", index, 1) }
                                            }
                                            Label {
                                                text: "✕"; color: "#e01d1d"; font.pixelSize: 11
                                                MouseArea {
                                                    anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                                                    onClicked: {
                                                        var next = []
                                                        var cur = pm.dnsServers || []
                                                        for (var k = 0; k < cur.length; k++) if (k !== index) next.push(cur[k])
                                                        pm.updateSetting("dns/nameservers", next)
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                Label {
                                    visible: pm.dnsServers.length === 0
                                    text: pm.isRu ? "Список пуст — используется системный DNS" : "List empty — using system/default DNS"
                                    color: "#76767e"; font.pixelSize: 11
                                }
                            }

                            RowLayout {
                                Layout.fillWidth: true
                                spacing: 8
                                TextField {
                                    id: quickDnsInput
                                    Layout.fillWidth: true
                                    placeholderText: pm.isRu ? "Добавить DNS (dot://..., https://.../dns-query, 9.9.9.9)..." : "Add custom DNS (dot://..., https://..., 9.9.9.9)..."
                                    placeholderTextColor: "#66666e"
                                    color: "#ffffff"; font.pixelSize: 11; font.family: "monospace"
                                    background: Rectangle { color: "#121213"; radius: 4; border.color: "#2e2e32" }
                                    function addCustom() {
                                        var v = quickDnsInput.text.trim()
                                        if (v === "") return
                                        pm.applyPreset("dns/nameservers", [v], true)
                                        quickDnsInput.text = ""
                                    }
                                    onAccepted: addCustom()
                                }
                                Rectangle {
                                    height: 28; radius: 4; color: "#4e97fa"
                                    implicitWidth: qdAddLbl.implicitWidth + 16
                                    Label { id: qdAddLbl; anchors.centerIn: parent; text: pm.isRu ? "+ Добавить" : "+ Add"; color: "#ffffff"; font.pixelSize: 11; font.bold: true }
                                    MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: quickDnsInput.addCustom() }
                                }
                            }

                            Flow {
                                Layout.fillWidth: true
                                spacing: 6
                                Repeater {
                                    model: [
                                        { label: pm.cfgText("qs.dns.allDot"), vals: ["dot://cloudflare-dns.com?ip=1.1.1.1&name=Cloudflare", "dot://dns.quad9.net?ip=9.9.9.9&name=Quad9&blockedif=empty", "dot://dns.adguard-dns.com?ip=94.140.14.14&name=AdGuard&blockedif=zeroip", "dot://dns.google?ip=8.8.8.8&name=Google"] },
                                        { label: pm.cfgText("qs.dns.allDoh"), vals: ["https://cloudflare-dns.com/dns-query?ip=1.1.1.1&name=Cloudflare", "https://dns.quad9.net/dns-query?ip=9.9.9.9&name=Quad9&blockedif=empty", "https://dns.adguard-dns.com/dns-query?ip=94.140.14.14&name=AdGuard&blockedif=zeroip", "https://dns.google/dns-query?ip=8.8.8.8&name=Google"] },
                                        { label: pm.cfgText("qs.dns.cloudflareMalwareDot"), vals: ["dot://cloudflare-dns.com?ip=1.1.1.2&name=Cloudflare&blockedif=zeroip", "dot://cloudflare-dns.com?ip=1.0.0.2&name=Cloudflare&blockedif=zeroip"] },
                                        { label: pm.cfgText("qs.dns.cloudflareMalwareDoh"), vals: ["https://security.cloudflare-dns.com/dns-query?ip=1.1.1.2&name=Cloudflare&blockedif=zeroip", "https://security.cloudflare-dns.com/dns-query?ip=1.0.0.2&name=Cloudflare&blockedif=zeroip"] },
                                        { label: pm.cfgText("qs.dns.quad9Dot"), vals: ["dot://dns.quad9.net?ip=9.9.9.9&name=Quad9&blockedif=empty", "dot://dns.quad9.net?ip=149.112.112.112&name=Quad9&blockedif=empty"] },
                                        { label: pm.cfgText("qs.dns.quad9Doh"), vals: ["https://dns.quad9.net/dns-query?ip=9.9.9.9&name=Quad9&blockedif=empty", "https://dns.quad9.net/dns-query?ip=149.112.112.112&name=Quad9&blockedif=empty"] },
                                        { label: pm.cfgText("qs.dns.adguardDot"), vals: ["dot://dns.adguard-dns.com?ip=94.140.14.14&name=AdGuard&blockedif=zeroip", "dot://dns.adguard-dns.com?ip=94.140.15.15&name=AdGuard&blockedif=zeroip"] },
                                        { label: pm.cfgText("qs.dns.adguardDoh"), vals: ["https://dns.adguard-dns.com/dns-query?ip=94.140.14.14&name=AdGuard&blockedif=zeroip", "https://dns.adguard-dns.com/dns-query?ip=94.140.15.15&name=AdGuard&blockedif=zeroip"] },
                                        { label: pm.cfgText("qs.dns.mullvadDot"), vals: ["dot://dns.mullvad.net?ip=194.242.2.3&name=Mullvad", "dot://dns.mullvad.net?ip=194.242.2.4&name=Mullvad"] },
                                        { label: pm.cfgText("qs.dns.mullvadDoh"), vals: ["https://dns.mullvad.net/dns-query?ip=194.242.2.3&name=Mullvad", "https://dns.mullvad.net/dns-query?ip=194.242.2.4&name=Mullvad"] },
                                        { label: pm.cfgText("qs.dns.controldDoh"), vals: ["https://freedns.controld.com/p2?ip=76.76.2.2&name=ControlD", "https://freedns.controld.com/p2?ip=76.76.10.2&name=ControlD"] },
                                        { label: pm.cfgText("qs.dns.appliedPrivacyDot"), vals: ["dot://dot1.applied-privacy.net?ip=146.255.56.98&name=AppliedPrivacy"] },
                                        { label: pm.cfgText("qs.dns.googleDoh"), vals: ["https://dns.google/dns-query?ip=8.8.8.8&name=Google", "https://dns.google/dns-query?ip=8.8.4.4&name=Google"] },
                                        { label: pm.cfgText("qs.dns.yandexSafeDoh"), vals: ["https://safe.dns.yandex.net/dns-query?ip=77.88.8.88&name=YandexSafe", "https://safe.dns.yandex.net/dns-query?ip=77.88.8.2&name=YandexSafe"] },
                                        { label: pm.cfgText("qs.dns.cloudflareFallback"), vals: ["dot://cloudflare-dns.com?ip=1.1.1.1&name=Cloudflare&blockedif=zeroip", "dot://cloudflare-dns.com?ip=1.0.0.1&name=Cloudflare&blockedif=zeroip"], forceAppend: true }
                                    ]
                                    delegate: Rectangle {
                                        height: 24; radius: 4; color: "#212530"; border.color: "#364054"
                                        implicitWidth: dPLbl.implicitWidth + 14
                                        Label { id: dPLbl; anchors.centerIn: parent; text: modelData.label; color: "#89b7ff"; font.pixelSize: 10; font.bold: true }
                                        MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: pm.applyPreset("dns/nameservers", modelData.vals, modelData.forceAppend ? true : root.appendDns) }
                                    }
                                }
                                Rectangle {
                                    height: 24; radius: 4; color: "#232328"; border.color: "#36363e"
                                    implicitWidth: dRstLbl.implicitWidth + 14
                                    Label { id: dRstLbl; anchors.centerIn: parent; text: pm.isRu ? "↺ По умолчанию" : "↺ Default DNS"; color: "#cacaca"; font.pixelSize: 10 }
                                    MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: pm.resetSetting("dns/nameservers") }
                                }
                            }

                            Flow {
                                Layout.fillWidth: true
                                spacing: 14
                                RowLayout {
                                    spacing: 8
                                    PmSwitch { checked: pm.dnsNoInsecure; onToggled: pm.updateSetting("dns/noInsecureProtocols", checked) }
                                    Label { text: pm.isRu ? "Только защищенный DNS (DoT/DoH)" : "Secure DNS Only (DoT/DoH)"; color: "#dcdce2"; font.pixelSize: 11 }
                                }
                                RowLayout {
                                    spacing: 8
                                    PmSwitch { checked: pm.dnsIgnoreSystem; onToggled: pm.updateSetting("dns/noAssignedNameservers", checked) }
                                    Label { text: pm.isRu ? "Игнорировать системный DNS" : "Ignore System DNS"; color: "#dcdce2"; font.pixelSize: 11 }
                                }
                                RowLayout {
                                    spacing: 8
                                    PmSwitch { checked: pm.dnsUseStaleCache; onToggled: pm.updateSetting("dns/useStaleCache", checked) }
                                    Label { text: pm.isRu ? "Всегда использовать кэш DNS" : "Always Use DNS Cache"; color: "#dcdce2"; font.pixelSize: 11 }
                                }
                            }
                        }
                    }

                    Item { height: 2 }
                }

                delegate: SettingCard {
                    width: ListView.view.width - 10
                    st: modelData
                    isPerApp: false
                    appendDns: root.appendDns
                    onAppendDnsToggled: function(checked) { root.appendDns = checked }
                    onRequestRuleBuilder: function(key, mode) { root.openRuleBuilder(key, mode) }
                }
            }
        }
    }
}
