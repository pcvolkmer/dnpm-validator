cmake_minimum_required(VERSION 3.31)
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

find_program(CARGO cargo REQUIRED)
include(FetchContent)
FetchContent_Declare(
        Corrosion
        GIT_REPOSITORY https://github.com/corrosion-rs/corrosion.git
        GIT_TAG v0.6
)
FetchContent_MakeAvailable(Corrosion)
corrosion_import_crate(
        MANIFEST_PATH ../dnpmvalidation/Cargo.toml
        CRATE_TYPES staticlib
        PROFILE release
)
corrosion_add_cxxbridge(dnpmvalidation-cxx
        CRATE dnpmvalidation
        FILES lib.rs
)

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

target_link_libraries(dnpm-validator-ui PRIVATE
        Qt::Core
        Qt::Gui
        Qt::Widgets
        dnpmvalidation-cxx
        ${WIN_LIBS}
)

target_include_directories(dnpm-validator-ui PRIVATE
        ${CMAKE_CURRENT_SOURCE_DIR}
        ${CMAKE_CURRENT_BINARY_DIR}
)

if (CMAKE_BUILD_TYPE STREQUAL "Release")
    add_custom_command(TARGET dnpm-validator-ui POST_BUILD
            COMMAND ${CMAKE_STRIP} "$<TARGET_FILE:dnpm-validator-ui>"
            COMMENT "Stripping dnpm-validator-ui"
    )
endif ()

# Windows Packaging
# Find local cross compile resources
if (WIN32 AND MINGW)
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
endif (WIN32 AND MINGW)
# Find resources from winqtdeploy
if (WIN32 AND NOT MINGW)
    install(TARGETS dnpm-validator-ui DESTINATION .)
    install(FILES ${CMAKE_CURRENT_SOURCE_DIR}/../LICENSE DESTINATION .)
endif (WIN32 AND NOT MINGW)

# Linux packaging
if (UNIX AND NOT APPLE)
    install(TARGETS dnpm-validator-ui DESTINATION /usr/local/bin/)
    install(FILES ${CMAKE_CURRENT_SOURCE_DIR}/src/resources/dnpm-validator.svg DESTINATION /usr/local/share/icons/)
    install(FILES ${CMAKE_CURRENT_SOURCE_DIR}/src/resources/dnpm-validator.desktop DESTINATION /usr/local/share/applications/)
endif (UNIX AND NOT APPLE)

include(InstallRequiredSystemLibraries)

set(CPACK_PACKAGE_NAME ${PROJECT_NAME})
set(CPACK_PACKAGE_VENDOR "Paul-Christian Volkmer")
set(CPACK_PACKAGE_VERSION "0.1.0")
set(CPACK_PACKAGE_FILE_NAME
        "${CPACK_PACKAGE_NAME}-${CPACK_PACKAGE_VERSION}"
)
set(CPACK_PACKAGE_INSTALL_DIRECTORY ${PROJECT_NAME})
set(CPACK_GENERATOR "NSIS")
set(CPACK_RESOURCE_FILE_LICENSE ${CMAKE_CURRENT_SOURCE_DIR}/../LICENSE)
set(CPACK_DEBIAN_PACKAGE_MAINTAINER "Paul-Christian Volkmer")
set(CPACK_DEBIAN_PACKAGE_LICENSE "GPLv3")
set(CPACK_DEBIAN_PACKAGE_DESCRIPTION "Application to edit and validate a data set in DNPM Datenmodell 2.1, SE:dip data model and GRZ Metadata format")
set(CPACK_RPM_PACKAGE_LICENSE "GPLv3")
set(CPACK_RPM_PACKAGE_DESCRIPTION "Application to edit and validate a data set in DNPM Datenmodell 2.1, SE:dip data model and GRZ Metadata format")
set(CPACK_NSIS_PACKAGE_NAME "DNPM-Validator")
set(CPACK_NSIS_DISPLAY_NAME "DNPM-Validator")
set(CPACK_NSIS_INSTALLED_ICON_NAME "${PROJECT_NAME}.exe")
set(CPACK_NSIS_DISPLAY_NAME "DNPM-Validator")
set(CPACK_NSIS_MENU_LINKS "${PROJECT_NAME}.exe" "DNPM-Validator")
include(CPack)