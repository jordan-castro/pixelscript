#pragma once

#include "includes/pixelscript.h"
#include <memory>

namespace pxs::var {
    /// Implementation behind `pxs_Var`

    /// Implementation behind `pxs_VarObject`.
    /// Holds a language value and potentially a host_ptr. -1 for not held.
    struct pxs_VarObject {
        /// Reference to language memory.
        pxs_Opaque object_val;

        /// A potential host idx.
        int32_t host_idx;

        pxs_VarObject(pxs_Opaque val, int32_t host_idx) : object_val(val), host_idx(host_idx) {}
    };

    /// Allocate a new `pxs_VarObject` with a host object in mind.
    std::unique_ptr<pxs_VarObject> make_as_host(pxs_Opaque val, int32_t host_idx);

    /// Allocate a new `pxs_VarObject` without host object in mind.
    std::unique_ptr<pxs_VarObject> make_lang_only(pxs_Opaque val);
};