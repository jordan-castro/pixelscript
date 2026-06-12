#ifndef PXS_LUA_H
#define PXS_LUA_H

#include "lua.h"

// Function signature in Rust.
// The state, the string ptr.
int pxslua_rustbridge(lua_State* L, char** err_buf);

// Free a rust string.
void pxslua_free_ruststring(char* ptr);

// C Callback.
// Push this to lua stack instead of unsafe rust code.
// This will call the rust code via the bridge uptop.
// It's up to the bridge to know what function to call. Use upvalues for that.
int pxslua_callback(lua_State* L);

#endif