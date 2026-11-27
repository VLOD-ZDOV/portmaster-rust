import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Rectangle {
    id: card
    property var st: ({})
    property bool isPerApp: false
    property string appKey: ""
    property bool appendDns: true
    signal appendDnsToggled(bool checked)
    signal requestRuleBuilder(string key, string mode)

    readonly property bool isArr: {
        var k = st.key || ""
        return st.optType === 2 ||
               (st.arrayItems && st.arrayItems.length > 0 && st.optType !== 1 && st.optType !== 3 && st.optType !== 4) ||
               k === "dns/nameservers" ||
               k === "xray/rules" ||
               k === "spn/usagePolicy" ||
               k === "splittun/usagePolicy" ||
               k === "filter/endpoints" ||
               k === "filter/serviceEndpoints" ||
               k === "filter/lists"
    }

    function saveValue(v) {
        if (card.isPerApp) {
            if (card.appKey !== "") pm.setAppSetting(card.appKey, st.key, v)
        } else {
            pm.updateSetting(st.key, v)
        }
    }

    function resetValue() {
        if (card.isPerApp) {
            if (card.appKey !== "") pm.resetAppSetting(card.appKey, st.key)
        } else {
            pm.resetSetting(st.key)
        }
    }

    function applyArrPreset(vals, append) {
        if (card.isPerApp) {
            if (card.appKey !== "") pm.applyAppPreset(card.appKey, st.key, vals, append)
        } else {
            pm.applyPreset(st.key, vals, append)
        }
    }

    implicitHeight: stCol.implicitHeight + 24
    radius: 5
    color: "#1a1a1d"
    border.color: !!st.modified ? "#2b4263" : "#26262b"

    // Left accent strip when overridden globally or per-app
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
                spacing: 3

                Flow {
                    Layout.fillWidth: true
                    spacing: 8

                    Label {
                        text: st.name || st.key || ""
                        color: "#ffffff"
                        font.pixelSize: 13
                        font.bold: true
                    }

                    Rectangle {
                        height: 18; radius: 3; color: "#25262b"
                        implicitWidth: kLbl.implicitWidth + 10
                        Label {
                            id: kLbl; anchors.centerIn: parent
                            text: st.key || ""
                            color: "#888894"; font.pixelSize: 10; font.family: "monospace"
                        }
                    }

                    Rectangle {
                        visible: card.isPerApp
                        height: 18; radius: 3
                        color: !!st.modified ? "#162d4d" : "#212227"
                        border.color: !!st.modified ? "#3b6ba8" : "#2f3038"
                        implicitWidth: inhLbl.implicitWidth + 12
                        Label {
                            id: inhLbl; anchors.centerIn: parent
                            text: !!st.modified
                                  ? (pm.isRu ? ("Свое • Глобальное: " + (st.globalDisplay || "-")) : ("Override • Global: " + (st.globalDisplay || "-")))
                                  : (pm.isRu ? ("Как глобально: " + (st.globalDisplay || "-")) : ("Global: " + (st.globalDisplay || "-")))
                            color: !!st.modified ? "#8fc1ff" : "#8b8b96"
                            font.pixelSize: 10
                        }
                    }

                    Rectangle {
                        visible: !!st.modified
                        height: 18; radius: 3
                        color: rstMa.containsMouse ? "#253a59" : "#1c2838"
                        border.color: "#4e97fa"
                        implicitWidth: rstLbl.implicitWidth + 12
                        Label {
                            id: rstLbl; anchors.centerIn: parent
                            text: card.isPerApp
                                  ? (pm.isRu ? "↺ Как глобально" : "↺ Use Global")
                                  : (pm.isRu ? "↺ Сбросить" : "↺ Reset")
                            color: "#7bb5ff"; font.pixelSize: 10; font.bold: true
                        }
                        MouseArea {
                            id: rstMa; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor
                            onClicked: card.resetValue()
                        }
                    }
                }

                Label {
                    visible: (st.description || "") !== ""
                    text: st.description || ""
                    color: "#84848e"; font.pixelSize: 11; wrapMode: Text.WordWrap
                    Layout.fillWidth: true
                }
            }

            // Bool Toggle Switch (optType == 4)
            PmSwitch {
                visible: st.optType === 4 && !card.isArr
                checked: !!st.boolVal
                onToggled: card.saveValue(checked)
            }
        }

        // Choice Selector (choices.length > 0 for non-Bool)
        Flow {
            visible: st.choices && st.choices.length > 0 && st.optType !== 4 && !card.isArr
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
                        onClicked: card.saveValue(modelData.Value)
                    }
                }
            }
        }

        // Array Editor + Presets (optType == 2 or array keys)
        ColumnLayout {
            id: arrCol
            visible: card.isArr
            Layout.fillWidth: true
            spacing: 6

            function prettyDnsLabel(url) {
                var u = (url || "").toLowerCase()
                var proto = u.indexOf("dot://") === 0 ? "DoT" : (u.indexOf("https://") === 0 || u.indexOf("doh://") === 0 ? "DoH" : "Plain")
                var name = "Custom"
                if (u.indexOf("cloudflare") >= 0 || u.indexOf("1.1.1.") >= 0 || u.indexOf("1.0.0.") >= 0) name = u.indexOf("1.1.1.2") >= 0 ? "Cloudflare Malware" : "Cloudflare"
                else if (u.indexOf("quad9") >= 0 || u.indexOf("9.9.9.9") >= 0) name = "Quad9"
                else if (u.indexOf("adguard") >= 0 || u.indexOf("94.140.14") >= 0) name = "AdGuard"
                else if (u.indexOf("mullvad") >= 0 || u.indexOf("194.242.") >= 0) name = "Mullvad"
                else if (u.indexOf("controld") >= 0 || u.indexOf("76.76.") >= 0) name = "Control D"
                else if (u.indexOf("applied-privacy") >= 0) name = "Applied Privacy"
                else if (u.indexOf("dns.google") >= 0 || u.indexOf("8.8.8.8") >= 0) name = "Google DNS"
                else if (u.indexOf("yandex") >= 0 || u.indexOf("77.88.") >= 0) name = "Yandex DNS"
                return name + " • " + proto
            }

            ColumnLayout {
                Layout.fillWidth: true
                spacing: 4
                visible: st.key === "dns/nameservers"
                Repeater {
                    model: st.arrayItems || []
                    delegate: Rectangle {
                        Layout.fillWidth: true
                        height: 30; radius: 4; color: "#20222a"; border.color: index === 0 ? "#3c5882" : "#30333e"
                        RowLayout {
                            anchors.fill: parent; anchors.leftMargin: 8; anchors.rightMargin: 8; spacing: 8
                            Label { text: "#" + (index + 1); color: index === 0 ? "#4e97fa" : "#76767e"; font.pixelSize: 10; font.bold: true }
                            Rectangle {
                                height: 18; radius: 3; color: "#192638"; border.color: "#2f4769"
                                implicitWidth: dnsTagLbl.implicitWidth + 10
                                Label { id: dnsTagLbl; anchors.centerIn: parent; text: arrCol.prettyDnsLabel(modelData); color: "#9fd0ff"; font.pixelSize: 10; font.bold: true }
                            }
                            Label { text: modelData; color: "#d4d6e0"; font.pixelSize: 10; font.family: "monospace"; elide: Text.ElideMiddle; Layout.fillWidth: true }
                            Label {
                                visible: index > 0
                                text: "↑"; color: "#89b7ff"; font.pixelSize: 12; font.bold: true
                                MouseArea {
                                    anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                                    onClicked: card.isPerApp ? pm.moveAppArrayItem(card.appKey, st.key, index, -1) : pm.moveSettingArrayItem(st.key, index, -1)
                                }
                            }
                            Label {
                                visible: index < ((st.arrayItems || []).length - 1)
                                text: "↓"; color: "#89b7ff"; font.pixelSize: 12; font.bold: true
                                MouseArea {
                                    anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                                    onClicked: card.isPerApp ? pm.moveAppArrayItem(card.appKey, st.key, index, 1) : pm.moveSettingArrayItem(st.key, index, 1)
                                }
                            }
                            Label {
                                text: "✕"; color: "#e01d1d"; font.pixelSize: 11
                                MouseArea {
                                    anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                                    onClicked: {
                                        var next = []
                                        var cur = st.arrayItems || []
                                        for (var k = 0; k < cur.length; k++) if (k !== index) next.push(cur[k])
                                        card.saveValue(next)
                                    }
                                }
                            }
                        }
                    }
                }
            }

            Flow {
                visible: st.key !== "dns/nameservers"
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
                                        card.saveValue(next)
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
                    placeholderText: st.key === "dns/nameservers"
                                     ? (pm.isRu ? "Добавить DNS (напр. dot://1.1.1.1, https://.../dns-query, 9.9.9.9)..." : "Add DNS URL or IP (dot://..., https://..., 9.9.9.9)...")
                                     : (pm.isRu ? "Добавить значение..." : "Add value...")
                    placeholderTextColor: "#66666e"
                    color: "#ffffff"; font.pixelSize: 11; font.family: "monospace"
                    background: Rectangle { color: "#121213"; radius: 4; border.color: "#2e2e32" }
                    function submitItem() {
                        var v = arrInput.text.trim()
                        if (v === "") return
                        card.applyArrPreset([v], true)
                        arrInput.text = ""
                    }
                    onAccepted: submitItem()
                }
                Rectangle {
                    height: 28; radius: 4; color: "#4e97fa"
                    implicitWidth: arrAddLbl.implicitWidth + 16
                    Label { id: arrAddLbl; anchors.centerIn: parent; text: "+"; color: "#ffffff"; font.pixelSize: 13; font.bold: true }
                    MouseArea {
                        anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                        onClicked: arrInput.submitItem()
                    }
                }
                Rectangle {
                    visible: st.key === "xray/rules" || st.key === "filter/endpoints" || st.key === "filter/serviceEndpoints" || st.key === "spn/usagePolicy" || st.key === "splittun/usagePolicy"
                    height: 28; radius: 4; color: "#1c283a"; border.color: "#4e97fa"
                    implicitWidth: scRbLbl.implicitWidth + 14
                    Label { id: scRbLbl; anchors.centerIn: parent; text: pm.isRu ? "⚙ Конструктор" : "⚙ Builder"; color: "#89b7ff"; font.pixelSize: 10; font.bold: true }
                    MouseArea {
                        anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                        onClicked: card.requestRuleBuilder(st.key || "", (st.key === "xray/rules" || st.key === "spn/usagePolicy" || st.key === "splittun/usagePolicy") ? "xray" : "firewall")
                    }
                }
            }

            PmCheckBox {
                visible: st.key === "dns/nameservers"
                text: pm.t("set.appendDns")
                checked: card.appendDns
                onToggled: card.appendDnsToggled(checked)
            }

            Flow {
                visible: st.key === "dns/nameservers" || st.key === "xray/rules" ||
                         st.key === "spn/usagePolicy" || st.key === "splittun/usagePolicy" ||
                         st.key === "filter/endpoints" || st.key === "filter/serviceEndpoints" ||
                         st.key === "filter/lists"
                Layout.fillWidth: true
                spacing: 6
                Repeater {
                    model: {
                        var k = st.key || ""
                        if (k === "dns/nameservers") return [
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
                        if (k === "xray/rules") return [
                            { label: pm.isRu ? "RU напрямую (geoip:ru)" : "RU Direct", vals: ["- .ru", "- .xn--p1ai", "- geoip:ru", "- geosite:category-ru"] },
                            { label: "YouTube + Discord -> Xray", vals: ["+ geosite:youtube", "+ geosite:discord"] },
                            { label: "OpenAI / AI -> Xray", vals: ["+ geosite:openai", "+ geosite:anthropic"] }
                        ]
                        if (k === "spn/usagePolicy" || k === "splittun/usagePolicy") return [
                            { label: pm.isRu ? "- RU домены напрямую" : "- RU domains direct", vals: ["- .ru", "- .xn--p1ai"] },
                            { label: "- LAN / Localhost", vals: ["- LAN", "- Localhost"] },
                            { label: "+ geosite:youtube", vals: ["+ geosite:youtube"] }
                        ]
                        if (k === "filter/serviceEndpoints") return [
                            { label: "+ SSH (tcp/22)", vals: ["+ * tcp/22"] },
                            { label: "+ HTTP/S (80,443)", vals: ["+ * tcp/80", "+ * tcp/443"] },
                            { label: "+ LAN", vals: ["+ LAN"] }
                        ]
                        if (k === "filter/lists") return [
                            { label: "Default (TRAC/MAL/BAD/UNBREAK)", vals: ["TRAC", "MAL", "BAD", "UNBREAK"] }
                        ]
                        return [
                            { label: "+ KDE Connect LAN (1714-1764)", vals: ["+ LAN 1714-1764"] },
                            { label: "- Ads (geosite)", vals: ["- geosite:category-ads-all"] },
                            { label: "+ LAN", vals: ["+ LAN"] }
                        ]
                    }
                    delegate: Rectangle {
                        height: 22; radius: 3; color: "#23262f"; border.color: "#384152"
                        implicitWidth: pL.implicitWidth + 12
                        Label { id: pL; anchors.centerIn: parent; text: modelData.label; color: "#89b7ff"; font.pixelSize: 10 }
                        MouseArea {
                            anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                            onClicked: card.applyArrPreset(modelData.vals, modelData.forceAppend ? true : (st.key === "dns/nameservers" ? card.appendDns : (st.key === "filter/lists" ? false : true)))
                        }
                    }
                }
            }
        }

        // Int Editor (optType == 3)
        RowLayout {
            visible: st.optType === 3 && (!st.choices || st.choices.length === 0) && !card.isArr
            Layout.fillWidth: true
            spacing: 8
            Rectangle {
                width: 28; height: 28; radius: 4; color: "#22242a"; border.color: "#343640"
                Label { anchors.centerIn: parent; text: "−"; color: "#ffffff"; font.pixelSize: 14 }
                MouseArea {
                    anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                    onClicked: {
                        var v = parseInt(intInput.text)
                        if (!isNaN(v)) card.saveValue(v - 1)
                    }
                }
            }
            TextField {
                id: intInput
                Layout.preferredWidth: 130
                text: String(st.intVal !== undefined ? st.intVal : 0)
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
                        if (!isNaN(v)) card.saveValue(v + 1)
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
                        if (!isNaN(v)) card.saveValue(v)
                    }
                }
            }
        }

        // Plain String Editor (optType == 1)
        RowLayout {
            visible: st.optType === 1 && (!st.choices || st.choices.length === 0) && !card.isArr
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
                    onClicked: card.saveValue(scalarInput.text)
                }
            }
        }
    }
}
