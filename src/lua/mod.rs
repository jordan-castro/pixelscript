// Copyright 2026 Jordan Castro <jordan@grupojvm.com>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
#![allow(non_snake_case)]

mod engine;
pub mod func;
pub mod module;
pub mod object;
pub mod var;

use etffi::cstring::CStringSafe;
use etffi::ptr_magic::{PtrMagic, ThreadSafePointer};

use crate::lua::func::LUA_MODULE_LOADER_BRIDGE_FUNCTION;
use crate::lua::module::preload_lua_module;
use crate::{
    borrow_string,
    lua::{
        engine::Engine,
        module::compile_chunk,
        var::{from_lua, push_lua_stack},
    },
    pxs_error,
    shared::{
        PixelScript, PxsRes, PxsResult,
        read_file,
        var::{ObjectMethods, pxs_Var, pxs_VarMap},
    },
    with_feature,
};

#[allow(unused)]
#[allow(non_camel_case_types)]
#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub(self) mod lua {
    include!(concat!(env!("OUT_DIR"), "/lua_bindings.rs"));
}

thread_local! {
    static LUASTATE: ThreadSafePointer<State> = ThreadSafePointer::new_owned(new_state());
}

/// This is the Lua state. Each language gets it's own private state
struct State {
    /// The lua engine.
    engine: *mut lua::lua_State,
}

impl PtrMagic for State {}

// Lua globals
const LUA_REGISTRYINDEX: i32 = -(core::ffi::c_int::MAX / 2 + 1000);
const LUA_OK: i32 = 0;
const LUA_TNONE: i32 = -1;
#[allow(unused)]
const LUA_TNIL: i32 = 0;
const LUA_TBOOLEAN: i32 = 1;
// const LUA_TLIGHTUSERDATA: i32 = 2;
const LUA_TNUMBER: i32 = 3;
const LUA_TSTRING: i32 = 4;
const LUA_TTABLE: i32 = 5;
const LUA_TFUNCTION: i32 = 6;
// const LUA_TUSERDATA: i32 = 7;
// const LUA_TTHREAD: i32 = 8;

/// Helper for safely referencing Lua table/functions.
/// Once out of scope, it will drop.
///
/// Can `clone`.
pub(self) struct LuaReference {
    pub idx: i32,
}

impl PtrMagic for LuaReference {}

impl Drop for LuaReference {
    fn drop(&mut self) {
        let state = get_lua_state();
        unsafe {
            lua::luaL_unref((*state).engine, LUA_REGISTRYINDEX, self.idx);
        }
    }
}

impl Clone for LuaReference {
    fn clone(&self) -> Self {
        let state = get_lua_state();
        self.push();
        let new_idx = unsafe { lua::luaL_ref((*state).engine, -1) };

        LuaReference { idx: new_idx }
    }
}

impl LuaReference {
    /// New reference based off position
    pub fn new() -> Self {
        let state = get_lua_state();
        let idx = unsafe { lua::luaL_ref((*state).engine, LUA_REGISTRYINDEX) };

        LuaReference { idx }
    }

    /// Push value to Lua stack
    pub fn push(&self) {
        let state = get_lua_state();
        unsafe {
            lua::lua_rawgeti((*state).engine, LUA_REGISTRYINDEX, self.idx as i64);
        }
    }
}

/// Push a string to lua
pub(self) fn push_string(L: *mut lua::lua_State, contents: &str) {
    let mut cstring = CStringSafe::new();
    unsafe {
        lua::lua_pushstring(L, cstring.new_string(contents));
    }
}

/// Pop the stack
pub(self) fn lua_pop(L: *mut lua::lua_State, amount: core::ffi::c_int) {
    // #define lua_pop(L,n)		lua_settop(L, -(n)-1)
    unsafe {
        lua::lua_settop(L, -(amount) - 1);
    }
}

#[allow(unused)]
/// #define lua_replace(L,idx)	(lua_copy(L, -1, (idx)), lua_pop(L, 1))
pub(self) fn lua_replace(L: *mut lua::lua_State, idx: core::ffi::c_int) {
    unsafe {
        lua::lua_copy(L, -1, idx);
    }
    lua_pop(L, 1);
}

/// #define lua_upvalueindex(i)	(LUA_REGISTRYINDEX - (i))
pub(self) fn lua_upvalueindex(i: core::ffi::c_int) -> core::ffi::c_int {
    LUA_REGISTRYINDEX - i
}

/// #define lua_remove(L,idx)	(lua_rotate(L, (idx), -1), lua_pop(L, 1))
pub(self) fn lua_remove(L: *mut lua::lua_State, idx: core::ffi::c_int) {
    unsafe {
        lua::lua_rotate(L, idx, -1);
        lua_pop(L, 1);
    }
}

/// Get the error as a string (handle it in PXS)
pub(self) fn lua_get_error(L: *mut lua::lua_State) -> String {
    unsafe {
        let lua_error = borrow_string!(lua::lua_tolstring(L, -1, core::ptr::null_mut()));
        // Pop the error obvio
        lua_pop(L, 1);
        lua_error.to_string()
    }
}

fn new_state() -> *mut State {
    unsafe {
        State {
            engine: lua::luaL_newstate(),
        }
        .into_raw()
    }
}

fn init(ptr: *mut State) {
    unsafe {
        let all_libs = !0;
        let safe_libs = all_libs & !(lua::LUA_IOLIBK | lua::LUA_OSLIBK | lua::LUA_DBLIBK);
        lua::luaL_openselectedlibs((*ptr).engine, safe_libs as i32, 0);

        let mut lua_globals = String::new();
        lua_globals.push_str(include_str!("../../core/lua/main.lua"));

        with_feature!("pxs_json", {
            // Load dkjson module
            let _ = preload_lua_module(
                (*ptr).engine,
                include_str!("../../libs/dkjson.lua"),
                "__dkjson__",
            );
            // Load in the pxs_json module
            let _ = preload_lua_module(
                (*ptr).engine,
                include_str!("../../core/lua/pxs_json.lua"),
                "pxs_json",
            );
            // Import it globally
            lua_globals.push_str("\npxs_json = require('pxs_json')\n");
        });
        let _ = execute(ptr, &lua_globals, "<lua_globals>");

        setup_module_loader((*ptr).engine);
    }
}

fn clear(ptr: *mut State) {
    unsafe {
        let L = (*ptr).engine;
        lua::lua_close(L);

        (*ptr).engine = lua::luaL_newstate();
    }
}

/// Get the state of LUA.
fn get_lua_state() -> *mut State {
    LUASTATE.with(|mutex| mutex.get_ptr())
}

/// Get a Engine wrapper of `LUASTATE`
fn get_lua_engine() -> Engine {
    unsafe { Engine::new((*get_lua_state()).engine) }
}

/// Will execute a lua function or chunk on -1 stack.
/// Will add result to stack if not error. If error, its popped from stack.
pub(self) fn lua_call(L: *mut lua::lua_State, args: i32, results: i32) -> PxsRes<()> {
    unsafe {
        // 1
        let code = lua::lua_pcallk(L, args, results, 0, 0, None); // results
        if code != LUA_OK {
            let lua_error = lua_get_error(L);
            return pxs_error!("{lua_error}");
        }
        Ok(())
    }
}

/// #define lua_pushglobaltable(L)  \
///	   ((void)lua_rawgeti(L, LUA_REGISTRYINDEX, LUA_RIDX_GLOBALS))
pub(self) fn lua_push_globals(L: *mut lua::lua_State) {
    unsafe {
        lua::lua_rawgeti(L, LUA_REGISTRYINDEX, lua::LUA_RIDX_GLOBALS as i64);
    }
}

/// Execute some orbituary lua code.
/// Returns a String. Empty means no error happened and was successful!
pub(self) fn execute(state: *mut State, code: &str, file_name: &str) -> String {
    unsafe {
        let L = (*state).engine;
        let chunk_res = compile_chunk(L, code, file_name); // 1
        if chunk_res.is_err() {
            return chunk_res.unwrap_err().to_string();
        }

        let call_res = lua_call(L, 0, 0); // 0
        if call_res.is_err() {
            return call_res.unwrap_err().to_string();
        }
    }
    // Otherwise we are good to go!
    String::from("")
}

/// package.searchers[] add this loader function to that table.
pub(self) fn module_loader_func(L: *mut lua::lua_State) -> PxsRes<i32> {
    let path_idx = 1;
    let mut engine = Engine::without_alloc(L);
    let path = engine.to_string(path_idx);

    // Path mut be grand.parent.child
    if path.contains("/") {
        return pxs_error!("Cannot have '/' in lua path.");
    } 
    let path = path.replace(".", "/");
    let path = format!("{path}.lua");

    let contents = read_file(&path);
    if contents.is_empty() {
        return pxs_error!("{path} was not found.");
    }

    // Compile chunk
    let _ = engine.compile_chunk(&contents, &path)?;

    // Donezo!
    Ok(1)
}

/// Custom moduile loader function
fn setup_module_loader(L: *mut lua::lua_State) {
    let mut engine = Engine::new(L);
    // Get package
    engine.get_global("package");
    let package_idx = engine.get_top();

    // Get searchers
    engine.push_string("searchers");
    engine.raw_get(package_idx);
    let s_idx = engine.get_top();

    // Pass Function type
    engine.push_integer(LUA_MODULE_LOADER_BRIDGE_FUNCTION);
    // Push module loader
    engine.push_function(lua::pxslua_callback, 1);
    // Add to table (redefine path searcher)
    engine.set_index(s_idx, 2 as i32);

    // Remove C Path searcher
    engine.push_nil();
    engine.set_index(s_idx, 3);
    
    // Remove all in one submodule searcher.
    engine.push_nil();
    engine.set_index(s_idx, 4);
}

/// Add variables to a Table from a Map
fn add_variables_to_table(state: *mut State, table: i32, map: &pxs_VarMap) -> PxsRes<()> {
    let keys = map.keys();
    let mut engine = Engine::from_state(state);
    for k in keys {
        // Push key to lua
        engine.push_pxs(k)?;
        let value = map.get_item(k).unwrap();
        engine.push_pxs(value)?;
        engine.set_table(table);
    }

    Ok(())
}

/// Remove variables from a Table.
fn remove_variables_from_table(state: *mut State, table: i32, map: &pxs_VarMap) -> PxsRes<()> {
    let keys = map.keys();
    let mut engine = Engine::from_state(state);
    for k in keys {
        // Convert to lua
        engine.push_pxs(k)?;
        engine.push_nil();
        engine.set_table(table);
    }

    Ok(())
}

pub struct LuaScripting;

impl PixelScript for LuaScripting {
    fn add_module(source: std::sync::Arc<crate::shared::module::pxs_Module>) {
        let state = get_lua_state();
        let res = module::add_module(state, source);
        if res.is_err() {
            panic!("{:#?}", res);
        }
    }

    fn execute(code: &str, file_name: &str) -> PxsResult {
        let state = get_lua_state();
        let err = execute(state, code, file_name);
        if err.is_empty() {
            Ok(pxs_Var::new_null())
        } else {
            Ok(pxs_Var::new_exception(err))
        }
    }

    fn start() {
        // Initalize the state
        init(get_lua_state());
    }

    fn stop() {
        clear(get_lua_state());
    }

    fn start_thread() {
        Self::start();
    }

    fn stop_thread() {
        Self::stop();
    }

    fn clear() {
        let state = get_lua_state();
        clear(state);
        init(state);
    }

    fn eval(code: &str, name: &str) -> PxsResult {
        let state = get_lua_state();
        let mut engine = Engine::from_state(state);
        engine.compile_chunk(code, name)?;
        engine.call(0, 1)?;
        engine.from_lua(-1)
    }

    fn compile(code: &str, global_scope: pxs_Var) -> PxsResult {
        let state = get_lua_state();
        let mut engine = Engine::from_state(state);
        // Compile chunk
        let chunk = engine.compile_chunk(code, "<lua_chunk>")?;
        // Create a scope table and set a meta table that has __index == globals
        let env_table = if global_scope.is_map() {
            // 2
            engine.push_pxs(&global_scope)?
        } else {
            engine.create_table(0, 0)
        };
        let mt_table = engine.create_table(0, 0);

        // Get globals
        engine.push_globals();
        // Assign to mt
        engine.set_field(mt_table, "__index");
        // Assign mt_table to env_table
        engine.set_meta(env_table);
        // save env_table
        let env_table_pxs = engine.from_lua(env_table)?; // 1
        // push back to stack
        engine.push_pxs(&env_table_pxs)?;
        // Set as _ENV
        engine.set_upvalue(chunk, 1);
        // Create Code Object
        let code_object = pxs_Var::new_list();
        let list = code_object.get_list().unwrap();

        // Push chunk
        engine.push_value(chunk);
        list.add_item(engine.from_lua(-1)?); // 0
        list.add_item(env_table_pxs);

        Ok(code_object)
    }

    fn exec_object(code: pxs_Var, local_scope: pxs_Var) -> PxsResult {
        let mut engine = get_lua_engine();
        // Get code object and globals
        let (code_object, global_scope) = {
            let list = code.get_list().unwrap();
            (list.get_item(1).unwrap(), list.get_item(2).unwrap())
        };

        // Add locals if necessary.
        if !local_scope.is_null() {
            // Add local scope to global scope
            engine.push_pxs(global_scope)?;
            add_variables_to_table(
                get_lua_state(),
                engine.get_top(),
                local_scope.get_map().unwrap(),
            )?;
            // Pop global scope.
            engine.pop(1);
        }

        // Push code object
        engine.push_pxs(code_object)?;
        engine.call(0, 1)?;

        let res = engine.get_top_pxs()?;

        // Remove locals if necessary
        if !local_scope.is_null() {
            engine.push_pxs(global_scope)?;
            remove_variables_from_table(
                get_lua_state(),
                engine.get_top(),
                local_scope.get_map().unwrap(),
            )?;
        }

        Ok(res)
    }

    fn debug() -> String {
        String::new()
        // let state = get_lua_state();
        // let tables = unsafe { &(*state).tables };
        // format!("{{tables: {:#?}}}", tables)
    }

    fn garbage_collect() {
        let state = get_lua_state();
        unsafe {
            lua::lua_gc((*state).engine, lua::LUA_GCCOLLECT as i32);
        }
    }
}

/// Push args to lua stack.
fn args_to_lua(engine: &mut Engine, args: &Vec<pxs_Var>) -> PxsRes<()> {
    for arg in args.iter() {
        engine.push_pxs(arg)?;
    }
    Ok(())
}

impl ObjectMethods for LuaScripting {
    fn object_call(
        var: &crate::shared::var::pxs_Var,
        method: &str,
        args: &mut crate::shared::var::pxs_VarList,
    ) -> PxsResult {
        let mut engine = get_lua_engine();
        engine.push_pxs(var)?;
        engine.get_field(-1, method);
        args_to_lua(&mut engine, &args.vars)?;
        engine.call(args.len() as i32, 1)?;
        engine.get_top_pxs()
    }

    fn call_method(method: &str, args: &mut crate::shared::var::pxs_VarList) -> PxsResult {
        let mut engine = get_lua_engine();
        engine.push_globals();
        engine.get_field(-1, method);
        args_to_lua(&mut engine, &args.vars)?;
        engine.call(args.len() as i32, 1)?;
        engine.get_top_pxs()
    }

    fn var_call(
        method: &crate::shared::var::pxs_Var,
        args: &mut crate::shared::var::pxs_VarList,
    ) -> PxsResult {
        let mut engine = get_lua_engine();
        engine.push_pxs(method)?;
        args_to_lua(&mut engine, &args.vars)?;
        engine.call(args.len() as i32, 1)?;
        engine.get_top_pxs()
    }

    fn get(var: &pxs_Var, key: &str) -> PxsResult {
        let mut engine = get_lua_engine();
        engine.push_pxs(var)?;
        engine.get_field(-1, key);
        let result = engine.get_top_pxs()?;
        Ok(result)
    }

    fn set(var: &pxs_Var, key: &str, value: &pxs_Var) -> PxsRes<()> {
        let mut engine = get_lua_engine();

        // push object to lua
        let table = engine.push_pxs(var)?;

        // Set key
        engine.push_string(key);
        engine.push_pxs(value)?;

        engine.set_table(table);

        Ok(())
    }

    fn get_from_name(name: &str) -> PxsResult {
        let mut engine = get_lua_engine();
        // Push the global table
        engine.push_globals();
        // Get field
        engine.get_field(-1, name);
        // result
        engine.from_lua(-1)
    }
}
