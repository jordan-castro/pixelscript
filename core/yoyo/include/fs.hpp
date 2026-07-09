#pragma once

#ifdef YOYO_FS

#include <pixelscript.h>
#include <pixelscript_cpp.hpp>
#include <string>
#include <vector>
#include <cstdint>
#include <memory>
#include <fstream>
#include <sstream>

namespace yoyo::fs {
    // Pass this as the final argument in `read_file` to return bytes or text.
    // Defaults to Text
    enum class FileReadType : uint8_t {
        Bytes,
        Text
    };

    // Pass this as the final argument in `open_file` to open as Read, Write, or Append
    // Defaults to Read.
    // You can also mix it via bitwise.
    enum class FileOpenType : uint8_t {
        Read = 1 << 0,
        Write = 2 << 1,
        Append = 3 << 2,
    };

    inline FileOpenType operator|(FileOpenType l, FileOpenType r) {
        return static_cast<FileOpenType>(
            static_cast<uint8_t>(l) | static_cast<uint8_t>(r)
        );
    }

    inline bool operator&(FileOpenType l, FileOpenType r) {
        return (static_cast<uint8_t>(l) & static_cast<uint8_t>(r));
    }

    // Pass this as the final argument in `remove_dir` to either remove only empty components or all components.
    // Defaults to Empty.
    enum class DirRemoveType : uint8_t {
        Empty,
        All
    };

    // Pass this as the final argument in `create_dir` to except on creating more than one internal directory.
    // Defaults to `Single`
    enum class CreateDirMode : uint8_t {
        Single,
        All
    };

    // Wraps a `ifstream` or `ofstream` depending on `FileOpenType`.
    // Use this when memory needs to be explicit.
    // `read_file` and `write_file` use `File` internally.
    class File : public pxs::type::Extension<File> {
        // @private
        // File path
        std::string path;
        // @private
        // File stream
        std::fstream stream;
        // @private
        // a buffer for writing, when the file is created not opened.
        std::stringstream buffer;
        // @private
        // Open type
        FileOpenType opentype;

        // @private
        // Was this file created or opened?
        bool created;

        // @private
        // When opening a file.
        File(const std::string& fpath, FileOpenType ot);
        // @private
        // When creating a file.
        File();
    
    public:
        ~File();

        // @prop
        // Get the file path set.
        // 
        // args:
        //  - self: `File`
        //
        // returns `string` string if this file was not created.
        static pxs_VarT get_path(pxs_VarT args);
        
        // @prop
        // Get the `FileOpenType`.
        // 
        // args:
        //  - self: `File`
        //
        // returns `FileOpenType`
        static pxs_VarT get_open_type(pxs_VarT args);

        // Create a new `File`. It won't write to the system until `save` is called.
        //
        // returns `File` a new instance.
        static pxs_VarT create(pxs_VarT args);

        // @except
        // 
        // Open a `File`. If it does not exist, it creates it on the fly, but `save` is not required.
        // args:
        //  - path: `string` path to the file.
        //  - open_type: @opt `FileOpenType` how to open the file. Defaults to `FileOpenType::Read`.
        //
        // returns `File` a new instance.
        static pxs_VarT open(pxs_VarT args);

        // @except
        // Read from `self`.
        // args:
        //  - self: `File`
        //  - read_type: @opt `FileReadType` what to return the file contents as. Defaults to `FileReadType::Text`.
        //
        // returns `string`|`[]uint` Text or Bytes depending on `read_type`.
        static pxs_VarT read(pxs_VarT args);

        // @except
        // Write into `self`.
        // args:
        //  - self: `File`
        //  - data: `string`|`[]uint` Data to write.
        static pxs_VarT write(pxs_VarT args);

        // @except
        // Append onto `self`.
        // args:
        //  - self: `File`
        //  - data: `string`|`[]uint` Data to append.
        // 
        // returns `int` the new size of the file.
        static pxs_VarT append(pxs_VarT args);

        // @except
        // Close a file.
        // args: 
        //  - self: `File`
        static pxs_VarT close(pxs_VarT args);

        // @except
        // Remove a file.
        // args:
        //  - self: `File`
        static pxs_VarT remove(pxs_VarT args);

        // @except
        // Save a file. This should be used when you created a file vs opened.
        // args:
        //  - self: `File`
        //  - path: `string`
        static pxs_VarT save(pxs_VarT args);
    };

    // @except
    // Read a file.
    // args:
    //  - path: `string` path to the file.
    //  - read_type: @opt `FileReadType` how to read the file. Defaults to `FileReadType::Text`.
    // 
    // returns `string`|`[]uint`
    pxs_VarT read_file(pxs_VarT args);
    
    // @except
    // Read the entries of a directory.
    // args:
    //  - path: `string` path to the directory.
    //
    // returns `[]string` file entries.
    pxs_VarT read_dir(pxs_VarT args);

    // @except
    // Write to a file.
    // args:
    //  - path: `string` path to the file.
    //  - data: `string`|`[]uint` the data to write.
    pxs_VarT write_file(pxs_VarT args);

    // Check if a file/dir exists.
    // args:
    //  - path: `string` file/dir path.
    //
    // returns: `bool`
    pxs_VarT exists(pxs_VarT args);

    // @except
    // Remove a directory.
    // args:
    //  - path: `string` path to the directory.
    //  - rt: @opt `DirRemoveType` to remove empty or all. Default empty
    pxs_VarT remove_dir(pxs_VarT args);

    // @except
    // Remove a file.
    // args:
    //  - path: `string` path to the file.
    pxs_VarT remove_file(pxs_VarT args);

    // @except
    // Create a directory.
    // args:
    //  - path: `string` directory path
    //  - mode: @opt `CreateDirMode` defaults to `Single`.
    pxs_VarT create_dir(pxs_VarT args);

    // Check if path is a directory.
    // args:
    //  - path: `string` path
    //
    // returns `bool`
    pxs_VarT is_dir(pxs_VarT args);

    // @private
    // 
    // Initialize the `yoyo.fs` module.
    void init(pxs_Module* yoyo);
};

#endif // YOYO_FS