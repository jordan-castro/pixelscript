#ifdef YOYO_SHELL

#include <pixelscript.h>
#include <pixelscript_cpp.hpp>
#include <cstdlib>
#include "shell.hpp"

// TODO: implement a better shell.

namespace yoyo::shell {
    // Run a shell command via `std::system`
    // Output is sent to terminal directly.
    pxs_VarT system(pxs_VarT args) {
        PXS_ARGC_EQ(1); // command
        auto command = pxs::Var::from_args(args, 0);
        PXS_ARG_IS_TYPE(command.raw(), pxs_String);

        int res = std::system(command.get_string().c_str());
        return pxs_newint(res);
    }

    void init(pxs_Module* yoyo) {
        auto shell_mod = pxs_newmod("shell");

        pxs_addfunc(shell_mod, "system", system);

        pxs_add_submod(yoyo, shell_mod);
    }
}

#endif // YOYO_SHELL