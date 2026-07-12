#ifdef YOYO_OS
#include <pixelscript_cpp.hpp>
#include <filesystem>
#include "os.hpp"

namespace yoyo::os {
    // Get the current working directory.
    pxs_VarT get_cwd(pxs_VarT args) {
        auto res = std::filesystem::current_path();
        return pxs_newstring(res.string().c_str());
    }

    // Change the current directory
    pxs_VarT ch_dir(pxs_VarT args) {
        PXS_ARGC_EQ(1); // new_path
        auto new_path = pxs::Var::from_args(args, 0);
        PXS_ARG_IS_TYPE(new_path.raw(), pxs_String);

        // TODO: Exception catching.
        // auto new_path_str = new_path.get_string();
        // if (new_path_str)

        std::filesystem::current_path(new_path.get_string());
        return pxs_newnull();
    }

    void init(pxs_Module* yoyo, pxs_VarT argv) {
        auto yoyo_os = pxs_newmod("os");

        // Add argv
        pxs_addvar(yoyo_os, "argv", argv);

        pxs_addfunc(yoyo_os, "get_cwd", get_cwd);
        pxs_addfunc(yoyo_os, "ch_dir", ch_dir);

        pxs_add_submod(yoyo, yoyo_os);
    }
};

#endif // YOYO_OS