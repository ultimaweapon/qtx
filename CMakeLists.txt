# Global options.
cmake_minimum_required(VERSION 3.20)

project(qtx)

if(WIN32)
    add_compile_options(/W4)
else()
    add_compile_options(-Wall -Wextra)
endif()

find_package(Qt6 REQUIRED COMPONENTS Widgets)

# Build FFI.
add_library(qtx STATIC
    src/app.cpp
    src/executor.cpp
    src/lib.cpp
    src/mem.cpp
    src/wake_task.cpp
    src/windows/main.cpp)

set_target_properties(qtx PROPERTIES AUTOMOC ON AUTORCC ON)

target_compile_features(qtx PRIVATE cxx_std_17)

target_link_libraries(qtx PRIVATE Qt6::Widgets)

# Install FFI.
install(TARGETS qtx ARCHIVE)
