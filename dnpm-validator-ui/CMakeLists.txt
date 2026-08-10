cmake_minimum_required(VERSION 4.2)
project(dnpm-validator-ui LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 23)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTOUIC ON)
set(CMAKE_AUTORCC ON)

find_package(Qt5 REQUIRED COMPONENTS
        Core
        Gui
        Widgets)

include_directories(${CMAKE_CURRENT_SOURCE_DIR}/libs)

qt5_add_resources(RESOURCES
        src/icons.qrc
)

add_executable(dnpm-validator-ui WIN32 src/main.cpp
        src/mainwindow.cpp
        src/mainwindow.h
        src/mainwindow.ui
        src/icons.qrc
        ${RESOURCES}
)

if (WIN32)
    set(WIN_LIBS
            ws2_32
            ntdll
            userenv
    )
endif (WIN32)

target_link_libraries(dnpm-validator-ui PUBLIC
        Qt::Core
        Qt::Gui
        Qt::Widgets
        ${CMAKE_CURRENT_BINARY_DIR}/../libs/libdnpmvalidation.a
        ${WIN_LIBS}
)

target_include_directories(dnpm-validator-ui PRIVATE
        ${CMAKE_CURRENT_SOURCE_DIR}
        ${CMAKE_CURRENT_BINARY_DIR}
)

message("${CMAKE_CURRENT_SOURCE_DIR}/../LICENSE")

if (WIN32)
    install(TARGETS dnpm-validator-ui DESTINATION .)
    install(FILES ${CMAKE_CURRENT_SOURCE_DIR}/../LICENSE DESTINATION .)
    set(WIN_LIBS
            # This part should contain DLL files to be included in package e.g.:
            ${Qt5_DIR}/../../../bin/libbz2-1.dll
            ${Qt5_DIR}/../../../bin/libfreetype-6.dll
            ${Qt5_DIR}/../../../bin/libgcc_s_seh-1.dll
            ${Qt5_DIR}/../../../bin/libGLESv2Qt5.dll
            ${Qt5_DIR}/../../../bin/libglib-2.0-0.dll
            ${Qt5_DIR}/../../../bin/libharfbuzz-0.dll
            ${Qt5_DIR}/../../../bin/libiconv.dll
            ${Qt5_DIR}/../../../bin/libicudt74.dll
            ${Qt5_DIR}/../../../bin/libicuin74.dll
            ${Qt5_DIR}/../../../bin/libicuuc74.dll
            ${Qt5_DIR}/../../../bin/libintl-8.dll
            ${Qt5_DIR}/../../../bin/libpcre2-8-0.dll
            ${Qt5_DIR}/../../../bin/libpcre2-16-0.dll
            ${Qt5_DIR}/../../../bin/libpng16-16.dll
            ${Qt5_DIR}/../../../bin/libQt5Core.dll
            ${Qt5_DIR}/../../../bin/libQt5Gui.dll
            ${Qt5_DIR}/../../../bin/libQt5Widgets.dll
            ${Qt5_DIR}/../../../bin/libstdc++-6.dll
            ${Qt5_DIR}/../../../bin/libwinpthread-1.dll
            ${Qt5_DIR}/../../../bin/libz.dll
            ${Qt5_DIR}/../../../bin/libzstd.dll
    )
    install(FILES ${WIN_LIBS}
            DESTINATION .
    )
    install(FILES ${Qt5_DIR}/../../qt5/plugins/platforms/libqwindows.dll DESTINATION ./platforms)
    install(FILES ${Qt5_DIR}/../../qt5/plugins/platformthemes/libqxdgdesktopportal.dll DESTINATION ./platformthemes)
    install(FILES ${Qt5_DIR}/../../qt5/plugins/styles/libqwindowsvistastyle.dll DESTINATION ./styles)
ENDIF (WIN32)

include(InstallRequiredSystemLibraries)

set(CPACK_PACKAGE_NAME ${PROJECT_NAME})
set(CPACK_PACKAGE_VENDOR "Paul-Christian Volkmer")
set(CPACK_NSIS_PACKAGE_NAME "DNPM-Validator")
set(CPACK_NSIS_DISPLAY_NAME "DNPM-Validator")
set(CPACK_PACKAGE_VERSION "0.1.0")
set(CPACK_PACKAGE_INSTALL_DIRECTORY ${PROJECT_NAME})
set(CPACK_GENERATOR "NSIS")
set(CPACK_RESOURCE_FILE_LICENSE ${CMAKE_CURRENT_SOURCE_DIR}/../LICENSE)
set(CPACK_NSIS_INSTALLED_ICON_NAME "${PROJECT_NAME}.exe")
set(CPACK_NSIS_DISPLAY_NAME "DNPM-Validator")
set(CPACK_NSIS_MENU_LINKS "${PROJECT_NAME}.exe" "DNPM-Validator")
include(CPack)