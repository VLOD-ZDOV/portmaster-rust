import QtQuick
import QtQuick.Controls.Basic

CheckBox {
    id: ctrl
    implicitHeight: 22

    indicator: Rectangle {
        implicitWidth: 16
        implicitHeight: 16
        x: ctrl.leftPadding
        y: parent.height / 2 - height / 2
        radius: 3
        color: ctrl.checked ? "#4e97fa" : "#18181b"
        border.color: ctrl.checked ? "#4e97fa" : "#44444c"

        Canvas {
            anchors.fill: parent
            visible: ctrl.checked
            onPaint: {
                var ctx = getContext("2d")
                ctx.clearRect(0, 0, width, height)
                ctx.strokeStyle = "#ffffff"
                ctx.lineWidth = 2.0
                ctx.lineCap = "round"
                ctx.lineJoin = "round"
                ctx.beginPath()
                ctx.moveTo(3.5, 8.2)
                ctx.lineTo(6.6, 11.4)
                ctx.lineTo(12.5, 4.6)
                ctx.stroke()
            }
        }
    }
    contentItem: Text {
        text: ctrl.text
        font: ctrl.font
        color: "#cacaca"
        verticalAlignment: Text.AlignVCenter
        leftPadding: ctrl.indicator.width + ctrl.spacing
    }
}
