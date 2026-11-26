import QtQuick

Canvas {
    id: root
    property string cutoutColor: "#121213"

    Connections {
        target: pm
        function onConnectionChanged() { root.requestPaint() }
        function onNotificationsChanged() { root.requestPaint() }
    }

    onPaint: {
        var ctx = getContext("2d")
        ctx.clearRect(0, 0, width, height)

        var c1 = !pm.connected ? "#e01d1d" : (pm.paused ? "#e9d81d" : "#1de966")
        var c2 = !pm.connected ? "#811818" : (pm.paused ? "#827918" : "#18823d")
        var c3 = !pm.connected ? "#3d1414" : (pm.paused ? "#3d3a14" : "#143d24")

        function traceShield(c) {
            // Exact smooth rounded Angular security-lock.html shield SVG geometry in 24x24 space
            c.beginPath()
            c.moveTo(20, 11.242)
            c.bezierCurveTo(20, 15.61, 16.843, 19.704, 12.52, 20.928)
            c.bezierCurveTo(12.182, 21.024, 11.818, 21.024, 11.48, 20.928)
            c.bezierCurveTo(7.157, 19.705, 4, 15.61, 4, 11.242)
            c.lineTo(4, 7.214)
            c.bezierCurveTo(4, 6.402, 4.491, 5.67, 5.243, 5.363)
            c.lineTo(10.107, 3.373)
            c.bezierCurveTo(11.321, 2.876, 12.681, 2.876, 13.894, 3.373)
            c.lineTo(18.758, 5.363)
            c.bezierCurveTo(19.509, 5.67, 20, 6.402, 20, 7.214)
            c.closePath()
        }

        var unit = Math.min(width, height) / 24.0
        ctx.save()
        ctx.translate(width / 2, height / 2)

        function drawLayer(sc, fill, alpha) {
            ctx.save()
            ctx.scale(unit * sc, unit * sc)
            ctx.translate(-12, -12)
            traceShield(ctx)
            ctx.globalAlpha = alpha
            ctx.fillStyle = fill
            ctx.fill()
            ctx.lineJoin = "round"
            ctx.lineCap = "round"
            ctx.lineWidth = 1.4
            ctx.strokeStyle = fill
            ctx.stroke()
            ctx.restore()
        }

        // 3 layered concentric shields matching security-lock.scss scales & opacities
        drawLayer(1.0,  c3, 0.48)
        drawLayer(0.80, c2, 0.72)
        drawLayer(0.62, c1, 1.0)

        // Dark background cutout glyph inside the bright center shield
        ctx.save()
        ctx.scale(unit * 0.54, unit * 0.54)
        ctx.translate(-12, -12)
        ctx.strokeStyle = root.cutoutColor
        ctx.lineCap = "round"
        ctx.lineJoin = "round"
        ctx.lineWidth = 2.4
        if (!pm.connected) {
            ctx.beginPath()
            ctx.moveTo(14.83, 9.17); ctx.lineTo(9.17, 14.83)
            ctx.moveTo(14.83, 14.83); ctx.lineTo(9.17, 9.17)
            ctx.stroke()
        } else if (pm.paused) {
            ctx.beginPath()
            ctx.moveTo(12, 8.5); ctx.lineTo(12, 12.6)
            ctx.stroke()
            ctx.fillStyle = root.cutoutColor
            ctx.beginPath()
            ctx.arc(12, 15.2, 1.0, 0, Math.PI * 2)
            ctx.fill()
        } else {
            ctx.beginPath()
            ctx.moveTo(8.712, 12.566)
            ctx.lineTo(10.905, 14.759)
            ctx.lineTo(15.692, 9.971)
            ctx.stroke()
        }
        ctx.restore()
        ctx.restore()
    }
    Component.onCompleted: requestPaint()
}
