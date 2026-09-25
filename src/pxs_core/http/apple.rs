// Apple native IMPL uses Fondation
// Works on all Apple OS.

// ============ BINDINGS ===============

use etffi::create_raw_string;

type ClassName = *const std::ffi::c_char;
type Class = *mut std::ffi::c_void;

#[link(name="objc", kind="dylib")]
#[link(name="System", kind="dylib")]
#[link(name="Foundation", kind="framework")]
unsafe extern "C" {
    fn objc_getClass(name: ClassName) -> Class;
    fn sel_registerName(name: ClassName) -> Class;

    fn objc_msgSend(receiver: Class, selector: *mut std::ffi::c_void, ...) -> *mut std::ffi::c_void;
}

struct ObjCClass {
    name: String,
    ptr: Class,
    class_name: ClassName
}
impl ObjCClass {
    fn new(name: String) -> ObjCClass {
        let class_name = create_raw_string!(name.as_str());
        
    }
}
impl Drop for ObjCClass {
    fn drop(&mut self) {
        todo!()
    }
}

// ============ END BDINGINS ===========

// #import <Foundation/Foundation.h>
// #include <string>
// #include <vector>
// #include <stdexcept>
// #include "http.hpp"
// #include "utils.hpp"
// #include <stdexcept>
// #include <dispatch/dispatch.h>
// #include <iostream>

// // Wrap the session using RAII.
// struct SessionWrapper {
//     NSURLSession* session;

//     SessionWrapper(NSURLSession* s) {
//         session = [s retain];
//     }
//     ~SessionWrapper() {
//         if (session) {
//             [session invalidateAndCancel];
//             [session release];
//             session = nil;
//         }
//     }
// };

// void Client::setup() {
//     // Already exists?
//     if (this->internal != nullptr) {
//         return;
//     }

//     auto user_agent = this->data.user_agent;
//     if (user_agent.empty()) {
//         user_agent = "yoyo_rt";
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