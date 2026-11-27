import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Rectangle {
    id: modal
    anchors.fill: parent
    color: "#cc09090b"
    z: 950
    visible: !!pm.editingNode && (pm.editingNode.id || "") !== ""

    signal openRuleBuilder()

    property string nodeId: ""
    property string nameVal: ""
    property string protoVal: "vless"
    property string addrVal: ""
    property string portVal: "443"
    property string credVal: ""
    property string secVal: "tls"
    property string netVal: "raw"
    property string sniVal: ""
    property string hostVal: ""
    property string pathVal: ""
    property string flowVal: ""
    property string pbkVal: ""
    property string sidVal: ""
    property string presetVal: "default"
    property var rulesList: []
    property string newRuleInput: ""

    Connections {
        target: pm
        function onEditingNodeChanged() {
            var n = pm.editingNode || ({})
            modal.nodeId = n.id || ""
            modal.nameVal = n.name || ""
            modal.protoVal = n.protocol || "vless"
            modal.addrVal = n.address || ""
            modal.portVal = String(n.port || 443)
            modal.credVal = ""
            modal.secVal = n.security || "none"
            modal.netVal = n.network || "raw"
            modal.sniVal = n.sni || ""
            modal.hostVal = n.host || ""
            modal.pathVal = n.path || ""
            modal.flowVal = n.flow || ""
            modal.pbkVal = n.realityPublicKey || ""
            modal.sidVal = n.realityShortId || ""
            modal.presetVal = (n.rulePreset && n.rulePreset !== "") ? n.rulePreset : "default"
            modal.rulesList = (n.routingRules || []).slice(0)
        }
    }

    function appendNodeRule(rule) {
        var r = (rule || "").trim()
        if (r === "") return
        var copy = (rulesList || []).slice(0)
        if (copy.indexOf(r) < 0) copy.push(r)
        rulesList = copy
    }

    MouseArea { anchors.fill: parent; onClicked: pm.clearEditingNode() }

    Rectangle {
        anchors.centerIn: parent
        width: Math.min(parent.width - 50, 740)
        height: Math.min(parent.height - 50, 680)
        radius: 8
        color: "#18191e"
        border.color: "#4e97fa"

        MouseArea { anchors.fill: parent; onClicked: function(m) { m.accepted = true } }

        ColumnLayout {
            anchors.fill: parent
            anchors.margins: 18
            spacing: 12

            RowLayout {
                Layout.fillWidth: true
                Label {
                    text: (pm.isRu ? "Редактор узла Xray: " : "Xray Node Editor: ") + modal.nameVal
                    color: "#ffffff"; font.pixelSize: 16; font.bold: true; elide: Text.ElideRight; Layout.fillWidth: true
                }
                Rectangle {
                    height: 26; radius: 4; color: "#3d1414"; border.color: "#7a2424"
                    implicitWidth: delLbl.implicitWidth + 14
                    Label { id: delLbl; anchors.centerIn: parent; text: pm.isRu ? "Удалить узел" : "Delete Node"; color: "#ff8888"; font.pixelSize: 11 }
                    MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: pm.deleteNode(modal.nodeId) }
                }
                Rectangle {
                    width: 26; height: 26; radius: 4; color: "#25262e"
                    Label { anchors.centerIn: parent; text: "✕"; color: "#cacaca"; font.pixelSize: 12 }
                    MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: pm.clearEditingNode() }
                }
            }

            Flickable {
                Layout.fillWidth: true
                Layout.fillHeight: true
                clip: true
                contentHeight: formCol.implicitHeight + 12
                ScrollBar.vertical: PmScrollBar {}
                PmWheelHelper {}

                ColumnLayout {
                    id: formCol
                    width: parent.width - 10
                    spacing: 10

                    GridLayout {
                        Layout.fillWidth: true
                        columns: 2
                        columnSpacing: 12
                        rowSpacing: 8

                        ColumnLayout {
                            Layout.fillWidth: true; spacing: 3
                            Label { text: pm.isRu ? "Название" : "Name"; color: "#9e9ea6"; font.pixelSize: 10 }
                            TextField {
                                Layout.fillWidth: true; text: modal.nameVal; onTextChanged: modal.nameVal = text
                                color: "#ffffff"; font.pixelSize: 12
                                background: Rectangle { color: "#121214"; radius: 4; border.color: "#2c2e38" }
                            }
                        }

                        ColumnLayout {
                            Layout.fillWidth: true; spacing: 3
                            Label { text: pm.isRu ? "Протокол" : "Protocol"; color: "#9e9ea6"; font.pixelSize: 10 }
                            Row {
                                spacing: 4
                                Repeater {
                                    model: ["vless", "trojan", "vmess", "shadowsocks", "hysteria", "wireguard"]
                                    delegate: Rectangle {
                                        height: 28; radius: 4; implicitWidth: pL.implicitWidth + 10
                                        color: modal.protoVal === modelData ? "#1c3354" : "#222329"
                                        border.color: modal.protoVal === modelData ? "#4e97fa" : "#343640"
                                        Label { id: pL; anchors.centerIn: parent; text: modelData.toUpperCase(); color: "#ffffff"; font.pixelSize: 9; font.bold: modal.protoVal === modelData }
                                        MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: modal.protoVal = modelData }
                                    }
                                }
                            }
                        }

                        ColumnLayout {
                            Layout.fillWidth: true; spacing: 3
                            Label { text: pm.isRu ? "Адрес / Сервер" : "Server Address"; color: "#9e9ea6"; font.pixelSize: 10 }
                            TextField {
                                Layout.fillWidth: true; text: modal.addrVal; onTextChanged: modal.addrVal = text
                                color: "#ffffff"; font.pixelSize: 12; font.family: "monospace"
                                background: Rectangle { color: "#121214"; radius: 4; border.color: "#2c2e38" }
                            }
                        }

                        ColumnLayout {
                            Layout.fillWidth: true; spacing: 3
                            Label { text: pm.isRu ? "Порт" : "Port"; color: "#9e9ea6"; font.pixelSize: 10 }
                            TextField {
                                Layout.fillWidth: true; text: modal.portVal; onTextChanged: modal.portVal = text
                                color: "#ffffff"; font.pixelSize: 12; font.family: "monospace"
                                background: Rectangle { color: "#121214"; radius: 4; border.color: "#2c2e38" }
                            }
                        }
                    }

                    ColumnLayout {
                        Layout.fillWidth: true; spacing: 3
                        Label {
                            text: pm.isRu ? "Новый UUID / Пароль (оставьте пустым, чтобы сохранить текущий)" : "New UUID / Password (leave blank to keep current)"
                            color: "#9e9ea6"; font.pixelSize: 10
                        }
                        TextField {
                            Layout.fillWidth: true; text: modal.credVal; onTextChanged: modal.credVal = text
                            echoMode: TextInput.Password
                            placeholderText: !!pm.editingNode.hasCredential ? "••••••••••••" : ""
                            placeholderTextColor: "#666670"; color: "#ffffff"; font.pixelSize: 12; font.family: "monospace"
                            background: Rectangle { color: "#121214"; radius: 4; border.color: "#2c2e38" }
                        }
                    }

                    GridLayout {
                        Layout.fillWidth: true
                        columns: 3
                        columnSpacing: 10
                        rowSpacing: 8

                        ColumnLayout {
                            Layout.fillWidth: true; spacing: 3
                            Label { text: pm.isRu ? "Безопасность (TLS)" : "Security"; color: "#9e9ea6"; font.pixelSize: 10 }
                            Row {
                                spacing: 4
                                Repeater {
                                    model: ["none", "tls", "reality"]
                                    delegate: Rectangle {
                                        height: 26; radius: 4; implicitWidth: sL.implicitWidth + 12
                                        color: modal.secVal === modelData ? "#1c3354" : "#222329"
                                        border.color: modal.secVal === modelData ? "#4e97fa" : "#343640"
                                        Label { id: sL; anchors.centerIn: parent; text: modelData; color: "#ffffff"; font.pixelSize: 10 }
                                        MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: modal.secVal = modelData }
                                    }
                                }
                            }
                        }

                        ColumnLayout {
                            Layout.fillWidth: true; spacing: 3
                            Label { text: pm.isRu ? "Транспорт" : "Network"; color: "#9e9ea6"; font.pixelSize: 10 }
                            Row {
                                spacing: 4
                                Repeater {
                                    model: ["raw", "ws", "grpc", "httpupgrade"]
                                    delegate: Rectangle {
                                        height: 26; radius: 4; implicitWidth: nL.implicitWidth + 10
                                        color: modal.netVal === modelData ? "#1c3354" : "#222329"
                                        border.color: modal.netVal === modelData ? "#4e97fa" : "#343640"
                                        Label { id: nL; anchors.centerIn: parent; text: modelData; color: "#ffffff"; font.pixelSize: 10 }
                                        MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: modal.netVal = modelData }
                                    }
                                }
                            }
                        }

                        ColumnLayout {
                            Layout.fillWidth: true; spacing: 3
                            Label { text: "Flow"; color: "#9e9ea6"; font.pixelSize: 10 }
                            TextField {
                                Layout.fillWidth: true; text: modal.flowVal; onTextChanged: modal.flowVal = text
                                placeholderText: "xtls-rprx-vision"
                                placeholderTextColor: "#60606a"; color: "#ffffff"; font.pixelSize: 11; font.family: "monospace"
                                background: Rectangle { color: "#121214"; radius: 4; border.color: "#2c2e38" }
                            }
                        }

                        ColumnLayout {
                            Layout.fillWidth: true; spacing: 3
                            Label { text: "SNI"; color: "#9e9ea6"; font.pixelSize: 10 }
                            TextField {
                                Layout.fillWidth: true; text: modal.sniVal; onTextChanged: modal.sniVal = text
                                color: "#ffffff"; font.pixelSize: 11; font.family: "monospace"
                                background: Rectangle { color: "#121214"; radius: 4; border.color: "#2c2e38" }
                            }
                        }

                        ColumnLayout {
                            Layout.fillWidth: true; spacing: 3
                            Label { text: "Host"; color: "#9e9ea6"; font.pixelSize: 10 }
                            TextField {
                                Layout.fillWidth: true; text: modal.hostVal; onTextChanged: modal.hostVal = text
                                color: "#ffffff"; font.pixelSize: 11; font.family: "monospace"
                                background: Rectangle { color: "#121214"; radius: 4; border.color: "#2c2e38" }
                            }
                        }

                        ColumnLayout {
                            Layout.fillWidth: true; spacing: 3
                            Label { text: "Path"; color: "#9e9ea6"; font.pixelSize: 10 }
                            TextField {
                                Layout.fillWidth: true; text: modal.pathVal; onTextChanged: modal.pathVal = text
                                color: "#ffffff"; font.pixelSize: 11; font.family: "monospace"
                                background: Rectangle { color: "#121214"; radius: 4; border.color: "#2c2e38" }
                            }
                        }
                    }

                    RowLayout {
                        visible: modal.secVal === "reality"
                        Layout.fillWidth: true; spacing: 10
                        ColumnLayout {
                            Layout.fillWidth: true; spacing: 3
                            Label { text: "Reality Public Key (pbk)"; color: "#9e9ea6"; font.pixelSize: 10 }
                            TextField {
                                Layout.fillWidth: true; text: modal.pbkVal; onTextChanged: modal.pbkVal = text
                                color: "#ffffff"; font.pixelSize: 11; font.family: "monospace"
                                background: Rectangle { color: "#121214"; radius: 4; border.color: "#2c2e38" }
                            }
                        }
                        ColumnLayout {
                            Layout.preferredWidth: 160; spacing: 3
                            Label { text: "Short ID (sid)"; color: "#9e9ea6"; font.pixelSize: 10 }
                            TextField {
                                Layout.fillWidth: true; text: modal.sidVal; onTextChanged: modal.sidVal = text
                                color: "#ffffff"; font.pixelSize: 11; font.family: "monospace"
                                background: Rectangle { color: "#121214"; radius: 4; border.color: "#2c2e38" }
                            }
                        }
                    }

                    // Per-Node Routing Preset + Per-Node Rules
                    Rectangle {
                        Layout.fillWidth: true; height: 1; color: "#272832"
                    }
                    Label {
                        text: pm.isRu ? "Пресет маршрутизации для этого узла" : "Per-Node Routing Preset"
                        color: "#ffffff"; font.pixelSize: 12; font.bold: true
                    }
                    Flow {
                        Layout.fillWidth: true; spacing: 6
                        Repeater {
                            model: pm.geodataHints.presets || [
                                { id: "default", name: pm.isRu ? "По умолчанию (глобальные)" : "Default (Global Rules)" },
                                { id: "ru-direct", name: pm.isRu ? "РФ напрямую + YouTube/AI в VPN" : "RU Direct + Streaming VPN" },
                                { id: "bypass-ads", name: pm.isRu ? "Без рекламы (AdBlock)" : "AdBlock Preset" },
                                { id: "global-vpn", name: pm.isRu ? "Весь трафик через этот узел" : "All Internet via Node" }
                            ]
                            delegate: Rectangle {
                                height: 26; radius: 4; implicitWidth: prL.implicitWidth + 14
                                color: modal.presetVal === modelData.id ? "#183324" : "#222329"
                                border.color: modal.presetVal === modelData.id ? "#1de966" : "#343640"
                                Label { id: prL; anchors.centerIn: parent; text: modelData.name; color: "#ffffff"; font.pixelSize: 10 }
                                MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: modal.presetVal = modelData.id }
                            }
                        }
                    }

                    RowLayout {
                        Layout.fillWidth: true; spacing: 8
                        Label { text: pm.isRu ? "Персональные правила узла:" : "Per-Node Rules:"; color: "#9e9ea6"; font.pixelSize: 11; Layout.fillWidth: true }
                        Rectangle {
                            height: 26; radius: 4; color: "#1c283a"; border.color: "#4e97fa"
                            implicitWidth: rbBtnL.implicitWidth + 14
                            Label { id: rbBtnL; anchors.centerIn: parent; text: pm.isRu ? "⚙ Конструктор" : "⚙ Rule Builder"; color: "#89b7ff"; font.pixelSize: 10; font.bold: true }
                            MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: modal.openRuleBuilder() }
                        }
                    }

                    RowLayout {
                        Layout.fillWidth: true; spacing: 6
                        TextField {
                            Layout.fillWidth: true; height: 30
                            text: modal.newRuleInput; onTextChanged: modal.newRuleInput = text
                            placeholderText: "- geoip:ru  /  + geosite:youtube  /  + example.com"
                            placeholderTextColor: "#60606a"; color: "#ffffff"; font.pixelSize: 11; font.family: "monospace"
                            background: Rectangle { color: "#121214"; radius: 4; border.color: "#2c2e38" }
                            onAccepted: { modal.appendNodeRule(modal.newRuleInput); modal.newRuleInput = "" }
                        }
                        Rectangle {
                            height: 30; radius: 4; color: "#4e97fa"; implicitWidth: addNRL.implicitWidth + 16
                            Label { id: addNRL; anchors.centerIn: parent; text: "+"; color: "#ffffff"; font.pixelSize: 13; font.bold: true }
                            MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: { modal.appendNodeRule(modal.newRuleInput); modal.newRuleInput = "" } }
                        }
                    }

                    Flow {
                        Layout.fillWidth: true; spacing: 5
                        Repeater {
                            model: modal.rulesList
                            delegate: Rectangle {
                                height: 24; radius: 4
                                color: modelData.trim().startsWith("+") ? "#122b1d" : "#2e1517"
                                border.color: modelData.trim().startsWith("+") ? "#1de966" : "#e01d1d"
                                implicitWidth: nrRow.implicitWidth + 14
                                RowLayout {
                                    id: nrRow; anchors.centerIn: parent; spacing: 6
                                    Label { text: modelData; color: "#ffffff"; font.pixelSize: 10; font.family: "monospace" }
                                    Label {
                                        text: "✕"; color: "#ff8888"; font.pixelSize: 10
                                        MouseArea {
                                            anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                                            onClicked: {
                                                var next = modal.rulesList.slice(0)
                                                next.splice(index, 1)
                                                modal.rulesList = next
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            RowLayout {
                Layout.fillWidth: true; spacing: 10
                Item { Layout.fillWidth: true }
                Rectangle {
                    height: 32; radius: 4; color: "#25262e"; implicitWidth: cnLbl.implicitWidth + 20
                    Label { id: cnLbl; anchors.centerIn: parent; text: pm.isRu ? "Отмена" : "Cancel"; color: "#dcdce2"; font.pixelSize: 12 }
                    MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: pm.clearEditingNode() }
                }
                Rectangle {
                    height: 32; radius: 4; color: "#4e97fa"; implicitWidth: svLbl.implicitWidth + 24
                    Label { id: svLbl; anchors.centerIn: parent; text: pm.isRu ? "Сохранить изменения" : "Save Node"; color: "#ffffff"; font.pixelSize: 12; font.bold: true }
                    MouseArea {
                        anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                        onClicked: {
                            var p = parseInt(modal.portVal) || 443
                            var body = {
                                "name": modal.nameVal,
                                "protocol": modal.protoVal,
                                "address": modal.addrVal,
                                "port": p,
                                "security": modal.secVal,
                                "network": modal.netVal,
                                "sni": modal.sniVal,
                                "host": modal.hostVal,
                                "path": modal.pathVal,
                                "flow": modal.flowVal,
                                "realityPublicKey": modal.pbkVal,
                                "realityShortId": modal.sidVal,
                                "rulePreset": modal.presetVal,
                                "routingRules": modal.rulesList
                            }
                            if (modal.credVal.trim() !== "") {
                                body["credential"] = modal.credVal.trim()
                                body["password"] = modal.credVal.trim()
                            }
                            pm.saveNodeEdit(modal.nodeId, body)
                        }
                    }
                }
            }
        }
    }
}
