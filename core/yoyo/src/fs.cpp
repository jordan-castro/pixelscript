#ifdef YOYO_FS

#include <pixelscript.h>
#include <pixelscript_cpp.hpp>
#include <fstream>
#include <filesystem>
#include <string>
#include <vector>
#include "fs.hpp"
#include "utils/bytes.hpp"
#include "utils/strutils.hpp"
#include "utils/debug.hpp"
#include <system_error>
#include "utils/types.hpp"
#include "utils/exceptions.hpp"
#include <iterator>

namespace yoyo::fs {
    // free a File
    void free_file(pxs_Opaque obj) {
        if (obj == nullptr) {
            return;  
        }

        auto val = static_cast<File*>(obj);
        delete val;
    }

    // Create the `File` object with all it's methods.
    pxs_VarT create_file_object(File* file) {
        auto obj = pxs_newtype(static_cast<pxs_Opaque>(file), free_file, "File", yoyo::types::FS_FILE_TYPE);
        pxs_object_addfunc(obj, "read", &File::read);
        pxs_object_addfunc(obj, "write", &File::write);
        pxs_object_addfunc(obj, "append", &File::append);
        pxs_object_addfunc(obj, "close", &File::close);
        pxs_object_addfunc(obj, "remove", &File::remove);
        pxs_object_addfunc(obj, "save", &File::save);
        pxs_object_addprop(obj, "path", &File::get_path);
        pxs_object_addprop(obj, "open_type", &File::get_open_type);
        return pxs_newhost(obj);
    }

    File::~File() {
        if (this->created) {
            return;
        }

        if (this->stream.is_open()) {
            this->stream.close();
        }
    }

    File::File(const std::string& path, FileOpenType ot) {
        this->path = path;

        this->opentype = ot;

        std::ios_base::openmode mode = 0;

        if (ot & FileOpenType::Read) {
            mode |= std::ios::in;
        }
        if (ot & FileOpenType::Write) {
            mode |= std::ios::out;
        }
        if (ot & FileOpenType::Append) {
            mode |= std::ios::app;
        }

        this->stream = std::fstream(path, mode);
        this->created = false;
    }

    File::File() {
        this->created = true;
        FileOpenType ot = FileOpenType::Read | FileOpenType::Write | FileOpenType::Append;
        this->opentype = ot;
    }

    pxs_VarT File::create(pxs_VarT args) {
        // create a file.
        auto cfile = new File();
        // wrap and return it.
        return create_file_object(cfile);
    }
    
    pxs_VarT File::open(pxs_VarT args) {
        // Get path and maybe open_type
        auto argc = pxs_argc(args);
        std::string path;
        FileOpenType ot = FileOpenType::Read;
        if (argc >= 1) {
            // Get string path or return exception
            auto path_arg = pxs_arg(args, 0);
            if (path_arg && pxs_varis(path_arg, pxs_String)) {
                auto path_c = pxs_getstring(path_arg);
                path = std::string(path_c);
                pxs_freestr(path_c);
            } else {
                return pxs_newexception("Expected path:String");
            }
            // Get open type if passed. No exception unless invalid index.
            if (argc >= 2) {
                auto ot_arg = pxs_arg(args, 1);
                auto ot_int = pxs_getint(ot_arg);
                if (ot_int > 1) {
                    return pxs_newexception("Expected valid FileReadType.");
                }
                if (ot_int != -1) {
                    ot = static_cast<FileOpenType>(ot_int);
                }
            }
        } else {
            return utils::exceptions::expected_nm(argc, 1);
        }

        // If the path does not exist, create it
        if (!std::filesystem::exists(path)) {
            // Create the file
            std::ofstream blank(path);
        }

        // Create file
        auto cfile = new File(path, ot);
        return create_file_object(cfile);
    }

    pxs_VarT File::read(pxs_VarT args) {
        auto self = static_cast<File*>(pxs_gettype(pxs_getrt(args), pxs_arg(args, 0), yoyo::types::FS_FILE_TYPE));
        if (!self) {
            return utils::exceptions::expected_self(PXS_ARG(0));
        }

        // Assert is read
        if (!(self->opentype & FileOpenType::Read)) {
            return pxs_newexception("`File` was not opened with `FileOpenType.Read`.");
        }
    
        // Check read type
        FileReadType rt = FileReadType::Text;
        auto rt_arg = pxs::Var::from_args(args, 1);
        if (rt_arg.is(pxs_Int64) || rt_arg.is(pxs_UInt64)) {
            rt = static_cast<FileReadType>(rt_arg.get_uint());
        }

        // Get contents as bytes first.
        std::vector<char> contents(
            (std::istreambuf_iterator<char>(self->stream)),
            std::istreambuf_iterator<char>());

        if (rt == FileReadType::Text) {
            // convert bytes to text
            std::string text(contents.begin(), contents.end());
            return pxs_newstring(text.c_str());
        } else {
            return pxs_newbytes(static_cast<pxs_Opaque>(contents.data()), sizeof(char), contents.size());
        }
    }

    pxs_VarT File::write(pxs_VarT args) {
        PXS_ARGC_EQ(2); // self, data
        auto self = static_cast<File*>(pxs_gettype(pxs_getrt(args), pxs_arg(args, 0), yoyo::types::FS_FILE_TYPE));
        if (!self) {
            return utils::exceptions::expected_self(PXS_ARG(0));
        }

        if (!(self->opentype & FileOpenType::Write)) {
            return pxs_newexception("`File` was not opened with `FileOpenType.Write`");
        }

        // CHeck data
        auto data = pxs::Var::from_args(args, 1);
        std::vector<char> bytes;

        if (data.is(pxs_String)) {
            auto str = data.get_string();
            bytes = std::vector<char>(str.begin(), str.end());
        } else if (data.is(pxs_List)) {
            bytes.resize(pxs_varsize(data.raw()));
            pxs_copybytes(data.raw(), static_cast<pxs_Opaque>(bytes.data()));
        } else {
            return utils::exceptions::expected_types(data.raw()->tag, {pxs_String, pxs_List});
        }

        // Check if path exists.
        // If not, create it.
        std::filesystem::path parent_path = std::filesystem::path(self->path).parent_path();
        if (!std::filesystem::exists(parent_path)) {
            auto res = pxs::call(create_dir, {parent_path.string(), static_cast<int>(CreateDirMode::All)});
            if (pxs_varis(res, pxs_Exception)) {
                return res;
            }
        }

        self->stream.write(reinterpret_cast<const char*>(bytes.data(), bytes.size()), bytes.size());

        return pxs_newnull();
    }

    pxs_VarT File::append(pxs_VarT args) {
        PXS_ARGC_EQ(2); // self, data
        auto self = static_cast<File*>(pxs_gettype(pxs_getrt(args), pxs_arg(args, 0), yoyo::types::FS_FILE_TYPE));
        if (!self) {
            return utils::exceptions::expected_self(PXS_ARG(0));
        }

        if (!(self->opentype & FileOpenType::Append)) {
            return pxs_newexception("`File` was not opened with `FileOpenType.Append`");
        }

        // CHeck data
        auto data = pxs::Var::from_args(args, 1);
        std::vector<char> bytes;

        if (data.is(pxs_String)) {
            auto str = data.get_string();
            bytes = std::vector<char>(str.begin(), str.end());
        } else if (data.is(pxs_List)) {
            pxs_copybytes(data.raw(), static_cast<pxs_Opaque>(bytes.data()));
        } else {
            return utils::exceptions::expected_types(data.raw()->tag, {pxs_String, pxs_List});
        }

        self->stream.write(reinterpret_cast<const char*>(bytes.data(), bytes.size()), bytes.size());

        return pxs_newnull();
    }

    pxs_VarT File::close(pxs_VarT args) {
        auto self = static_cast<File*>(pxs_gettype(pxs_getrt(args), pxs_arg(args, 0), yoyo::types::FS_FILE_TYPE));
        if (!self) {
            return utils::exceptions::expected_self(PXS_ARG(0));
        }

        // Check if created
        if (self->created) {
            return pxs_newnull();
        }

        if (self->stream.is_open()) {
            // Actually close the file.
            self->stream.close();
        }
        return pxs_newnull();
    }

    pxs_VarT File::remove(pxs_VarT args) {
        auto self = static_cast<File*>(pxs_gettype(pxs_getrt(args), pxs_arg(args, 0), yoyo::types::FS_FILE_TYPE));
        if (!self) {
            return utils::exceptions::expected_self(PXS_ARG(0));
        }

        if (self->created) {
            // reset buffer
            self->buffer.clear();
        } else {
            // remove file.
            std::error_code ec;
            std::filesystem::remove(self->path, ec);

            if (ec) {
                return pxs_newexception(ec.message().c_str());
            }
        }
        return pxs_newnull();
    }

    pxs_VarT File::save(pxs_VarT args) {
        PXS_ARGC_EQ(2); // self, path to save to.
        auto self = static_cast<File*>(pxs_gettype(pxs_getrt(args), pxs_arg(args, 0), yoyo::types::FS_FILE_TYPE));
        if (!self) {
            return utils::exceptions::expected_self(PXS_ARG(0));
        }

        auto path_var = pxs::Var::from_args(args, 1);
        PXS_ARG_IS_TYPE(path_var.raw(), pxs_String);

        if (self->created) {
            // Creates it automatically yo!
            std::ofstream w(path_var.get_string(), std::ios::out);
            w << self->buffer.rdbuf();
            w.close();
        }

        return pxs_newnull();
    }

    pxs_VarT File::get_path(pxs_VarT args) {
        PXS_ARGC_EQ(1); // self
        auto self = static_cast<File*>(pxs_gettype(pxs_getrt(args), pxs_arg(args, 0), yoyo::types::FS_FILE_TYPE));
        if (!self) {
            return utils::exceptions::expected_self(PXS_ARG(0));
        }

        return pxs_newstring(self->path.c_str());
    }

    pxs_VarT File::get_open_type(pxs_VarT args) {
        PXS_ARGC_EQ(1); // self
        auto self = static_cast<File*>(pxs_gettype(pxs_getrt(args), pxs_arg(args, 0), yoyo::types::FS_FILE_TYPE));
        if (!self) {
            return utils::exceptions::expected_self(PXS_ARG(0));
        }

        return pxs_newint(static_cast<int>(self->opentype));
    }

    pxs_VarT read_file(pxs_VarT args) {
        PXS_ARGC_GT(0); // at least path.
        auto path = pxs::Var::from_args(args, 0);
        PXS_ARG_IS_TYPE(path.raw(), pxs_String);
        auto read_type = pxs::Var::from_args(args, 1);
        if (!read_type.is(pxs_Int64) && !read_type.is(pxs_UInt64)) {
            return utils::exceptions::expected_types(read_type.raw()->tag, {pxs_Int64, pxs_UInt64});            
        }

        // open file
        auto file = pxs::Var(pxs::call(&File::open, {path.shallow().raw()}));
        if (!file.is(pxs_HostObject)) {
            // Exception?
            return file.raw();
        }

        // This will automatically drop file.
        return pxs::call(&File::read, {file.raw(), path.shallow().raw(), read_type.shallow().raw()});
    }

    pxs_VarT read_dir(pxs_VarT args) {
        auto path = pxs::Var::from_args(args, 0);
        PXS_ARG_IS_TYPE(path.raw(), pxs_String);
        auto path_str = path.get_string();
        std::filesystem::path dpath(path_str);
        if (!std::filesystem::exists(dpath)) {
            return pxs_newexception("Directory does not exist.");
        }
        if (std::filesystem::is_directory(dpath)) {
            return pxs_newexception("Path is not a directory.");
        }
        auto list = pxs::Var::new_list();
        for (const auto& entry : std::filesystem::directory_iterator(dpath)) {
            auto res = entry.path().filename();
            list.add(pxs::from_variant(res.string()));
        }
        return list.raw();
    }

    pxs_VarT write_file(pxs_VarT args) {
        PXS_ARGC_EQ(2); // path, data
        auto path = pxs::Var::from_args(args, 0);
        PXS_ARG_IS_TYPE(path.raw(), pxs_String);

        auto data = pxs::Var::from_args(args, 1);
        if (!data.is(pxs_String) && !data.is(pxs_List)) {
            return utils::exceptions::expected_types(data.raw()->tag, {pxs_String, pxs_List});
        }

        // Open the file
        auto file = pxs::Var(pxs::call(&File::open, {path.shallow().raw()}));
        if (!file.is(pxs_HostObject)) {
            return file.raw();
        }

        return pxs::call(&File::write, {file.raw(), data.shallow().raw()});
    }

    pxs_VarT exists(pxs_VarT args) {
        PXS_ARGC_EQ(1); // path
        PXS_ARG_STRING_VAL(path, 0);
        
        return pxs_newbool(std::filesystem::exists(path));
    }

    pxs_VarT remove_dir(pxs_VarT args) {
        PXS_ARGC_GT(0); // path, rt
        PXS_ARG_STRING_VAL(path, 0);
        auto rt = pxs::Var::from_args(args, 1);
        DirRemoveType drt = DirRemoveType::Empty;
        if (rt.is(pxs_Int64) || rt.is(pxs_UInt64)) {
            drt = static_cast<DirRemoveType>(rt.get_int());
        }
        
        if (drt == DirRemoveType::All) {
            std::filesystem::remove_all(path);
        } else if (drt == DirRemoveType::Empty) {
            std::filesystem::remove(path);
        } else {
            return pxs_newexception("Please provide a valid remove type.");
        }

        return pxs_newnull();
    }

    pxs_VarT remove_file(pxs_VarT args) {
        PXS_ARGC_EQ(1); // path
        PXS_ARG_STRING_VAL(path, 0);
        std::error_code ec;
        std::filesystem::remove(path, ec);

        if (ec) {
            return pxs_newexception(ec.message().c_str());
        }

        return pxs_newnull();
    }

    pxs_VarT create_dir(pxs_VarT args) {
        PXS_ARGC_GT(0); // path, mode
        PXS_ARG_STRING_VAL(path, 0);
        CreateDirMode crmode = pxs::Var::from_args(args, 1).to_enum<CreateDirMode>();

        std::error_code ec;
        if (crmode == CreateDirMode::All) {
            std::filesystem::create_directories(path, ec);
        } else if (crmode == CreateDirMode::Single) {
            std::filesystem::create_directory(path, ec);
        } else {
            return pxs_newexception("Please provide a valid createmode.");
        }

        if (ec) {
            return pxs_newexception(ec.message().c_str());
        }

        return pxs_newnull();
    }

    pxs_VarT is_dir(pxs_VarT args) {
        PXS_ARGC_EQ(1); // path
        PXS_ARG_STRING_VAL(path, 0);
        return pxs_newbool(std::filesystem::is_directory(path));
    }

    // Initialize the `yoyo.fs` module.
    void init(pxs_Module* yoyo) {
        auto _fs = pxs_newmod("fs");

        pxs_addfunc(_fs, "read_file", read_file);
        pxs_addfunc(_fs, "write_file", write_file);
        pxs_addfunc(_fs, "read_dir", read_dir);
        pxs_addfunc(_fs, "exists", exists);
        pxs_addfunc(_fs, "remove_dir", remove_dir);
        pxs_addfunc(_fs, "remove_file", remove_file);
        pxs_addfunc(_fs, "create_dir", create_dir);
        pxs_addfunc(_fs, "is_dir", is_dir);
        pxs_addfunc(_fs, "create", &File::create);
        pxs_addfunc(_fs, "open", &File::open);
        pxs_addvar(_fs, "FILE_READ_TYPE_TEXT", pxs_newint(static_cast<int>(FileReadType::Text)));
        pxs_addvar(_fs, "FILE_READ_TYPE_BYTES", pxs_newint(static_cast<int>(FileReadType::Bytes)));
        pxs_addvar(_fs, "CREATE_DIR_MODE_ALL", pxs_newint(static_cast<int>(CreateDirMode::All)));
        pxs_addvar(_fs, "CREATE_DIR_MODE_SINGLE", pxs_newint(static_cast<int>(CreateDirMode::Single)));
        pxs_addvar(_fs, "DIR_REMOVE_TYPE_EMPTY", pxs_newint(static_cast<int>(DirRemoveType::Empty)));
        pxs_addvar(_fs, "DIR_REMOVE_TYPE_ALL", pxs_newint(static_cast<int>(DirRemoveType::All)));
        pxs_addvar(_fs, "FILE_OPEN_TYPE_READ", pxs_newint(static_cast<int>(FileOpenType::Read)));
        pxs_addvar(_fs, "FILE_OPEN_TYPE_WRITE", pxs_newint(static_cast<int>(FileOpenType::Write)));
        pxs_addvar(_fs, "FILE_OPEN_TYPE_APPEND", pxs_newint(static_cast<int>(FileOpenType::Append)));

        pxs_add_submod(yoyo, _fs);
    }
};

#endif // YOYO_FS