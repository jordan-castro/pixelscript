#include "yoyo.hpp"

#ifdef YOYO_OS
#include "os.hpp"
#endif
#ifdef YOYO_PXS
#include "pxs.hpp"
#endif
#ifdef YOYO_FS
#include "fs.hpp"
#endif
#ifdef YOYO_SHELL
#include "shell.hpp"
#endif
#ifdef YOYO_NET
#include "net.hpp"
#endif
#ifdef YOYO_ZIP
#include "zip.hpp"
#endif

#include <pixelscript.h>

#include <string>
#include <iostream>

#ifdef YOYO_CORE
namespace yoyo {
    /// `yoyo.print`
    pxs_VarT print(pxs_VarT args) {
        std::string msg;
        pxs::Var arg_wrapper = pxs::Var(pxs_listget(args, 0), args);
        int argc = arg_wrapper.list_len();
        for (int i = 1; i < argc; i++) {
            auto var = arg_wrapper.list_get(i).to_string();
            msg += var;
            if (i < argc - 1) {
                msg += " ";
            }
        }

        std::cout << msg;
        return pxs_newnull();
    }

    /// `yoyo.println`
    pxs_VarT println(pxs_VarT args) {
        pxs_freevar(print(args));
        std::cout << std::endl;
        return pxs_newnull();
    }

    /// `yoyo.readln`
    pxs_VarT readln(pxs_VarT args) {
        std::string line;
        std::getline(std::cin, line);

        return pxs_newstring(line.c_str());
    }
}
#endif // YOYO_CORE

void yoyo_init() {
    auto yoyo = pxs_newmod("yoyo");

    #ifdef YOYO_CORE
    pxs_addfunc(yoyo, "print", yoyo::print);
    pxs_addfunc(yoyo, "println", yoyo::println);
    pxs_addfunc(yoyo, "readln", yoyo::readln);
    #endif // YOYO_CORE

    #ifdef YOYO_OS
    // TODO(jc) make OS use rust callbacks.
    // yoyo::os::init(yoyo, argc, argv);
    #endif // YOYO_OS

    #ifdef YOYO_PXS
    yoyo::ipxs::init(yoyo);
    #endif // YOYO_PXS

    #ifdef YOYO_FS
    yoyo::fs::init(yoyo);
    #endif // YOYO_FS

    #ifdef YOYO_SHELL
    yoyo::shell::init(yoyo);
    #endif // YOYO_SHELL

    #ifdef YOYO_NET
    yoyo::net::init(yoyo);
    #endif // YOYO_NET

    #ifdef YOYO_ZIP
    yoyo::zip::init(yoyo);
    #endif // YOYO_ZIP
    
    pxs_addmod(yoyo);
}
