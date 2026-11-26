import QtQuick
import QtQuick.Controls.Basic

ScrollBar {
    id: sb
    implicitWidth: 8
    implicitHeight: 8
    policy: ScrollBar.AsNeeded

    background: Rectangle {
        color: "transparent"
    }

    contentItem: Rectangle {
        implicitWidth: 6
        implicitHeight: 6
        radius: 3
        color: sb.pressed ? "#686870" : (sb.hovered ? "#4a4a52" : "#323238")
        Behavior on color { ColorAnimation { duration: 120 } }
    }
}
