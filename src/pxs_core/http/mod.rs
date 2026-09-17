use std::collections::HashMap;

use etffi::{cstring::CStringSafe, own_string, ptr_magic::PtrMagic};

use crate::{
    expected_argc, pxs_add_submod, pxs_addfunc, pxs_addobject, pxs_addvar, pxs_arg, pxs_argc,
    pxs_copybytes,
    pxs_core::PxsCoreType,
    pxs_error, pxs_freevar, pxs_getbool, pxs_getint, pxs_getrt, pxs_getstring, pxs_gettype,
    pxs_isbool, pxs_isexception, pxs_isfloat, pxs_isint, pxs_islist, pxs_isstring, pxs_listadd,
    pxs_listget, pxs_listlen, pxs_new_shallowcopy, pxs_newbool, pxs_newbytes, pxs_newcopy,
    pxs_newexception, pxs_newhost, pxs_newint, pxs_newlist, pxs_newmod, pxs_newnull, pxs_newstring,
    pxs_newtype, pxs_object_addfunc, pxs_object_addprop, pxs_smart_getstring, pxs_varsize,
    shared::{PxsRes, module::pxs_Module, pxs_Opaque, var::pxs_VarT},
};

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_vendor = "apple")]
pub mod apple;

#[cfg(target_os = "linux")]
pub mod linux;

/// Request Type
#[derive(Clone, Copy, PartialEq, Debug)]
#[repr(i32)]
enum RequestType {
    GET = 0,
    POST = 1,
    PUT = 2,
    PATCH = 3,
    DELETE = 4,
}

impl RequestType {
    fn from_int(val: i32) -> PxsRes<Self> {
        match val {
            0 => Ok(Self::GET),
            1 => Ok(Self::POST),
            2 => Ok(Self::PUT),
            3 => Ok(Self::PATCH),
            4 => Ok(Self::DELETE),
            _ => pxs_error!("Val {val} is not valid RequestType"),
        }
    }
}

/// HTTP Version
#[derive(Clone, Copy, PartialEq)]
#[repr(i32)]
enum HTTPVersion {
    HTTP_1_1 = 0,
    HTTP_2 = 1,
    HTTP_3 = 2,
}

impl HTTPVersion {
    fn from_int(val: i32) -> PxsRes<Self> {
        match val {
            0 => Ok(Self::HTTP_1_1),
            1 => Ok(Self::HTTP_2),
            2 => Ok(Self::HTTP_3),
            _ => pxs_error!("Val {val} is not valid HTTPVersion"),
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
    domain_name: String,
}

struct ClientResponse {
    /// Response data
    data: ResponseData,
    /// Status
    status: i32,
}

impl PtrMagic for ClientResponse {}

impl ClientResponse {
    fn new() -> Self {
        Self {
            data: ResponseData {
                headers: HashMap::new(),
                body: String::new(),
                request_type: RequestType::GET,
                version: HTTPVersion::HTTP_1_1,
                timeout: 0,
                user_agent: String::from("pixelscript_user_agent"),
                domain_name: String::new(),
            },
            status: 0,
        }
    }

    /// Free the client response.
    extern "C" fn free(ptr: pxs_Opaque) {
        if !ptr.is_null() {
            let _ = unsafe { Self::from_raw_void(ptr) };
        }
    }

    /// Convert into PXS native.
    fn into_pxs(ptr: *mut Self) -> pxs_VarT {
        let obj = pxs_newtype(
            ptr as pxs_Opaque,
            Self::free,
            c"ClientResponse".as_ptr(),
            PxsCoreType::ClientResponse as i32,
        );
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
        let ptr = pxs_gettype(
            pxs_getrt(args),
            pxs_arg(args, 0),
            PxsCoreType::ClientResponse as i32,
        );
        if ptr.is_null() {
            return pxs_newexception(c"Expected `ClientResponse`".as_ptr());
        }
        let this = unsafe { Self::from_borrow_void(ptr) };

        pxs_newint(this.data.version as i64)
    }

    /// @self
    /// @prop(get)
    /// The response status.
    ///
    /// returns `int`
    extern "C" fn prop_status(args: pxs_VarT) -> pxs_VarT {
        expected_argc!(args, 1);
        let ptr = pxs_gettype(
            pxs_getrt(args),
            pxs_arg(args, 0),
            PxsCoreType::ClientResponse as i32,
        );
        if ptr.is_null() {
            return pxs_newexception(c"Expected `ClientResponse`".as_ptr());
        }
        let this = unsafe { Self::from_borrow_void(ptr) };

        pxs_newint(this.status as i64)
    }

    /// @self
    /// @prop(get)
    /// The response bytes.
    ///
    /// returns `[]uint`
    extern "C" fn prop_bytes(args: pxs_VarT) -> pxs_VarT {
        expected_argc!(args, 1);
        let ptr = pxs_gettype(
            pxs_getrt(args),
            pxs_arg(args, 0),
            PxsCoreType::ClientResponse as i32,
        );
        if ptr.is_null() {
            return pxs_newexception(c"Expected `ClientResponse`".as_ptr());
        }
        let this = unsafe { Self::from_borrow_void(ptr) };
        // Convert response into bytes
        let mut response = this.data.body.clone();
        pxs_newbytes(
            response.as_mut_ptr() as pxs_Opaque,
            size_of::<u8>(),
            response.len(),
        )
    }

    /// @self
    /// @prop(get)
    /// The response text.
    ///
    /// returns `string`
    extern "C" fn prop_text(args: pxs_VarT) -> pxs_VarT {
        expected_argc!(args, 1);
        let ptr = pxs_gettype(
            pxs_getrt(args),
            pxs_arg(args, 0),
            PxsCoreType::ClientResponse as i32,
        );
        if ptr.is_null() {
            return pxs_newexception(c"Expected `ClientResponse`".as_ptr());
        }
        let this = unsafe { Self::from_borrow_void(ptr) };
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
    data: ResponseData,
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

/// Get the header parts as a Vec<"key:    value">
fn get_header_parts(headers: &HashMap<String, String>) -> Vec<String> {
    let mut res = vec![];

    for i in headers {
        res.push(format!("{}:\t{}", i.0, i.1));
    }

    res
}

impl Drop for Client {
    fn drop(&mut self) {
        // No freeing needed.
        if self.value.is_null() {
            return;
        }

        // Handle platform specific (value).
        #[cfg(target_os = "windows")] 
        {
            windows::WindowsHTTP::free(self.value);
        }
        #[cfg(target_os = "linux")]
        {
            linux::LinuxHTTP::free(self.value);
        }
    }
}

trait ClientCallbacks {
    fn setup(client: &mut Client) -> Result<(), String>;
    fn create_request(client: &mut Client, path: String, rt: RequestType) -> Result<ClientResponse, String>;
    fn free(v:pxs_Opaque);
}

impl Client {
    fn new() -> Self {
        let mut client = Self {
            value: core::ptr::null_mut(),
            https: true,
            data: ResponseData {
                headers: HashMap::new(),
                body: String::new(),
                request_type: RequestType::GET,
                version: HTTPVersion::HTTP_1_1,
                timeout: 6000,
                user_agent: String::from("pixelscript_user_agent"),
                domain_name: String::new(),
            },
        };

        #[cfg(target_os = "windows")]
        {
            let _ = windows::WindowsHTTP::setup(&mut client);
        }
        #[cfg(target_os = "linux")]
        {
            let _ = linux::LinuxHTTP::setup(&mut client);
        }
        client
    }

    /// Crate a non pxs `ClientResponse`.
    fn create_request(
        &mut self,
        path: String,
        request_type: RequestType,
    ) -> Result<ClientResponse, String> {
        // TODO: Call the correct platform function here.
        #[cfg(target_os = "windows")]
        return windows::WindowsHTTP::create_request(self, path, request_type);

        // #[cfg(target_vendor="apple")]
        // pub mod apple;

        #[cfg(target_os="linux")]
        return linux::LinuxHTTP::create_request(self, path, request_type);
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

        let obj = pxs_newtype(
            client_ptr,
            Self::free,
            c"Client".as_ptr(),
            PxsCoreType::Client as i32,
        );

        // Methods
        pxs_object_addfunc(obj, c"get_header".as_ptr(), Self::get_header);
        pxs_object_addfunc(obj, c"set_header".as_ptr(), Self::set_header);
        pxs_object_addfunc(obj, c"make_request".as_ptr(), Self::make_request);
        // Prototypes
        pxs_object_addprop(obj, c"headers".as_ptr(), Self::prop_headers);
        pxs_object_addprop(obj, c"body".as_ptr(), Self::prop_body);
        pxs_object_addprop(obj, c"version".as_ptr(), Self::prop_version);
        pxs_object_addprop(obj, c"domain".as_ptr(), Self::prop_domain);
        pxs_object_addprop(obj, c"timeout".as_ptr(), Self::prop_timeout);
        pxs_object_addprop(obj, c"https".as_ptr(), Self::prop_https);

        pxs_newhost(obj)
    }

    /// @self
    /// @prop(get,set)
    /// The headers.
    /// args:
    ///  - headers: @set `[][]string` the headers to set.
    ///
    /// returns `[][]string`|`null`
    extern "C" fn prop_headers(args: pxs_VarT) -> pxs_VarT {
        let ptr = pxs_gettype(
            pxs_getrt(args),
            pxs_arg(args, 0),
            PxsCoreType::Client as i32,
        );
        if ptr.is_null() {
            return pxs_newexception(c"Expected self (Client.prop_headers)".as_ptr());
        }
        let this = unsafe { Self::from_borrow_void(ptr) };
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
        let ptr = pxs_gettype(
            pxs_getrt(args),
            pxs_arg(args, 0),
            PxsCoreType::Client as i32,
        );
        if ptr.is_null() {
            return pxs_newexception(c"Expected self (Client.get_header)".as_ptr());
        }
        let this = unsafe { Self::from_borrow_void(ptr) };

        let key_arg = pxs_arg(args, 1);
        if !pxs_isstring(key_arg) {
            return pxs_newexception(c"Expected string (Client.get_header)".as_ptr());
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
        let ptr = pxs_gettype(
            pxs_getrt(args),
            pxs_arg(args, 0),
            PxsCoreType::Client as i32,
        );
        if ptr.is_null() {
            return pxs_newexception(c"Expected self (Client.set_header)".as_ptr());
        }
        let this = unsafe { Self::from_borrow_void(ptr) };

        let key_arg = pxs_arg(args, 1);
        if !pxs_isstring(key_arg) {
            return pxs_newexception(c"Expected string (Client.set_header)".as_ptr());
        }
        let key = own_string!(pxs_getstring(key_arg));

        let value_arg = pxs_arg(args, 2);
        if !pxs_isstring(value_arg) {
            return pxs_newexception(c"Expected string (Client.set_header)".as_ptr());
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
        let ptr = pxs_gettype(
            pxs_getrt(args),
            pxs_arg(args, 0),
            PxsCoreType::Client as i32,
        );
        if ptr.is_null() {
            return pxs_newexception(c"Expected self (Client.body)".as_ptr());
        }
        let this = unsafe { Self::from_borrow_void(ptr) };

        let argc = pxs_argc(args);

        if argc == 1 {
            // GET
            let mut cstring = CStringSafe::new();
            return pxs_newstring(cstring.new_string(&this.data.body));
        } else if argc == 2 {
            // SET
            let body_arg = pxs_arg(args, 1);
            if pxs_isexception(body_arg) {
                return pxs_newnull();
            }
            let size = pxs_varsize(body_arg);
            if size == 0 {
                return pxs_newnull();
            }
            let mut bytes: Vec<u8> = vec![0; size];
            pxs_copybytes(body_arg, bytes.as_mut_ptr() as pxs_Opaque);
            let string = String::from_utf8(bytes);
            if string.is_err() {
                let mut cstring = CStringSafe::new();
                return pxs_newexception(cstring.new_string(&format!(
                    "Error setting body: {}",
                    string.unwrap_err().to_string()
                )));
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
        let ptr = pxs_gettype(
            pxs_getrt(args),
            pxs_arg(args, 0),
            PxsCoreType::Client as i32,
        );
        if ptr.is_null() {
            return pxs_newexception(c"Expected self (Client.version)".as_ptr());
        }
        let this = unsafe { Self::from_borrow_void(ptr) };
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
            match v {
                Ok(val) => this.data.version = val,
                Err(err) => {
                    let mut cstring = CStringSafe::new();
                    return pxs_newexception(cstring.new_string(&err));
                }
            }
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
        let ptr = pxs_gettype(
            pxs_getrt(args),
            pxs_arg(args, 0),
            PxsCoreType::Client as i32,
        );
        if ptr.is_null() {
            return pxs_newexception(c"Expected self (Client.domain)".as_ptr());
        }
        let this = unsafe { Self::from_borrow_void(ptr) };
        let argc = pxs_argc(args);

        if argc == 1 {
            // GET
            let mut cstring = CStringSafe::new();
            return pxs_newstring(cstring.new_string(&this.data.domain_name));
        } else if argc == 2 {
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
        let ptr = pxs_gettype(
            pxs_getrt(args),
            pxs_arg(args, 0),
            PxsCoreType::Client as i32,
        );
        if ptr.is_null() {
            return pxs_newexception(c"Expected self (Client.timeout)".as_ptr());
        }
        let this = unsafe { Self::from_borrow_void(ptr) };
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
        let ptr = pxs_gettype(
            pxs_getrt(args),
            pxs_arg(args, 0),
            PxsCoreType::Client as i32,
        );
        if ptr.is_null() {
            return pxs_newexception(c"Expected self (Client.make_request) ".as_ptr());
        }
        let client = unsafe { Self::from_borrow_void(ptr) };

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
        match response {
            Ok(val) => ClientResponse::into_pxs(val.into_raw()),
            Err(val) => {
                let mut cstring = CStringSafe::new();
                return pxs_newexception(cstring.new_string(&val));
            }
        }
    }

    /// @self
    /// @prop(get, set)
    /// Use HTTPS.
    /// args:
    ///   - use: @set `bool` true for using false for not.
    ///
    /// returns `bool`
    extern "C" fn prop_https(args: pxs_VarT) -> pxs_VarT {
        let ptr = pxs_gettype(
            pxs_getrt(args),
            pxs_arg(args, 0),
            PxsCoreType::Client as i32,
        );
        if ptr.is_null() {
            return pxs_newexception(c"Expected self (Client.https)".as_ptr());
        }
        let client = unsafe { Self::from_borrow_void(ptr) };
        let argc = pxs_argc(args);

        if argc == 1 {
            // GET
            return pxs_newbool(client.https);
        } else if argc == 2 {
            // SET
            let val_arg = pxs_arg(args, 1);
            if !pxs_isbool(val_arg) {
                return pxs_newexception(c"Expecte bool".as_ptr());
            }
            client.https = pxs_getbool(val_arg);
        }
        pxs_newnull()
    }
}

/// Get domain name and path from a pxs_VarT url
fn get_domain_and_path(url: pxs_VarT) -> [String; 2] {
    let mut result = [String::from(""), String::from("")];

    let cstr = pxs_getstring(url);
    if cstr.is_null() {
        return result;
    }
    let url_string = own_string!(cstr);

    let paths: Vec<&str> = url_string.split("/").collect();

    if paths.len() >= 3 {
        result[0] = paths[2].to_string();

        if paths.len() > 3 {
            let mut path = String::new();
            for i in 3..paths.len() {
                path.push_str(paths[i]);
                if i < path.len() - 1 {
                    path.push_str("/");
                }
            }
            result[1] = path;
        }
    }

    result
}

/// @except
/// Make a HTTP Get request.
/// args:
///  - url: `string` the url to request to.
///  - headers: @opt `[][]string` the headers to apply.
///  - version: @opt `int` the HTTP version to use.
///
/// returns `ClientResponse`
extern "C" fn get(args: pxs_VarT) -> pxs_VarT {
    // Check URL
    let argc = pxs_argc(args);
    if argc == 0 {
        return pxs_newexception(c"Expected URL".as_ptr());
    }

    let paths = get_domain_and_path(pxs_arg(args, 0));
    let mut cstring = CStringSafe::new();

    let client = Client::new_client(core::ptr::null_mut());
    let runtime = pxs_getrt(args);

    // Domain
    let params = pxs_newlist();
    pxs_listadd(params, pxs_newcopy(runtime));
    pxs_listadd(params, pxs_new_shallowcopy(client));
    pxs_listadd(params, pxs_newstring(cstring.new_string(&paths[0])));
    let _ = Client::prop_domain(params);
    pxs_freevar(params);

    // Headers
    let params = pxs_newlist();
    pxs_listadd(params, pxs_newcopy(runtime));
    pxs_listadd(params, pxs_new_shallowcopy(client));
    pxs_listadd(params, pxs_newcopy(pxs_arg(args, 1)));
    let _ = Client::prop_headers(params);
    pxs_freevar(params);

    // Version
    let params = pxs_newlist();
    pxs_listadd(params, pxs_newcopy(runtime));
    pxs_listadd(params, pxs_new_shallowcopy(client));
    pxs_listadd(params, pxs_newcopy(pxs_arg(args, 2)));
    let _ = Client::prop_version(params);
    pxs_freevar(params);

    // make request
    let params = pxs_newlist();
    pxs_listadd(params, pxs_newcopy(runtime));
    pxs_listadd(params, pxs_new_shallowcopy(client));
    pxs_listadd(params, pxs_newstring(cstring.new_string(&paths[1])));
    pxs_listadd(params, pxs_newint(RequestType::GET as i64));
    let result = Client::make_request(params);
    pxs_freevar(params);

    result
}

/// @except
/// Make a HTTP Post request.
/// args:
///  - url: `string` the url to request to.
///  - body: `string` the body to send.
///  - headers: @opt `[][]string` the headers to apply.
///  - version: @opt `int` the HTTP version to use.
///
/// returns `ClientResponse`
extern "C" fn post(args: pxs_VarT) -> pxs_VarT {
    // Check URL
    let argc = pxs_argc(args);
    if argc == 0 {
        return pxs_newexception(c"Expected URL".as_ptr());
    }

    let paths = get_domain_and_path(pxs_arg(args, 0));
    let mut cstring = CStringSafe::new();

    let client = Client::new_client(core::ptr::null_mut());
    let runtime = pxs_getrt(args);

    // Domain
    let params = pxs_newlist();
    pxs_listadd(params, pxs_newcopy(runtime));
    pxs_listadd(params, pxs_new_shallowcopy(client));
    pxs_listadd(params, pxs_newstring(cstring.new_string(&paths[0])));
    let _ = Client::prop_domain(params);
    pxs_freevar(params);

    // Body
    //     pxs_freevar(pxs::call(Client::prop_body, {pxs_new_shallowcopy(client), pxs_arg(args, 1)}));
    let params = pxs_newlist();
    pxs_listadd(params, pxs_newcopy(runtime));
    pxs_listadd(params, pxs_new_shallowcopy(client));
    pxs_listadd(params, pxs_newcopy(pxs_arg(args, 1)));
    let _ = Client::prop_body(params);
    pxs_freevar(params);

    // Headers
    let params = pxs_newlist();
    pxs_listadd(params, pxs_newcopy(runtime));
    pxs_listadd(params, pxs_new_shallowcopy(client));
    pxs_listadd(params, pxs_newcopy(pxs_arg(args, 2)));
    let _ = Client::prop_headers(params);
    pxs_freevar(params);

    // Version
    let params = pxs_newlist();
    pxs_listadd(params, pxs_newcopy(runtime));
    pxs_listadd(params, pxs_new_shallowcopy(client));
    pxs_listadd(params, pxs_newcopy(pxs_arg(args, 3)));
    let _ = Client::prop_version(params);
    pxs_freevar(params);

    // make request
    let params = pxs_newlist();
    pxs_listadd(params, pxs_newcopy(runtime));
    pxs_listadd(params, pxs_new_shallowcopy(client));
    pxs_listadd(params, pxs_newstring(cstring.new_string(&paths[1])));
    pxs_listadd(params, pxs_newint(RequestType::POST as i64));
    let result = Client::make_request(params);
    pxs_freevar(params);

    result
}

pub(super) fn init(module: *mut pxs_Module) {
    let mut cstring = CStringSafe::new();
    let http = pxs_newmod(cstring.new_string("http"));

    // Variables
    pxs_addvar(
        http,
        c"HTTP_VERSION_1_1".as_ptr(),
        pxs_newint(HTTPVersion::HTTP_1_1 as i64),
    );
    pxs_addvar(
        http,
        c"HTTP_VERSION_2".as_ptr(),
        pxs_newint(HTTPVersion::HTTP_2 as i64),
    );
    pxs_addvar(
        http,
        c"HTTP_VERSION_3".as_ptr(),
        pxs_newint(HTTPVersion::HTTP_3 as i64),
    );

    // Sub module
    let client_mod = pxs_newmod(c"client".as_ptr());
    pxs_addfunc(client_mod, c"get".as_ptr(), get);
    pxs_addfunc(client_mod, c"post".as_ptr(), post);
    pxs_addobject(client_mod, c"Client".as_ptr(), Client::new_client);
    pxs_add_submod(http, client_mod);

    // TODO: server_mod. (Yes pixelscript is getting a server module.)

    pxs_add_submod(module, http);
}
