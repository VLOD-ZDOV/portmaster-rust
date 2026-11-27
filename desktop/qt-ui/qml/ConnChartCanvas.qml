import QtQuick

Canvas {
    id: root
    property var bins: pm.connChartBins || []
    property bool compact: false
    property bool showAxes: !compact
    property bool dataMode: false
    property int hoverIndex: -1

    signal chartClicked()

    property bool activePaint: visible && (!Window.window || Window.window.visible)
    onBinsChanged: if (activePaint) requestPaint()
    onWidthChanged: if (activePaint) requestPaint()
    onHeightChanged: if (activePaint) requestPaint()
    onDataModeChanged: if (activePaint) requestPaint()
    onShowAxesChanged: if (activePaint) requestPaint()
    onHoverIndexChanged: if (activePaint) requestPaint()
    onActivePaintChanged: if (activePaint) requestPaint()

    function fmtBytes(b) {
        if (b >= 1073741824) return (b / 1073741824).toFixed(1) + " GB"
        if (b >= 1048576) return (b / 1048576).toFixed(1) + " MB"
        if (b >= 1024) return Math.round(b / 1024) + " KB"
        return Math.round(b) + " B"
    }

    function computePlotBox() {
        var padL = root.showAxes ? (root.dataMode ? 56 : 36) : 0
        var padR = root.showAxes ? 14 : 0
        var padT = root.showAxes ? (root.compact ? 6 : 8) : 4
        var padB = root.showAxes ? 18 : 4
        var plotW = Math.max(12, width - padL - padR)
        var plotH = Math.max(10, height - padT - padB)
        return { l: padL, t: padT, w: plotW, h: plotH, r: padL + plotW, b: padT + plotH }
    }

    onPaint: {
        var ctx = getContext("2d")
        ctx.clearRect(0, 0, width, height)
        if (width <= 8 || height <= 8) return

        var list = root.bins || []
        var n = list.length > 0 ? list.length : 36
        var box = computePlotBox()

        var fieldA = root.dataMode ? "rxBytes" : "allowed"
        var fieldB = root.dataMode ? "txBytes" : "blocked"

        var rawMax = 0
        for (var i = 0; i < list.length; i++) {
            var va = list[i][fieldA] || 0
            var vb = list[i][fieldB] || 0
            if (va > rawMax) rawMax = va
            if (vb > rawMax) rawMax = vb
        }

        var tickMax
        if (root.dataMode) {
            tickMax = rawMax > 0 ? rawMax : 1024
        } else {
            tickMax = rawMax <= 1 ? 1 : (Math.ceil(rawMax / 2) * 2)
        }

        var stepX = box.w / Math.max(1, n - 1)

        // Horizontal Y-axis gridlines & labels
        ctx.lineWidth = 1
        ctx.strokeStyle = "#22242c"
        ctx.beginPath()
        ctx.moveTo(box.l, box.t + box.h * 0.5)
        ctx.lineTo(box.r, box.t + box.h * 0.5)
        ctx.moveTo(box.l, box.t)
        ctx.lineTo(box.r, box.t)
        ctx.stroke()

        // Vertical time gridlines + X-axis timestamp ticks
        if (root.showAxes) {
            var tickIndices = box.w >= 540
                ? [0, 7, 14, 21, 28, n - 1]
                : (box.w >= 280 ? [0, Math.floor((n - 1) / 3), Math.floor(2 * (n - 1) / 3), n - 1] : [0, Math.floor((n - 1) / 2), n - 1])

            ctx.strokeStyle = "#1e2028"
            ctx.beginPath()
            for (var ti = 0; ti < tickIndices.length; ti++) {
                var tx = box.l + tickIndices[ti] * stepX
                ctx.moveTo(tx, box.t)
                ctx.lineTo(tx, box.b + 3)
            }
            ctx.stroke()

            ctx.fillStyle = "#7a7d8a"
            ctx.font = "10px sans-serif"
            ctx.textAlign = "right"
            ctx.fillText(root.dataMode ? "0 B" : "0", box.l - 6, box.b + 3)
            if (root.dataMode || tickMax >= 2) {
                var midLabel = root.dataMode ? fmtBytes(tickMax * 0.5) : String(Math.round(tickMax * 0.5))
                ctx.fillText(midLabel, box.l - 6, box.t + box.h * 0.5 + 3)
            }
            var topLabel = root.dataMode ? fmtBytes(tickMax) : String(tickMax)
            ctx.fillText(topLabel, box.l - 6, box.t + 7)

            for (var tj = 0; tj < tickIndices.length; tj++) {
                var idx = tickIndices[tj]
                var tStr = (idx < list.length && list[idx].timeLabel) ? list[idx].timeLabel : ""
                if (tStr === "") continue
                var lx = box.l + idx * stepX
                ctx.textAlign = (tj === 0) ? "left" : (tj === tickIndices.length - 1 ? "right" : "center")
                ctx.fillText(tStr, lx, height - 3)
            }
        }

        // Baseline slightly stronger than inner gridlines
        ctx.strokeStyle = "#2b2e38"
        ctx.beginPath()
        ctx.moveTo(box.l, box.b)
        ctx.lineTo(box.r, box.b)
        ctx.stroke()

        // Smooth clipped area curves
        ctx.save()
        ctx.beginPath()
        ctx.rect(box.l, box.t - 2, box.w, box.h + 3)
        ctx.clip()

        function drawArea(field, lineColor, fillTop, fillBot) {
            ctx.beginPath()
            for (var j = 0; j < n; j++) {
                var val = j < list.length ? (list[j][field] || 0) : 0
                var x = box.l + j * stepX
                var y = box.b - (val / tickMax) * box.h
                if (j === 0) ctx.moveTo(x, y)
                else {
                    var px = box.l + (j - 1) * stepX
                    var pval = (j - 1) < list.length ? (list[j - 1][field] || 0) : 0
                    var py = box.b - (pval / tickMax) * box.h
                    var cx = (px + x) / 2
                    ctx.bezierCurveTo(cx, py, cx, y, x, y)
                }
            }
            ctx.strokeStyle = lineColor
            ctx.lineWidth = (root.compact && !root.showAxes) ? 1.5 : 2.0
            ctx.stroke()

            ctx.lineTo(box.r, box.b)
            ctx.lineTo(box.l, box.b)
            ctx.closePath()
            var grad = ctx.createLinearGradient(0, box.t, 0, box.b)
            grad.addColorStop(0, fillTop)
            grad.addColorStop(1, fillBot)
            ctx.fillStyle = grad
            ctx.fill()
        }

        if (root.dataMode) {
            drawArea("rxBytes", "#2be876", "#381de966", "#051de966")
            drawArea("txBytes", "#4e97fa", "#444e97fa", "#054e97fa")
        } else {
            drawArea("allowed", "#2be876", "#381de966", "#051de966")
            drawArea("blocked", "#f03c3c", "#44e01d1d", "#05e01d1d")
        }
        ctx.restore()

        // Interactive hover cursor & tooltip
        if (root.hoverIndex >= 0 && root.hoverIndex < list.length) {
            var hi = root.hoverIndex
            var hx = box.l + hi * stepX
            var pt = list[hi]
            ctx.strokeStyle = "#4e97fa"
            ctx.lineWidth = 1
            ctx.beginPath()
            ctx.moveTo(hx, box.t)
            ctx.lineTo(hx, box.b)
            ctx.stroke()

            var vaH = pt[fieldA] || 0
            var vbH = pt[fieldB] || 0
            var yaH = box.b - (vaH / tickMax) * box.h
            var ybH = box.b - (vbH / tickMax) * box.h

            ctx.fillStyle = "#2be876"
            ctx.beginPath(); ctx.arc(hx, yaH, 3.2, 0, Math.PI * 2); ctx.fill()
            ctx.fillStyle = root.dataMode ? "#4e97fa" : "#f03c3c"
            ctx.beginPath(); ctx.arc(hx, ybH, 3.2, 0, Math.PI * 2); ctx.fill()

            var tTxt = pt.timeLabel ? (pt.timeLabel + "  •  ") : ""
            var infoTxt = root.dataMode
                ? (tTxt + "↓ " + fmtBytes(pt.rxBytes || 0) + "   ↑ " + fmtBytes(pt.txBytes || 0) + "   (" + (pt.total || 0) + ")")
                : (tTxt + "OK: " + (pt.allowed || 0) + "   Blk: " + (pt.blocked || 0) + "   •   " + fmtBytes(pt.dataBytes || 0))

            ctx.font = "10px sans-serif"
            var tw = ctx.measureText(infoTxt).width + 14
            var txBox = Math.min(Math.max(box.l + 2, hx - tw / 2), box.r - tw - 2)
            var tyBox = box.t + 2
            ctx.fillStyle = "rgba(18, 19, 24, 0.92)"
            ctx.strokeStyle = "#3a4052"
            ctx.lineWidth = 1
            ctx.beginPath()
            ctx.roundedRect(txBox, tyBox, tw, 18, 4, 4)
            ctx.fill()
            ctx.stroke()
            ctx.fillStyle = "#eef0f6"
            ctx.textAlign = "left"
            ctx.fillText(infoTxt, txBox + 7, tyBox + 12)
        }
    }

    MouseArea {
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        onPositionChanged: function(mouse) {
            if (Qt.application.arguments.indexOf("--screenshot") >= 0) return
            var list = root.bins || []
            if (list.length <= 1) return
            var box = root.computePlotBox()
            if (mouse.x < box.l || mouse.x > box.r || mouse.y < box.t || mouse.y > box.b) {
                root.hoverIndex = -1
                return
            }
            var idx = Math.round(((mouse.x - box.l) / box.w) * (list.length - 1))
            root.hoverIndex = Math.max(0, Math.min(list.length - 1, idx))
        }
        onExited: root.hoverIndex = -1
        onClicked: root.chartClicked()
    }

    Component.onCompleted: requestPaint()
}
