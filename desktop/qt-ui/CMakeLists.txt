cmake_minimum_required(VERSION 3.16)
project(portmaster-qt VERSION 2.2.36 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTORCC ON)

find_package(Qt6 REQUIRED COMPONENTS
    Core
    Gui
    Qml
    Quick
    QuickControls2
    Widgets
    Network
    WebSockets
)

add_executable(portmaster-qt
    src/main.cpp
    src/PortApiClient.h
    src/PortApiClient.cpp
    src/PortmasterBackend.h
    src/PortmasterBackend.cpp
    resources.qrc
)

target_link_libraries(portmaster-qt PRIVATE
    Qt6::Core
    Qt6::Gui
    Qt6::Qml
    Qt6::Quick
    Qt6::QuickControls2
    Qt6::Widgets
    Qt6::Network
    Qt6::WebSockets
)

install(TARGETS portmaster-qt RUNTIME DESTINATION bin)
