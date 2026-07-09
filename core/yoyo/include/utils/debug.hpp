#pragma once

#ifdef YOYO_DEBUG

#include <iostream>
#include <sstream>
#include <string>
#include <utility>

/// Print a line to the console from FT, works in Godot and WASM.
inline void _PrintLine(const char* str) {
    std::cout << str << std::endl;
}

inline void _EPrintLine(const char* str) {
    std::cout << "ERROR: " << str << std::endl;
}

// 1. Core Variadic Formatting Function
// This template accepts any types and concatenates them into a std::string.
template <typename... Args>
std::string format_args(Args&&... args) {
    std::ostringstream ss;
    // The "Fold Expression" is used to stream all arguments into the stringstream
    (ss << ... << std::forward<Args>(args));
    return ss.str();
}

/**
 * Use this to println any kind of to_string argument.
 * 
 * Will call PrintLine automatically
 */
template <typename... Args>
void println(Args&&... args) {
    std::string formatted_str = format_args(std::forward<Args>(args)...);
    _PrintLine(formatted_str.c_str());
}

/**
 * Use this to debug print.
 */
template <typename... Args>
void dprintln(Args&&... args) {
    std::string formatted_str = std::string("DEBUG: ") + format_args(std::forward<Args>(args)...);
    _PrintLine(formatted_str.c_str());
}

/**
 * Use this to error print.
 */
template <typename... Args>
void eprintln(Args&&... args) {
    std::string formatted_str = format_args(std::forward<Args>(args)...);
    _EPrintLine(formatted_str.c_str());
}

/**
 * Use this for debug print.
 */
#define D_PRINTLN(msg) dprintln(__func__, msg)

#endif // YOYO_DEBUG