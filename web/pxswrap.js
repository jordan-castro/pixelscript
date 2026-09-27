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

    if (start) {
        pxs_initialize();
    }
}