#include "lua.h"
#include "lauxlib.h"
#include <stdlib.h>
#include <string.h>
#include "pxs_lua.h"

// C Callback.
// Push this to lua stack instead of unsafe rust code.
// This will call the rust code via the bridge uptop.
// It's up to the bridge to know what function to call. Use upvalues for that.
int pxslua_callback(lua_State* L) {
    char* err_buf = NULL;
    // char* buffer[256] = {0};
    // char* buffer = (char*)malloc(strlen())
    // Call the bridge
    int result = pxslua_rustbridge(L, &err_buf);

    if (result < 0) {
        if (err_buf == NULL) {
            lua_pushstring(L, "Unknown error.");
        } else {
            lua_pushstring(L, err_buf);
            pxslua_free_ruststring(err_buf);
        }
        return lua_error(L);
    }

    // Num returned (1)
    return result;
}