import QtQuick
import QtQuick.Controls.Basic

Switch {
    id: ctrl
    implicitWidth: 40
    implicitHeight: 22

    indicator: Rectangle {
        implicitWidth: 40
        implicitHeight: 22
        x: ctrl.leftPadding
        y: parent.height / 2 - height / 2
        radius: 11
        color: ctrl.checked ? "#4e97fa" : "#2c2c30"
        border.color: ctrl.checked ? "#6aa9ff" : "#44444a"

        Behavior on color { ColorAnimation { duration: 130 } }

        Rectangle {
            x: ctrl.checked ? parent.width - width - 3 : 3
            y: (parent.height - height) / 2
            width: 16
            height: 16
            radius: 8
            color: "#ffffff"
            Behavior on x { NumberAnimation { duration: 130; easing.type: Easing.OutCubic } }
        }
    }
}
