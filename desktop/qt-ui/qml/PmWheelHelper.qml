import QtQuick

Item {
    id: helper
    property var flickable: (parent && parent.contentY !== undefined)
                            ? parent
                            : ((parent && parent.contentItem && parent.contentItem.contentY !== undefined) ? parent.contentItem : null)
    property real targetY: 0
    property bool animating: false
    anchors.fill: parent

    NumberAnimation {
        id: wheelAnim
        target: helper.flickable
        property: "contentY"
        duration: 145
        easing.type: Easing.OutCubic
        onFinished: helper.animating = false
    }

    WheelHandler {
        target: null
        acceptedDevices: PointerDevice.Mouse
        onWheel: function(e) {
            var f = helper.flickable
            if (!f || f.contentHeight <= f.height) {
                e.accepted = false
                return
            }
            if (e.pixelDelta && Math.abs(e.pixelDelta.y) > 0) {
                e.accepted = false
                return
            }
            e.accepted = true
            var originY = f.originY || 0
            var maxY = originY + Math.max(0, f.contentHeight - f.height)
            if (!helper.animating) helper.targetY = f.contentY
            var step = (e.angleDelta.y / 120.0) * -96.0
            helper.targetY = Math.max(originY, Math.min(maxY, helper.targetY + step))
            wheelAnim.from = f.contentY
            wheelAnim.to = helper.targetY
            helper.animating = true
            wheelAnim.restart()
        }
    }
}
