import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

RowLayout {
    id: hdr
    Layout.fillWidth: true
    spacing: compact ? 6 : 8

    property string title: ""
    property bool showInternetToggle: false
    property bool internetOnly: false
    property int okCount: 0
    property int blockedCount: 0
    property string rxStr: "0 B"
    property string txStr: "0 B"
    property bool compact: false

    signal toggleInternet()

    Label {
        text: hdr.title
        color: hdr.compact ? "#7e8290" : "#dcdce4"
        font.pixelSize: hdr.compact ? 10 : 12
        font.bold: true
    }

    Rectangle {
        visible: hdr.showInternetToggle
        height: hdr.compact ? 19 : 22; radius: 4
        color: hdr.internetOnly ? "#173252" : "#222328"
        border.color: hdr.internetOnly ? "#4e97fa" : "#343640"
        implicitWidth: inetLbl.implicitWidth + (hdr.compact ? 10 : 14)
        Label {
            id: inetLbl; anchors.centerIn: parent
            text: pm.isRu ? "Только интернет" : "Internet Only"
            color: hdr.internetOnly ? "#89b7ff" : "#8e8e98"
            font.pixelSize: hdr.compact ? 9 : 10; font.bold: hdr.internetOnly
        }
        MouseArea {
            anchors.fill: parent; cursorShape: Qt.PointingHandCursor
            onClicked: hdr.toggleInternet()
        }
    }

    // Connections vs Data volume switcher pill
    Rectangle {
        height: hdr.compact ? 19 : 22
        radius: 4
        color: "#141519"
        border.color: "#2c2f3a"
        implicitWidth: modeRow.implicitWidth + 4

        Row {
            id: modeRow
            anchors.centerIn: parent
            spacing: 2

            Repeater {
                model: [
                    { data: false, labelRu: "Соединения", labelEn: "Connections" },
                    { data: true,  labelRu: "Данные",     labelEn: "Data" }
                ]
                delegate: Rectangle {
                    height: hdr.compact ? 15 : 18
                    radius: 3
                    implicitWidth: mLbl.implicitWidth + 10
                    color: (pm.chartDataMode === modelData.data) ? "#1d3557" : "transparent"
                    Label {
                        id: mLbl; anchors.centerIn: parent
                        text: pm.isRu ? modelData.labelRu : modelData.labelEn
                        color: (pm.chartDataMode === modelData.data) ? "#ffffff" : "#7e8290"
                        font.pixelSize: hdr.compact ? 9 : 10
                        font.bold: pm.chartDataMode === modelData.data
                    }
                    MouseArea {
                        anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                        onClicked: pm.setChartDataMode(modelData.data)
                    }
                }
            }
        }
    }

    Item { Layout.fillWidth: true }

    Rectangle {
        width: hdr.compact ? 7 : 8
        height: hdr.compact ? 7 : 8
        radius: hdr.compact ? 3.5 : 4
        color: "#2be876"
    }
    Label {
        text: pm.chartDataMode
              ? ("↓ " + hdr.rxStr)
              : ("OK: " + hdr.okCount)
        color: "#2be876"
        font.pixelSize: hdr.compact ? 10 : 11
        font.bold: true
    }

    Item { width: hdr.compact ? 4 : 8 }

    Rectangle {
        width: hdr.compact ? 7 : 8
        height: hdr.compact ? 7 : 8
        radius: hdr.compact ? 3.5 : 4
        color: pm.chartDataMode ? "#4e97fa" : "#f03c3c"
    }
    Label {
        text: pm.chartDataMode
              ? ("↑ " + hdr.txStr)
              : ((pm.isRu ? "Блок: " : "Blocked: ") + hdr.blockedCount)
        color: pm.chartDataMode ? "#4e97fa" : "#f03c3c"
        font.pixelSize: hdr.compact ? 10 : 11
        font.bold: true
    }
}
