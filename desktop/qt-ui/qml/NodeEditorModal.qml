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
    property string shareLinkVal: ""
    property string encVal: "none"
    property string secVal: "tls"
    property string netVal: "raw"
    property string sniVal: ""
    property string hostVal: ""
    property string pathVal: ""
    property string flowVal: ""
    property string svcVal: ""
    property string hdrTypeVal: "none"
    property string xhttpModeVal: "auto"
    property string xhttpExtraVal: ""
    property bool grpcMultiVal: false
    property string grpcAuthVal: ""
    property string kcpSeedVal: ""
    property string wsHeartbeatVal: "0"
    property string fpVal: ""
    property string alpnVal: ""
    property string pbkVal: ""
    property string sidVal: ""
    property string spxVal: ""
    property string pcsVal: ""
    property string vcnVal: ""
    property string echVal: ""
    property bool muxEnabledVal: false
    property string muxConcVal: "8"
    property bool fragmentVal: false
    property bool tfoVal: false
    property string keepAliveVal: "0"
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
            modal.shareLinkVal = ""
            modal.encVal = (n.encryption && n.encryption !== "") ? n.encryption : "none"
            modal.secVal = n.security || "none"
            modal.netVal = n.network || "raw"
            modal.sniVal = n.sni || ""
            modal.hostVal = n.host || ""
            modal.pathVal = n.path || ""
            modal.flowVal = n.flow || ""
            modal.svcVal = n.serviceName || ""
            modal.hdrTypeVal = (n.headerType && n.headerType !== "") ? n.headerType : "none"
            modal.xhttpModeVal = (n.xhttpMode && n.xhttpMode !== "") ? n.xhttpMode : "auto"
            modal.xhttpExtraVal = n.xhttpExtra || ""
            modal.grpcMultiVal = !!n.grpcMultiMode
            modal.grpcAuthVal = n.grpcAuthority || ""
            modal.kcpSeedVal = n.kcpSeed || ""
            modal.wsHeartbeatVal = String(n.wsHeartbeatPeriod || 0)
            modal.fpVal = n.fingerprint || ""
            modal.alpnVal = (n.alpn && n.alpn.length > 0) ? n.alpn.join(",") : ""
            modal.pbkVal = n.realityPublicKey || ""
            modal.sidVal = n.realityShortId || ""
            modal.spxVal = n.realitySpiderX || ""
            modal.pcsVal = n.pinnedPeerCert || ""
            modal.vcnVal = n.verifyPeerCertByName || ""
            modal.echVal = n.echConfigList || ""
            modal.muxEnabledVal = !!n.muxEnabled
            modal.muxConcVal = String((n.muxConcurrency !== undefined && n.muxConcurrency !== 0) ? n.muxConcurrency : 8)
            modal.fragmentVal = !!n.fragmentEnabled
            modal.tfoVal = !!n.tcpFastOpen
            modal.keepAliveVal = String(n.tcpKeepAliveInterval || 0)
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
        width: Math.min(parent.width - 40, 820)
        height: Math.min(parent.height - 40, 720)
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

                    ColumnLayout {
                        Layout.fillWidth: true; spacing: 3
                        Label {
                            text: pm.isRu ? "Заменить ссылкой целиком (опционально: vless:// vmess:// trojan:// ss:// hy2:// wg://)" : "Replace from Share Link (optional URI)"
                            color: "#9e9ea6"; font.pixelSize: 10
                        }
                        TextField {
                            Layout.fillWidth: true; text: modal.shareLinkVal; onTextChanged: modal.shareLinkVal = text
                            placeholderText: pm.isRu ? "Вставьте новую URI-ссылку, если нужно заменить весь конфиг..." : "Paste URI if replacing entire node config..."
                            placeholderTextColor: "#545662"; color: "#89b7ff"; font.pixelSize: 11; font.family: "monospace"
                            background: Rectangle { color: "#121214"; radius: 4; border.color: "#2c2e38" }
                        }
                    }

                    // Transport & Security Selectors
                    Rectangle { Layout.fillWidth: true; height: 1; color: "#272832" }
                    Label {
                        text: pm.isRu ? "Транспорт и протокол V2Ray / Xray" : "V2Ray / Xray Transport & Protocol"
                        color: "#ffffff"; font.pixelSize: 12; font.bold: true
                    }

                    GridLayout {
                        Layout.fillWidth: true
                        columns: 2
                        columnSpacing: 12
                        rowSpacing: 8

                        ColumnLayout {
                            Layout.fillWidth: true; spacing: 3
                            Label { text: pm.isRu ? "Безопасность (TLS)" : "Security"; color: "#9e9ea6"; font.pixelSize: 10 }
                            Flow {
                                Layout.fillWidth: true; spacing: 4
                                Repeater {
                                    model: ["none", "tls", "reality"]
                                    delegate: Rectangle {
                                        height: 26; radius: 4; implicitWidth: sL.implicitWidth + 12
                                        color: modal.secVal === modelData ? "#1c3354" : "#222329"
                                        border.color: modal.secVal === modelData ? "#4e97fa" : "#343640"
                                        Label { id: sL; anchors.centerIn: parent; text: modelData.toUpperCase(); color: "#ffffff"; font.pixelSize: 10; font.bold: modal.secVal === modelData }
                                        MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: modal.secVal = modelData }
                                    }
                                }
                            }
                        }

                        ColumnLayout {
                            Layout.fillWidth: true; spacing: 3
                            Label { text: pm.isRu ? "Транспорт (Network)" : "Network"; color: "#9e9ea6"; font.pixelSize: 10 }
                            Flow {
                                Layout.fillWidth: true; spacing: 4
                                Repeater {
                                    model: ["raw", "ws", "grpc", "httpupgrade", "xhttp", "kcp"]
                                    delegate: Rectangle {
                                        height: 26; radius: 4; implicitWidth: nL.implicitWidth + 10
                                        color: modal.netVal === modelData ? "#1c3354" : "#222329"
                                        border.color: modal.netVal === modelData ? "#4e97fa" : "#343640"
                                        Label { id: nL; anchors.centerIn: parent; text: modelData.toUpperCase(); color: "#ffffff"; font.pixelSize: 10; font.bold: modal.netVal === modelData }
                                        MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: modal.netVal = modelData }
                                    }
                                }
                            }
                        }

                        ColumnLayout {
                            visible: modal.protoVal === "vless" || modal.flowVal !== ""
                            Layout.fillWidth: true; spacing: 3
                            Label { text: "Flow (XTLS Vision)"; color: "#9e9ea6"; font.pixelSize: 10 }
                            Flow {
                                Layout.fillWidth: true; spacing: 4
                                Repeater {
                                    model: [
                                        { id: "", label: pm.isRu ? "Нет" : "None" },
                                        { id: "xtls-rprx-vision", label: "vision" },
                                        { id: "xtls-rprx-vision-udp443", label: "vision-udp443" }
                                    ]
                                    delegate: Rectangle {
                                        height: 24; radius: 4; implicitWidth: flL.implicitWidth + 10
                                        color: modal.flowVal === modelData.id ? "#1c3354" : "#222329"
                                        border.color: modal.flowVal === modelData.id ? "#4e97fa" : "#343640"
                                        Label { id: flL; anchors.centerIn: parent; text: modelData.label; color: "#ffffff"; font.pixelSize: 10 }
                                        MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: modal.flowVal = modelData.id }
                                    }
                                }
                            }
                        }

                        ColumnLayout {
                            visible: modal.protoVal === "vless"
                            Layout.fillWidth: true; spacing: 3
                            Label { text: pm.isRu ? "Шифрование VLESS (encryption)" : "VLESS Encryption"; color: "#9e9ea6"; font.pixelSize: 10 }
                            TextField {
                                Layout.fillWidth: true; text: modal.encVal; onTextChanged: modal.encVal = text
                                placeholderText: "none"
                                placeholderTextColor: "#60606a"; color: "#ffffff"; font.pixelSize: 11; font.family: "monospace"
                                background: Rectangle { color: "#121214"; radius: 4; border.color: "#2c2e38" }
                            }
                        }

                        ColumnLayout {
                            visible: modal.netVal === "ws" || modal.netVal === "httpupgrade" || modal.netVal === "xhttp" || modal.netVal === "raw"
                            Layout.fillWidth: true; spacing: 3
                            Label { text: "Host / HTTP Header"; color: "#9e9ea6"; font.pixelSize: 10 }
                            TextField {
                                Layout.fillWidth: true; text: modal.hostVal; onTextChanged: modal.hostVal = text
                                color: "#ffffff"; font.pixelSize: 11; font.family: "monospace"
                                background: Rectangle { color: "#121214"; radius: 4; border.color: "#2c2e38" }
                            }
                        }

                        ColumnLayout {
                            visible: modal.netVal === "ws" || modal.netVal === "httpupgrade" || modal.netVal === "xhttp" || modal.netVal === "raw"
                            Layout.fillWidth: true; spacing: 3
                            Label { text: "Path"; color: "#9e9ea6"; font.pixelSize: 10 }
                            TextField {
                                Layout.fillWidth: true; text: modal.pathVal; onTextChanged: modal.pathVal = text
                                placeholderText: "/"
                                placeholderTextColor: "#60606a"; color: "#ffffff"; font.pixelSize: 11; font.family: "monospace"
                                background: Rectangle { color: "#121214"; radius: 4; border.color: "#2c2e38" }
                            }
                        }
                    }

                    // Transport-Specific Options (XHTTP / gRPC / KCP / RAW / WebSocket)
                    RowLayout {
                        visible: modal.netVal === "raw"
                        Layout.fillWidth: true; spacing: 10
                        Label { text: "RAW Header:"; color: "#9e9ea6"; font.pixelSize: 10 }
                        Row {
                            spacing: 4
                            Repeater {
                                model: ["none", "http"]
                                delegate: Rectangle {
                                    height: 24; radius: 4; implicitWidth: rhL.implicitWidth + 12
                                    color: modal.hdrTypeVal === modelData ? "#1c3354" : "#222329"
                                    border.color: modal.hdrTypeVal === modelData ? "#4e97fa" : "#343640"
                                    Label { id: rhL; anchors.centerIn: parent; text: modelData; color: "#ffffff"; font.pixelSize: 10 }
                                    MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: modal.hdrTypeVal = modelData }
                                }
                            }
                        }
                    }

                    ColumnLayout {
                        visible: modal.netVal === "xhttp"
                        Layout.fillWidth: true; spacing: 6
                        RowLayout {
                            Layout.fillWidth: true; spacing: 10
                            Label { text: "XHTTP Mode:"; color: "#9e9ea6"; font.pixelSize: 10 }
                            Flow {
                                Layout.fillWidth: true; spacing: 4
                                Repeater {
                                    model: ["auto", "packet-up", "stream-up", "stream-one"]
                                    delegate: Rectangle {
                                        height: 24; radius: 4; implicitWidth: xhmL.implicitWidth + 10
                                        color: modal.xhttpModeVal === modelData ? "#1c3354" : "#222329"
                                        border.color: modal.xhttpModeVal === modelData ? "#4e97fa" : "#343640"
                                        Label { id: xhmL; anchors.centerIn: parent; text: modelData; color: "#ffffff"; font.pixelSize: 10 }
                                        MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: modal.xhttpModeVal = modelData }
                                    }
                                }
                            }
                        }
                        ColumnLayout {
                            Layout.fillWidth: true; spacing: 3
                            Label { text: pm.isRu ? "XHTTP Extra JSON (xmux, scMaxEachPostBytes, noSSEHeader)" : "XHTTP Extra JSON"; color: "#9e9ea6"; font.pixelSize: 10 }
                            TextField {
                                Layout.fillWidth: true; text: modal.xhttpExtraVal; onTextChanged: modal.xhttpExtraVal = text
                                placeholderText: "{\"noSSEHeader\":false,\"xmux\":{\"maxConcurrency\":\"16-32\"}}"
                                placeholderTextColor: "#545662"; color: "#ffffff"; font.pixelSize: 11; font.family: "monospace"
                                background: Rectangle { color: "#121214"; radius: 4; border.color: "#2c2e38" }
                            }
                        }
                    }

                    GridLayout {
                        visible: modal.netVal === "grpc"
                        Layout.fillWidth: true; columns: 3; columnSpacing: 10; rowSpacing: 6
                        ColumnLayout {
                            Layout.fillWidth: true; spacing: 3
                            Label { text: "gRPC Service Name"; color: "#9e9ea6"; font.pixelSize: 10 }
                            TextField {
                                Layout.fillWidth: true; text: modal.svcVal; onTextChanged: modal.svcVal = text
                                color: "#ffffff"; font.pixelSize: 11; font.family: "monospace"
                                background: Rectangle { color: "#121214"; radius: 4; border.color: "#2c2e38" }
                            }
                        }
                        ColumnLayout {
                            Layout.fillWidth: true; spacing: 3
                            Label { text: "gRPC Authority"; color: "#9e9ea6"; font.pixelSize: 10 }
                            TextField {
                                Layout.fillWidth: true; text: modal.grpcAuthVal; onTextChanged: modal.grpcAuthVal = text
                                color: "#ffffff"; font.pixelSize: 11; font.family: "monospace"
                                background: Rectangle { color: "#121214"; radius: 4; border.color: "#2c2e38" }
                            }
                        }
                        ColumnLayout {
                            spacing: 4
                            Label { text: "gRPC MultiMode"; color: "#9e9ea6"; font.pixelSize: 10 }
                            PmSwitch { checked: modal.grpcMultiVal; onToggled: modal.grpcMultiVal = checked }
                        }
                    }

                    ColumnLayout {
                        visible: modal.netVal === "kcp"
                        Layout.fillWidth: true; spacing: 6
                        RowLayout {
                            Layout.fillWidth: true; spacing: 8
                            Label { text: "mKCP Obfuscation:"; color: "#9e9ea6"; font.pixelSize: 10 }
                            Flow {
                                Layout.fillWidth: true; spacing: 4
                                Repeater {
                                    model: ["none", "srtp", "utp", "wechat-video", "dtls", "wireguard"]
                                    delegate: Rectangle {
                                        height: 24; radius: 4; implicitWidth: khL.implicitWidth + 10
                                        color: modal.hdrTypeVal === modelData ? "#1c3354" : "#222329"
                                        border.color: modal.hdrTypeVal === modelData ? "#4e97fa" : "#343640"
                                        Label { id: khL; anchors.centerIn: parent; text: modelData; color: "#ffffff"; font.pixelSize: 10 }
                                        MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: modal.hdrTypeVal = modelData }
                                    }
                                }
                            }
                        }
                        ColumnLayout {
                            Layout.fillWidth: true; spacing: 3
                            Label { text: "mKCP Seed"; color: "#9e9ea6"; font.pixelSize: 10 }
                            TextField {
                                Layout.fillWidth: true; text: modal.kcpSeedVal; onTextChanged: modal.kcpSeedVal = text
                                color: "#ffffff"; font.pixelSize: 11; font.family: "monospace"
                                background: Rectangle { color: "#121214"; radius: 4; border.color: "#2c2e38" }
                            }
                        }
                    }

                    RowLayout {
                        visible: modal.netVal === "ws"
                        Layout.fillWidth: true; spacing: 10
                        Label { text: pm.isRu ? "WebSocket Heartbeat (сек, 0=выкл):" : "WS Heartbeat Period (sec):"; color: "#9e9ea6"; font.pixelSize: 10 }
                        TextField {
                            Layout.preferredWidth: 100; text: modal.wsHeartbeatVal; onTextChanged: modal.wsHeartbeatVal = text
                            color: "#ffffff"; font.pixelSize: 11; font.family: "monospace"
                            background: Rectangle { color: "#121214"; radius: 4; border.color: "#2c2e38" }
                        }
                    }

                    // TLS / REALITY Security Parameters
                    Rectangle { visible: modal.secVal !== "none"; Layout.fillWidth: true; height: 1; color: "#272832" }
                    Label {
                        visible: modal.secVal !== "none"
                        text: modal.secVal === "reality" ? "REALITY & uTLS" : "TLS & uTLS"
                        color: "#ffffff"; font.pixelSize: 12; font.bold: true
                    }

                    GridLayout {
                        visible: modal.secVal !== "none"
                        Layout.fillWidth: true; columns: 2; columnSpacing: 12; rowSpacing: 8

                        ColumnLayout {
                            Layout.fillWidth: true; spacing: 3
                            Label { text: "SNI (Server Name)"; color: "#9e9ea6"; font.pixelSize: 10 }
                            TextField {
                                Layout.fillWidth: true; text: modal.sniVal; onTextChanged: modal.sniVal = text
                                color: "#ffffff"; font.pixelSize: 11; font.family: "monospace"
                                background: Rectangle { color: "#121214"; radius: 4; border.color: "#2c2e38" }
                            }
                        }

                        ColumnLayout {
                            Layout.fillWidth: true; spacing: 3
                            Label { text: "ALPN (comma-separated, e.g. h2,http/1.1)"; color: "#9e9ea6"; font.pixelSize: 10 }
                            TextField {
                                Layout.fillWidth: true; text: modal.alpnVal; onTextChanged: modal.alpnVal = text
                                placeholderText: "h2,http/1.1"
                                placeholderTextColor: "#545662"; color: "#ffffff"; font.pixelSize: 11; font.family: "monospace"
                                background: Rectangle { color: "#121214"; radius: 4; border.color: "#2c2e38" }
                            }
                        }

                        ColumnLayout {
                            Layout.columnSpan: 2; Layout.fillWidth: true; spacing: 3
                            Label { text: "uTLS Fingerprint (fp)"; color: "#9e9ea6"; font.pixelSize: 10 }
                            Flow {
                                Layout.fillWidth: true; spacing: 4
                                Repeater {
                                    model: ["", "chrome", "firefox", "safari", "ios", "android", "edge", "random", "randomized"]
                                    delegate: Rectangle {
                                        height: 24; radius: 4; implicitWidth: fpL.implicitWidth + 10
                                        color: modal.fpVal === modelData ? "#1c3354" : "#222329"
                                        border.color: modal.fpVal === modelData ? "#4e97fa" : "#343640"
                                        Label { id: fpL; anchors.centerIn: parent; text: modelData === "" ? (pm.isRu ? "По ум." : "Auto") : modelData; color: "#ffffff"; font.pixelSize: 10 }
                                        MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: modal.fpVal = modelData }
                                    }
                                }
                            }
                        }
                    }

                    GridLayout {
                        visible: modal.secVal === "tls"
                        Layout.fillWidth: true; columns: 3; columnSpacing: 10; rowSpacing: 6
                        ColumnLayout {
                            Layout.fillWidth: true; spacing: 3
                            Label { text: "Pinned Cert SHA-256 (pcs)"; color: "#9e9ea6"; font.pixelSize: 10 }
                            TextField {
                                Layout.fillWidth: true; text: modal.pcsVal; onTextChanged: modal.pcsVal = text
                                color: "#ffffff"; font.pixelSize: 10; font.family: "monospace"
                                background: Rectangle { color: "#121214"; radius: 4; border.color: "#2c2e38" }
                            }
                        }
                        ColumnLayout {
                            Layout.fillWidth: true; spacing: 3
                            Label { text: "Verify Cert Name (vcn)"; color: "#9e9ea6"; font.pixelSize: 10 }
                            TextField {
                                Layout.fillWidth: true; text: modal.vcnVal; onTextChanged: modal.vcnVal = text
                                color: "#ffffff"; font.pixelSize: 10; font.family: "monospace"
                                background: Rectangle { color: "#121214"; radius: 4; border.color: "#2c2e38" }
                            }
                        }
                        ColumnLayout {
                            Layout.fillWidth: true; spacing: 3
                            Label { text: "ECH Config List (ech)"; color: "#9e9ea6"; font.pixelSize: 10 }
                            TextField {
                                Layout.fillWidth: true; text: modal.echVal; onTextChanged: modal.echVal = text
                                color: "#ffffff"; font.pixelSize: 10; font.family: "monospace"
                                background: Rectangle { color: "#121214"; radius: 4; border.color: "#2c2e38" }
                            }
                        }
                    }

                    GridLayout {
                        visible: modal.secVal === "reality"
                        Layout.fillWidth: true; columns: 3; columnSpacing: 10; rowSpacing: 6
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
                            Layout.preferredWidth: 130; spacing: 3
                            Label { text: "Short ID (sid)"; color: "#9e9ea6"; font.pixelSize: 10 }
                            TextField {
                                Layout.fillWidth: true; text: modal.sidVal; onTextChanged: modal.sidVal = text
                                color: "#ffffff"; font.pixelSize: 11; font.family: "monospace"
                                background: Rectangle { color: "#121214"; radius: 4; border.color: "#2c2e38" }
                            }
                        }
                        ColumnLayout {
                            Layout.fillWidth: true; spacing: 3
                            Label { text: "SpiderX (spx)"; color: "#9e9ea6"; font.pixelSize: 10 }
                            TextField {
                                Layout.fillWidth: true; text: modal.spxVal; onTextChanged: modal.spxVal = text
                                placeholderText: "/"
                                placeholderTextColor: "#545662"; color: "#ffffff"; font.pixelSize: 11; font.family: "monospace"
                                background: Rectangle { color: "#121214"; radius: 4; border.color: "#2c2e38" }
                            }
                        }
                    }

                    // Mux, TLS Fragmentation & Sockopt
                    Rectangle { Layout.fillWidth: true; height: 1; color: "#272832" }
                    Label {
                        text: pm.isRu ? "Оптимизация сокетов, Mux и анти-DPI (Fragment)" : "Socket Options, Mux & Anti-DPI (Fragment)"
                        color: "#ffffff"; font.pixelSize: 12; font.bold: true
                    }

                    Flow {
                        Layout.fillWidth: true; spacing: 18

                        RowLayout {
                            spacing: 6
                            PmSwitch { checked: modal.muxEnabledVal; onToggled: modal.muxEnabledVal = checked }
                            Label { text: "Mux.cool"; color: "#dcdce4"; font.pixelSize: 11 }
                        }
                        RowLayout {
                            visible: modal.muxEnabledVal
                            spacing: 6
                            Label { text: pm.isRu ? "Потоков:" : "Concurrency:"; color: "#9e9ea6"; font.pixelSize: 10 }
                            TextField {
                                Layout.preferredWidth: 56; text: modal.muxConcVal; onTextChanged: modal.muxConcVal = text
                                color: "#ffffff"; font.pixelSize: 11; font.family: "monospace"
                                background: Rectangle { color: "#121214"; radius: 4; border.color: "#2c2e38" }
                            }
                        }
                        RowLayout {
                            spacing: 6
                            PmSwitch { checked: modal.fragmentVal; onToggled: modal.fragmentVal = checked }
                            Label { text: pm.isRu ? "TLS Fragment (от ТСПУ/DPI)" : "TLS Fragment (Anti-DPI)"; color: "#dcdce4"; font.pixelSize: 11 }
                        }
                        RowLayout {
                            spacing: 6
                            PmSwitch { checked: modal.tfoVal; onToggled: modal.tfoVal = checked }
                            Label { text: "TCP Fast Open"; color: "#dcdce4"; font.pixelSize: 11 }
                        }
                        RowLayout {
                            spacing: 6
                            Label { text: "KeepAlive (sec):"; color: "#9e9ea6"; font.pixelSize: 10 }
                            TextField {
                                Layout.preferredWidth: 56; text: modal.keepAliveVal; onTextChanged: modal.keepAliveVal = text
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
                            var conc = parseInt(modal.muxConcVal)
                            if (isNaN(conc) || conc === 0) conc = 8
                            var ka = parseInt(modal.keepAliveVal) || 0
                            var wsh = parseInt(modal.wsHeartbeatVal) || 0
                            var alpnList = []
                            var parts = (modal.alpnVal || "").split(",")
                            for (var ai = 0; ai < parts.length; ai++) {
                                var item = parts[ai].trim()
                                if (item !== "") alpnList.push(item)
                            }
                            var body = {
                                "name": modal.nameVal,
                                "protocol": modal.protoVal,
                                "address": modal.addrVal,
                                "port": p,
                                "encryption": modal.encVal.trim(),
                                "security": modal.secVal,
                                "network": modal.netVal,
                                "sni": modal.sniVal.trim(),
                                "host": modal.hostVal.trim(),
                                "path": modal.pathVal.trim(),
                                "flow": modal.flowVal.trim(),
                                "serviceName": modal.svcVal.trim(),
                                "headerType": modal.hdrTypeVal,
                                "xhttpMode": modal.xhttpModeVal,
                                "xhttpExtra": modal.xhttpExtraVal.trim(),
                                "grpcMultiMode": modal.grpcMultiVal,
                                "grpcAuthority": modal.grpcAuthVal.trim(),
                                "kcpSeed": modal.kcpSeedVal.trim(),
                                "wsHeartbeatPeriod": wsh,
                                "fingerprint": modal.fpVal.trim(),
                                "alpn": alpnList,
                                "realityPublicKey": modal.pbkVal.trim(),
                                "realityShortId": modal.sidVal.trim(),
                                "realitySpiderX": modal.spxVal.trim(),
                                "pinnedPeerCert": modal.pcsVal.trim(),
                                "verifyPeerCertByName": modal.vcnVal.trim(),
                                "echConfigList": modal.echVal.trim(),
                                "muxEnabled": modal.muxEnabledVal,
                                "muxConcurrency": conc,
                                "fragmentEnabled": modal.fragmentVal,
                                "tcpFastOpen": modal.tfoVal,
                                "tcpKeepAliveInterval": ka,
                                "rulePreset": modal.presetVal,
                                "routingRules": modal.rulesList
                            }
                            if (modal.shareLinkVal.trim() !== "") {
                                body["shareLink"] = modal.shareLinkVal.trim()
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
