#ifdef YOYO_ZIP

#include "zip.hpp"
#include <cstdint>
#include <string>
#include <vector>
#include "miniz.hpp"
#include <utility>
#include "utils/types.hpp"
#include <sstream>
#include "utils/bytes.hpp"
#include <filesystem>
#include "utils/debug.hpp"
#include <pixelscript.h>
#include <pixelscript_cpp.hpp>
#include "utils/exceptions.hpp"

#ifndef YOYO_FS
#error "YOYO_FS is required to use YOYO_ZIP."
#endif

#include "fs.hpp"

namespace yoyo::zip {
    void free_zip_file(pxs_Opaque obj) {
        if (obj) {
            delete static_cast<ZipFile*>(obj);
        }
    }

    ZipFile::~ZipFile() {
        if (zf) {
            delete zf;
        }
    }

    pxs_VarT ZipFile::topxs() {
        auto obj = pxs_newtype(static_cast<pxs_Opaque>(this), free_zip_file, "ZipFile", yoyo::types::ZIP_ZIP_FILE_TYPE);
        pxs_object_addfunc(obj, "read", &ZipFile::read);
        pxs_object_addfunc(obj, "write", &ZipFile::write);
        pxs_object_addfunc(obj, "listdir", &ZipFile::listdir);
        pxs_object_addfunc(obj, "rmdir", &ZipFile::rmdir);
        pxs_object_addfunc(obj, "rmfile", &ZipFile::rmfile);
        pxs_object_addfunc(obj, "extract", &ZipFile::extract);
        return pxs_newhost(obj);
    }

    // pxs_VarT ZipFile::open(const std::string& path) {
    //     // Open a new zip file using fs.hpp
    //     auto contents = pxs::call(fs::read_file, {path, static_cast<int>(fs::FileReadType::Bytes)});
    //     if (pxs_varis(contents, pxs_Exception)) {
    //         return contents; // propogate.
    //     }
    //     // Extarct contents
    //     std::vector<uint8_t> bytes;
    //     bytes.resize(pxs_varsize(contents));
    //     pxs_copybytes(contents, static_cast<pxs_Opaque>(bytes.data()));
    //     pxs_freevar(contents);

    //     // Create zip file
    //     auto zip = new ZipFile{miniz_cpp::zip_file(bytes)};
        
    //     return zip->topxs();
    // }

    // pxs_VarT ZipFile::open(const std::vector<uint8_t> archive_bytes) {
    //     // Just create the zip
    //     auto zip = new ZipFile{miniz_cpp::zip_file(archive_bytes)};
    //     return zip->topxs();
    // }

    pxs_VarT ZipFile::read(pxs_VarT args) {
        auto self = static_cast<ZipFile*>(pxs_gettype(pxs_getrt(args), pxs_arg(args, 0), yoyo::types::ZIP_ZIP_FILE_TYPE));
        if (!self) {
            return yoyo::utils::exceptions::expected_self(pxs_arg(args, 0));
        }

        // Get path
        auto path_arg = pxs_arg(args, 1);
        auto path_c = pxs_getstring(path_arg);
        if (!path_c) {
            return yoyo::utils::exceptions::expected_type(pxs_vartype(path_arg), pxs_String);
        }
        // copy it over. (No allocations needed...) pretty neat I know.
        std::string path;
        path.resize(pxs_varsize(path_arg) / sizeof(char));
        pxs_copystring(path_arg, path.data());

        // Get rt
        auto rt_int = pxs_getint(pxs_arg(args, 2));
        if (rt_int > 1) {
            return yoyo::utils::exceptions::invalid_enum();
        }
        fs::FileReadType read_type(fs::FileReadType::Text);
        if (rt_int != -1) {
            read_type = static_cast<fs::FileReadType>(rt_int);
        }

        // Lets go!
        auto res = self->zf->read(path);

        // If read_type is bytes we gotta convert it
        pxs_VarT result;
        if (read_type == fs::FileReadType::Bytes) {
            result = pxs_newbytes(static_cast<pxs_Opaque>(res.data()), sizeof(char), res.size());
        } else {
            result = pxs_newstring(res.c_str());
        }

        return result;
    }

    pxs_VarT ZipFile::write(pxs_VarT args) {
        auto self = static_cast<ZipFile*>(pxs_gettype(pxs_getrt(args), pxs_arg(args, 0), yoyo::types::ZIP_ZIP_FILE_TYPE));
        if (!self) {
            return yoyo::utils::exceptions::expected_self(pxs_arg(args, 0));
        }

        // Get path
        auto path_arg = pxs_arg(args, 1);
        if (!pxs_varis(path_arg, pxs_String)) {
            return yoyo::utils::exceptions::expected_type(pxs_vartype(path_arg), pxs_String);
        }
        std::string path;
        path.resize(pxs_varsize(path_arg) / sizeof(char));
        pxs_copystring(path_arg, path.data());

        // Get data
        auto data_arg = pxs_arg(args, 2);
        std::string data;
        if (!pxs_varis(data_arg, pxs_String) && !pxs_varis(data_arg, pxs_List)) {
            return yoyo::utils::exceptions::expected_types(pxs_vartype(data_arg), {pxs_String, pxs_List});
        }
        pxs_copybytes(data_arg, static_cast<pxs_Opaque>(data.data()));
        
        // Write it yo!
        self->zf->writestr(path, data);
        return pxs_newnull();
    }

    pxs_VarT ZipFile::listdir(pxs_VarT args) {
        auto self = static_cast<ZipFile*>(pxs_gettype(pxs_getrt(args), pxs_arg(args, 0), yoyo::types::ZIP_ZIP_FILE_TYPE));
        if (!self) {
            return yoyo::utils::exceptions::expected_self(pxs_arg(args, 0));
        }
        
        // Get path
        auto path_arg = pxs_arg(args, 1);
        if (!pxs_varis(path_arg, pxs_String)) {
            return utils::exceptions::expected_type(pxs_vartype(path_arg), pxs_String);
        }
        std::string path;
        path.resize(pxs_varsize(path_arg) / sizeof(char));
        pxs_copystring(path_arg, path.data());

        // Grab and pass the contents.
        auto full_contents = self->zf->infolist();
        pxs_VarT list = pxs_newlist();
        for (const auto& item : full_contents) {
            if (path.empty()) {
                pxs_listadd(list, pxs_newstring(item.filename.c_str()));
                continue;
            }
            // Check starts with.
            if (item.filename.rfind(path, 0) == 0) {
                pxs_listadd(list, pxs_newstring(item.filename.c_str()));
            }
        }

        return list;
    }

    // todo(jordanmc)
    pxs_VarT ZipFile::rmdir(pxs_VarT args) {
        auto self = static_cast<ZipFile*>(pxs_gettype(pxs_getrt(args), pxs_arg(args, 0), yoyo::types::ZIP_ZIP_FILE_TYPE));
        if (!self) {
            return yoyo::utils::exceptions::expected_self(pxs_arg(args, 0));
        }
        
        return pxs_newnull();
    }
    
    // todo(jordanmc)
    pxs_VarT ZipFile::rmfile(pxs_VarT args) {
        auto self = static_cast<ZipFile*>(pxs_gettype(pxs_getrt(args), pxs_arg(args, 0), yoyo::types::ZIP_ZIP_FILE_TYPE));
        if (!self) {
            return yoyo::utils::exceptions::expected_self(pxs_arg(args, 0));
        }
        
        return pxs_newnull();
    }

    pxs_VarT ZipFile::extract(pxs_VarT args) {
        auto self = static_cast<ZipFile*>(pxs_gettype(pxs_getrt(args), pxs_arg(args, 0), yoyo::types::ZIP_ZIP_FILE_TYPE));
        if (!self) {
            return yoyo::utils::exceptions::expected_self(pxs_arg(args, 0));
        }
        
        // Get src_path
        auto src_path_arg = pxs_arg(args, 1);
        if (!pxs_varis(src_path_arg, pxs_String)) {
            return utils::exceptions::expected_type(pxs_vartype(src_path_arg), pxs_String);
        }
        std::string src_path;
        src_path.resize(pxs_varsize(src_path_arg) / sizeof(char));
        pxs_copystring(src_path_arg, src_path.data());

        // Get dst_path
        auto dst_path_arg = pxs_arg(args, 2);
        if (!pxs_varis(dst_path_arg, pxs_String)) {
            return utils::exceptions::expected_type(pxs_vartype(dst_path_arg), pxs_String);
        }
        std::string dst_path;
        dst_path.resize(pxs_varsize(dst_path_arg) / sizeof(char));
        pxs_copystring(dst_path_arg, dst_path.data());

        bool is_dir = src_path.back() == '/';

        if (is_dir) {
            // A full dir
            auto contents = pxs::call(ZipFile::listdir, {pxs_new_shallowcopy(pxs_arg(args, 0)), src_path});
            if (!pxs_varis(contents, pxs_List)) {
                return pxs_newexception("Could not get contents of directory in archive.");
            }
            
            // Loop through and write them
            for (int i = 0; i < pxs_listlen(contents); i++) {
                auto item = pxs_listget(contents, i);
                if (pxs_varis(item, pxs_String)) {
                    std::string str;
                    str.resize(pxs_varsize(item) / sizeof(char));
                    pxs_copystring(item, str.data());
                    // Get file contents and save them at this (dst + item) path
                    auto file_contents = self->zf->read(str);
                    auto path = dst_path + "/" + str;
                    auto res = pxs::call(fs::write_file, {dst_path, file_contents});
                    if (pxs_varis(res, pxs_Exception)) {
                        pxs_freevar(contents);
                        return res;
                    }
                    // free res
                    pxs_freevar(res);
                }
            }

            pxs_freevar(contents);
            return pxs_newnull();
        } else {
            // just a file
            auto contents = self->zf->read(src_path);
            return pxs::call(fs::write_file, {dst_path, contents});
        }
    }

    pxs_VarT open(pxs_VarT args) {
        // Get PD
        auto pd_arg = pxs_arg(args, 0);
        ZipFile* zf = nullptr;
        if (pxs_varis(pd_arg, pxs_String)) {
            // This is a string argument.
            auto str_c = pxs_getstring(pd_arg);
            if (!str_c) {
                return pxs_newexception("pd:string argument is null.");
            }
            // Read file bytes.
            auto contents = pxs::call(fs::read_file, {std::string(str_c), static_cast<int>(fs::FileReadType::Bytes)});
            pxs_freestr(str_c);
            // Bubble up.
            if (pxs_varis(contents, pxs_Exception)) {
                return contents;
            } else {
                // Copy bytes. 
                std::vector<uint8_t> bytes;
                pxs_copybytes(contents, static_cast<pxs_Opaque>(bytes.data()));
                pxs_freevar(contents);
                zf = new ZipFile{new miniz_cpp::zip_file(bytes)};
            }
        } else if (pxs_varis(pd_arg, pxs_List)) {
            // Its already bytes
            // Lets read them y listo!
            std::vector<uint8_t> bytes;
            pxs_copybytes(pd_arg, static_cast<pxs_Opaque>(bytes.data()));
            zf = new ZipFile{new miniz_cpp::zip_file(bytes)};
        } else if (!pd_arg) {
            // Empty dog.
            zf = new ZipFile{new miniz_cpp::zip_file()};
        }

        if (!zf) {
            return pxs_newexception("Could not create ZipFile.");
        }

        return zf->topxs();
    }

    void init(pxs_Module* yoyo) {
        auto zip_mod = pxs_newmod("zip");

        pxs_addfunc(zip_mod, "open", open);

        pxs_add_submod(yoyo, zip_mod);
    }
};

#endif // YOYO_ZIP