#include "utils/exceptions.hpp"
#include <string>
#include <pixelscript_cpp.hpp>

namespace yoyo::utils::exceptions {
    pxs_VarT expected_argc(int found, int expected) {
        std::string message = std::string("Expected " + std::to_string(expected) + ", but got " + std::to_string(found));
        return pxs_newexception(message.c_str());
    }
    pxs_VarT expected_nm(int found, int n) {
        std::string msg = std::string("Expected " + std::to_string(n) + " or more, but got " + std::to_string(found));
        return pxs_newexception(msg.c_str());
    }
    pxs_VarT expected_self(pxs_VarT arg) {
        auto type_str = pxs::string_type(arg);
        std::string msg = std::string("Expected `self`. Found type " + type_str + " instead");
        return pxs_newexception(msg.c_str());
    }
    pxs_VarT expected_type(pxs_VarType found, pxs_VarType expected) {
        auto found_type = pxs::string_type(found);
        auto expected_type = pxs::string_type(expected);
        std::string msg = std::string("Expected " + expected_type + ", found " + found_type + " instead.");
        return pxs_newexception(msg.c_str());
    }
    pxs_VarT expected_types(pxs_VarType found, const std::vector<pxs_VarType>& expected) {
        auto found_type = pxs::string_type(found);
        std::string expected_types;
        for (const auto& t : expected) {
            expected_types += pxs::string_type(t);
            expected_types += " ";
        }
        std::string msg = std::string("Expected " + expected_types + ", found " + found_type + " instead.");
        return pxs_newexception(msg.c_str());
    }
};