#pragma once
#ifdef YOYO_OS
#include <pixelscript_cpp.hpp>

namespace yoyo::os {
    void init(pxs_Module* yoyo, pxs_VarT argv);
};

#endif // YOYO_OS