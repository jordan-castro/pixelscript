// Apple native IMPL uses Fondation
// Works on all Apple OS.

#![allow(non_snake_case)]

// ============ BINDINGS ===============

use std::ptr::null_mut;

use etffi::{create_raw_string, free_raw_string};

use crate::pxs_core::http::ClientCallbacks;

type ClassName = *const std::ffi::c_char;
type Class = *mut std::ffi::c_void;
type Id = *mut std::ffi::c_void;

const DISPATCH_TIME_FOREVER: u64 = !0;

#[link(name="objc", kind="dylib")]
#[link(name="System", kind="dylib")]
#[link(name="Foundation", kind="framework")]
unsafe extern "C" {
    fn objc_getClass(name: ClassName) -> Class;
    fn sel_registerName(name: ClassName) -> Class;

    fn objc_msgSend(receiver: Class, selector: *mut std::ffi::c_void, ...) -> *mut std::ffi::c_void;
}

#[link(name="System", kind="dylib")]
unsafe extern "C" {
    fn dispatch_semaphore_create(value: std::ffi::c_long) -> Id;
    fn dispatch_semaphore_wait(dsema: Id, timeout: u64) -> std::ffi::c_long;
    fn dispatch_semaphore_signal(dsema: Id) -> std::ffi::c_long;
    fn dispatch_release(object: Id);
}

struct ObjCClass {
    ptr: Class,
}

impl ObjCClass {
    fn new(name: String) -> ObjCClass {
        let class_name = create_raw_string!(name.as_str());
        unsafe {
            let ptr = objc_getClass(class_name); 
            free_raw_string!(class_name);
            ObjCClass { ptr } 
        }
    }

    fn from_ptr(ptr: Class) -> ObjCClass {
        ObjCClass { ptr }
    }

    fn invalidateAndCancel(&self) {
        unsafe {
            let sel = sel_registerName(c"invalidateAndCancel".as_ptr());
            objc_msgSend(self.ptr, sel);
        }
    }

    fn release(&self) {
        unsafe {
            let sel = sel_registerName(c"release".as_ptr());
            objc_msgSend(self.ptr, sel);
        }
    }

    fn retain(&self) {
        unsafe {
            let sel = sel_registerName(c"retain".as_ptr());
            Self::from_ptr(objc_msgSend(self.ptr, sel));
        }
    }
}

struct NSURLSessionConfiguration {
    class: ObjCClass
}

impl NSURLSessionConfiguration {
    /// This is specific to NSURLSessionConfiguration class.
    fn defaultSessionConfiguration() -> ObjCClass {
        unsafe {
            let class = ObjCClass::new("NSURLSessionConfiguration".to_string());
            let sel = sel_registerName(c"defaultSessionConfiguration".as_ptr());

            // class.
            ObjCClass { ptr: objc_msgSend(class.ptr, sel) }
        }
    }
}


// ============ END BINDINGS ===========

struct SessionWrapper {
    session: ObjCClass
}

impl SessionWrapper {
    fn new(session: ObjCClass) -> Self {
        session.retain();
        SessionWrapper { session }
    }
}

impl Drop for SessionWrapper {
    fn drop(&mut self) {
        self.session.release();
    }
}

pub(super) struct MacOSHttp {}
impl ClientCallbacks for MacOSHttp {
    fn setup(client: &mut super::Client) -> Result<(), String> {
        if client.value != null_mut() {
            return Ok(());
        }   

        // user agent
        let config = NSURLSessionConfiguration::defaultSessionConfiguration();
    }

    fn create_request(client: &mut super::Client, path: String, rt: super::RequestType) -> Result<super::ClientResponse, String> {
        todo!()
    }

    fn free(v:crate::shared::pxs_Opaque) {
        todo!()
    }
}
// pub(super) struct WindowsHTTP {}


// void Client::setup() {
//     // Already exists?
//     if (this->internal != nullptr) {
//         return;
//     }

//     NSURLSessionConfiguration* config = [NSURLSessionConfiguration defaultSessionConfiguration];
//     config.HTTPAdditionalHeaders = @{
//         @"User-Agent": [NSString stringWithUTF8String:user_agent.c_str()]
//     };
//     NSURLSession* session = [NSURLSession sessionWithConfiguration:config];
//     auto wrapper = new SessionWrapper(session);

//     this->internal = static_cast<void*>(wrapper);
// }

// ClientResponse* Client::create_request(const std::string &path, const RequestType &rt) {
//     if (this->internal == nullptr) {
//         throw std::runtime_error("Client.apple.internal is null.");
//     }

//     SessionWrapper* wrapper = static_cast<SessionWrapper*>(this->internal);
//     if (wrapper->session == nil) {
//         throw std::runtime_error("Client.apple.session is null.");
//     }

//     // A little simpler than WinHTTP.
//     std::string scheme = this->use_https ? "https://" : "http://";
//     std::string full_url_str = scheme + this->data.domain_name;

//     // TODO(jc) is this necessary?
//     if (!path.empty() && path[0] != '/') {
//         full_url_str += "/";
//     }
//     full_url_str += path;

//     NSString* ns_url_str = [NSString stringWithUTF8String:full_url_str.c_str()];
//     NSURL* url = [NSURL URLWithString:ns_url_str];

//     // TODO(jc) yeah again is this necessary?
//     if (!url) {
//         throw std::runtime_error("Invalid URL construction: " + full_url_str);
//     }
//     NSMutableURLRequest* request = [NSMutableURLRequest requestWithURL:url];
//     NSTimeInterval timeout_sec = static_cast<NSTimeInterval>(this->data.timeout) / 1000.0;
//     [request setTimeoutInterval:timeout_sec];

//     switch (rt) {
//         case RequestType::GET:
//             [request setHTTPMethod:@"GET"];
//             break;
//         case RequestType::POST:
//             [request setHTTPMethod:@"POST"];
//             break;
//         case RequestType::PATCH:
//             [request setHTTPMethod:@"PATCH"];
//             break;
//         case RequestType::DELETE:
//             [request setHTTPMethod:@"DELETE"];
//             break;
//         case RequestType::PUT:
//             [request setHTTPMethod:@"PUT"];
//             break;
//     }

//     // header setup.
//     std::vector<std::string> header_parts = get_header_parts();
//     for (const std::string& header : header_parts) {
//         // Check for key:value
//         size_t colon_pos = header.find(":");
//         if (colon_pos == std::string::npos) {
//             continue; // skip it.
//         }
//         std::string key = header.substr(0, colon_pos);
//         std::string value = utils::trim_left(header.substr(colon_pos + 1));

//         NSString* ns_key = [NSString stringWithUTF8String:key.c_str()];
//         NSString* ns_value = [NSString stringWithUTF8String:value.c_str()];

//         if (ns_key && ns_value) {
//             [request setValue:ns_value forHTTPHeaderField:ns_key];
//         }
//     }

//     // data setup if not GET.
//     if (rt != RequestType::GET && !this->data.body.empty()) {
//         size_t body_size = this->data.body.size();
        
//         NSData* body_data = [NSData dataWithBytes:this->data.body.data() length:body_size];
//         [request setHTTPBody:body_data];
//     }

//     // setup for response!
//     __block NSData* response_data = nil;
//     __block NSHTTPURLResponse* http_response = nil;
//     __block NSError* execution_error = nil;

//     // To make it synchronous
//     dispatch_semaphore_t semaphore = dispatch_semaphore_create(0);

//     NSURLSessionDataTask* task = [wrapper->session dataTaskWithRequest:request
//         completionHandler:^(NSData * _Nullable data, NSURLResponse * _Nullable response, NSError * _Nullable error) {
//             // I use retain because I dont use ARC.
//             if (data) {
//                 response_data = [data retain];
//             }
//             if (response && [response isKindOfClass:[NSHTTPURLResponse class]]) {
//                 http_response = [(NSHTTPURLResponse*)response retain];
//             }
//             if (error) {
//                 execution_error = [error retain];
//             }
//             dispatch_semaphore_signal(semaphore);
//         }
//     ];
//     [task resume];
//     // TODO(jc) do I really want forever here?
//     dispatch_semaphore_wait(semaphore, DISPATCH_TIME_FOREVER);
//     // We don't use ARC.
//     dispatch_release(semaphore);
//     // Error checking
//     if (execution_error != nil) {
//         std::string msg = [[execution_error localizedDescription] UTF8String];
//         [execution_error release];
//         throw std::runtime_error(msg);
//     }

//     // Body
//     std::string response_body_str;
//     if (response_data != nil && [response_data length] > 0) {
//         response_body_str.assign(static_cast<const char*>([response_data bytes]), [response_data length]);
//         [response_data release];
//     }

//     NSInteger status_code = 0;
//     if (http_response != nil) {
//         status_code = [http_response statusCode];
//         [http_response release];
//     }

//     // Client response fr fr.
//     ClientResponse* cr = new ClientResponse();
//     cr->fill(this->data);
//     cr->data.body = response_body_str;
//     cr->status = status_code;

//     return cr;
// }

// Client::~Client() {
//     if (!this->internal) {
//         return;
//     }

//     delete static_cast<SessionWrapper*>(this->internal);
// }