#pragma once

#ifdef YOYO_ZIP

#include <pixelscript.h>
// #include "miniz.hpp"
#include <string>
#include <vector>
#include <cstdint>

namespace miniz_cpp {
    class zip_file;
}

namespace yoyo::zip {
    struct ZipFile {
        // @private
        // Internal zip file.
        miniz_cpp::zip_file* zf;

        ~ZipFile();

        // @private
        // CONVERT into a pxs host object.
        pxs_VarT topxs();

        // // @private
        // // Open from a path.
        // // Uses `yoyo.fs` internally.
        // static pxs_VarT open(const std::string& archive_path);
        // // @private
        // // Direct open a archive from it's bytes.
        // static pxs_VarT open(const std::vector<uint8_t> archive_bytes);

        // @except
        // @self
        // Read a file in the archive.
        // args:
        //  - path: `string` the path to read in the archive.
        //  - rt: @opt `FileReadType` how to read and return the results (default is Text).
        //
        // returns `string`|`[]uint` either file contents as a string or list of bytes.
        static pxs_VarT read(pxs_VarT args);

        // @except
        // @self
        // Write into a archive.
        // args:
        //  - path: `string` the path to write to.
        //  - data: `string`|`[]uint` the data to write, either a string or bytes.
        //
        static pxs_VarT write(pxs_VarT args);

        // @except
        // @self
        // List contents of a directory in the archive.
        // args:
        //  - path: `string` the path to the directory.
        //
        // returns `[]string` return a list of items.
        static pxs_VarT listdir(pxs_VarT args);

        // @except
        // @self
        // Remove a directory.
        // args:
        //  - path: `string` path to directory.
        //
        static pxs_VarT rmdir(pxs_VarT args);

        // @except
        // @self
        // Remove a file.
        // args:
        //  - path: `string` path to file.
        //
        static pxs_VarT rmfile(pxs_VarT args);

        // @except
        // @self
        // Extract files to a destination.
        // args:
        //  - src_path: `string` path to files. Can be '/'
        //  - dest_path: `string` path to destination.
        //
        static pxs_VarT extract(pxs_VarT args);
    };

    // Open a new zip file.
    // args:
    //  - pd: `string`|`[]uint` path to the zip file or the bytes.
    //
    // returns `ZipFile` a new instance.
    pxs_VarT open(pxs_VarT args);

    void init(pxs_Module* yoyo);
};

#endif // YOYO_ZIP