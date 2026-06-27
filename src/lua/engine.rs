#![allow(unused)]
use etffi::cstring::CStringSafe;

use crate::{
    borrow_string,
    lua::{
        State, compile_chunk, from_lua, lua, lua_call, lua_pop, lua_push_globals, lua_remove,
        lua_upvalueindex, push_lua_stack, push_string,
    },
    shared::{PxsRes, PxsResult, var::pxs_Var},
};

/// Engine that handles all Lua calls.
///
/// Keeps track of allocations.
///
/// You should instance this in every function using lua callbacks. It will clear the stack when it goes out of scope.
pub struct Engine {
    /// The internal engine
    L: *mut lua::lua_State,
    /// The number of allocations
    num_allocated: u32,
    /// Uses allocation tracking
    use_allocation_tracking: bool,
}

impl Drop for Engine {
    fn drop(&mut self) {
        self.reset();
    }
}

impl Engine {
    pub fn new(L: *mut lua::lua_State) -> Self {
        Engine {
            L,
            num_allocated: 0,
            use_allocation_tracking: true,
        }
    }

    pub fn from_state(state: *mut State) -> Self {
        unsafe { Engine::new((*state).engine) }
    }

    /// Engine without allocation tracking. Use this in lua functions.
    pub fn without_alloc(L: *mut lua::lua_State) -> Self {
        Engine {
            L: L,
            num_allocated: 0,
            use_allocation_tracking: false,
        }
    }

    /// Reset the allocations of the engine
    pub fn reset(&mut self) {
        if !self.use_allocation_tracking {
            return;
        }
        if self.num_allocated == 0 {
            return;
        }
        self.pop(self.num_allocated as i32);
    }

    /// Update num allocated
    pub fn decrease(&mut self, amount: u32) {
        if !self.use_allocation_tracking {
            return;
        }
        self.num_allocated = if amount > self.num_allocated {
            0
        } else {
            self.num_allocated - amount
        };
    }

    /// Update num allocated
    pub fn increase(&mut self, amount: u32) {
        if !self.use_allocation_tracking {
            return;
        }
        self.num_allocated += amount;
    }

    /// Push a `pxs_Var` to the lua stack.
    pub fn push_pxs(&mut self, var: &pxs_Var) -> PxsRes<i32> {
        let res = push_lua_stack(var)?;
        self.num_allocated += 1;
        Ok(res)
    }

    /// Call `lua_getfield`.
    pub fn get_field(&mut self, idx: i32, field: &str) {
        let mut cstring = CStringSafe::new();
        unsafe {
            lua::lua_getfield(self.L, idx, cstring.new_string(field));
        }
        self.num_allocated += 1;
    }

    /// Call `lua_gettop`
    pub fn get_top(&self) -> i32 {
        unsafe { lua::lua_gettop(self.L) }
    }

    /// Get the top value as `pxs_Var`
    pub fn get_top_pxs(&self) -> PxsResult {
        from_lua(-1)
    }

    /// Call `lua_pop`
    pub fn pop(&mut self, amount: i32) {
        lua_pop(self.L, amount);
        self.decrease(amount as u32);
    }

    /// Call `lua_call`
    pub fn call(&mut self, args_len: i32, result_len: i32) -> PxsRes<()> {
        lua_call(self.L, args_len, result_len)?;
        self.decrease(1 + args_len as u32); // function and args are popped automatically.
        self.num_allocated += result_len as u32;

        Ok(())
    }

    /// Push function
    pub fn push_function(
        &mut self,
        func: unsafe extern "C" fn(*mut lua::lua_State) -> core::ffi::c_int,
        upvalues: i32,
    ) {
        unsafe {
            lua::lua_pushcclosure(self.L, Some(func), upvalues);
        }
        self.increase(1);
        self.decrease(upvalues as u32);
        // self.num_allocated += 1;
    }

    /// Push string
    pub fn push_string(&mut self, contents: &str) {
        push_string(self.L, contents);
        self.num_allocated += 1;
    }

    /// Push integer
    pub fn push_integer(&mut self, i: i32) {
        unsafe {
            lua::lua_pushinteger(self.L, i as i64);
        }
        self.increase(1);
    }

    /// Push boolean
    pub fn push_boolean(&mut self, b: bool) {
        unsafe {
            lua::lua_pushboolean(self.L, b as i32);
        }
        self.increase(1);
    }

    /// Push value
    pub fn push_value(&mut self, value: i32) {
        unsafe {
            lua::lua_pushvalue(self.L, value);
        }
        self.num_allocated += 1;
    }

    /// Push globals
    pub fn push_globals(&mut self) {
        lua_push_globals(self.L);
        self.num_allocated += 1;
    }

    /// Call `lua_settable`
    pub fn set_table(&mut self, table: i32) {
        unsafe {
            lua::lua_settable(self.L, table);
        }
        self.decrease(2);
    }

    /// Call `lua_gettable`
    pub fn get_table(&mut self, table: i32) {
        unsafe {
            lua::lua_gettable(self.L, table);
        }
        // self.decrease(1);
        // self.increase(1);
    }

    /// Push nil
    pub fn push_nil(&mut self) {
        unsafe {
            lua::lua_pushnil(self.L);
        }
        self.increase(1);
    }

    /// Compile lua code into a chunk
    pub fn compile_chunk(&mut self, code: &str, name: &str) -> PxsRes<i32> {
        let res = compile_chunk(self.L, code, name)?;
        self.increase(1);
        Ok(res)
    }

    /// Call `lua_createtable` and get its index
    pub fn create_table(&mut self, narr: i32, nrec: i32) -> i32 {
        unsafe {
            lua::lua_createtable(self.L, narr, nrec);
        }
        self.increase(1);
        self.get_top()
    }

    /// Call `lua_setfield`
    pub fn set_field(&mut self, table: i32, field: &str) {
        let mut cstring = CStringSafe::new();
        unsafe {
            lua::lua_setfield(self.L, table, cstring.new_string(field));
        }
        self.decrease(1);
    }

    /// Call `lua_setmetatable`
    pub fn set_meta(&mut self, table: i32) {
        unsafe {
            lua::lua_setmetatable(self.L, table);
        }
        self.decrease(1);
    }

    /// Call `lua_setupvalue`
    pub fn set_upvalue(&mut self, func_idx: i32, num: i32) {
        unsafe {
            lua::lua_setupvalue(self.L, func_idx, num);
        }
        self.decrease(1);
    }

    /// Call `from_lua`
    pub fn from_lua(&mut self, idx: i32) -> PxsResult {
        self.decrease(1);
        from_lua(idx)
    }

    /// Call `lua_getglobal`
    pub fn get_global(&mut self, key: &str) {
        let mut cstring = CStringSafe::new();
        unsafe {
            lua::lua_getglobal(self.L, cstring.new_string(key));
        }
        self.increase(1);
    }

    /// Call `lua_rawget`
    pub fn raw_get(&mut self, table: i32) {
        unsafe {
            lua::lua_rawget(self.L, table);
        }
        // We remove the key BUT add its value.
        // self.decrease(1);
        // self.increase(1);
    }

    /// Call `lua_rawgeti`
    pub fn raw_get_index(&mut self, table: i32, i: i32) {
        unsafe {
            lua::lua_rawgeti(self.L, table, i as i64);
        }
        self.increase(1);
    }

    /// Call `lua_remove`
    pub fn remove(&self, i: i32) {
        if !self.use_allocation_tracking {
            return;
        }
        lua_remove(self.L, i);
    }

    /// Call `lua_len`
    ///
    /// Also returns the length.
    pub fn len(&mut self, table: i32) -> i64 {
        unsafe {
            lua::lua_len(self.L, table);
        }
        self.increase(1);

        unsafe { lua::lua_tointegerx(self.L, -1, core::ptr::null_mut()) }
    }

    /// Call `lua_toboolean`
    pub fn to_boolean(&self, idx: i32) -> bool {
        unsafe { lua::lua_toboolean(self.L, idx) == 1 }
    }

    /// Call `lua_tointeger`
    pub fn to_integer(&self, idx: i32) -> i64 {
        unsafe { lua::lua_tointegerx(self.L, idx, core::ptr::null_mut()) }
    }

    /// Call `lua_tostring`
    pub fn to_string(&self, idx: i32) -> String {
        let s = unsafe { lua::lua_tolstring(self.L, idx, core::ptr::null_mut()) };
        borrow_string!(s).to_string()
    }

    /// Call `lua_seti`
    pub fn set_index(&mut self, table: i32, index: i32) {
        unsafe {
            lua::lua_seti(self.L, table, index as i64);
        }
        self.decrease(1);
    }

    /// Call `lua_rawset`
    pub fn raw_set(&mut self, table: i32) {
        unsafe {
            lua::lua_rawset(self.L, table);
        }
        self.decrease(2);
    }

    /// Call `lua_rawseti`
    pub fn raw_set_index(&mut self, table: i32, index: i32) {
        unsafe {
            lua::lua_rawseti(self.L, table, index as i64);
        }
        self.decrease(1);
    }

    /// Call `lua_getmetatable`
    pub fn get_meta(&mut self, table: i32) {
        unsafe {
            lua::lua_getmetatable(self.L, table);
        }
        self.increase(1);
    }

    /// Call `lua_type`
    pub fn get_type(&self, idx: i32) -> i32 {
        unsafe { lua::lua_type(self.L, idx) }
    }

    /// Call `lua_getupvalue`
    pub fn get_upvalue(&self, idx: i32) -> i32 {
        unsafe {
            let position = lua_upvalueindex(idx);
            self.to_integer(position) as i32
        }
    }

    /// Call `luaL_newmetatable`
    ///
    /// Returns 0, 1
    ///
    /// 0 = Already exists
    /// 1 = New
    pub fn new_meta(&mut self, name: &str) -> i32 {
        let mut cstring = CStringSafe::new();
        unsafe {
            let res = lua::luaL_newmetatable(self.L, cstring.new_string(name));
            self.increase(1);
            res
        }
    }

    /// Get type of current top
    pub fn get_top_type(&self) -> i32 {
        self.get_type(self.get_top())
    }
}
