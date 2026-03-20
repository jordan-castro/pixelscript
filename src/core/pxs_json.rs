use crate::{create_raw_string, free_raw_string, pxs_objectget, pxs_var_fromname, pxs_varcall, shared::var::pxs_VarT};

/// Encode a `pxs_Object` into JSON.
pub extern "C" fn encode(rt: pxs_VarT, args: pxs_VarT) -> pxs_VarT {
    // Get json module
    let cname = create_raw_string!("pxs_json");
    let json = pxs_var_fromname(rt, cname);
    unsafe{
        free_raw_string!(cname);
    }

    let cmethod = create_raw_string!("encode");
    // Lets get the encode method as a var!
    let encode_method = pxs_objectget(rt, json, cmethod);
    // Now let's call it
    let res = pxs_varcall(rt, encode_method, args);
    unsafe{
        free_raw_string!(cmethod);
    }

    res
} 

/// Decode a JSON string into a `pxs_Object`
pub extern "C" fn decode(rt: pxs_VarT, args: pxs_VarT) -> pxs_VarT {
    let cname = create_raw_string!("pxs_json");
    let json = pxs_var_fromname(rt, cname);
    unsafe{
        free_raw_string!(cname);
    }

    let cmethod = create_raw_string!("decode");
    // Lets get the decode method as a var!
    let decode_method = pxs_objectget(rt, json, cmethod);
    // Now let's call it
    let res = pxs_varcall(rt, decode_method, args);
    unsafe {
        free_raw_string!(cmethod);
    }
    res
}