#include "zip.hpp"
#include "pixelscript.h"
#include "lib/miniz.hpp"
#include <vector>
#include <string>
#include <filesystem>

namespace fs = std::filesystem;

const int ZIP_FILE_TYPE = 3;

struct ZipFile {
    // @private
    // Internal zip file.
    miniz_cpp::zip_file* zf;
    // @private
    // The path to the zip file.
    std::string path;

    // @private
    ~ZipFile() {
        if (zf) {
            delete zf;
        }
    }

    // @private
    static void free_zip_file(pxs_Opaque ptr) {
        if (ptr) {
            delete static_cast<ZipFile*>(ptr);
        }
    }

    // @private
    // CONVERT into a pxs host object.
    pxs_VarT topxs() {
        auto obj = pxs_newtype(static_cast<pxs_Opaque>(this), free_zip_file, "ZipFile", ZIP_FILE_TYPE);
        pxs_object_addfunc(obj, "read", &ZipFile::read);
        pxs_object_addfunc(obj, "write", &ZipFile::write);
        pxs_object_addfunc(obj, "listdir", &ZipFile::listdir);
        pxs_object_addfunc(obj, "extract", &ZipFile::extract);
        pxs_object_addfunc(obj, "save", &ZipFile::save);
        return pxs_newhost(obj);
    }

    // @private
    // Get items from a path.
    std::vector<std::string> get_contents(const std::string& dir_path) {
        std::vector<std::string> res;
        // Grab and pass the contents.
        auto full_contents = this->zf->infolist();
        pxs_VarT list = pxs_newlist();
        for (const auto& item : full_contents) {
            if (dir_path.empty()) {
                res.push_back(item.filename);
                continue;
            }
            // Check starts with.
            if (item.filename.rfind(dir_path, 0) == 0) {
                res.push_back(item.filename);
            }
        }

        return res;
    }

    // @except
    // @self
    // Save contents to path.
    // args:
    //  - path: `string`? the path to save to. If none provided, uses file path.
    static pxs_VarT save(pxs_VarT args) {
        // Get self
        auto self = static_cast<ZipFile*>(pxs_gettype(pxs_getrt(args), pxs_arg(args, 0), ZIP_FILE_TYPE));
        if (!self) {
            return pxs_newexception("Expected self");
        }
        // Get path
        auto path = pxs_getstring(pxs_arg(args, 1));
        auto path_str = self->path;
        if (path) {
            path_str = std::string(path);
            pxs_freestr(path);
        }

        // Now call save
        self->zf->save(path_str);
        return pxs_newnull();
    }

    // @except
    // @self
    // Read a file in the archive.
    // args:
    //  - path: `string` the path to read in the archive.
    //
    // returns `string` file contents.
    static pxs_VarT read(pxs_VarT args) {
        auto self = static_cast<ZipFile*>(pxs_gettype(pxs_getrt(args), pxs_arg(args, 0), ZIP_FILE_TYPE));
        if (!self) {
            return pxs_newexception("Expected self");
        }

        // Get path
        auto path_arg = pxs_arg(args, 1);
        auto path_c = pxs_getstring(path_arg);
        if (!path_c) {
            return pxs_newexception("Expected string");
        }
        // copy it over. (No allocations needed...) pretty neat, I know.
        std::string path;
        path.resize(pxs_varsize(path_arg) / sizeof(char));
        pxs_copystring(path_arg, path.data());

        // Lets go!
        auto res = self->zf->read(path);
        return pxs_newstring(res.c_str());
    }

    // @except
    // @self
    // Write into a archive.
    // args:
    //  - path: `string` the path to write to.
    //  - data: `string`|`[]uint` the data to write.
    //
    static pxs_VarT write(pxs_VarT args) {
        auto self = static_cast<ZipFile*>(pxs_gettype(pxs_getrt(args), pxs_arg(args, 0), ZIP_FILE_TYPE));
        if (!self) {
            return pxs_newexception("Expected self");
        }

        // Get path
        auto path_arg = pxs_arg(args, 1);
        if (!pxs_varis(path_arg, pxs_String)) {
            return pxs_newexception("Expected String");
        }
        std::string path;
        path.resize(pxs_varsize(path_arg) / sizeof(char));
        pxs_copystring(path_arg, path.data());

        // Get data
        auto data_arg = pxs_arg(args, 2);
        std::string data;
        if (!pxs_varis(data_arg, pxs_String) && !pxs_varis(data_arg, pxs_List)) {
            return pxs_newexception("Expected String or List[Byte]");
        }
        pxs_copybytes(data_arg, static_cast<pxs_Opaque>(data.data()));
        
        // Write it yo!
        self->zf->writestr(path, data);
        return pxs_newnull();
    }

    // @except
    // @self
    // List contents of a directory in the archive.
    // args:
    //  - path: `string` the path to the directory.
    //
    // returns `[]string` return a list of items.
    static pxs_VarT listdir(pxs_VarT args) {
        auto self = static_cast<ZipFile*>(pxs_gettype(pxs_getrt(args), pxs_arg(args, 0), ZIP_FILE_TYPE));
        if (!self) {
            return pxs_newexception("Expected self");
        }
        
        // Get path
        auto path_arg = pxs_arg(args, 1);
        if (!pxs_varis(path_arg, pxs_String)) {
            return pxs_newexception("Expected String");
        }
        std::string path;
        path.resize(pxs_varsize(path_arg) / sizeof(char));
        pxs_copystring(path_arg, path.data());

        // Grab and pass the contents.
        auto contents = self->get_contents(path);
        auto list = pxs_newlist();
        for (const auto& i : contents) {
            pxs_listadd(list, pxs_newstring(i.c_str()));
        }

        return list;
    }

    // @except
    // @self
    // Extract files to a destination.
    // args:
    //  - src_path: `string` path to files. Can be '/'
    //  - dest_path: `string` path to destination.
    //
    static pxs_VarT extract(pxs_VarT args) {
        auto self = static_cast<ZipFile*>(pxs_gettype(pxs_getrt(args), pxs_arg(args, 0), ZIP_FILE_TYPE));
        if (!self) {
            return pxs_newexception("Expected self");
        }
        
        // Get src_path
        auto src_path_arg = pxs_arg(args, 1);
        if (!pxs_varis(src_path_arg, pxs_String)) {
            return pxs_newexception("Expected String");
            // return utils::exceptions::expected_type(pxs_vartype(src_path_arg), pxs_String);
        }
        std::string src_path;
        src_path.resize(pxs_varsize(src_path_arg) / sizeof(char));
        pxs_copystring(src_path_arg, src_path.data());

        // Get dst_path
        auto dst_path_arg = pxs_arg(args, 2);
        if (!pxs_varis(dst_path_arg, pxs_String)) {
            return pxs_newexception("Expected String");
        }
        std::string dst_path;
        dst_path.resize(pxs_varsize(dst_path_arg) / sizeof(char));
        pxs_copystring(dst_path_arg, dst_path.data());

        bool is_dir = !fs::path(dst_path).has_extension();

        if (is_dir) {
            // A full dir
            auto contents = self->get_contents(src_path);
            
            // Loop through and write them
            for (const auto& item : contents) {
                // Get file contents and save them at this (dst + item) path
                auto file_contents = self->zf->read(item);
                auto path = fs::path(dst_path);
                path.append(item);
                std::ofstream outfile(path);
                if (!outfile) {
                    return pxs_newexception(std::string("Could not write " + path.string()).c_str());
                }
                outfile << file_contents;
                outfile.close();
            }

            return pxs_newnull();
        } else {
            // just a file
            auto contents = self->zf->read(src_path);
            std::ofstream outfile(dst_path);
            if (!outfile) {
                return pxs_newexception(std::string("Could not write " + dst_path).c_str());
            }
            outfile << contents;
            outfile.close();
            return pxs_newnull();
        }
    }
};

// Open a new zip file.
// args:
//  - pd: `string`|`[]uint` path to the zip file or the bytes.
//
// returns `ZipFile` a new instance.
pxs_VarT open(pxs_VarT args) {
    // Get PD
    auto pd_arg = pxs_arg(args, 0);
    ZipFile* zf = nullptr;
    if (pxs_isstring(pd_arg)) {
        // This is a string argument.
        auto str_c = pxs_getstring(pd_arg);
        if (!str_c) {
            return pxs_newexception("pd:string argument is null.");
        }

        // Check if exists
        if (fs::exists(str_c)) {
            zf = new ZipFile{new miniz_cpp::zip_file(str_c)};
        } else {
            // otherwise we create a new one.
            zf = new ZipFile{new miniz_cpp::zip_file()};
        }
        pxs_freestr(str_c);
    } else if (pxs_islist(pd_arg)) {
        // Read bytes
        std::vector<uint8_t> bytes;
        pxs_copybytes(pd_arg, static_cast<pxs_Opaque>(bytes.data()));
        zf = new ZipFile{new miniz_cpp::zip_file(bytes)};
    } else {
        return pxs_newexception("Expected String of List[Byte]");
    }

    return zf->topxs();
}

void pxs_corelib_zip_init() {
    auto mod = pxs_newmod("pxs_zip");
    
    // Functions
    pxs_addfunc(mod, "open", open);

    pxs_addmod(mod);
}