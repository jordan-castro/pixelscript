// Exposes `pxs_wrap`.

// pub extern "C" fn pxs_eval(script: *const c_char, rt: pxs_Runtime) -> pxs_VarT
let pxs_eval;
/**
  pub extern "C" fn pxs_exec(
    runtime: pxs_Runtime,
    code: *const c_char,
    file_name: *const c_char,
) -> pxs_VarT {
 */ 
let pxs_exec;
// pub extern "C" fn pxs_newmod(name: *const c_char) -> *mut pxs_Module {
let pxs_newmod;
// pub extern "C" fn pxs_finalize() {
let pxs_finalize;
// pub extern "C" fn pxs_initialize() {
let pxs_initialize;
// pub extern "C" fn pxs_addfunc(module_ptr: *mut pxs_Module, name: *const c_char, func: pxs_Func) {
let pxs_addfunc;
// unsafe extern "C" fn(args: *mut pxs_Var) -> *mut pxs_Var
const pxs_Func = 'pp';
/**
Deleter Function type. It takes a *void, and returns void.

pub type pxs_DeleterFn = unsafe extern "C" fn(*mut c_void);
 */
const pxs_DeleterFn = 'vp';

/**
Add the module finally to the runtime.

After this you can forget about the ptr since PM handles it.

module_ptr:TRANSFER

pub extern "C" fn pxs_addmod(module_ptr: *mut pxs_Module) {
 */
let pxs_addmod;

/**
Add a Module to a Module

This transfers ownership.

parent_ptr:BORROW
child_ptr:TRANSFER

pub extern "C" fn pxs_add_submod(parent_ptr: *mut pxs_Module, child_ptr: *mut pxs_Module) {
 */
let pxs_add_submod;

/** 
Make a new Null var.

return:OWNED
pub extern "C" fn pxs_newnull() -> pxs_VarT {

*/
let pxs_newnull;

/**
Free the string created by the pixelscript library

string:TRANSFER

pub extern "C" fn pxs_freestr(string: *mut c_char) {
 */
let pxs_freestr;

/** 
Add the same function under different names.
 
module_ptr:BORROW
func_list:TRANSFER

pub extern "C" fn pxs_addfuncs(module_ptr: *mut pxs_Module, func_list: pxs_VarT, func: pxs_Func) {
*/
let pxs_addfuncs;

/**
Add a Varible to a module.

Pass in the module pointer and variable params.

Variable ownership is transfered.

module_ptr:BORROW
variable:TRANSFER

pub extern "C" fn pxs_addvar(
    module_ptr: *mut pxs_Module,
    name: *const c_char,
    variable: *mut pxs_Var,
) {
 */
let pxs_addvar;

/**
Free a module.

module_ptr:TRANSFER

pub extern "C" fn pxs_freemod(module_ptr: *mut pxs_Module) {
 */
let pxs_freemod;

/**
Create a new object with a Type.
 
This is the same as `pxs_newobject` but it defines a `type` on the `pxs_PixelObject`.
 
This will not cause UB. Retrieve the host pointer using `pxs_gettype`. A `type_id` < 0 means no type.
 
ptr:OWNED
return:OWNED

pub extern "C" fn pxs_newtype(
    ptr: pxs_Opaque,
    free_method: pxs_DeleterFn,
    type_name: *const c_char,
    type_id: i32
) -> *mut pxs_PixelObject {
 */
let pxs_newtype;

/**
Free a PixelScript var.

You should only free results from `pxs_object_call`

var:TRANSFER

pub extern "C" fn pxs_freevar(var: *mut pxs_Var) {
 */
let pxs_freevar;

/**
Check if variable is `pxs_Exception`.
 
```c
bool is = pxs_isexception(pxs_arg(args, 0));
```
 
var:BORROW

pub extern "C" fn pxs_isexception(var: pxs_VarT) -> bool {
 */
let pxs_isexception;


const pxs_Runtime = {
    pxs_Lua: 0,
    pxs_Python: 1,
    pxs_JavaScript: 2
};

function pxs_wrap(module, start=true) {
    pxs_isexception = module.cwrap('pxs_isexception', 'bool', ['number']);
    pxs_newtype = module.cwrap('pxs_newtype', 'number', ['number', 'number', 'string', 'number']);
    pxs_freemod = module.cwrap('pxs_freemod', '', ['number']);
    pxs_addvar = module.cwrap('pxs_addvar', '', ['number', 'string', 'number']);
    pxs_initialize = module.cwrap('pxs_initialize');
    pxs_finalize = module.cwrap('pxs_finalize');
    pxs_eval = module.cwrap('pxs_eval', 'number', ['string', 'number']);
    pxs_exec = module.cwrap('pxs_exec', 'number', ['number', 'string', 'string']);
    pxs_newmod = module.cwrap('pxs_newmod', 'number', ['string']);
    pxs_addfunc = (...args) => {
        const wrapper = module.cwrap('pxs_addfunc', '', ['number', 'string', 'number']);
        
        let ptr = args[0];
        let name = args[1];
        let func = arsg[2];

        let func_p = module.addFunction(func, 'pp');

        wrapper(ptr, name, func_p);
    };
    pxs_addmod = module.cwrap('pxs_addmod', '', ['number']);
    pxs_add_submod = module.cwrap('pxs_add_submod', '', ['number', 'number']);
    pxs_newnull = module.cwrap('pxs_newnull');
    pxs_freestr = module.cwrap('pxs_freestr', '', ['number']);
    pxs_addfuncs = (...args) => {
        const wrapper = module.cwrap('pxs_addfuncs', '', ['number', 'number', 'number']);

        let ptr = args[0];
        let list = args[1];
        let func = args[2];

        let func_p = module.addFunction(func, 'pp');
        wrapper(ptr, list, func_p);
    }
    pxs_freevar = module.cwrap('pxs_freevar', '', ['number']);

    if (start) {
        pxs_initialize();
    }
}
