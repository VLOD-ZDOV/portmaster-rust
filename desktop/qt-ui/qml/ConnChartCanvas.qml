import QtQuick

Canvas {
    id: root
    property var bins: pm.connChartBins || []
    property bool compact: false

    onBinsChanged: requestPaint()
    onWidthChanged: requestPaint()
    onHeightChanged: requestPaint()

    onPaint: {
        var ctx = getContext("2d")
        ctx.clearRect(0, 0, width, height)
        if (width <= 8 || height <= 8) return

        var list = root.bins || []
        var n = list.length > 0 ? list.length : 36
        var maxV = 1
        for (var i = 0; i < list.length; i++) {
            var t = (list[i].allowed || 0) + (list[i].blocked || 0)
            if (t > maxV) maxV = t
        }

        // Subtle baseline grid lines
        ctx.strokeStyle = "#24262d"
        ctx.lineWidth = 1
        var padY = root.compact ? 4 : 8
        var plotH = height - padY * 2
        ctx.beginPath()
        ctx.moveTo(0, height - padY)
        ctx.lineTo(width, height - padY)
        if (!root.compact) {
            ctx.moveTo(0, padY + plotH * 0.5)
            ctx.lineTo(width, padY + plotH * 0.5)
        }
        ctx.stroke()

        // Smooth area curves (green allowed + red blocked) + thin subtle vertical bars
        var stepX = width / Math.max(1, n - 1)
        function drawArea(field, lineColor, fillTop, fillBot) {
            ctx.beginPath()
            for (var j = 0; j < n; j++) {
                var val = j < list.length ? (list[j][field] || 0) : 0
                var x = j * stepX
                var y = height - padY - (val / maxV) * plotH
                if (j === 0) ctx.moveTo(x, y)
                else {
                    var px = (j - 1) * stepX
                    var pval = (j - 1) < list.length ? (list[j - 1][field] || 0) : 0
                    var py = height - padY - (pval / maxV) * plotH
                    var cx = (px + x) / 2
                    ctx.bezierCurveTo(cx, py, cx, y, x, y)
                }
            }
            ctx.strokeStyle = lineColor
            ctx.lineWidth = root.compact ? 1.5 : 2.0
            ctx.stroke()

            ctx.lineTo(width, height - padY)
            ctx.lineTo(0, height - padY)
            ctx.closePath()
            var grad = ctx.createLinearGradient(0, padY, 0, height - padY)
            grad.addColorStop(0, fillTop)
            grad.addColorStop(1, fillBot)
            ctx.fillStyle = grad
            ctx.fill()
        }

        drawArea("allowed", "#2be876", "#381de966", "#051de966")
        drawArea("blocked", "#f03c3c", "#44e01d1d", "#05e01d1d")
    }
    Component.onCompleted: requestPaint()
}
