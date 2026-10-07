cmake_minimum_required(VERSION 3.16)
project(routeopt_cpp CXX)

set(CMAKE_CXX_STANDARD 14)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

add_library(routeopt_points INTERFACE)
target_include_directories(routeopt_points INTERFACE ${CMAKE_CURRENT_SOURCE_DIR}/include)

add_executable(clonecheck src/points_cli.cpp)
target_link_libraries(clonecheck PRIVATE routeopt_points)

enable_testing()
add_executable(points_test test/points_test.cpp)
target_link_libraries(points_test PRIVATE routeopt_points)
add_test(NAME points_test COMMAND points_test)
