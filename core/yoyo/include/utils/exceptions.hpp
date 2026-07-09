#pragma once

#include <pixelscript.h>
#include <vector>

namespace yoyo::utils::exceptions {
    // ARGC exception
    pxs_VarT expected_argc(int found, int expected);
    // n or more exception
    pxs_VarT expected_nm(int found, int n);
    // Expected `self` found `type`
    pxs_VarT expected_self(pxs_VarT found);
    // Expected `type` found `type`.
    pxs_VarT expected_type(pxs_VarType found, pxs_VarType expected);
    // Expected `type` or ... found `type`.
    pxs_VarT expected_types(pxs_VarType found, const std::vector<pxs_VarType>& expected);
};