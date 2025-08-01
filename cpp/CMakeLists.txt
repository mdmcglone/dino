cmake_minimum_required(VERSION 3.10)
project(NeonPangaea)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Find SDL2
find_package(SDL2 REQUIRED)
find_package(SDL2_ttf REQUIRED)

# Include directories
include_directories(${SDL2_INCLUDE_DIRS})
include_directories(${SDL2_TTF_INCLUDE_DIRS})
include_directories(${CMAKE_SOURCE_DIR}/include)

# Source files
set(SOURCES
    src/main.cpp
    src/pangaea.cpp
    src/renderer.cpp
)

# Create executable
add_executable(neon_pangaea ${SOURCES})

# Link libraries
target_link_libraries(neon_pangaea ${SDL2_LIBRARIES} ${SDL2_TTF_LIBRARIES})

# macOS specific
if(APPLE)
    target_link_libraries(neon_pangaea "-framework SDL2" "-framework SDL2_ttf")
endif()

# Copy resources if needed
# file(COPY resources DESTINATION ${CMAKE_BINARY_DIR}) 