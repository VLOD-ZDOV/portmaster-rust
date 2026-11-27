import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Rectangle {
    id: overlay
    anchors.fill: parent
    color: "#cc09090b"
    z: 960
    visible: false

    property string modeLabel: "firewall" // "firewall" | "xray"
    property string actionSign: "+"       // "+" | "-"
    property string ruleKind: "domain"    // "domain" | "geoip" | "geosite" | "cidr" | "port"
    property string patternText: ""
    property string portText: ""

    signal ruleCommitted(string rule)

    function openFor(mode) {
        modeLabel = mode || "firewall"
        actionSign = "+"
        ruleKind = "domain"
        patternText = ""
        portText = ""
        pm.fetchGeodataHints()
        visible = true
    }

    function formattedRule() {
        var pat = patternText.trim()
        if (pat === "" && ruleKind !== "port") return ""
        var prefix = ""
        if (ruleKind === "geoip") prefix = pat.startsWith("geoip:") ? pat : ("geoip:" + pat.toLowerCase())
        else if (ruleKind === "geosite") prefix = pat.startsWith("geosite:") ? pat : ("geosite:" + pat.toLowerCase())
        else if (ruleKind === "port") prefix = "* " + (portText.trim() !== "" ? portText.trim() : pat)
        else prefix = pat
        var extraPort = (ruleKind !== "port" && portText.trim() !== "") ? (" " + portText.trim()) : ""
        return actionSign + " " + prefix + extraPort
    }

    function filteredHints() {
        var src = (ruleKind === "geoip") ? (pm.geodataHints.geoipCodes || [])
                : (ruleKind === "geosite") ? (pm.geodataHints.geositeCategories || [])
                : []
        var q = patternText.toLowerCase().trim()
        var out = []
        for (var i = 0; i < src.length && out.length < 30; i++) {
            var s = String(src[i])
            if (q === "" || s.toLowerCase().indexOf(q) >= 0) out.push(s)
        }
        return out
    }

    MouseArea { anchors.fill: parent; onClicked: overlay.visible = false }

    Rectangle {
        anchors.centerIn: parent
        width: Math.min(parent.width - 60, 620)
        implicitHeight: col.implicitHeight + 36
        radius: 8
        color: "#18191e"
        border.color: "#4e97fa"

        MouseArea { anchors.fill: parent; onClicked: function(m) { m.accepted = true } }

        ColumnLayout {
            id: col
            anchors.fill: parent
            anchors.margins: 18
            spacing: 12

            RowLayout {
                Layout.fillWidth: true
                Label {
                    text: pm.isRu ? "Визуальный конструктор правил" : "Visual Rule Builder"
                    color: "#ffffff"; font.pixelSize: 16; font.bold: true
                }
                Item { Layout.fillWidth: true }
                Rectangle {
                    width: 26; height: 26; radius: 4; color: "#25262e"
                    Label { anchors.centerIn: parent; text: "✕"; color: "#cacaca"; font.pixelSize: 12 }
                    MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: overlay.visible = false }
                }
            }

            // Step 1: Action (+ vs -)
            RowLayout {
                Layout.fillWidth: true
                spacing: 8
                Label { text: pm.isRu ? "Действие:" : "Action:"; color: "#9e9ea6"; font.pixelSize: 11; Layout.preferredWidth: 85 }
                Repeater {
                    model: [
                        { s: "+", l: overlay.modeLabel === "xray" ? (pm.isRu ? "+ Через VPN" : "+ Route via VPN") : (pm.isRu ? "+ Разрешить" : "+ Allow"), bg: "#134e2a", fg: "#1de966" },
                        { s: "-", l: overlay.modeLabel === "xray" ? (pm.isRu ? "- Напрямую / Блок рекламы" : "- Direct / AdBlock") : (pm.isRu ? "- Блокировать" : "- Block"), bg: "#5a1616", fg: "#f03c3c" }
                    ]
                    delegate: Rectangle {
                        height: 28; radius: 4; implicitWidth: acLbl.implicitWidth + 18
                        color: overlay.actionSign === modelData.s ? modelData.bg : "#222328"
                        border.color: overlay.actionSign === modelData.s ? modelData.fg : "#343640"
                        Label { id: acLbl; anchors.centerIn: parent; text: modelData.l; color: "#ffffff"; font.pixelSize: 11; font.bold: overlay.actionSign === modelData.s }
                        MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: overlay.actionSign = modelData.s }
                    }
                }
            }

            // Step 2: Rule Kind
            Flow {
                Layout.fillWidth: true
                spacing: 6
                Repeater {
                    model: [
                        { id: "domain",  l: pm.isRu ? "Домен (.example.com)" : "Domain" },
                        { id: "geoip",   l: pm.isRu ? "Страна (GeoIP)" : "Country GeoIP" },
                        { id: "geosite", l: pm.isRu ? "Сервис (GeoSite)" : "Service GeoSite" },
                        { id: "cidr",    l: "IP / CIDR" },
                        { id: "port",    l: pm.isRu ? "Порт / Протокол" : "Port / Proto" }
                    ]
                    delegate: Rectangle {
                        height: 26; radius: 4; implicitWidth: kLbl.implicitWidth + 16
                        color: overlay.ruleKind === modelData.id ? "#1f324d" : "#222328"
                        border.color: overlay.ruleKind === modelData.id ? "#4e97fa" : "#343640"
                        Label { id: kLbl; anchors.centerIn: parent; text: modelData.l; color: overlay.ruleKind === modelData.id ? "#ffffff" : "#b4b4c0"; font.pixelSize: 11 }
                        MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: { overlay.ruleKind = modelData.id; overlay.patternText = "" } }
                    }
                }
            }

            // Step 3: Target Pattern + Optional Port
            RowLayout {
                Layout.fillWidth: true
                spacing: 8
                TextField {
                    visible: overlay.ruleKind !== "port"
                    Layout.fillWidth: true
                    height: 34
                    text: overlay.patternText
                    onTextChanged: overlay.patternText = text
                    placeholderText: {
                        if (overlay.ruleKind === "geoip") return "ru, us, de, private..."
                        if (overlay.ruleKind === "geosite") return "youtube, openai, discord, category-ads-all..."
                        if (overlay.ruleKind === "cidr") return "192.168.1.0/24 or 1.1.1.1"
                        return ".google.com or example.org"
                    }
                    placeholderTextColor: "#66666e"; color: "#ffffff"; font.pixelSize: 12; font.family: "monospace"
                    background: Rectangle { color: "#121214"; radius: 4; border.color: parent.activeFocus ? "#4e97fa" : "#2e3038" }
                }
                TextField {
                    Layout.preferredWidth: overlay.ruleKind === "port" ? 300 : 140
                    Layout.fillWidth: overlay.ruleKind === "port"
                    height: 34
                    text: overlay.portText
                    onTextChanged: overlay.portText = text
                    placeholderText: overlay.ruleKind === "port" ? "TCP/443 or 22 or UDP/53" : (pm.isRu ? "Порт (опц.)" : "Port (opt.)")
                    placeholderTextColor: "#66666e"; color: "#ffffff"; font.pixelSize: 12; font.family: "monospace"
                    background: Rectangle { color: "#121214"; radius: 4; border.color: parent.activeFocus ? "#4e97fa" : "#2e3038" }
                }
            }

            // Step 4: GeoIP / GeoSite Autocomplete Chips
            ColumnLayout {
                Layout.fillWidth: true
                visible: overlay.ruleKind === "geoip" || overlay.ruleKind === "geosite"
                spacing: 6
                Label {
                    text: (overlay.ruleKind === "geoip"
                           ? (pm.geodataHints.geoipAvailable ? (pm.isRu ? "Категории из geoip.dat:" : "From geoip.dat:") : (pm.isRu ? "Популярные коды стран:" : "Popular country codes:"))
                           : (pm.geodataHints.geositeAvailable ? (pm.isRu ? "Категории из geosite.dat:" : "From geosite.dat:") : (pm.isRu ? "Популярные категории GeoSite:" : "Popular GeoSite categories:")))
                    color: "#767c8a"; font.pixelSize: 10; font.bold: true
                }
                Flow {
                    Layout.fillWidth: true
                    spacing: 5
                    Repeater {
                        model: overlay.filteredHints()
                        delegate: Rectangle {
                            height: 22; radius: 4; color: "#21232b"; border.color: "#333642"
                            implicitWidth: hLbl.implicitWidth + 12
                            Label { id: hLbl; anchors.centerIn: parent; text: modelData; color: "#89b7ff"; font.pixelSize: 10; font.family: "monospace" }
                            MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: overlay.patternText = modelData }
                        }
                    }
                }
            }

            // Live Preview + Submit
            RowLayout {
                Layout.fillWidth: true
                spacing: 10
                Rectangle {
                    Layout.fillWidth: true; height: 32; radius: 4; color: "#121316"; border.color: "#262832"
                    Label {
                        anchors.fill: parent; anchors.leftMargin: 10; verticalAlignment: Text.AlignVCenter
                        text: (pm.isRu ? "Правило: " : "Rule: ") + (overlay.formattedRule() || "—")
                        color: overlay.actionSign === "+" ? "#1de966" : "#ff8585"
                        font.pixelSize: 12; font.family: "monospace"; elide: Text.ElideRight
                    }
                }
                Rectangle {
                    height: 32; radius: 4; color: overlay.formattedRule() !== "" ? "#4e97fa" : "#25262e"
                    implicitWidth: okBtnL.implicitWidth + 24
                    Label { id: okBtnL; anchors.centerIn: parent; text: pm.isRu ? "Добавить" : "Add Rule"; color: "#ffffff"; font.pixelSize: 12; font.bold: true }
                    MouseArea {
                        anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                        onClicked: {
                            var r = overlay.formattedRule()
                            if (r !== "") {
                                overlay.ruleCommitted(r)
                                overlay.visible = false
                            }
                        }
                    }
                }
            }
        }
    }
}
