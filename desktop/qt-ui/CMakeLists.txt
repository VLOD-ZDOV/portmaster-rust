cmake_minimum_required(VERSION 3.16)
project(portmaster-qt VERSION 2.2.38 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTORCC ON)
set(CMAKE_POSITION_INDEPENDENT_CODE ON)

include(CheckCXXCompilerFlag)
if(UNIX AND NOT APPLE)
    # Emit GOT-indirect accesses instead of R_X86_64_COPY so binaries built on
    # older toolchains run cleanly on distros with Qt6 protected-symbol checks.
    check_cxx_compiler_flag("-mno-direct-extern-access" HAS_NO_DIRECT_EXTERN_ACCESS)
    check_cxx_compiler_flag("-fno-direct-access-external-data" HAS_NO_DIRECT_ACCESS_EXTERNAL_DATA)
endif()

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

if(UNIX AND NOT APPLE)
    target_compile_options(portmaster-qt PRIVATE
        -fPIC
        $<$<BOOL:${HAS_NO_DIRECT_EXTERN_ACCESS}>:-mno-direct-extern-access>
        $<$<BOOL:${HAS_NO_DIRECT_ACCESS_EXTERNAL_DATA}>:-fno-direct-access-external-data>
    )
    target_link_options(portmaster-qt PRIVATE -Wl,-z,nocopyreloc)
endif()

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
