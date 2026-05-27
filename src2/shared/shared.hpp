#pragma once

#include "include/pixelscript.h"
#include <optional>

namespace pxs::shared {
    /// Exposes methods for loading files, writing files, and reading directories.
    /// Said methods come from the Host. Implemented using `pxs_set_filereader`, `pxs_set_filewriter`, and `pxs_set_dirreader` methods.
    struct PixelState {
        /// Load a file.
        std::optional<pxs_LoadFileFn> load_file;
        /// Write a file.
        std::optional<pxs_WriteFileFn> write_file;
        /// Read contents of a directory.
        std::optional<pxs_ReadDirFn> read_dir;

        PixelState() {
            // Set functions empty to begin with.
            load_file = std::nullopt;
            write_file = std::nullopt;
            read_dir = std::nullopt;
        }
    };

    /// Global PixelState variable
    static inline PixelState PIXEL_STATE = PixelState(); 
};