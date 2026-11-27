import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Item {
    id: root
    signal openGlobalRuleBuilder()

    property bool showSubDrawer: false
    property bool showLinkDrawer: false
    property string nodeSearch: ""
    property string selectedCountry: "" // Clicking a country on map or sidebar opens its nodes + flag panel!
    property var cachedWorld: []
    property string lastWorldSig: ""

    function activeWin() { return !Window.window || Window.window.visible }

    function syncXrayMap(force) {
        if (!root.visible || !activeWin()) return
        var pins = pm.mapPins || []
        var st = pm.xrayStatus || ({})
        var sig = (st.connected ? "1:" : "0:") + (st.nodeId || "") + "|"
        for (var i = 0; i < pins.length; i++) {
            sig += (pins[i].country || "") + (pins[i].active ? "*," : ",")
        }
        if (!force && sig === root.lastWorldSig) return
        root.lastWorldSig = sig
        worldCanvas.requestPaint()
        if (mapBox.hoveredHitKey !== "" || root.selectedCountry !== "") hoverCanvas.requestPaint()
    }

    function filteredNodes(countryOverride) {
        var src = pm.xrayNodes || []
        var q = nodeSearch.toLowerCase().trim()
        var targetCC = countryOverride !== undefined ? countryOverride : selectedCountry
        var cc = (targetCC || "").trim().toUpperCase()
        var out = []
        for (var i = 0; i < src.length; i++) {
            var n = src[i]
            if (cc !== "" && (n.country || "").trim().toUpperCase() !== cc) continue
            if (q !== "") {
                var hay = ((n.name || "") + " " + (n.country || "") + " " + (n.address || "") + " " + (n.protocol || "")).toLowerCase()
                if (hay.indexOf(q) < 0) continue
            }
            out.push(n)
        }
        return out
    }

    function availableCountries() {
        var src = pm.xrayNodes || []
        var seen = {}
        var out = []
        for (var i = 0; i < src.length; i++) {
            var cc = (src[i].country || "").trim().toUpperCase()
            if (cc !== "" && !seen[cc]) {
                seen[cc] = true
                out.push(cc)
            }
        }
        out.sort()
        return out
    }

    Connections {
        target: pm
        function onWorldMapReady() {
            root.cachedWorld = pm.worldCountries || []
            root.syncXrayMap(true)
            if (root.visible && root.activeWin()) hoverCanvas.requestPaint()
        }
        function onXrayChanged() { root.syncXrayMap(false) }
    }
    onVisibleChanged: if (visible) { root.syncXrayMap(true); hoverCanvas.requestPaint() }
    onSelectedCountryChanged: if (root.visible && root.activeWin()) hoverCanvas.requestPaint()

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 18
        spacing: 12

        // 1. Top Toolbar Matching Angular xray-page.html (Buttons above map so map keeps full space!)
        Rectangle {
            Layout.fillWidth: true
            height: 54
            radius: 6
            color: "#191a1e"
            border.color: "#282a32"

            RowLayout {
                anchors.fill: parent
                anchors.leftMargin: 16; anchors.rightMargin: 16
                spacing: 12

                Label {
                    text: pm.t("xray.title")
                    color: "#ffffff"; font.pixelSize: 16; font.bold: true
                }

                Rectangle {
                    height: 20; radius: 4
                    color: (pm.xrayStatus.geoipAvailable && pm.xrayStatus.geositeAvailable) ? "#13293e" : "#3a2c12"
                    implicitWidth: geoBadge.implicitWidth + 12
                    Label {
                        id: geoBadge; anchors.centerIn: parent
                        text: (pm.xrayStatus.geoipAvailable && pm.xrayStatus.geositeAvailable)
                              ? "GeoIP + GeoSite" : (pm.isRu ? "Нет баз Geo" : "Geo Missing")
                        color: (pm.xrayStatus.geoipAvailable && pm.xrayStatus.geositeAvailable) ? "#89b7ff" : "#e9d81d"
                        font.pixelSize: 9; font.bold: true
                    }
                }

                RowLayout {
                    spacing: 6
                    PmSwitch { checked: pm.globalVpn; onToggled: pm.toggleGlobalVpn() }
                    Label { text: pm.isRu ? "Весь трафик" : "Route All"; color: "#cacad4"; font.pixelSize: 11 }
                }

                RowLayout {
                    spacing: 6
                    PmSwitch { checked: pm.autoConnectXray; onToggled: pm.updateSetting("xray/autoConnect", checked) }
                    Label { text: pm.isRu ? "Автоподкл." : "Auto"; color: "#cacad4"; font.pixelSize: 11 }
                }

                Item { Layout.fillWidth: true }

                Rectangle {
                    height: 28; radius: 4; color: "#1c283a"; border.color: "#4e97fa"
                    implicitWidth: gRbLbl.implicitWidth + 14
                    Label { id: gRbLbl; anchors.centerIn: parent; text: pm.isRu ? "⚙ Правила Xray" : "⚙ Xray Rules"; color: "#89b7ff"; font.pixelSize: 10; font.bold: true }
                    MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: root.openGlobalRuleBuilder() }
                }

                Rectangle {
                    height: 28; radius: 4; color: "#22242a"; border.color: "#363842"
                    implicitWidth: rstLbl.implicitWidth + 14
                    Label { id: rstLbl; anchors.centerIn: parent; text: pm.t("apps.resetDirect"); color: "#cacaca"; font.pixelSize: 10 }
                    MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: pm.resetAllAppOverrides() }
                }

                Rectangle {
                    height: 28; radius: 4; color: "#22242a"; border.color: "#363842"
                    implicitWidth: geoLbl.implicitWidth + 14
                    Label { id: geoLbl; anchors.centerIn: parent; text: pm.isRu ? "Базы Geo" : "Geodata"; color: "#cacaca"; font.pixelSize: 10 }
                    MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: pm.downloadGeodata() }
                }

                Rectangle {
                    height: 28; radius: 4; color: "#22242a"; border.color: "#363842"
                    implicitWidth: binLbl.implicitWidth + 14
                    Label { id: binLbl; anchors.centerIn: parent; text: "xray-rust"; color: "#cacaca"; font.pixelSize: 10 }
                    MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: pm.downloadXrayBinary() }
                }

                Rectangle {
                    height: 28; radius: 4; color: root.showSubDrawer ? "#4e97fa" : "#22242a"; border.color: "#363842"
                    implicitWidth: subBtnLbl.implicitWidth + 14
                    Label { id: subBtnLbl; anchors.centerIn: parent; text: "+ " + pm.t("xray.addSub"); color: "#ffffff"; font.pixelSize: 10 }
                    MouseArea {
                        anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                        onClicked: { root.showSubDrawer = !root.showSubDrawer; if (root.showSubDrawer) root.showLinkDrawer = false }
                    }
                }

                Rectangle {
                    height: 28; radius: 4; color: root.showLinkDrawer ? "#4e97fa" : "#22242a"; border.color: "#363842"
                    implicitWidth: lnkBtnLbl.implicitWidth + 14
                    Label { id: lnkBtnLbl; anchors.centerIn: parent; text: "+ " + pm.t("xray.addManual"); color: "#ffffff"; font.pixelSize: 10 }
                    MouseArea {
                        anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                        onClicked: { root.showLinkDrawer = !root.showLinkDrawer; if (root.showLinkDrawer) root.showSubDrawer = false }
                    }
                }
            }
        }

        // Inline Drawers (+ Subscription / + Link)
        Rectangle {
            visible: root.showSubDrawer
            Layout.fillWidth: true
            height: 50; radius: 6; color: "#1d1e24"; border.color: "#4e97fa"
            RowLayout {
                anchors.fill: parent; anchors.margins: 10; spacing: 10
                TextField {
                    id: subUrlField
                    Layout.fillWidth: true
                    placeholderText: "https://example.com/subscription"
                    placeholderTextColor: "#66666e"; color: "#ffffff"; font.pixelSize: 12
                    background: Rectangle { color: "#121213"; radius: 4; border.color: "#2e2e32" }
                }
                Rectangle {
                    height: 28; radius: 4; color: "#4e97fa"
                    implicitWidth: saveSubLbl.implicitWidth + 18
                    Label { id: saveSubLbl; anchors.centerIn: parent; text: pm.isRu ? "Добавить" : "Import"; color: "#ffffff"; font.pixelSize: 11; font.bold: true }
                    MouseArea {
                        anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                        onClicked: {
                            if (subUrlField.text.trim() !== "") {
                                pm.addSubscription(subUrlField.text.trim())
                                subUrlField.text = ""; root.showSubDrawer = false
                            }
                        }
                    }
                }
            }
        }

        Rectangle {
            visible: root.showLinkDrawer
            Layout.fillWidth: true
            height: 50; radius: 6; color: "#1d1e24"; border.color: "#4e97fa"
            RowLayout {
                anchors.fill: parent; anchors.margins: 10; spacing: 10
                TextField {
                    id: linkField
                    Layout.fillWidth: true
                    placeholderText: "vless://... or trojan://... or ss://..."
                    placeholderTextColor: "#66666e"; color: "#ffffff"; font.pixelSize: 12; font.family: "monospace"
                    background: Rectangle { color: "#121213"; radius: 4; border.color: "#2e2e32" }
                }
                Rectangle {
                    height: 28; radius: 4; color: "#4e97fa"
                    implicitWidth: saveLnkLbl.implicitWidth + 18
                    Label { id: saveLnkLbl; anchors.centerIn: parent; text: pm.isRu ? "Добавить" : "Add Node"; color: "#ffffff"; font.pixelSize: 11; font.bold: true }
                    MouseArea {
                        anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                        onClicked: {
                            if (linkField.text.trim() !== "") {
                                pm.addManualLink(linkField.text.trim())
                                linkField.text = ""; root.showLinkDrawer = false
                            }
                        }
                    }
                }
            }
        }

        // 2. Full-Height Split Layout: Interactive Map Left + Node/Sub Panel Right (1:1 Angular xray-page.html)
        RowLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            spacing: 14

            // LEFT/CENTER: World Map Canvas + Floating Overlays + Bottom Country Nodes Strip
            Rectangle {
                id: mapBox
                Layout.fillWidth: true
                Layout.fillHeight: true
                radius: 6
                color: "#111112"
                border.color: "#25262e"
                clip: true

                property real zoomScale: 1.0
                property real panX: 0.0
                property real panY: 0.0
                property string hoveredCode: ""
                property string hoveredHitKey: ""
                property string hoveredName: ""
                property int hoveredNodes: 0

                function clampPan() {
                    var minX = mapBox.width - mapViewport.width
                    var minY = mapBox.height - mapViewport.height
                    panX = Math.max(minX, Math.min(0, panX))
                    panY = Math.max(minY, Math.min(0, panY))
                }

                function setZoom(nextZoom, anchorX, anchorY) {
                    var z = Math.max(1.0, Math.min(5.0, nextZoom))
                    if (Math.abs(z - zoomScale) < 0.001) return
                    var relX = (anchorX - panX) / zoomScale
                    var relY = (anchorY - panY) / zoomScale
                    zoomScale = z
                    panX = anchorX - relX * zoomScale
                    panY = anchorY - relY * zoomScale
                    clampPan()
                }

                function pinInfoFor(code) {
                    var pins = pm.mapPins || []
                    for (var i = 0; i < pins.length; i++) {
                        if ((pins[i].country || "").toUpperCase() === code) return pins[i]
                    }
                    return null
                }

                function hitTestCountry(nx, ny) {
                    var hit = pm.hitTestMapCountry(nx, ny)
                    return (hit && (hit.hitKey || hit.code)) ? hit : null
                }

                function drawCountryFlat(ctx, c, w, h) {
                    var cpolys = c.polygons || []
                    for (var k = 0; k < cpolys.length; k++) {
                        var polyRings = cpolys[k]
                        if (!polyRings || polyRings.length === 0) continue
                        if (typeof polyRings[0] === "number") polyRings = [polyRings]
                        ctx.beginPath()
                        for (var r = 0; r < polyRings.length; r++) {
                            var pts = polyRings[r]
                            if (!pts || pts.length < 6) continue
                            ctx.moveTo(pts[0] * w, pts[1] * h)
                            for (var j = 2; j < pts.length; j += 2) {
                                ctx.lineTo(pts[j] * w, pts[j + 1] * h)
                            }
                            ctx.closePath()
                        }
                        ctx.fill("evenodd")
                        ctx.stroke()
                    }
                }

                onWidthChanged: clampPan()
                onHeightChanged: clampPan()
                onHoveredCodeChanged: if (root.visible && root.activeWin()) hoverCanvas.requestPaint()
                onHoveredHitKeyChanged: if (root.visible && root.activeWin()) hoverCanvas.requestPaint()

                Item {
                    id: mapViewport
                    x: mapBox.panX
                    y: mapBox.panY
                    width: mapBox.width * mapBox.zoomScale
                    height: mapBox.height * mapBox.zoomScale

                    // Static Base Layer: Countries WITHOUT nodes have NO borders (fill == stroke == #1e1e1e)!
                    Canvas {
                        id: worldCanvas
                        anchors.fill: parent
                        onPaint: {
                            var ctx = getContext("2d")
                            ctx.clearRect(0, 0, width, height)
                            var countries = (root.cachedWorld && root.cachedWorld.length > 0) ? root.cachedWorld : (pm.worldCountries || [])
                            if (countries.length === 0) return

                            var pins = pm.mapPins || []
                            var nodeCountries = {}
                            var activeCountries = {}
                            for (var pi = 0; pi < pins.length; pi++) {
                                var pc = (pins[pi].country || "").toUpperCase()
                                if (pc !== "") {
                                    nodeCountries[pc] = true
                                    if (pins[pi].active) activeCountries[pc] = true
                                }
                            }

                            // 3-pass painting prevents adjacent borderless land fills from overdrawing active country borders
                            ctx.fillStyle = "#1e1e1e"
                            ctx.strokeStyle = "#1e1e1e"
                            ctx.lineWidth = 1.6
                            for (var c0 = 0; c0 < countries.length; c0++) {
                                var code0 = countries[c0].code || ""
                                if (!nodeCountries[code0] && !activeCountries[code0]) {
                                    mapBox.drawCountryFlat(ctx, countries[c0], width, height)
                                }
                            }

                            ctx.fillStyle = "#2b2c32"
                            ctx.strokeStyle = "#3a3d48"
                            ctx.lineWidth = 1.1
                            for (var c1 = 0; c1 < countries.length; c1++) {
                                var code1 = countries[c1].code || ""
                                if (nodeCountries[code1] && !activeCountries[code1]) {
                                    mapBox.drawCountryFlat(ctx, countries[c1], width, height)
                                }
                            }

                            ctx.fillStyle = "#175432"
                            ctx.strokeStyle = "#1de966"
                            ctx.lineWidth = 1.3
                            for (var c2 = 0; c2 < countries.length; c2++) {
                                var code2 = countries[c2].code || ""
                                if (activeCountries[code2]) {
                                    mapBox.drawCountryFlat(ctx, countries[c2], width, height)
                                }
                            }
                        }
                        Component.onCompleted: requestPaint()
                    }

                    // Dynamic Hover/Selected Overlay Layer: Borders appear ONLY on hovered or selected country!
                    Canvas {
                        id: hoverCanvas
                        anchors.fill: parent
                        onPaint: {
                            var ctx = getContext("2d")
                            ctx.clearRect(0, 0, width, height)
                            var cs = (root.cachedWorld && root.cachedWorld.length > 0) ? root.cachedWorld : (pm.worldCountries || [])
                            if (cs.length === 0) return
                            var hovKey = mapBox.hoveredHitKey
                            var sel = root.selectedCountry
                            if (hovKey === "" && sel === "") return

                            for (var i = 0; i < cs.length; i++) {
                                var c = cs[i]
                                var code = c.code || ""
                                var hKey = c.hitKey || code
                                var isSel = (sel !== "" && code === sel)
                                var isHov = (hovKey !== "" && hKey === hovKey)
                                if (!isSel && !isHov) continue

                                ctx.fillStyle = isHov ? (mapBox.hoveredNodes > 0 ? "#315382" : "#3a3a40") : "#28364a"
                                ctx.strokeStyle = isSel ? "#4e97fa" : "#85858e"
                                ctx.lineWidth = isSel ? 1.8 : 1.4
                                mapBox.drawCountryFlat(ctx, c, width, height)
                            }
                        }
                    }

                    MouseArea {
                        id: mapDragMa
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: mapBox.hoveredCode !== "" ? Qt.PointingHandCursor : Qt.OpenHandCursor
                        property real dragStartX: 0
                        property real dragStartY: 0
                        property bool dragged: false

                        onPressed: function(mouse) {
                            var pt = mapToItem(mapBox, mouse.x, mouse.y)
                            dragStartX = pt.x; dragStartY = pt.y; dragged = false
                        }
                        onPositionChanged: function(mouse) {
                            if (pressed) {
                                var pt = mapToItem(mapBox, mouse.x, mouse.y)
                                var dx = pt.x - dragStartX; var dy = pt.y - dragStartY
                                dragStartX = pt.x; dragStartY = pt.y
                                if (Math.abs(dx) > 2 || Math.abs(dy) > 2) dragged = true
                                mapBox.panX += dx; mapBox.panY += dy
                                mapBox.clampPan()
                            } else {
                                var nx = mouse.x / mapViewport.width
                                var ny = mouse.y / mapViewport.height
                                var hit = mapBox.hitTestCountry(nx, ny)
                                if (hit) {
                                    mapBox.hoveredHitKey = hit.hitKey || hit.code || ""
                                    mapBox.hoveredCode = hit.code || ""
                                    mapBox.hoveredName = hit.name || hit.code || ""
                                    var pin = mapBox.pinInfoFor(mapBox.hoveredCode)
                                    mapBox.hoveredNodes = pin ? pin.count : 0
                                } else {
                                    mapBox.hoveredHitKey = ""; mapBox.hoveredCode = ""; mapBox.hoveredName = ""; mapBox.hoveredNodes = 0
                                }
                            }
                        }
                        onExited: { mapBox.hoveredHitKey = ""; mapBox.hoveredCode = ""; mapBox.hoveredName = ""; mapBox.hoveredNodes = 0 }
                        onClicked: function(mouse) {
                            if (dragged) return
                            var nx = mouse.x / mapViewport.width
                            var ny = mouse.y / mapViewport.height
                            var hit = mapBox.hitTestCountry(nx, ny)
                            if (hit && hit.code && hit.code !== "-99") {
                                var cc = hit.code.toUpperCase()
                                root.selectedCountry = (root.selectedCountry === cc) ? "" : cc
                            } else {
                                root.selectedCountry = ""
                            }
                        }
                        onWheel: function(wheel) {
                            var factor = wheel.angleDelta.y > 0 ? 1.25 : 0.8
                            var boxPt = mapToItem(mapBox, wheel.x, wheel.y)
                            mapBox.setZoom(mapBox.zoomScale * factor, boxPt.x, boxPt.y)
                            wheel.accepted = true
                        }
                    }

                    // Minimal Glowing Pins on Map
                    Repeater {
                        model: pm.mapPins || []
                        delegate: Item {
                            x: modelData.nx * worldCanvas.width - width / 2
                            y: modelData.ny * worldCanvas.height - height / 2
                            width: 18; height: 18

                            Rectangle {
                                anchors.centerIn: parent
                                width: modelData.active ? 16 : (pinMa.containsMouse ? 14 : 10)
                                height: width; radius: width / 2
                                color: modelData.active ? "#1de966" : (root.selectedCountry === modelData.country ? "#fde047" : "#4e97fa")
                                border.color: "#111112"; border.width: 1.5
                                ToolTip.visible: pinMa.containsMouse
                                ToolTip.text: pm.countryName(modelData.country) + " (" + modelData.count + ")"
                                ToolTip.delay: 150
                                MouseArea {
                                    id: pinMa; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor
                                    onClicked: root.selectedCountry = modelData.country
                                }
                            }
                        }
                    }
                }

                // Floating Top-Left Connection Status Card on Map (1:1 Angular xray-page.html)
                Rectangle {
                    anchors.left: parent.left; anchors.top: parent.top; anchors.margins: 12
                    height: 36; radius: 5
                    color: "#e01a1b1f"; border.color: pm.xrayStatus.connected ? "#195a33" : "#2f313b"
                    implicitWidth: flStatusRow.implicitWidth + 20

                    RowLayout {
                        id: flStatusRow
                        anchors.centerIn: parent
                        spacing: 8
                        Rectangle { width: 8; height: 8; radius: 4; color: pm.xrayStatus.connected ? "#1de966" : "#76767e" }
                        FlagBadge {
                            visible: !!pm.xrayStatus.connected && (pm.xrayStatus.exitCountry || "") !== ""
                            code: pm.xrayStatus.exitCountry || ""
                            badgeWidth: 18; badgeHeight: 12
                        }
                        Label {
                            text: pm.xrayStatus.connected
                                  ? (pm.xrayStatus.nodeName || "Xray Connected") + ((pm.xrayStatus.exitIp || "") !== "" ? (" • " + pm.xrayStatus.exitIp) : "")
                                  : (pm.isRu ? "Xray отключен" : "Xray Disconnected")
                            color: "#ffffff"; font.pixelSize: 11; font.bold: true
                        }
                        Rectangle {
                            height: 22; radius: 3
                            color: pm.xrayStatus.connected ? "#7a1515" : "#4e97fa"
                            implicitWidth: flBtnLbl.implicitWidth + 14
                            Label {
                                id: flBtnLbl; anchors.centerIn: parent
                                text: pm.xrayStatus.connected ? pm.t("xray.disconnect") : pm.t("xray.connect")
                                color: "#ffffff"; font.pixelSize: 10; font.bold: true
                            }
                            MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: pm.toggleXrayConnect() }
                        }
                    }
                }

                // Floating Hover Country Overlay OR Legend + Zoom Buttons (Bottom-Right, shifts above drawer if open)
                Row {
                    anchors.right: parent.right
                    anchors.bottom: countryDrawer.visible ? countryDrawer.top : parent.bottom
                    anchors.margins: 12
                    spacing: 8

                    Rectangle {
                        visible: mapBox.hoveredName !== ""
                        height: 28; radius: 4; color: "#e01a1b1f"; border.color: mapBox.hoveredNodes > 0 ? "#4e97fa" : "#3a3d48"
                        implicitWidth: hovRow.implicitWidth + 16
                        RowLayout {
                            id: hovRow; anchors.centerIn: parent; spacing: 7
                            FlagBadge { visible: mapBox.hoveredCode !== "" && mapBox.hoveredCode !== "-99"; code: mapBox.hoveredCode; badgeWidth: 20; badgeHeight: 14 }
                            Label {
                                text: mapBox.hoveredName + (mapBox.hoveredNodes > 0 ? (" • " + mapBox.hoveredNodes + (pm.isRu ? " узл." : " nodes")) : "")
                                color: "#ffffff"; font.pixelSize: 11; font.bold: true
                            }
                        }
                    }

                    Row {
                        spacing: 4
                        Repeater {
                            model: [ { txt: "+", fn: "in" }, { txt: "−", fn: "out" }, { txt: "1:1", fn: "reset" } ]
                            delegate: Rectangle {
                                width: 28; height: 28; radius: 4; color: "#e01a1b1f"; border.color: "#363842"
                                Label { anchors.centerIn: parent; text: modelData.txt; color: "#ffffff"; font.pixelSize: 11; font.bold: true }
                                MouseArea {
                                    anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                                    onClicked: {
                                        if (modelData.fn === "in") mapBox.setZoom(mapBox.zoomScale * 1.35, mapBox.width / 2, mapBox.height / 2)
                                        else if (modelData.fn === "out") mapBox.setZoom(mapBox.zoomScale / 1.35, mapBox.width / 2, mapBox.height / 2)
                                        else { mapBox.zoomScale = 1.0; mapBox.panX = 0; mapBox.panY = 0 }
                                    }
                                }
                            }
                        }
                    }
                }

                // Bottom Selected Country Panel inside Map (Flag PNG + Country Name + Country Nodes!)
                Rectangle {
                    id: countryDrawer
                    visible: root.selectedCountry !== ""
                    anchors.left: parent.left; anchors.right: parent.right; anchors.bottom: parent.bottom
                    anchors.margins: 12
                    height: 158
                    radius: 6
                    color: "#eb191a1f"
                    border.color: "#4e97fa"

                    ColumnLayout {
                        anchors.fill: parent
                        anchors.margins: 12
                        spacing: 8

                        RowLayout {
                            Layout.fillWidth: true
                            spacing: 10
                            FlagBadge { code: root.selectedCountry; badgeWidth: 28; badgeHeight: 19 }
                            Label {
                                text: pm.countryName(root.selectedCountry) + " (" + root.selectedCountry + ")"
                                color: "#ffffff"; font.pixelSize: 14; font.bold: true
                            }
                            Rectangle {
                                height: 20; radius: 10; color: "#262832"
                                implicitWidth: cdCnt.implicitWidth + 12
                                Label {
                                    id: cdCnt; anchors.centerIn: parent
                                    text: root.filteredNodes(root.selectedCountry).length + (pm.isRu ? " узлов" : " nodes")
                                    color: "#89b7ff"; font.pixelSize: 10; font.bold: true
                                }
                            }
                            Item { Layout.fillWidth: true }
                            Rectangle {
                                width: 24; height: 24; radius: 4; color: "#262830"
                                Label { anchors.centerIn: parent; text: "✕"; color: "#cacaca"; font.pixelSize: 12 }
                                MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: root.selectedCountry = "" }
                            }
                        }

                        ListView {
                            Layout.fillWidth: true
                            Layout.fillHeight: true
                            orientation: ListView.Horizontal
                            clip: true
                            spacing: 10
                            model: root.filteredNodes(root.selectedCountry)

                            delegate: Rectangle {
                                property bool isAct: !!pm.xrayStatus.connected && pm.xrayStatus.nodeId === modelData.id
                                width: 235; height: ListView.view.height
                                radius: 5
                                color: isAct ? "#153020" : "#22232a"
                                border.color: isAct ? "#1de966" : "#343742"

                                ColumnLayout {
                                    anchors.fill: parent; anchors.margins: 9; spacing: 3
                                    RowLayout {
                                        Layout.fillWidth: true; spacing: 6
                                        FlagBadge { code: modelData.country || ""; badgeWidth: 20; badgeHeight: 14 }
                                        Label { text: modelData.name || modelData.address || "Node"; color: "#ffffff"; font.pixelSize: 12; font.bold: true; elide: Text.ElideRight; Layout.fillWidth: true }
                                        Rectangle {
                                            width: 22; height: 20; radius: 3; color: "#272933"; border.color: "#383b4a"
                                            Label { anchors.centerIn: parent; text: "✎"; color: "#89b7ff"; font.pixelSize: 11 }
                                            MouseArea {
                                                anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                                                onClicked: pm.openNodeEditor(modelData.id)
                                            }
                                        }
                                    }
                                    Label {
                                        text: (modelData.protocol || "vless").toUpperCase() + " • " + (modelData.address || "") + ":" + (modelData.port || "")
                                        color: "#8e8e98"; font.pixelSize: 10; font.family: "monospace"; elide: Text.ElideRight; Layout.fillWidth: true
                                    }
                                    Item { Layout.fillHeight: true }
                                    Rectangle {
                                        Layout.fillWidth: true; height: 24; radius: 4
                                        color: isAct ? "#7a1515" : "#4e97fa"
                                        Label {
                                            anchors.centerIn: parent
                                            text: isAct ? pm.t("xray.disconnect") : pm.t("xray.connect")
                                            color: "#ffffff"; font.pixelSize: 10; font.bold: true
                                        }
                                        MouseArea {
                                            anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                                            onClicked: isAct ? pm.toggleXrayConnect() : pm.connectNode(modelData.id)
                                        }
                                    }
                                }
                            }

                            Label {
                                anchors.centerIn: parent
                                visible: root.filteredNodes(root.selectedCountry).length === 0
                                text: pm.isRu ? "В этой стране пока нет добавленных узлов." : "No Xray nodes available in this country."
                                color: "#787c88"; font.pixelSize: 11
                            }
                        }
                    }
                }
            }

            // RIGHT SIDEBAR: Subscriptions, Country Chips, Server List with Country Flags (330px fixed matching Angular w-96)
            Rectangle {
                Layout.preferredWidth: 330
                Layout.fillHeight: true
                radius: 6
                color: "#16171a"
                border.color: "#25262e"

                ColumnLayout {
                    anchors.fill: parent
                    anchors.margins: 12
                    spacing: 10

                    // Subscriptions section
                    ColumnLayout {
                        Layout.fillWidth: true
                        visible: (pm.xraySubs || []).length > 0
                        spacing: 4
                        Label { text: pm.isRu ? "ПОДПИСКИ" : "SUBSCRIPTIONS"; color: "#767884"; font.pixelSize: 10; font.bold: true }
                        Repeater {
                            model: pm.xraySubs || []
                            delegate: Rectangle {
                                Layout.fillWidth: true; height: 30; radius: 4; color: "#1d1e24"; border.color: "#2a2c36"
                                RowLayout {
                                    anchors.fill: parent; anchors.leftMargin: 8; anchors.rightMargin: 8; spacing: 6
                                    Label { text: (modelData.name || "Sub") + " (" + (modelData.node_count || 0) + ")"; color: "#d4d4dc"; font.pixelSize: 11; elide: Text.ElideRight; Layout.fillWidth: true }
                                    Label { text: "↻"; color: "#4e97fa"; font.pixelSize: 13; MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: pm.refreshSubscription(modelData.id) } }
                                    Label { text: "✕"; color: "#e01d1d"; font.pixelSize: 11; MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: pm.deleteSubscription(modelData.id) } }
                                }
                            }
                        }
                    }

                    // Country Filter Chips with Flags
                    ColumnLayout {
                        Layout.fillWidth: true
                        spacing: 5
                        Label { text: pm.isRu ? "СТРАНЫ" : "COUNTRIES"; color: "#767884"; font.pixelSize: 10; font.bold: true }
                        Flow {
                            Layout.fillWidth: true
                            spacing: 4
                            Rectangle {
                                height: 24; radius: 4
                                color: root.selectedCountry === "" ? "#263348" : "#1d1e24"
                                border.color: root.selectedCountry === "" ? "#4e97fa" : "#2c2e38"
                                implicitWidth: allCcLbl.implicitWidth + 14
                                Label { id: allCcLbl; anchors.centerIn: parent; text: pm.isRu ? "Все" : "All"; color: "#ffffff"; font.pixelSize: 10; font.bold: root.selectedCountry === "" }
                                MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: root.selectedCountry = "" }
                            }
                            Repeater {
                                model: root.availableCountries()
                                delegate: Rectangle {
                                    height: 24; radius: 4
                                    color: root.selectedCountry === modelData ? "#263348" : "#1d1e24"
                                    border.color: root.selectedCountry === modelData ? "#4e97fa" : "#2c2e38"
                                    implicitWidth: ccChipRow.implicitWidth + 12
                                    RowLayout {
                                        id: ccChipRow; anchors.centerIn: parent; spacing: 5
                                        FlagBadge { code: modelData; badgeWidth: 16; badgeHeight: 11 }
                                        Label { text: modelData; color: "#ffffff"; font.pixelSize: 10; font.bold: root.selectedCountry === modelData }
                                    }
                                    MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: root.selectedCountry = (root.selectedCountry === modelData ? "" : modelData) }
                                }
                            }
                        }
                    }

                    TextField {
                        Layout.fillWidth: true; height: 30
                        placeholderText: pm.isRu ? "Поиск узла..." : "Search nodes..."
                        placeholderTextColor: "#66666e"; color: "#ffffff"; font.pixelSize: 11
                        onTextChanged: root.nodeSearch = text
                        background: Rectangle { color: "#121214"; radius: 4; border.color: parent.activeFocus ? "#4e97fa" : "#2c2c30" }
                    }

                    Label {
                        text: (pm.isRu ? "УЗЛЫ" : "NODES") + " (" + root.filteredNodes().length + ")"
                        color: "#767884"; font.pixelSize: 10; font.bold: true
                    }

                    ListView {
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        clip: true
                        cacheBuffer: 800
                        spacing: 6
                        model: root.filteredNodes()
                        ScrollBar.vertical: PmScrollBar {}
                        PmWheelHelper {}

                        delegate: Rectangle {
                            property bool isAct: !!pm.xrayStatus.connected && pm.xrayStatus.nodeId === modelData.id
                            width: ListView.view.width
                            height: 58
                            radius: 5
                            color: isAct ? "#142a1d" : (srvMa.containsMouse ? "#25262c" : "#1c1d22")
                            border.color: isAct ? "#1de966" : (srvMa.containsMouse ? "#4e97fa" : "#282932")

                            MouseArea {
                                id: srvMa; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor
                                onClicked: isAct ? pm.toggleXrayConnect() : pm.connectNode(modelData.id)
                            }

                            RowLayout {
                                anchors.fill: parent; anchors.margins: 10; spacing: 8
                                FlagBadge { code: modelData.country || ""; badgeWidth: 28; badgeHeight: 19 }
                                ColumnLayout {
                                    Layout.fillWidth: true; spacing: 2
                                    Label { text: modelData.name || modelData.address || "Node"; color: "#ffffff"; font.pixelSize: 12; font.bold: true; elide: Text.ElideRight; Layout.fillWidth: true }
                                    Label {
                                        text: (modelData.protocol || "vless").toUpperCase() + " • " + (modelData.address || "") + ":" + (modelData.port || "")
                                        color: "#7e8290"; font.pixelSize: 10; font.family: "monospace"; elide: Text.ElideRight; Layout.fillWidth: true
                                    }
                                }
                                Rectangle {
                                    visible: (modelData.pingMs || 0) > 0
                                    height: 18; radius: 3; color: "#22242b"
                                    implicitWidth: pingL.implicitWidth + 8
                                    Label { id: pingL; anchors.centerIn: parent; text: modelData.pingMs + "ms"; color: modelData.pingMs < 120 ? "#1de966" : "#e9d81d"; font.pixelSize: 9 }
                                }
                                Rectangle {
                                    width: 24; height: 24; radius: 4; color: "#252730"; border.color: "#383b4a"
                                    Label { anchors.centerIn: parent; text: "✎"; color: "#89b7ff"; font.pixelSize: 11 }
                                    MouseArea {
                                        anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                                        onClicked: pm.openNodeEditor(modelData.id)
                                    }
                                }
                            }
                        }

                        Label {
                            anchors.centerIn: parent
                            visible: root.filteredNodes().length === 0
                            text: pm.isRu ? "Узлы не найдены." : "No Xray nodes found."
                            color: "#76767e"; font.pixelSize: 12
                        }
                    }
                }
            }
        }
    }
}
