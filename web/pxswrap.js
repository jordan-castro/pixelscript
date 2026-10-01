const pxs_VarType ={ pxs_Int64 : 0,pxs_UInt64 : 1,pxs_String : 2,pxs_Bool : 3,pxs_Float64 : 4,pxs_Null : 5,pxs_Object : 6,pxs_HostObject : 7,pxs_List : 8,pxs_Function : 9,pxs_Factory : 10,pxs_Exception : 11,pxs_Map : 12,pxs_Byte : 13 };
const pxs_Runtime ={ pxs_Lua : 0,pxs_Python : 1,pxs_JavaScript : 2,pxs_Wren : 3 };
const pxs_ModuleFlag ={ pxs_NONE : 0,pxs_JSON : 1,pxs_MEM : 2,pxs_OS : 4,pxs_PXS : 8,pxs_FS : 16,pxs_SHELL : 32,pxs_ZIP : 64,pxs_HTTP : 128 };let pxs_version;
let pxs_initialize;
let pxs_finalize;
let pxs_exec;
let pxs_freestr;
let pxs_newmod;
let pxs_addfunc;
let pxs_addfuncs;
let pxs_addvar;
let pxs_add_submod;
let pxs_addmod;
let pxs_freemod;
let pxs_newtype;
let pxs_newobject;
let pxs_object_addfunc;
let pxs_object_add_reffunc;
let pxs_object_addprop;
let pxs_addobject;
let pxs_newstring;
let pxs_newnull;
let pxs_newhost;
let pxs_newint;
let pxs_newuint;
let pxs_newbool;
let pxs_newfloat;
let pxs_object_callrt;
let pxs_objectcall;
let pxs_getint;
let pxs_getuint;
let pxs_getfloat;
let pxs_getbool;
let pxs_getstring;
let pxs_varis;
let pxs_set_filereader;
let pxs_set_dirreader;
let pxs_freevar;
let pxs_startthread;
let pxs_stopthread;
let pxs_clear;
let pxs_call;
let pxs_tostring;
let pxs_newlist;
let pxs_listadd;
let pxs_listget;
let pxs_listset;
let pxs_listlen;
let pxs_varcall;
let pxs_newcopy;
let pxs_objectget;
let pxs_objectset;
let pxs_eval;
let pxs_evalnamed;
let pxs_newfactory;
let pxs_gettype;
let pxs_gethost;
let pxs_debugvar;
let pxs_newexception;
let pxs_var_fromname;
let pxs_listdel;
let pxs_new_shallowcopy;
let pxs_compile;
let pxs_execobject;
let pxs_newmap;
let pxs_map_addpair;
let pxs_map_delitem;
let pxs_maplen;
let pxs_mapkeys;
let pxs_mapget;
let pxs_listinsert;
let pxs_newarena;
let pxs_freearena;
let pxs_arenaput;
let pxs_arena_putstr;
let pxs_debugstate;
let pxs_garbagecollect;
let pxs_getidx;
let pxs_arg;
let pxs_getrt;
let pxs_argc;
let pxs_newbytes;
let pxs_varsize;
let pxs_copybytes;
let pxs_copystring;
let pxs_smart_getstring;
let pxs_smart_copystring;
let pxs_vartype;
let pxs_isstring;
let pxs_isint;
let pxs_isuint;
let pxs_isbool;
let pxs_isfloat;
let pxs_isnull;
let pxs_isobject;
let pxs_is_hostobject;
let pxs_islist;
let pxs_isfunction;
let pxs_isfactory;
let pxs_isexception;
let pxs_ismap;
let pxs_isbyte;
let pxs_addmod2;
let pxs_json_encode;
let pxs_json_decode;
let pxs_core_init;
let pxs_core_initall;
function pxs_wrap(module, start=true) {
        pxs_version = module.cwrap("pxs_version","number" );
pxs_initialize = module.cwrap("pxs_initialize" );
pxs_finalize = module.cwrap("pxs_finalize" );
pxs_exec = module.cwrap("pxs_exec","number",["number","string","string"] );
pxs_freestr = module.cwrap("pxs_freestr","",["string"] );
pxs_newmod = module.cwrap("pxs_newmod","number",["string"] );
pxs_addfunc = (...args) => {
    let a_0 = args[0];
let a_1 = args[1];
let a_2 = args[2];

    let wrapper = module.cwrap("pxs_addfunc","",["number","string","number"] );

    a_2 = module.addFunction(a_2, 'pp');

    return wrapper(a_0,a_1,a_2);
}
;
pxs_addfuncs = (...args) => {
    let a_0 = args[0];
let a_1 = args[1];
let a_2 = args[2];

    let wrapper = module.cwrap("pxs_addfuncs","",["number","number","number"] );

    a_2 = module.addFunction(a_2, 'pp');

    return wrapper(a_0,a_1,a_2);
}
;
pxs_addvar = module.cwrap("pxs_addvar","",["number","string","number"] );
pxs_add_submod = module.cwrap("pxs_add_submod","",["number","number"] );
pxs_addmod = module.cwrap("pxs_addmod","",["number"] );
pxs_freemod = module.cwrap("pxs_freemod","",["number"] );
pxs_newtype = (...args) => {
    let a_0 = args[0];
let a_1 = args[1];
let a_2 = args[2];
let a_3 = args[3];

    let wrapper = module.cwrap("pxs_newtype","number",["number","number","string","number"] );

    a_1 = module.addFunction(a_1, 'p');

    return wrapper(a_0,a_1,a_2,a_3);
}
;
pxs_newobject = (...args) => {
    let a_0 = args[0];
let a_1 = args[1];
let a_2 = args[2];

    let wrapper = module.cwrap("pxs_newobject","number",["number","number","string"] );

    a_1 = module.addFunction(a_1, 'p');

    return wrapper(a_0,a_1,a_2);
}
;
pxs_object_addfunc = (...args) => {
    let a_0 = args[0];
let a_1 = args[1];
let a_2 = args[2];

    let wrapper = module.cwrap("pxs_object_addfunc","",["number","string","number"] );

    a_2 = module.addFunction(a_2, 'pp');

    return wrapper(a_0,a_1,a_2);
}
;
pxs_object_add_reffunc = (...args) => {
    let a_0 = args[0];
let a_1 = args[1];
let a_2 = args[2];

    let wrapper = module.cwrap("pxs_object_add_reffunc","",["number","string","number"] );

    a_2 = module.addFunction(a_2, 'pp');

    return wrapper(a_0,a_1,a_2);
}
;
pxs_object_addprop = (...args) => {
    let a_0 = args[0];
let a_1 = args[1];
let a_2 = args[2];

    let wrapper = module.cwrap("pxs_object_addprop","",["number","string","number"] );

    a_2 = module.addFunction(a_2, 'pp');

    return wrapper(a_0,a_1,a_2);
}
;
pxs_addobject = (...args) => {
    let a_0 = args[0];
let a_1 = args[1];
let a_2 = args[2];

    let wrapper = module.cwrap("pxs_addobject","",["number","string","number"] );

    a_2 = module.addFunction(a_2, 'pp');

    return wrapper(a_0,a_1,a_2);
}
;
pxs_newstring = module.cwrap("pxs_newstring","number",["string"] );
pxs_newnull = module.cwrap("pxs_newnull","number" );
pxs_newhost = module.cwrap("pxs_newhost","number",["number"] );
pxs_newint = module.cwrap("pxs_newint","number",["number"] );
pxs_newuint = module.cwrap("pxs_newuint","number",["number"] );
pxs_newbool = module.cwrap("pxs_newbool","number",["bool"] );
pxs_newfloat = module.cwrap("pxs_newfloat","number",["number"] );
pxs_object_callrt = module.cwrap("pxs_object_callrt","number",["number","number","string","number"] );
pxs_objectcall = module.cwrap("pxs_objectcall","number",["number","number","string","number"] );
pxs_getint = module.cwrap("pxs_getint","number",["number"] );
pxs_getuint = module.cwrap("pxs_getuint","number",["number"] );
pxs_getfloat = module.cwrap("pxs_getfloat","number",["number"] );
pxs_getbool = module.cwrap("pxs_getbool","bool",["number"] );
pxs_getstring = module.cwrap("pxs_getstring","string",["number"] );
pxs_varis = module.cwrap("pxs_varis","bool",["number","number"] );
pxs_set_filereader = (...args) => {
    let a_0 = args[0];

    let wrapper = module.cwrap("pxs_set_filereader","",["number"] );

    a_0 = module.addFunction(a_0, 'ps');

    return wrapper(a_0);
}
;
pxs_set_dirreader = (...args) => {
    let a_0 = args[0];

    let wrapper = module.cwrap("pxs_set_dirreader","",["number"] );

    a_0 = module.addFunction(a_0, 'ps');

    return wrapper(a_0);
}
;
pxs_freevar = module.cwrap("pxs_freevar","",["number"] );
pxs_startthread = module.cwrap("pxs_startthread" );
pxs_stopthread = module.cwrap("pxs_stopthread" );
pxs_clear = module.cwrap("pxs_clear" );
pxs_call = module.cwrap("pxs_call","number",["number","string","number"] );
pxs_tostring = module.cwrap("pxs_tostring","number",["number","number"] );
pxs_newlist = module.cwrap("pxs_newlist","number" );
pxs_listadd = module.cwrap("pxs_listadd","number",["number","number"] );
pxs_listget = module.cwrap("pxs_listget","number",["number","number"] );
pxs_listset = module.cwrap("pxs_listset","bool",["number","number","number"] );
pxs_listlen = module.cwrap("pxs_listlen","number",["number"] );
pxs_varcall = module.cwrap("pxs_varcall","number",["number","number","number"] );
pxs_newcopy = module.cwrap("pxs_newcopy","number",["number"] );
pxs_objectget = module.cwrap("pxs_objectget","number",["number","number","string"] );
pxs_objectset = module.cwrap("pxs_objectset","bool",["number","number","string","number"] );
pxs_eval = module.cwrap("pxs_eval","number",["string","number"] );
pxs_evalnamed = module.cwrap("pxs_evalnamed","number",["string","string","number"] );
pxs_newfactory = (...args) => {
    let a_0 = args[0];
let a_1 = args[1];

    let wrapper = module.cwrap("pxs_newfactory","number",["number","number"] );

    a_0 = module.addFunction(a_0, 'pp');

    return wrapper(a_0,a_1);
}
;
pxs_gettype = module.cwrap("pxs_gettype","number",["number","number","number"] );
pxs_gethost = module.cwrap("pxs_gethost","number",["number","number"] );
pxs_debugvar = module.cwrap("pxs_debugvar","string",["number"] );
pxs_newexception = module.cwrap("pxs_newexception","number",["string"] );
pxs_var_fromname = module.cwrap("pxs_var_fromname","number",["number","string"] );
pxs_listdel = module.cwrap("pxs_listdel","bool",["number","number"] );
pxs_new_shallowcopy = module.cwrap("pxs_new_shallowcopy","number",["number"] );
pxs_compile = module.cwrap("pxs_compile","number",["number","string","number","string"] );
pxs_execobject = module.cwrap("pxs_execobject","number",["number","number"] );
pxs_newmap = module.cwrap("pxs_newmap","number" );
pxs_map_addpair = module.cwrap("pxs_map_addpair","",["number","number","number"] );
pxs_map_delitem = module.cwrap("pxs_map_delitem","",["number","number"] );
pxs_maplen = module.cwrap("pxs_maplen","number",["number"] );
pxs_mapkeys = module.cwrap("pxs_mapkeys","number",["number"] );
pxs_mapget = module.cwrap("pxs_mapget","number",["number","number"] );
pxs_listinsert = module.cwrap("pxs_listinsert","",["number","number","number"] );
pxs_newarena = module.cwrap("pxs_newarena","number" );
pxs_freearena = module.cwrap("pxs_freearena","",["number"] );
pxs_arenaput = module.cwrap("pxs_arenaput","number",["number","number"] );
pxs_arena_putstr = module.cwrap("pxs_arena_putstr","string",["number","string"] );
pxs_debugstate = module.cwrap("pxs_debugstate","string",["number"] );
pxs_garbagecollect = module.cwrap("pxs_garbagecollect" );
pxs_getidx = module.cwrap("pxs_getidx","number",["number"] );
pxs_arg = module.cwrap("pxs_arg","number",["number","number"] );
pxs_getrt = module.cwrap("pxs_getrt","number",["number"] );
pxs_argc = module.cwrap("pxs_argc","number",["number"] );
pxs_newbytes = module.cwrap("pxs_newbytes","number",["number","number","number"] );
pxs_varsize = module.cwrap("pxs_varsize","number",["number"] );
pxs_copybytes = module.cwrap("pxs_copybytes","",["number","number"] );
pxs_copystring = module.cwrap("pxs_copystring","",["number","string"] );
pxs_smart_getstring = module.cwrap("pxs_smart_getstring","string",["number","number"] );
pxs_smart_copystring = module.cwrap("pxs_smart_copystring","",["number","number","string"] );
pxs_vartype = module.cwrap("pxs_vartype","number",["number"] );
pxs_isstring = module.cwrap("pxs_isstring","bool",["number"] );
pxs_isint = module.cwrap("pxs_isint","bool",["number"] );
pxs_isuint = module.cwrap("pxs_isuint","bool",["number"] );
pxs_isbool = module.cwrap("pxs_isbool","bool",["number"] );
pxs_isfloat = module.cwrap("pxs_isfloat","bool",["number"] );
pxs_isnull = module.cwrap("pxs_isnull","bool",["number"] );
pxs_isobject = module.cwrap("pxs_isobject","bool",["number"] );
pxs_is_hostobject = module.cwrap("pxs_is_hostobject","bool",["number"] );
pxs_islist = module.cwrap("pxs_islist","bool",["number"] );
pxs_isfunction = module.cwrap("pxs_isfunction","bool",["number"] );
pxs_isfactory = module.cwrap("pxs_isfactory","bool",["number"] );
pxs_isexception = module.cwrap("pxs_isexception","bool",["number"] );
pxs_ismap = module.cwrap("pxs_ismap","bool",["number"] );
pxs_isbyte = module.cwrap("pxs_isbyte","bool",["number"] );
pxs_addmod2 = module.cwrap("pxs_addmod2","",["number","number"] );
pxs_json_encode = module.cwrap("pxs_json_encode","number",["number","number"] );
pxs_json_decode = module.cwrap("pxs_json_decode","number",["number","number"] );
pxs_core_init = module.cwrap("pxs_core_init","",["number"] );
pxs_core_initall = module.cwrap("pxs_core_initall" )
        if (start) {
            pxs_initialize();
        }
    }
