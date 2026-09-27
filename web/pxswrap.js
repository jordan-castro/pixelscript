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

const pxs_Runtime = {
    pxs_Lua: 0,
    pxs_Python: 1,
    pxs_JS: 2
};

function pxs_wrap(module, start=true) {
    pxs_initialize = module.cwrap('pxs_initialize');
    pxs_finalize = module.cwrap('pxs_finalize');
    pxs_eval = module.cwrap('pxs_eval', 'number', ['string', 'number']);
    pxs_exec = module.cwrap('pxs_exec', 'number', ['number', 'string', 'string']);
    pxs_newmod = module.cwrap('pxs_newmod', 'number', ['string']);
    pxs_addfunc = (args) => {
        const wrapper = module.cwrap('pxs_addfunc', '', ['number', 'string', 'number']);
        
        let ptr = args[0];
        let name = args[1];
        let func = arsg[2];

        let func_p = module.addFunction(func, 'pp');

        wrapper(ptr, name, func_p);
    };
    pxs_addmod = module.cwrap('pxs_addmod', '', ['number']);
    pxs_add_submod = module.cwrap('pxs_add_submod', '', ['number', 'number']);
    pxs_newnull = module.cwrap('pxs_newnull', '', []);

    if (start) {
        pxs_initialize();
    }
}