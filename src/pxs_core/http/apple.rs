// Apple native IMPL uses Fondation
// Works on all Apple OS.

#![allow(non_snake_case)]

// ============ BINDINGS ===============

use core::slice;
use std::ptr::null_mut;

use etffi::{
    borrow_string, create_raw_string, cstring::CStringSafe, free_raw_string, ptr_magic::PtrMagic,
};

use crate::{
    pxs_core::http::{ClientCallbacks, ClientResponse, RequestType, get_header_parts},
    pxs_error,
};

type ClassName = *const std::ffi::c_char;
type Class = *mut std::ffi::c_void;
type Id = *mut std::ffi::c_void;

const DISPATCH_TIME_FOREVER: u64 = !0;

#[repr(C)]
struct HttpCompletionBlock {
    isa: Class,
    flags: i32,
    reserved: i32,
    invoke: extern "C" fn(Class, Class, Class, Class),
    response_data_ptr: Class,
    http_response_ptr: Class,
    execution_error_ptr: Class,
    semaphore: Id,
}
impl PtrMagic for HttpCompletionBlock {}

#[link(name = "objc", kind = "dylib")]
#[link(name = "System", kind = "dylib")]
#[link(name = "Foundation", kind = "framework")]
unsafe extern "C" {
    fn objc_getClass(name: ClassName) -> Class;
    fn sel_registerName(name: ClassName) -> Class;

    fn objc_msgSend(receiver: Class, selector: *mut std::ffi::c_void, ...)
    -> *mut std::ffi::c_void;
}

#[link(name = "System", kind = "dylib")]
unsafe extern "C" {
    fn dispatch_semaphore_create(value: std::ffi::c_long) -> Id;
    fn dispatch_semaphore_wait(dsema: Id, timeout: u64) -> std::ffi::c_long;
    fn dispatch_semaphore_signal(dsema: Id) -> std::ffi::c_long;
    fn dispatch_release(object: Id);
}

macro_rules! create_class {
    ($name:expr) => {{
        let class_name = create_raw_string!($name);
        let ptr = objc_getClass(class_name);
        free_raw_string!(class_name);
        ptr
    }};
}

macro_rules! call_function {
    ($class:expr, $name:expr $(, $args:expr)* $(,)?) => {{
        unsafe {
            let cstr = create_raw_string!($name);
            let sel = sel_registerName(cstr);
            free_raw_string!(cstr);

            objc_msgSend($class, sel $(, $args)*)
        }
    }};
}

// ============ END BINDINGS ===========

struct SessionWrapper {
    session: Class,
}

impl SessionWrapper {
    fn new(session: Class) -> Self {
        // session.retain();
        SessionWrapper { session }
    }
}

impl Drop for SessionWrapper {
    fn drop(&mut self) {
        call_function!(self.session, "release");
    }
}

impl PtrMagic for SessionWrapper {}

pub(super) struct MacOSHttp {}

extern "C" fn http_completion_invoke(block_ptr: Class, data: Class, response: Class, error: Class) {
    let block = unsafe { HttpCompletionBlock::from_borrow_void(block_ptr) };

    if !data.is_null() {
        block.response_data_ptr = call_function!(data, "retain");
    }

    if !response.is_null() {
        let is_http = call_function!(
            response,
            "isKindOfClass:",
            create_class!("NSHTTPURLResponse")
        );
        if !is_http.is_null() {
            block.http_response_ptr = call_function!(response, "retain");
        }
    }

    if !error.is_null() {
        block.execution_error_ptr = call_function!(error, "retain");
    }

    unsafe {
        dispatch_semaphore_signal(block.semaphore);
    }
}

impl ClientCallbacks for MacOSHttp {
    fn setup(client: &mut super::Client) -> Result<(), String> {
        if client.value != null_mut() {
            return Ok(());
        }

        // user agent
        let config = call_function!(
            create_class!("NSURLSessionConfiguration"),
            "defaultSessionConfiguration"
        );

        // Create strings
        let user_agent_key = call_function!(
            create_class!("NSString"),
            "stringWithUTF8String:",
            c"User-Agent".as_ptr()
        );
        let user_agent_cval = create_raw_string!(client.data.user_agent.clone());
        let user_agent_value = call_function!(
            create_class!("NSString"),
            "stringWithUTF8String:",
            user_agent_cval
        );
        unsafe {
            free_raw_string!(user_agent_cval);
        }
        let headers_dict = call_function!(
            create_class!("NSDictionary"),
            "dictionaryWithObject:forKey:",
            user_agent_value,
            user_agent_key
        );
        // Set it to config
        call_function!(config, "setHTTPAdditionalHeaders:", headers_dict);

        let session = call_function!(
            create_class!("NSURLSession"),
            "sessionWithConfiguration:",
            config
        );

        client.value = SessionWrapper::new(session).into_void();

        Ok(())
    }

    fn create_request(
        client: &mut super::Client,
        path: String,
        rt: super::RequestType,
    ) -> Result<super::ClientResponse, String> {
        if client.value.is_null() {
            return pxs_error!("Client.apple.value is null.");
        }

        let mut cstring = CStringSafe::new();

        let wrapper = unsafe { SessionWrapper::from_borrow_void(client.value) };
        if wrapper.session.is_null() {
            return pxs_error!("Client.apple.session is null.");
        }

        // HTTP scheme
        let scheme = if client.https { "https://" } else { "http://" };

        let mut full_url = format!("{scheme}/{}", client.data.domain_name);

        if !path.is_empty() && path.chars().nth(0).unwrap() != '/' {
            full_url.push('/');
        }
        full_url.push_str(&path);

        // Create ns url
        let ns_url_str = call_function!(
            create_class!("NSString"),
            "stringWithUTF8String:",
            cstring.new_string(&full_url)
        );
        let ns_url = call_function!(create_class!("NSURL"), "URLWithString:", ns_url_str);

        // Check nullable
        if ns_url.is_null() {
            return pxs_error!("Invalid URL construction: {full_url}");
        }

        // Create request
        let request = call_function!(
            create_class!("NSMutableURLRequest"),
            "requestWithURL:",
            ns_url
        );
        let timeout_sec = (client.data.timeout as f64) / 1000.0;
        call_function!(request, "setTimeoutInterval:", timeout_sec);

        // Method setup
        let http_method = match rt {
            crate::pxs_core::http::RequestType::GET => "GET",
            crate::pxs_core::http::RequestType::POST => "POST",
            crate::pxs_core::http::RequestType::PUT => "PATCH",
            crate::pxs_core::http::RequestType::PATCH => "DELETE",
            crate::pxs_core::http::RequestType::DELETE => "PUT",
        };

        let ns_string_http_method = call_function!(
            create_class!("NSString"),
            "stringWithUTF8String:",
            cstring.new_string(&http_method)
        );
        call_function!(request, "setHTTPMethod:", ns_string_http_method);

        // headers string
        let mut cheaders = CStringSafe::new();
        // Headers
        let headers = get_header_parts(&client.data.headers);
        for h in headers {
            // Split at ':'
            let split = h.split(':').collect::<Vec<&str>>();
            if split.len() < 2 {
                continue;
            }

            let key = split[0];
            let value = split[1].trim_start();

            let ns_key = call_function!(
                create_class!("NSString"),
                "stringWithUTF8String:",
                cheaders.new_string(&key)
            );
            let ns_value = call_function!(
                create_class!("NSString"),
                "stringWithUTF8String:",
                cheaders.new_string(&value)
            );

            if !ns_key.is_null() && !ns_value.is_null() {
                call_function!(request, "setValue:forHTTPHeaderField:", ns_value, ns_key);
            }
        }
        drop(cheaders);

        // Data setup if NOT GET.
        if rt != RequestType::GET && !client.data.body.is_empty() {
            let body_size = client.data.body.len();
            let body_ptr = client.data.body.clone().as_mut_ptr();
            let body_data = call_function!(
                create_class!("NSData"),
                "dataWithBytes:length:",
                body_ptr,
                body_size
            );
            call_function!(request, "setHTTPBody:", body_data);
        }

        // Setup for response.
        #[allow(unused_mut)]
        let mut response_data = null_mut();
        #[allow(unused_mut)]
        let mut http_response = null_mut();
        #[allow(unused_mut)]
        let mut execution_error = null_mut();
        let semaphore = unsafe { dispatch_semaphore_create(0) };

        #[allow(unused_mut)]
        let mut block = HttpCompletionBlock {
            isa: null_mut(),
            flags: 0,
            reserved: 0,
            invoke: http_completion_invoke,
            response_data_ptr: response_data,
            http_response_ptr: http_response,
            execution_error_ptr: execution_error,
            semaphore,
        };
        let block_ptr = block.into_void();
        let task = call_function!(
            wrapper.session,
            "dataTaskWithRequest:completionHandler:",
            request,
            block_ptr
        );
        call_function!(task, "resume");
        unsafe {
            dispatch_semaphore_wait(semaphore, DISPATCH_TIME_FOREVER);
            dispatch_release(semaphore);
        }

        // Automatically get dropped when leaving scope.
        let _response_wrapper = SessionWrapper::new(response_data);
        let _http_wrapper = SessionWrapper::new(http_response);
        let _execution_error_wrapper = SessionWrapper::new(execution_error);

        // Check errors
        if !execution_error.is_null() {
            let localized_description = call_function!(execution_error, "localizedDescription");
            let c_string = call_function!(localized_description, "UTF8String");
            if c_string.is_null() {
                return pxs_error!("Client.execution_error is null.");
            }
            let msg = borrow_string!(c_string as *const std::ffi::c_char);
            let owned_string = msg.to_string();

            return pxs_error!("{owned_string}");
        }

        // Get response body.
        let response_body = if !response_data.is_null() {
            let bytes_ptr = call_function!(response_data, "bytes");
            if bytes_ptr.is_null() {
                return pxs_error!("Client.response_data.bytes is null");
            }
            let bytes_length = call_function!(response_data, "length");
            if bytes_length.is_null() {
                return pxs_error!("Client.response_data.lenght is null.");
            }

            // Convert from C to rust.
            let bytes = unsafe {
                slice::from_raw_parts(bytes_ptr as *mut std::ffi::c_char, bytes_length as usize)
            };
            let str = String::from_utf8(bytes.iter().map(|v| *v as u8).collect());
            if str.is_err() {
                return pxs_error!("Client.str = {}", str.unwrap_err().to_string());
            }
            str.unwrap()
        } else {
            String::new()
        };

        // status code
        let status_code = if !http_response.is_null() {
            let res = call_function!(http_response, "statusCode");
            if res.is_null() {
                return pxs_error!("Client.status_code is null");
            }
            unsafe { *(res as *mut i32) }
        } else {
            0
        };

        let mut client_response = ClientResponse::new();
        client_response.fill(&client.data);
        client_response.status = status_code;
        client_response.data.body = response_body;

        Ok(client_response)
    }

    fn free(v: crate::shared::pxs_Opaque) {
        if v.is_null() {
            return;
        }

        let _ = unsafe { SessionWrapper::from_raw_void(v) };
    }
}
