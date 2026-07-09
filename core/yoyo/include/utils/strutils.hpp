#pragma once

#include <string>
#include <algorithm>
#include <cctype>
#include <vector>
#include <string_view>

#ifdef _WIN32
#include <windows.h>
#endif // _WIN32

namespace yoyo::utils::str {
    // clone a string and convert it to lowercase.
    inline std::string to_lower(const std::string& source) {
        std::string copy = source;
        std::transform(copy.begin(), copy.end(), copy.begin(), [](unsigned char c) {
            return std::tolower(c);
        });
        return copy;
    }

    // Join a vector<string> into a string with a delimiter
    inline std::string join(const std::string& dil, const std::vector<std::string>& v) {
        std::string res;
        for (size_t i = 0; i < v.size(); i++) {
            res += v[i];
            if (i < v.size() - 1) {
                res += dil;
            }
        }
        return res;
    }

    #ifdef _WIN32
    // Convert a std::string into a wide string.
    std::wstring to_wstring(std::string_view utf8_str) {
        if (utf8_str.empty()) {
            return L"";
        }

        int required_chars = MultiByteToWideChar(
            CP_UTF8,
            0,
            utf8_str.data(),
            static_cast<int>(utf8_str.size()),
            nullptr,
            0
        );

        if (required_chars <= 0) {
            return L"";
        }

        std::wstring wide_str;
        wide_str.resize(required_chars);

        MultiByteToWideChar(
            CP_UTF8,
            0,
            utf8_str.data(),
            static_cast<int>(utf8_str.size()),
            wide_str.data(),
            required_chars
        );

        return wide_str;
    }
    #endif // _WIN32
};
