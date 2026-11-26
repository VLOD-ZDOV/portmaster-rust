import QtQuick
import QtQuick.Controls

Rectangle {
    id: root
    property string code: ""
    property int badgeWidth: 24
    property int badgeHeight: 16
    property string normCode: (code || "").trim().toUpperCase()
    property bool hasFlag: flagUrl !== ""
    property string flagUrl: pm.countryFlagUrl(normCode)

    width: badgeWidth
    height: badgeHeight
    radius: 3
    color: hasFlag ? "#15161a" : "#252730"
    border.color: hasFlag ? "#363945" : "#343742"
    border.width: 1
    clip: true

    Image {
        id: flagImg
        anchors.fill: parent
        anchors.margins: 1
        fillMode: Image.PreserveAspectCrop
        smooth: true
        asynchronous: false
        source: root.flagUrl
        visible: root.hasFlag
    }

    Label {
        visible: !root.hasFlag
        anchors.centerIn: parent
        text: root.normCode !== "" ? root.normCode.slice(0, 2) : "??"
        color: "#b4b8c8"
        font.pixelSize: Math.max(8, Math.round(root.height * 0.56))
        font.bold: true
    }
}
