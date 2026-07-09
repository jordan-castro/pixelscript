#pragma once
#ifdef YOYO_OS
#include <pixelscript_cpp.hpp>

namespace yoyo::os {
    void init(pxs_Module* yoyo, int argc, char* argv[]);
};

#endif // YOYO_OS