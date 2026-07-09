#ifdef YOYO_ZIP

#include "zip.hpp"
#include <cstdint>
#include <string>
#include <vector>
#include "miniz.hpp"
#include <utility>
#include "types.hpp"
#include <pixelscript_cpp.hpp>
#include <sstream>
#include "utils/bytes.hpp"
#include <filesystem>
#include "utils/debug.hpp"

#ifdef YOYO_FS
#include "fs.hpp"
#endif // YOYO_FS

namespace yoyo::zip {
    // The structure for working with zip files.
    struct ZipFile {
        miniz_cpp::zip_file file;

        static ZipFile* get(const pxs::Var& arg) {
            return pxs::type::Wrapper::get<ZipFile>(arg, yoyo::types::ZIP_ZIP_FILE_TYPE);
        }

        std::string read(const std::string& file_path) {
            return file.read(file_path);
        }

        std::vector<unsigned char> readb(const std::string& file_path) {
            auto contents = this->read(file_path);
            return std::vector<unsigned char>(contents.begin(), contents.end());
        }
    };

    // Free a ZipFile
    void free_zip_file(pxs_Opaque obj) {
        if (obj == nullptr) {
            return;
        }

        auto val = static_cast<ZipFile*>(obj);
        delete val;
    }

    // List the contents of the archive.
    pxs_VarT ZF_list_dir(pxs_VarT args) {
        auto self = ZipFile::get(pxs::Var::from_args(args, 0));
        if (self == nullptr) {
            return pxs_newnull();
        }

        // Get the contents!
        auto members = self->file.infolist();
        auto contents = pxs_newlist();
        for (size_t i = 0; i < members.size(); i++) {
            pxs_listadd(contents, pxs_newstring(members[i].filename.c_str()));
        }
        return contents;
    }

    // Read a specific file within the archive.
    pxs_VarT ZF_read(pxs_VarT args) {
        auto self = ZipFile::get(pxs::Var::from_args(args, 0));
        if (self == nullptr) {
            return pxs_newnull();
        }

        auto path = pxs::Var::from_args(args, 1);
        PXS_ARG_IS_TYPE(path.raw(), pxs_String);

        auto contents = self->file.read(path.get_string());
        return pxs_newstring(contents.c_str());
    }

    // Read a specific files bytes within the archive.
    pxs_VarT ZF_readb(pxs_VarT args) {
        auto self = ZipFile::get(pxs::Var::from_args(args, 0));
        if (self == nullptr) {
            return pxs_newnull();
        }

        auto path = pxs::Var::from_args(args, 1);
        PXS_ARG_IS_TYPE(path.raw(), pxs_String);

        return yoyo::utils::bytes::make_byte_list(self->readb(path.get_string()));
    }

    // Extract a file to a specific location
    pxs_VarT ZF_extract(pxs_VarT args) {
        #ifdef YOYO_FS
        PXS_ARGC_EQ(3); // self, path, location
        auto self = ZipFile::get(pxs::Var::from_args(args, 0));
        if (self == nullptr) {
            return pxs_newbool(false);
        }

        auto path = pxs::Var::from_args(args, 1);
        PXS_ARG_IS_TYPE(path.raw(), pxs_String);
        auto location = pxs::Var::from_args(args, 2);
        PXS_ARG_IS_TYPE(location.raw(), pxs_String);

        auto contents = self->readb(path.get_string());
        return pxs_newbool(yoyo::fs::iwrite_file(location.get_string(), contents));
        #else
        return pxs_newexception("YOYO_FS is required.");
        #endif
    }

    // Extract the entire archive.
    pxs_VarT ZF_extract_all(pxs_VarT args) {
        #ifdef YOYO_FS
        PXS_ARGC_EQ(2); // self, location
        auto self = ZipFile::get(pxs::Var::from_args(args, 0));
        if (self == nullptr) {
            return pxs_newbool(false);
        }

        auto location = pxs::Var::from_args(args, 1);
        PXS_ARG_IS_TYPE(location.raw(), pxs_String);
        auto loc_string = location.get_string();
        auto entries = self->file.infolist();
        for (const auto& entry : entries) {
            auto bytes = self->readb(entry.filename);
            std::filesystem::path p(loc_string);
            p.append(entry.filename);
            if (!yoyo::fs::iwrite_file(p.string(), bytes)) {
                return pxs_newbool(false);
            }
        }

        return pxs_newbool(true);
        #else
        return pxs_newexception("YOYO_FS is required.");
        #endif
    }

    // Write text to the archive.
    pxs_VarT ZF_write(pxs_VarT args) {
        PXS_ARGC_EQ(3); // self, path, text
        auto self = ZipFile::get(pxs::Var::from_args(args, 0));
        if (self == nullptr) {
            return pxs_newbool(false);
        }

        auto path = pxs::Var::from_args(args, 1);
        PXS_ARG_IS_TYPE(path.raw(), pxs_String);

        auto text = pxs::Var::from_args(args, 2);
        PXS_ARG_IS_TYPE(text.raw(), pxs_String);

        self->file.writestr(path.get_string(), text.get_string());
        return pxs_newbool(true);
    }

    // Write bytes to the archive.
    pxs_VarT ZF_writeb(pxs_VarT args) {
        PXS_ARGC_EQ(3); // self, path, text
        auto self = ZipFile::get(pxs::Var::from_args(args, 0));
        if (self == nullptr) {
            return pxs_newbool(false);
        }

        auto path = pxs::Var::from_args(args, 1);
        PXS_ARG_IS_TYPE(path.raw(), pxs_String);

        auto bytes_var = pxs::Var::from_args(args, 2);
        PXS_ARG_IS_TYPE(bytes_var.raw(), pxs_List);

        auto bytes = yoyo::utils::bytes::convert_list_into<uint8_t>(bytes_var.raw());
        std::string bytes_str(bytes.begin(), bytes.end());
        self->file.writestr(path.get_string(), bytes_str);
        return pxs_newbool(true);
    }
    
    // Save the archive
    pxs_VarT ZF_save(pxs_VarT args) {
        PXS_ARGC_EQ(2); // self, path
        auto self = ZipFile::get(pxs::Var::from_args(args, 0));
        if (self == nullptr) {
            return pxs_newbool(false);
        }

        auto path = pxs::Var::from_args(args, 1);
        PXS_ARG_IS_TYPE(path.raw(), pxs_String);

        auto p = path.get_string();
        if (p.size() == 0) {
            return pxs_newbool(false);
        }

        self->file.save(path.get_string());
        return pxs_newbool(true);
    }

    // Create a ZipFile pxs object
    pxs_VarT _setup_zipfile(ZipFile* ptr) {
        auto wrapper = new pxs::type::Wrapper(static_cast<pxs_Opaque>(ptr), free_zip_file, yoyo::types::ZIP_ZIP_FILE_TYPE);
        auto obj = pxs::Object(static_cast<pxs_Opaque>(wrapper), pxs::type::free_wrapper, "ZipFile");

        obj.add_method("list_dir", ZF_list_dir);
        obj.add_method("read", ZF_read);
        obj.add_method("readb", ZF_readb);
        obj.add_method("extract", ZF_extract);
        obj.add_method("extract_all", ZF_extract_all);
        obj.add_method("write", ZF_write);
        obj.add_method("writeb", ZF_writeb);
        obj.add_method("save", ZF_save);

        return obj.make().raw();
    }

    // Open a archive via file path or stream directly.
    // You can also write to this archive.
    // But the main difference between `open` and `create` is that
    // `create` does not attempt to read a file or bytes.
    pxs_VarT open(pxs_VarT args) {
        PXS_ARGC_EQ(1); // file path
        auto file_path_var = pxs::Var::from_args(args, 0);

        std::vector<uint8_t> bytes;
        if (file_path_var.is(pxs_String)) {
            #ifdef YOYO_FS
            // Read the file and stream it
            bytes = yoyo::fs::iread_file_bytes(file_path_var.get_string());
            #else
                return pxs_newexception("`zip.read` expects bytes not string.");
            #endif // YOYO_FS
        } else if (file_path_var.is(pxs_List)) {
            // Get bytes
            std::vector<uint8_t> bytes;
            for (int i = 0; i < file_path_var.list_len(); i++) {
                auto byte = file_path_var.list_get(i).get_uint();
                bytes.push_back(byte);
            }
        }

        auto zip_file = new ZipFile{ miniz_cpp::zip_file(bytes) };
        return _setup_zipfile(zip_file);
    }

    // Create a new empty archive.
    pxs_VarT create(pxs_VarT args) {
        auto zip_file = new ZipFile{ miniz_cpp::zip_file() };
        return _setup_zipfile(zip_file);
    }

    void init(pxs_Module* yoyo) {
        auto zip_mod = pxs_newmod("zip");

        pxs_addfunc(zip_mod, "open", open);
        pxs_addfunc(zip_mod, "create", create);

        pxs_add_submod(yoyo, zip_mod);
    }
};

#endif // YOYO_ZIP