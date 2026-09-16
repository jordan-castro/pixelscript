use std::{collections::HashMap, hash::Hash};

use etffi::{cstring::CStringSafe, own_string, ptr_magic::PtrMagic};

use crate::{expected_argc, pxs_add_submod, pxs_arg, pxs_argc, pxs_copybytes, pxs_core::PxsCoreType, pxs_error, pxs_getint, pxs_getrt, pxs_getstring, pxs_gettype, pxs_isfloat, pxs_isint, pxs_islist, pxs_isstring, pxs_listadd, pxs_listget, pxs_listlen, pxs_newbytes, pxs_newexception, pxs_newhost, pxs_newint, pxs_newlist, pxs_newmod, pxs_newnull, pxs_newstring, pxs_newtype, pxs_object_addfunc, pxs_object_addprop, pxs_smart_getstring, pxs_varsize, shared::{PxsRes, func::pxs_Func, module::pxs_Module, pxs_Opaque, var::pxs_VarT}};

#[cfg(target_os="windows")]
pub mod windows;

#[cfg(target_vendor="apple")]
pub mod apple;

#[cfg(target_os="linux")]
pub mod linux;

/// Request Type
#[derive(Clone, Copy, PartialEq)]
#[repr(i32)]
enum RequestType {
    GET = 0,
    POST = 1,
    PUT = 2,
    PATCH = 3,
    DELETE = 4
}

impl RequestType {
    fn from_int(val: i32) -> PxsRes<Self> {
        match val {
            0 => Ok(Self::GET),
            1 => Ok(Self::POST),
            2 => Ok(Self::PUT),
            3 => Ok(Self::PATCH),
            4 => Ok(Self::DELETE),
            _ => pxs_error!("Val {val} is not valid RequestType")
        }
    }
}

/// HTTP Version
#[derive(Clone, Copy, PartialEq)]
#[repr(i32)]
enum HTTPVersion {
    HTTP_1_1 = 0,
    HTTP_2 = 1,
    HTTP_3 = 2
}

impl HTTPVersion {
    fn from_int(val: i32) -> PxsRes<Self> {
        match val {
            0 => Ok(Self::HTTP_1_1),
            1 => Ok(Self::HTTP_2),
            2 => Ok(Self::HTTP_3),
            _ => pxs_error!("Val {val} is not valid HTTPVersion")
        }
    }
}

/// The response data that is sent to and fro.
/// It is used when we send a request, and is also used for the response.
struct ResponseData {
    /// Headers sent in a request.
    headers: HashMap<String, String>,
    /// Request body
    body: String,
    /// Request stype
    request_type: RequestType,
    /// The HTTP version to use (if supported.)
    version: HTTPVersion,
    /// Timeout in milliseconds
    timeout: i64,
    /// The user agent. This can only be set once per client.
    user_agent: String,
    /// The domain name
    domain_name: String
}

struct ClientResponse {
    /// Response data
    data: ResponseData,
    /// Status
    status: i32
}

impl PtrMagic for ClientResponse {}

impl ClientResponse {
    /// Free the client response.
    extern "C" fn free(ptr: pxs_Opaque) {
        if !ptr.is_null() {
            let _ = unsafe{Self::from_raw_void(ptr)};
        }
    }

    /// Convert into PXS native.
    fn into_pxs(ptr: *mut Self) -> pxs_VarT {
        let obj = pxs_newtype(ptr as pxs_Opaque, Self::free, c"ClientResponse".as_ptr(), PxsCoreType::ClientResponse as i32);
        pxs_object_addprop(obj, c"version".as_ptr(), Self::prop_version);
        pxs_object_addprop(obj, c"status".as_ptr(), Self::prop_status);
        pxs_object_addprop(obj, c"bytes".as_ptr(), Self::prop_bytes);
        pxs_object_addprop(obj, c"text".as_ptr(), Self::prop_text);
        pxs_newhost(obj)
    }

    /// Fill the `data` from another `other` `ResponseData`.
    /// This does not overwrite `body`.
    fn fill(&mut self, other: &ResponseData) {
        self.data.headers = other.headers.clone();
        self.data.request_type = other.request_type;
        self.data.version = other.version;
        self.data.timeout = other.timeout;
        self.data.user_agent = other.user_agent.clone();
        self.data.domain_name = other.domain_name.clone();
    }

    /// @self
    /// @prop(get)
    /// The HTTP Version.
    /// 
    /// returns `int`
    extern "C" fn prop_version(args: pxs_VarT) -> pxs_VarT {
        expected_argc!(args, 1);
        let ptr = pxs_gettype(pxs_getrt(args), pxs_arg(args, 0), PxsCoreType::ClientResponse as i32);
        if ptr.is_null() {
            return pxs_newexception(c"Expected `ClientResponse`".as_ptr());
        }
        let this = unsafe{ Self::from_borrow_void(ptr) };

        pxs_newint(this.data.version as i64)
    }

    /// @self
    /// @prop(get)
    /// The response status.
    /// 
    /// returns `int`
    extern "C" fn prop_status(args: pxs_VarT) -> pxs_VarT {
        expected_argc!(args, 1);
        let ptr = pxs_gettype(pxs_getrt(args), pxs_arg(args, 0), PxsCoreType::ClientResponse as i32);
        if ptr.is_null() {
            return pxs_newexception(c"Expected `ClientResponse`".as_ptr());
        }
        let this = unsafe{ Self::from_borrow_void(ptr) };

        pxs_newint(this.status as i64)
    }

    /// @self
    /// @prop(get)
    /// The response bytes.
    ///
    /// returns `[]uint`
    extern "C" fn prop_bytes(args: pxs_VarT) -> pxs_VarT {
        expected_argc!(args, 1);
        let ptr = pxs_gettype(pxs_getrt(args), pxs_arg(args, 0), PxsCoreType::ClientResponse as i32);
        if ptr.is_null() {
            return pxs_newexception(c"Expected `ClientResponse`".as_ptr());
        }
        let this = unsafe{ Self::from_borrow_void(ptr) };
        // Convert response into bytes
        let mut response = this.data.body.clone();
        pxs_newbytes(response.as_mut_ptr() as pxs_Opaque, size_of::<u8>(), response.len())
    }

    /// @self
    /// @prop(get)
    /// The response text.
    /// 
    /// returns `string`
    extern "C" fn prop_text(args: pxs_VarT) -> pxs_VarT {
        expected_argc!(args, 1);
        let ptr = pxs_gettype(pxs_getrt(args), pxs_arg(args, 0), PxsCoreType::ClientResponse as i32);
        if ptr.is_null() {
            return pxs_newexception(c"Expected `ClientResponse`".as_ptr());
        }
        let this = unsafe{ Self::from_borrow_void(ptr) };
        // Create string.
        let response = this.data.body.clone();
        let mut cstring = CStringSafe::new();
        pxs_newstring(cstring.new_string(&response))
    }
}

struct Client {
    /// Internal type/value
    value: pxs_Opaque,

    /// To use or not to use HTTPS
    https: bool,

    /// The data to CREATE a response with
    data: ResponseData
}

impl PtrMagic for Client {}

/// Get the headers from a pxs_VarT.
fn get_headers(rt: pxs_VarT, arg: pxs_VarT) -> HashMap<String, String> {
    let mut result = HashMap::new();

    // Loop through values
    for i in 0..pxs_listlen(arg) {
        // Get string values if strings.
        let item = pxs_listget(arg, i);
        // Gotta be a list tho
        if !pxs_islist(item) {
            continue;
        }

        // Key, value
        let key_arg = pxs_smart_getstring(rt, pxs_listget(item, 0));
        if key_arg.is_null() {
            continue;
        }
        let key = own_string!(key_arg);

        let value_arg = pxs_smart_getstring(rt, pxs_listget(item, 1));
        if value_arg.is_null() {
            continue;
        }
        let value = own_string!(value_arg);

        result.insert(key, value);
    }

    result
}

impl Drop for Client {
    fn drop(&mut self) {
        // Handle platform specific (value).
        // todo!()
    }
}

impl Client {
    fn new() -> Self {
        Self {
            value: core::ptr::null_mut(),
            https: false,
            data: ResponseData { 
                headers: HashMap::new(), 
                body: String::new(), 
                request_type: RequestType::GET, 
                version: HTTPVersion::HTTP_1_1, 
                timeout: 6000, 
                user_agent: String::from("pixelscript_user_agent"), 
                domain_name: String::new() 
            }
        }
    }

    /// Crate a non pxs `ClientResponse`.
    fn create_request(&mut self, path: String, request_type: RequestType) -> Result<ClientResponse, String> {
        // TODO: Call the correct platform function here.
    }

    extern "C" fn free(ptr: pxs_Opaque) {
        if !ptr.is_null() {
            let _ = unsafe { Self::from_raw_void(ptr) };
        }
    }

    /// @name(Client)
    /// Create a new `Client`
    ///
    /// returns `Client`
    extern "C" fn new_client(_: pxs_VarT) -> pxs_VarT {
        let client = Self::new();
        
        let client_ptr = client.into_void();

        let obj = pxs_newtype(client_ptr, Self::free, c"Client".as_ptr(), PxsCoreType::Client as i32);

        // Methods
        pxs_object_addfunc(object, c"get_header".as_ptr(), Self::get_header);
        pxs_object_addfunc(object, c"set_header".as_ptr(), Self::set_header);
        pxs_object_addfunc(object, c"make_request".as_ptr(), Self::make_request);
        // Prototypes
        pxs_object_addprop(object, c"headers".as_ptr(), Self::prop_headers);
        pxs_object_addprop(object, c"body".as_ptr(), Self::prop_body);
        pxs_object_addprop(object, c"version".as_ptr(), Self::prop_version);
        pxs_object_addprop(object, c"domain".as_ptr(), Self::prop_domain);
        pxs_object_addprop(object, c"timeout".as_ptr(), Self::prop_timeout);

        pxs_newhost(obj)
    }

        // @self
    /// @prop(get,set)
    /// The headers.
    /// args:
    ///  - headers: @set `[][]string` the headers to set.
    ///
    /// returns `[][]string`|`null`
    extern "C" fn prop_headers(args: pxs_VarT) -> pxs_VarT {
        let ptr = pxs_gettype(pxs_getrt(args), pxs_arg(args, 0), PxsCoreType::Client as i32);
        if ptr.is_null() {
            return pxs_newexception(c"Expected self".as_ptr());
        }
        let this = unsafe{ Self::from_borrow_void(ptr) };
        let argc = pxs_argc(args);
        if argc == 1 {
            // GET
            let headers = this.data.headers.clone();
            let result = pxs_newlist();

            let mut cstring = CStringSafe::new();
            for item in headers {
                let it = pxs_newlist();
                pxs_listadd(it, pxs_newstring(cstring.new_string(&item.0)));
                pxs_listadd(it, pxs_newstring(cstring.new_string(&item.1)));
                pxs_listadd(result, it);
            }

            return result;
        } else if argc == 2 {
            let headers = pxs_arg(args, 1);
            this.data.headers = get_headers(pxs_getrt(args), headers);
        }
        pxs_newnull()
    }
    
    /// @self
    /// Get a single header.
    /// args:
    ///  - key: `string` the header key.
    ///
    /// returns `string` value if found.
    extern "C" fn get_header(args: pxs_VarT) -> pxs_VarT {
        let ptr = pxs_gettype(pxs_getrt(args), pxs_arg(args, 0), PxsCoreType::Client as i32);
        if ptr.is_null() {
            return pxs_newexception(c"Expected self".as_ptr());
        }
        let this = unsafe{ Self::from_borrow_void(ptr) };
        
        let key_arg = pxs_arg(args, 1);
        if !pxs_isstring(key_arg) {
            return pxs_newexception(c"Expected string".as_ptr());
        }
        let key = own_string!(pxs_getstring(key_arg));

        let item = this.data.headers.get(&key);
        if item.is_none() {
            pxs_newnull()
        } else {
            let mut cstring = CStringSafe::new();
            pxs_newstring(cstring.new_string(item.unwrap()))
        }
    }

    /// @self
    /// Set a single header.
    /// args:
    ///  - key: `string` header key.
    ///  - value: `string` header value.
    extern "C" fn set_header(args: pxs_VarT) -> pxs_VarT {
        let ptr = pxs_gettype(pxs_getrt(args), pxs_arg(args, 0), PxsCoreType::Client as i32);
        if ptr.is_null() {
            return pxs_newexception(c"Expected self".as_ptr());
        }
        let this = unsafe{ Self::from_borrow_void(ptr) };

        let key_arg = pxs_arg(args, 1);
        if !pxs_isstring(key_arg) {
            return pxs_newexception(c"Expected string".as_ptr());
        }
        let key = own_string!(pxs_getstring(key_arg));

        let value_arg = pxs_arg(args, 2);
        if !pxs_isstring(value_arg) {
            return pxs_newexception(c"Expected string".as_ptr());
        }
        let value = own_string!(pxs_getstring(value_arg));

        let _ = this.data.headers.insert(key, value);

        pxs_newnull()
    }

    /// @self
    /// @prop(get,set)
    /// 
    /// args:
    ///  - body: @set `string`|`[]uint` body as string or bytes.
    ///
    /// returns `string`|`null`
    extern "C" fn prop_body(args: pxs_VarT) -> pxs_VarT {
        let ptr = pxs_gettype(pxs_getrt(args), pxs_arg(args, 0), PxsCoreType::Client as i32);
        if ptr.is_null() {
            return pxs_newexception(c"Expected self".as_ptr());
        }
        let this = unsafe{ Self::from_borrow_void(ptr) };

        let argc = pxs_argc(args);

        if argc == 1 {
            // GET
            let mut cstring = CStringSafe::new();
            return pxs_newstring(cstring.new_string(&this.data.body));
        } else if argc == 2 {
            // SET
            let body_arg = pxs_arg(args, 1);
            let size = pxs_varsize(body_arg);
            let mut bytes: Vec<u8> = vec![0;size];
            pxs_copybytes(body_arg, bytes.as_mut_ptr() as pxs_Opaque);
            let string = String::from_utf8(bytes);
            if string.is_err() {
                let mut cstring = CStringSafe::new();
                return pxs_newexception(cstring.new_string(&format!("Error setting body: {}", string.unwrap_err().to_string())));
            }
            this.data.body = string.unwrap();
        }
        pxs_newnull()
    }

    /// @self
    /// @prop(get,set)
    /// Version
    /// args:
    ///  - version: @set `int` the http version to use.
    ///
    /// returns `int`|`null`
    extern "C" fn prop_version(args: pxs_VarT) -> pxs_VarT {
        let ptr = pxs_gettype(pxs_getrt(args), pxs_arg(args, 0), PxsCoreType::Client as i32);
        if ptr.is_null() {
            return pxs_newexception(c"Expected self".as_ptr());
        }
        let this = unsafe{ Self::from_borrow_void(ptr) };
        let argc = pxs_argc(args);

        if argc == 1 {
            // GET
            let version = this.data.version as i32;
            return pxs_newint(version as i64);
        } else if argc == 2 {
            // SET
            let version_arg = pxs_arg(args, 1);
            if !pxs_isint(version_arg) && !pxs_isfloat(version_arg) {
                return pxs_newexception(c"Expected int".as_ptr());
            }
            let version = pxs_getint(version_arg);
            let v = HTTPVersion::from_int(version as i32);
            if v.is_err() {
                let mut cstring = CStringSafe::new();
                return pxs_newexception(cstring.new_string(&format!("Error setting version: {}", v.unwrap_err().to_string())));
            }
            this.data.version = v.unwrap();
        }

        pxs_newnull()
    }

    /// @self
    /// @prop(get,set)
    /// Domain name
    /// args:
    ///  - dn: @set `string` the domain name.
    ///
    /// returns `string`|`null`
    extern "C" fn prop_domain(args: pxs_VarT) -> pxs_VarT {
        let ptr = pxs_gettype(pxs_getrt(args), pxs_arg(args, 0), PxsCoreType::Client as i32);
        if ptr.is_null() {
            return pxs_newexception(c"Expected self".as_ptr());
        }
        let this = unsafe{ Self::from_borrow_void(ptr) };
        let argc = pxs_argc(args);

        if argc == 1 {
            // GET
            let mut cstring = CStringSafe::new();
            return pxs_newstring(cstring.new_string(&this.data.domain_name));
        } else if argc == 2{
            // SET
            let dn_arg = pxs_arg(args, 1);
            if !pxs_isstring(dn_arg) {
                return pxs_newexception(c"Expected string".as_ptr());
            }
            this.data.domain_name = own_string!(pxs_getstring(dn_arg));
        }

        pxs_newnull()
    }

    /// @self
    /// @prop(get,set)
    /// Tiemout in MS
    /// args:
    ///  - ms: @set `int` milliseconds.
    ///
    /// returns `int`
    extern "C" fn prop_timeout(args: pxs_VarT) -> pxs_VarT {
        let ptr = pxs_gettype(pxs_getrt(args), pxs_arg(args, 0), PxsCoreType::Client as i32);
        if ptr.is_null() {
            return pxs_newexception(c"Expected self".as_ptr());
        }
        let this = unsafe{ Self::from_borrow_void(ptr) };
        let argc = pxs_argc(args);

        if argc == 1 {
            // GET
            return pxs_newint(this.data.timeout);
        } else if argc == 2 {
            // SET
            let ms_arg = pxs_arg(args, 1);
            if !pxs_isint(ms_arg) && !pxs_isfloat(ms_arg) {
                return pxs_newexception(c"Expected int".as_ptr());
            }
            let ms = pxs_getint(ms_arg);
            this.data.timeout = ms;
        }

        pxs_newnull()
    }

    /// @except
    /// @self
    /// Make a request
    /// args:
    ///  - url: `string` the url to make the request to.
    ///  - rt: `RequestType` the request type to send.
    ///
    /// returns `string`
    extern "C" fn make_request(args: pxs_VarT) -> pxs_VarT {
        let ptr = pxs_gettype(pxs_getrt(args), pxs_arg(args, 0), PxsCoreType::Client as i32);
        if ptr.is_null() {
            return pxs_newexception(c"Expected self".as_ptr());
        }
        let client = unsafe{ Self::from_borrow_void(ptr) };

        // Get the URL
        let url_arg = pxs_arg(args, 1);
        if !pxs_isstring(url_arg) {
            return pxs_newexception(c"Expectes URL to be string.".as_ptr());
        }
        let url = own_string!(pxs_getstring(url_arg));

        // Get request type
        let rt_arg = pxs_arg(args, 2);
        if !pxs_isint(rt_arg) {
            return pxs_newexception(c"Expected int".as_ptr());
        }
        let rt = RequestType::from_int(pxs_getint(rt_arg) as i32);
        if rt.is_err() {
            return pxs_newexception(c"Invalid RequestType".as_ptr());
        }

        // Make request
        let response = client.create_request(url, rt.unwrap());
        if response.is_err() {
            let mut cstring = CStringSafe::new();
            return pxs_newexception(cstring.new_string(&response.unwrap_err().to_string()));
        }

        ClientResponse::into_pxs(response.unwrap().into_raw())
    }
}

pub(super) fn init(module: *mut pxs_Module) {
    let mut cstring = CStringSafe::new();
    let http = pxs_newmod(cstring.new_string("http"));

    pxs_add_submod(module, http);
}