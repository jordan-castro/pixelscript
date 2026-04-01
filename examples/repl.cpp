// This is a multi lang repl.

#include "pixelscript.h"
#include <string>
#include <iostream>

// Define a function for println
pxs_VarT println(pxs_VarT args) {
    std::string msg; 

    // Loop through args
    for (int i = 0; i < pxs_listlen(args) - 1; i++) {
        // Convert to string.
        pxs_VarT res = pxs_tostring(pxs_listget(args, 0), pxs_listget(args, 1));
        // Check for error?
        if (pxs_varis(res, pxs_VarType::pxs_Exception)) {
            // Just return it, the backend will take care of it.
            return res;
        }
        // We have our string!
        char* raw = pxs_getstring(res);
        msg += std::string(raw);
        // Free it
        pxs_freestr(raw);
    }

    // Use internal std::cout
    std::cout << msg << std::endl;

    // Return null
    return pxs_newnull();
}

pxs_VarT eval(pxs_VarT args) {
    // Get runtime first
    int runtime = pxs_getint(pxs_listget(args, 1));
    // Get script next
    char* script = pxs_getstring(pxs_listget(args, 2));
    // Eval result
    auto res = pxs_eval(script, static_cast<pxs_Runtime>(runtime));
    pxs_freestr(script);
    return res;
}

int main(int argc) {
    pxs_Runtime runtime;
    if (argc <= 1) {
        runtime = pxs_Python;
    } else {
        runtime = pxs_Lua;
    }

    // Initialize
    pxs_initialize();

    // Create pxs module
    pxs_Module* pxs_module = pxs_newmod("pxs");
    // Add println function
    pxs_addfunc(pxs_module, "println", println);
    pxs_addfunc(pxs_module, "eval", eval);

    // Add the module
    pxs_addmod(pxs_module);

    std::string full;    
    while (true) {
        // Get input from user
        std::string input;
        std::cout << ">> ";
        if (!std::getline(std::cin, input) || input == "exit") {
            break;
        }
        
        if (input.empty()) {
            continue;
        }
        full += "\n" + input;

        // Execute
        pxs_VarT res = pxs_exec(runtime, full.c_str(), "<test>");
        if (!pxs_varis(res, pxs_VarType::pxs_Null)) {
            char* msg = pxs_getstring(res);
            std::cout << msg << std::endl;
            pxs_freestr(msg);
        }
    }

    // Finalize
    pxs_finalize();
}