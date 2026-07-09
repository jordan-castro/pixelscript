#pragma once

#include <pixelscript.h>

// Helpful utilities for PXS support.
namespace yoyo::utils::pxs {
    template<typename T>
    pxs_VarT enum_to_int(T e) {
        return pxs_newint(static_cast<int>(e));
    }

    template<typename T>
    T* get_type(pxs_VarT args, int index, int type) {
        return static_cast<T*>(pxs_gettype(pxs_getrt(args), pxs_arg(args, index), type));
    }
};