cmake_minimum_required(VERSION 3.21)
project(fcitx5-lekhani VERSION 3.0.0 LANGUAGES C CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

find_package(Fcitx5Core REQUIRED)

get_filename_component(WORKSPACE_ROOT "${CMAKE_CURRENT_SOURCE_DIR}/../.." ABSOLUTE)

include_directories(
    ${CMAKE_CURRENT_SOURCE_DIR}/src
    ${WORKSPACE_ROOT}/crates/lekhani-ffi/include
)

add_library(fcitx5-lekhani SHARED
    src/lekhani_addon.cpp
)

target_link_libraries(fcitx5-lekhani PRIVATE
    Fcitx5::Core
    ${WORKSPACE_ROOT}/target/release/liblekhani_ffi.a
    pthread
    dl
    m
)

set_target_properties(fcitx5-lekhani PROPERTIES PREFIX "")

if(NOT DEFINED FCITX_INSTALL_LIBDIR)
    set(FCITX_INSTALL_LIBDIR "${CMAKE_INSTALL_LIBDIR}")
endif()

if(NOT DEFINED FCITX_INSTALL_PKGDATADIR)
    set(FCITX_INSTALL_PKGDATADIR "${CMAKE_INSTALL_DATADIR}/fcitx5")
endif()

install(TARGETS fcitx5-lekhani DESTINATION "${FCITX_INSTALL_LIBDIR}/fcitx5")
