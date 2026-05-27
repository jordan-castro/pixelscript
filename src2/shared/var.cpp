#include "shared/var.hpp"

namespace pxs::var {
    std::unique_ptr<pxs_VarObject> make_as_host(pxs_Opaque val, int32_t host_idx) {
        return std::make_unique<pxs_VarObject>(val, host_idx);
    }

    std::unique_ptr<pxs_VarObject> make_lang_only(pxs_Opaque val) {
        return std::make_unique<pxs_VarObject>(val, -1);
    }
};