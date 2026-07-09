#pragma once

#ifdef YOYO_SHELL

#include <pixelscript.h>
#include <pixelscript_cpp.hpp>

// TODO: implement a better shell.

namespace yoyo::shell {
    // Run a shell command via `std::system`
    // Output is sent to terminal directly.
    pxs_VarT system(pxs_VarT args);

    void init(pxs_Module* yoyo);
}

#endif // YOYO_SHELL