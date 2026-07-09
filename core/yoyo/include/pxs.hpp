#pragma once
#ifdef YOYO_PXS

#include <pixelscript_cpp.hpp>
#include "utils/debug.hpp"

// ipxs = internal pxs module.
// since pixelscript_cpp already uses `pxs` as a module.

namespace yoyo::ipxs {
    // Execute code
    // Null means it was executed succesfully.
    pxs_VarT exec(pxs_VarT args);

    void init(pxs_Module* yoyo);
};

#endif // YOYO_PXS