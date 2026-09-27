//! Lua bindings. 

// == GLOBALS ==
const LUA_GLIBK : core::ffi::c_int = 1;
const LUA_LOADLIBK : core::ffi::c_int = LUA_GLIBK << 1;
const LUA_COLIBK : core::ffi::c_int = LUA_LOADLIBK << 1;
pub const LUA_DBLIBK : core::ffi::c_int = LUA_COLIBK << 1;
pub const LUA_IOLIBK : core::ffi::c_int = LUA_DBLIBK << 1;
const LUA_MATHLIBK : core::ffi::c_int = LUA_IOLIBK << 1;
pub const LUA_OSLIBK : core::ffi::c_int = LUA_MATHLIBK << 1;
#[allow(unused)]
const LUA_STRLIBK : core::ffi::c_int = LUA_OSLIBK << 1;
#[allow(unused)]
const LUA_TABLIBK : core::ffi::c_int = LUA_STRLIBK << 1;
#[allow(unused)]
const LUA_UTF8LIBK : core::ffi::c_int = LUA_TABLIBK << 1;
pub const LUA_RIDX_GLOBALS : core::ffi::c_int = 2;
pub const LUA_GCCOLLECT : core::ffi::c_int = 2;
pub const LUA_TNONE : core::ffi::c_int = -1;

pub const LUA_TNIL : core::ffi::c_int =	0;
pub const LUA_TBOOLEAN : core::ffi::c_int = 1;
pub const LUA_TLIGHTUSERDATA : core::ffi::c_int = 2;
pub const LUA_TNUMBER : core::ffi::c_int = 3;
pub const LUA_TSTRING : core::ffi::c_int = 4;
pub const LUA_TTABLE : core::ffi::c_int = 5;
pub const LUA_TFUNCTION : core::ffi::c_int = 6;

// == GLOBALS END ==

// == TYPES == 

/// Lua VM.
#[repr(C)]
pub struct lua_State {
    _unused: [u8; 0],
}

/// Just a shorthand for the pointer
pub type lua_StatePtr = *mut lua_State;

type lua_Integer = core::ffi::c_longlong;
type lua_Number = core::ffi::c_double;
type lua_Unsigned = core::ffi::c_ulonglong;

// The type for continuation-function contexts. It must be a numeric type. This type is defined as intptr_t when intptr_t is available, so that it can store pointers too. Otherwise, it is defined as ptrdiff_t.
pub type lua_KContext = isize; 
/// Type for continuation functions
pub type lua_KFunction = unsafe extern "C" fn(l: lua_StatePtr, status: core::ffi::c_int, ctx: lua_KContext) -> core::ffi::c_int;
/// Type for C functions.
/// In order to communicate properly with Lua, a C function must use the following protocol, which defines the way parameters and results are passed: a C function receives its arguments from Lua in its stack in direct order (the first argument is pushed first). So, when the function starts, lua_gettop(L) returns the number of arguments received by the function. The first argument (if any) is at index 1 and its last argument is at index lua_gettop(L). To return values to Lua, a C function just pushes them onto the stack, in direct order (the first result is pushed first), and returns in C the number of results. Any other value in the stack below the results will be properly discarded by Lua. Like a Lua function, a C function called by Lua can also return many results.
/// As an example, the following function receives a variable number of numeric arguments and returns their average and their sum:
/// ```c
///     static int foo (lua_State *L) {
///       int n = lua_gettop(L);    /* number of arguments */
///       lua_Number sum = 0.0;
///       int i;
///       for (i = 1; i <= n; i++) {
///         if (!lua_isnumber(L, i)) {
///           lua_pushliteral(L, "incorrect argument");
///           lua_error(L);
///         }
///         sum += lua_tonumber(L, i);
///       }
///       lua_pushnumber(L, sum/n);        /* first result */
///       lua_pushnumber(L, sum);         /* second result */
///       return 2;                   /* number of results */
///     }
/// ```
pub type lua_CFunction = unsafe extern "C" fn(l: lua_StatePtr) -> core::ffi::c_int;


// == TYPES END ==

// == FUNCTIONS ==
unsafe extern "C" {
    /// Releases reference ref from the table at index t (see luaL_ref). The entry is removed from the table, so that the referred object can be collected. The reference ref is also freed to be used again.
    /// If ref is LUA_NOREF or LUA_REFNIL, luaL_unref does nothing.
    pub fn luaL_unref(L: lua_StatePtr, t: core::ffi::c_int, reference: core::ffi::c_int);
    /// Creates and returns a reference, in the table at index t, for the object at the top of the stack (and pops the object).
    /// A reference is a unique integer key. As long as you do not manually add integer keys into table t, luaL_ref ensures the uniqueness of the key it returns. You can retrieve an object referred by reference r by calling lua_rawgeti(L, t, r). Function luaL_unref frees a reference and its associated object.
    /// If the object at the top of the stack is nil, luaL_ref returns the constant LUA_REFNIL. The constant LUA_NOREF is guaranteed to be different from any reference returned by luaL_ref.
    pub fn luaL_ref(L: lua_StatePtr, t: core::ffi::c_int) -> core::ffi::c_int;
    /// Pushes onto the stack the value t[n], where t is the value at the given valid index. The access is raw; that is, it does not invoke metamethods.
    pub fn lua_rawgeti(L: lua_StatePtr, idx: core::ffi::c_int, n: lua_Integer) -> core::ffi::c_int;
    /// Pushes the zero-terminated string pointed to by s onto the stack. Lua makes (or reuses) an internal copy of the given string, so the memory at s can be freed or reused immediately after the function returns. The string cannot contain embedded zeros; it is assumed to end at the first zero.
    pub fn lua_pushstring(L: lua_StatePtr, s: *const core::ffi::c_char) -> *const core::ffi::c_char;
    /// Accepts any acceptable index, or 0, and sets the stack top to this index. If the new top is larger than the old one, then the new elements are filled with nil. If index is 0, then all stack elements are removed.
    pub fn lua_settop(L: lua_StatePtr, index: core::ffi::c_int);
    pub fn lua_copy(L: lua_StatePtr, fromidx: core::ffi::c_int, toidx: core::ffi::c_int);
    pub fn lua_rotate(L: lua_StatePtr, idx: core::ffi::c_int, n: core::ffi::c_int);
    /// Converts the Lua value at the given acceptable index to a C string. If len is not NULL, it also sets *len with the string length. The Lua value must be a string or a number; otherwise, the function returns NULL. If the value is a number, then lua_tolstring also changes the actual value in the stack to a string. (This change confuses lua_next when lua_tolstring is applied to keys during a table traversal.)
    /// lua_tolstring returns a fully aligned pointer to a string inside the Lua state. This string always has a zero ('\0') after its last character (as in C), but can contain other zeros in its body. Because Lua has garbage collection, there is no guarantee that the pointer returned by lua_tolstring will be valid after the corresponding value is removed from the stack.
    pub fn  lua_tolstring(L: lua_StatePtr, index: core::ffi::c_int, len: *mut usize) -> *const core::ffi::c_char;
    /// Creates a new Lua state. It calls lua_newstate with luaL_alloc as the allocator function and the result of luaL_makeseed(NULL) as the seed, and then sets a warning function and a panic function (see §4.4) that print messages to the standard error output.
    /// Returns the new state, or NULL if there is a memory allocation error.
    pub fn luaL_newstate() -> lua_StatePtr;
    /// Opens (loads) and preloads selected standard libraries into the state L. (To preload means to add the library loader into the table package.preload, so that the library can be required later by the program. Keep in mind that require itself is provided by the package library. If a program does not load that library, it will be unable to require anything.)
    /// The integer load selects which libraries to load; the integer preload selects which to preload, among those not loaded. Both are masks formed by a bitwise OR of the following constants:
    /// LUA_GLIBK : the basic library.
    /// LUA_LOADLIBK : the package library.
    /// LUA_COLIBK : the coroutine library.
    /// LUA_STRLIBK : the string library.
    /// LUA_UTF8LIBK : the UTF-8 library.
    /// LUA_TABLIBK : the table library.
    /// LUA_MATHLIBK : the mathematical library.
    /// LUA_IOLIBK : the I/O library.
    /// LUA_OSLIBK : the operating system library.
    /// LUA_DBLIBK : the debug library.
    pub fn luaL_openselectedlibs(L: lua_StatePtr, load: core::ffi::c_int, preload: core::ffi::c_int);
    /// Close all active to-be-closed variables in the main thread, release all objects in the given Lua state (calling the corresponding garbage-collection metamethods, if any), and frees all dynamic memory used by this state.
    /// On several platforms, you may not need to call this function, because all resources are naturally released when the host program ends. On the other hand, long-running programs that create multiple states, such as daemons or web servers, will probably need to close states as soon as they are not needed.
    pub fn lua_close(L: lua_StatePtr);
    /// This function behaves exactly like lua_pcall, except that it allows the called function to yield (see §4.5).
    pub fn lua_pcallk(L: lua_StatePtr, nargs: core::ffi::c_int, nresults: core::ffi::c_int, msgh: core::ffi::c_int, ctx: lua_KContext, k: Option<lua_KFunction>) -> core::ffi::c_int;
    /// If the registry already has the key tname, returns 0. Otherwise, creates a new table to be used as a metatable for userdata, adds to this new table the pair __name = tname, adds to the registry the pair [tname] = new table, and returns 1.
    /// In both cases, the function pushes onto the stack the final value associated with tname in the registry.
    /// Usage note: Beware the use of the return value of this function to conditionally initializes the new metatable (e.g., by adding metamethods to it). If the initialization raises an error, the metatable will not be properly initialized, but a subsequent execution of that code will detect that the metatable already exists and then skip the initialization.
    pub fn luaL_newmetatable(L: lua_StatePtr, tname: *const  core::ffi::c_char) -> core::ffi::c_int;
    /// Returns the type of the value in the given valid index, or LUA_TNONE for a non-valid but acceptable index. The types returned by lua_type are coded by the following constants defined in lua.h: LUA_TNIL, LUA_TNUMBER, LUA_TBOOLEAN, LUA_TSTRING, LUA_TTABLE, LUA_TFUNCTION, LUA_TUSERDATA, LUA_TTHREAD, and LUA_TLIGHTUSERDATA.
    pub fn lua_type(L: lua_StatePtr, index: core::ffi::c_int) -> core::ffi::c_int;
    /// If the value at the given index has a metatable, the function pushes that metatable onto the stack and returns 1. Otherwise, the function returns 0 and pushes nothing on the stack.
    pub fn lua_getmetatable(L: lua_StatePtr, index: core::ffi::c_int) -> core::ffi::c_int; 
    /// Does the equivalent of t[i] = v, where t is the table at the given index and v is the value on the top of the stack.
    /// This function pops the value from the stack. The assignment is raw, that is, it does not use the __newindex metavalue.
    pub fn lua_rawseti(L: lua_StatePtr, index: core::ffi::c_int, i: lua_Integer);
    /// Similar to lua_settable, but does a raw assignment (i.e., without metamethods). The value at index must be a table.
    pub fn lua_rawset(L: lua_StatePtr, index: core::ffi::c_int); 
    /// Does the equivalent to t[n] = v, where t is the value at the given index and v is the value on the top of the stack.
    /// This function pops the value from the stack. As in Lua, this function may trigger a metamethod for the "newindex" event (see §2.4).
    pub fn lua_seti(L: lua_StatePtr, index: core::ffi::c_int, n: lua_Integer);
    /// Converts the Lua value at the given index to the signed integral type lua_Integer. The Lua value must be an integer, or a number or string convertible to an integer (see §3.4.3); otherwise, lua_tointegerx returns 0.
    /// If isnum is not NULL, its referent is assigned a boolean value that indicates whether the operation succeeded.
    pub fn lua_tointegerx(L: lua_StatePtr, index: core::ffi::c_int, isnum: *mut core::ffi::c_int) -> lua_Integer; 
    /// Converts the Lua value at the given index to a C boolean value (0 or 1). Like all tests in Lua, lua_toboolean returns true for any Lua value different from false and nil; otherwise it returns false. (If you want to accept only actual boolean values, use lua_isboolean to test the value's type.)
    pub fn lua_toboolean(L: lua_StatePtr, index: core::ffi::c_int) -> core::ffi::c_int;
    /// Returns the length of the value at the given index. It is equivalent to the '#' operator in Lua (see §3.4.7) and may trigger a metamethod for the "length" event (see §2.4). The result is pushed on the stack.
    pub fn lua_len(L: lua_StatePtr, index: core::ffi::c_int);
    /// Similar to lua_gettable, but does a raw access (i.e., without metamethods). The value at index must be a table.
    pub fn lua_rawget(L: lua_StatePtr, index: core::ffi::c_int) -> core::ffi::c_int;
    /// Pushes onto the stack the value of the global name. Returns the type of that value.
    pub fn  lua_getglobal(L: lua_StatePtr, name: *const core::ffi::c_char) -> core::ffi::c_int;
    /// Sets the value of a closure's upvalue. It assigns the value on the top of the stack to the upvalue and returns its name. It also pops the value from the stack.
    /// Returns NULL (and pops nothing) when the index n is greater than the number of upvalues.
    /// Parameters funcindex and n are as in the function lua_getupvalue.
    pub fn lua_setupvalue(L: lua_StatePtr, funcindex: core::ffi::c_int, n: core::ffi::c_int) -> *const core::ffi::c_char;
    /// Pops a table or nil from the stack and sets that value as the new metatable for the value at the given index. (nil means no metatable.)
    /// (For historical reasons, this function returns an int, which now is always 1.)
    pub fn lua_setmetatable(L: lua_StatePtr, index: core::ffi::c_int) -> core::ffi::c_int; 
/// Does the equivalent to t[k] = v, where t is the value at the given index and v is the value on the top of the stack.
/// This function pops the value from the stack. As in Lua, this function may trigger a metamethod for the "newindex" event (see §2.4).
pub fn lua_setfield(L: lua_StatePtr, index: core::ffi::c_int, k: *const core::ffi::c_char);
/// Creates a new empty table and pushes it onto the stack. Parameter nseq is a hint for how many elements the table will have as a sequence; parameter nrec is a hint for how many other elements the table will have. Lua may use these hints to preallocate memory for the new table. This preallocation may help performance when you know in advance how many elements the table will have. Otherwise you should use the function lua_newtable.
pub fn lua_createtable(L: lua_StatePtr, nseq: core::ffi::c_int, nrec: core::ffi::c_int);
/// Pushes a nil value onto the stack.
pub fn lua_pushnil(L: lua_StatePtr);
/// Pushes onto the stack the value t[k], where t is the value at the given index and k is the value on the top of the stack.
/// This function pops the key from the stack, pushing the resulting value in its place. As in Lua, this function may trigger a metamethod for the "index" event (see §2.4).
/// Returns the type of the pushed value.
pub fn  lua_gettable(L: lua_StatePtr, index: core::ffi::c_int) -> core::ffi::c_int;
/// Does the equivalent to t[k] = v, where t is the value at the given index, v is the value on the top of the stack, and k is the value just below the top.
/// This function pops both the key and the value from the stack. As in Lua, this function may trigger a metamethod for the "newindex" event (see §2.4).
pub fn lua_settable(L: lua_StatePtr, index: core::ffi::c_int);
/// Pushes a copy of the element at the given index onto the stack.
pub fn lua_pushvalue(L: lua_StatePtr, index: core::ffi::c_int);
/// Pushes a boolean value with value b onto the stack.
pub fn lua_pushboolean(L: lua_StatePtr, b: core::ffi::c_int);
/// Pushes an integer with value n onto the stack.
pub fn lua_pushinteger(L: lua_StatePtr, n: lua_Integer);
/// Pushes a new C closure onto the stack. This function receives a pointer to a C function and pushes onto the stack a Lua value of type function that, when called, invokes the corresponding C function. The parameter n tells how many upvalues this function will have (see §4.2).
/// Any function to be callable by Lua must follow the correct protocol to receive its parameters and return its results (see lua_CFunction).
/// When a C function is created, it is possible to associate some values with it, the so called upvalues; these upvalues are then accessible to the function whenever it is called. This association is called a C closure (see §4.2). To create a C closure, first the initial values for its upvalues must be pushed onto the stack. (When there are multiple upvalues, the first value is pushed first.) Then lua_pushcclosure is called to create and push the C function onto the stack, with the argument n telling how many values will be associated with the function. lua_pushcclosure also pops these values from the stack.
/// The maximum value for n is 255.
/// When n is zero, this function creates a light C function, which is just a pointer to the C function. In that case, it never raises a memory error.
pub fn lua_pushcclosure(L: lua_StatePtr, func: Option<lua_CFunction>, n: core::ffi::c_int);
/// Returns the index of the top element in the stack. Because indices start at 1, this result is equal to the number of elements in the stack; in particular, 0 means an empty stack.
pub fn lua_gettop(L: lua_StatePtr) -> core::ffi::c_int;
/// Pushes onto the stack the value t[k], where t is the value at the given index. As in Lua, this function may trigger a metamethod for the "index" event (see §2.4).
/// Returns the type of the pushed value.
pub fn lua_getfield(L: lua_StatePtr, index: core::ffi::c_int, k: *const core::ffi::c_char) -> core::ffi::c_int; 
/// C Callback.
/// 
/// Push this to lua stack instead of unsafe rust code.
/// This will call the rust code via the bridge uptop.
/// 
/// It's up to the bridge to know what function to call. Use upvalues for that.
pub fn pxslua_callback(L: lua_StatePtr) -> core::ffi::c_int;
/// Controls the garbage collector.
/// 
/// This function performs several tasks, according to the value of the parameter what. For options that need extra arguments, they are listed after the option.
/// 
/// LUA_GCCOLLECT: Performs a full garbage-collection cycle.
/// LUA_GCSTOP: Stops the garbage collector.
/// LUA_GCRESTART: Restarts the garbage collector.
/// LUA_GCCOUNT: Returns the current amount of memory (in Kbytes) in use by Lua.
/// LUA_GCCOUNTB: Returns the remainder of dividing the current amount of bytes of memory in use by Lua by 1024.
/// LUA_GCSTEP (size_t n): Performs a step of garbage collection.
/// LUA_GCISRUNNING: Returns a boolean that tells whether the collector is running (i.e., not stopped).
/// LUA_GCINC: Changes the collector to incremental mode. Returns the previous mode (LUA_GCGEN or LUA_GCINC).
/// LUA_GCGEN: Changes the collector to generational mode. Returns the previous mode (LUA_GCGEN or LUA_GCINC).
/// LUA_GCPARAM (int param, int val): Changes and/or returns the value of a parameter of the collector. If val is -1, the call only returns the current value. The argument param must have one of the following values:
/// LUA_GCPMINORMUL: The minor multiplier.
/// LUA_GCPMAJORMINOR: The major-minor multiplier.
/// LUA_GCPMINORMAJOR: The minor-major multiplier.
/// LUA_GCPPAUSE: The garbage-collector pause.
/// LUA_GCPSTEPMUL: The step multiplier.
/// LUA_GCPSTEPSIZE: The step size.
/// 
/// For more details about these options, see collectgarbage.
/// This function should not be called by a finalizer.
pub fn lua_gc(L: lua_StatePtr, what: core::ffi::c_int, ...) -> core::ffi::c_int;
/// Loads a buffer as a Lua chunk. This function uses lua_load to load the chunk in the buffer pointed to by buff with size sz.
/// 
/// This function returns the same results as lua_load. name is the chunk name, used for debug information and error messages. The string mode works as in the function lua_load. In particular, this function supports mode 'B' for fixed buffers.
pub fn luaL_loadbufferx(L: lua_StatePtr, buff: *const core::ffi::c_char, sz: usize, name: *const core::ffi::c_char, mode: *const core::ffi::c_char) -> core::ffi::c_int;
/// Returns 1 if the value at the given index is an integer (that is, the value is a number and is represented as an integer), and 0 otherwise.
pub fn lua_isinteger(L: lua_StatePtr, index: core::ffi::c_int) -> core::ffi::c_int;
/// Converts the Lua value at the given index to the C type lua_Number (see lua_Number). The Lua value must be a number or a string convertible to a number (see §3.4.3); otherwise, lua_tonumberx returns 0.
/// 
/// If isnum is not NULL, its referent is assigned a boolean value that indicates whether the operation succeeded.
pub fn lua_tonumberx(L: lua_StatePtr, index: core::ffi::c_int, isnum: *mut core::ffi::c_int) -> lua_Number;
/// Pushes onto the stack the value t[i], where t is the value at the given index. As in Lua, this function may trigger a metamethod for the "index" event (see §2.4).
/// 
/// Returns the type of the pushed value.
pub fn lua_geti(L: lua_StatePtr, index: core::ffi::c_int, i: lua_Integer) -> core::ffi::c_int;
/// Returns the raw "length" of the value at the given index: for strings, this is the string length; for tables, this is the result of the length operator ('#') with no metamethods; for userdata, this is the size of the block of memory allocated for the userdata. For other values, this call returns 0.
pub fn lua_rawlen(L: lua_StatePtr, index: core::ffi::c_int) -> lua_Unsigned;
/// Pushes a float with value n onto the stack.
pub fn lua_pushnumber(L: lua_StatePtr, n: lua_Number );

}
// == FUNCTIONS END