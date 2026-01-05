cmake_minimum_required(VERSION 3.15)

project(PixelScript LANGUAGES C)
# C flags
set(CMAKE_C_STANDARD 99)
set(CMAKE_C_STANDARD_REQUIRED true)

# dirs
include_directories(
    ${PROJECT_SOURCE_DIR}/
)

# source
add_executable(PixelTest c_tests/test.c)
set_target_properties(PixelTest PROPERTIES LINK_FLAGS "/NODEFAULTLIB:MSVCRT")

# Link with pixel_script
target_link_libraries(PixelTest PRIVATE 
"${CMAKE_CURRENT_SOURCE_DIR}/pixel_script/pixel_script.lib"
"${CMAKE_CURRENT_SOURCE_DIR}/pixel_script/lua5.4.lib"
ws2_32.lib    # Fixes: WSAStartup, socket, send, recv, etc.
userenv.lib   # Fixes: GetUserProfileDirectoryW
ntdll.lib     # Fixes: NtReadFile, NtWriteFile, NtOpenFile
bcrypt.lib    # Fixes: BCryptGenRandom (often used by Rust for hashing/ids)
advapi32.lib  # Fixes: Security/Registry calls
)

