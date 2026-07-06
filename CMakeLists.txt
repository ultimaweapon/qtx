cmake_minimum_required(VERSION 3.20)

project(qtx)

if(WIN32)
    add_compile_options(/W4)
else()
    add_compile_options(-Wall -Wextra)
endif()

find_package(Qt6 REQUIRED COMPONENTS Widgets)

add_library(qtx STATIC
    src/application.cpp)

set_target_properties(qtx PROPERTIES AUTOMOC ON AUTORCC ON)

target_compile_features(qtx PRIVATE cxx_std_17)
