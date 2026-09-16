// Windows native IMPL. Uses WinHTTP.
// https://simplifycpp.org/?id=a0912

use std::os::windows::ffi::OsStrExt;
use std::ffi::OsStr;
use std::ptr::{null, null_mut};

use crate::shared::pxs_Opaque;

// ======== WinHTTP Binding ========

type LPVOID = pxs_Opaque;
type DWORD = u32;
type BOOL = i32;
type HINTERNET = pxs_Opaque;
type LPCWSTR = *mut u16;

const WINHTTP_ERROR_BASE: i32 = 12000;
const ERROR_WINHTTP_INCORRECT_HANDLE_TYPE: i32 = WINHTTP_ERROR_BASE + 18;
const ERROR_WINHTTP_INTERNAL_ERROR: i32 = WINHTTP_ERROR_BASE + 4;
const ERROR_WINHTTP_INVALID_URL: i32 = WINHTTP_ERROR_BASE + 5;
const ERROR_WINHTTP_OPERATION_CANCELLED: i32 = WINHTTP_ERROR_BASE + 17;
const ERROR_WINHTTP_UNRECOGNIZED_SCHEME: i32 = WINHTTP_ERROR_BASE + 6;
const ERROR_WINHTTP_SHUTDOWN: i32 = WINHTTP_ERROR_BASE + 12;
const ERROR_NOT_ENOUGH_MEMORY: i32 = 8;

// Native linking
#[link(name="winhttp")]
unsafe extern "system" {
    fn WinHttpOpen(
        pszAgentW: LPCWSTR,
        dwAcessType: DWORD,
        pszProxyW: LPCWSTR,
        pszProxyBypassW: LPCWSTR,
        dwFlags: DWORD
    ) -> HINTERNET;

    fn WinHttpCloseHandle(
        hInternet: HINTERNET
    );


}

#[link(name="kernel32")]
unsafe extern "system" {
    fn GetLastError() -> DWORD;
}

// ======== End WinHTTP Binding ========

/// Convert a &str into a wide string.
fn to_wstring(utf8_str: &str) -> Vec<u16> {
    OsStr::new(utf8_str).encode_wide()
    .chain(std::iter::once(0))
    .collect()
}

/// Get lat error for WinHTTP
fn get_error() -> String {
    let error = unsafe { GetLastError() };
    if (error == ERROR_WINHTTP_INCORRECT_HANDLE_TYPE) {
        return "The type of handle supplied is incorrect for this operation.";
    } else if (error == ERROR_WINHTTP_INTERNAL_ERROR) {
        return "An internal error has occurred.";
    } else if (error == ERROR_WINHTTP_INVALID_URL) {
        return "The URL is invalid.";
    } else if (error == ERROR_WINHTTP_OPERATION_CANCELLED) {
        return "The operation was canceled, usually because the handle on which the request was operating was closed before the operation completed.";
    } else if (error == ERROR_WINHTTP_UNRECOGNIZED_SCHEME) {
        return "The URL scheme could not be recognized, or is not supported.";
    } else if (error == ERROR_WINHTTP_SHUTDOWN) {
        return "The WinHTTP function support is being shut down or unloaded.";
    } else if (error == ERROR_NOT_ENOUGH_MEMORY) {
        return "Not enough memory was available to complete the requested operation. (Windows error code)";
    } else {
        return "Unkown error code: " + std::to_string(error);
    }

    // let error_code = 
    String::from("Unkown error code: ")
}

// #include <windows.h>
// #include <winhttp.h>
// #include <cwchar>
// #pragma comment(lib, "winhttp.lib")
// #undef DELETE
// #include "http.hpp"
// #include <stdexcept>
// #include "utils.hpp"


// // Get last error for WinHttp
// std::string get_error() {
//     int error = GetLastError();
//     if (error == ERROR_WINHTTP_INCORRECT_HANDLE_TYPE) {
//         return "The type of handle supplied is incorrect for this operation.";
//     } else if (error == ERROR_WINHTTP_INTERNAL_ERROR) {
//         return "An internal error has occurred.";
//     } else if (error == ERROR_WINHTTP_INVALID_URL) {
//         return "The URL is invalid.";
//     } else if (error == ERROR_WINHTTP_OPERATION_CANCELLED) {
//         return "The operation was canceled, usually because the handle on which the request was operating was closed before the operation completed.";
//     } else if (error == ERROR_WINHTTP_UNRECOGNIZED_SCHEME) {
//         return "The URL scheme could not be recognized, or is not supported.";
//     } else if (error == ERROR_WINHTTP_SHUTDOWN) {
//         return "The WinHTTP function support is being shut down or unloaded.";
//     } else if (error == ERROR_NOT_ENOUGH_MEMORY) {
//         return "Not enough memory was available to complete the requested operation. (Windows error code)";
//     } else {
//         return "Unkown error code: " + std::to_string(error);
//     }
// }

// // Get request type for windows
// std::wstring get_request_type(RequestType rt) {
//     switch (rt) {
//         case RequestType::GET:
//             return L"GET";
//         case RequestType::POST:
//             return L"POST";
//         case RequestType::PATCH:
//             return L"PATCH";
//         case RequestType::PUT:
//             return L"PUT";
//         case RequestType::DELETE:
//             return L"DELETE";
//     }
// }

// // RAII wrapper for HINTERNET handles.
// struct HInternetWrapper {
//     HINTERNET handle;

//     HInternetWrapper(HINTERNET o) : handle(o) {}
//     ~HInternetWrapper() {
//         if (handle) {
//             WinHttpCloseHandle(handle);
//         }
//     }
// };

// void Client::setup() {
//     // Already setup.
//     if (this->internal != nullptr) {
//         return;
//     }

//     auto user_agent = this->data.user_agent;
//     if (user_agent.empty()) {
//         user_agent = "yoyo_rt";
//     }

//     std::wstring wuser_agent = to_wstring(user_agent);

//     HINTERNET h_session = WinHttpOpen(
//         wuser_agent.c_str(),
//         WINHTTP_ACCESS_TYPE_NO_PROXY,
//         WINHTTP_NO_PROXY_NAME,
//         WINHTTP_NO_PROXY_BYPASS,
//         0
//     );

//     if (!h_session) {
//         throw std::runtime_error(get_error());
//     }

//     // Wrap it and save it.
//     auto wrapper = new HInternetWrapper(h_session);

//     this->internal = static_cast<void*>(wrapper);
// }

// ClientResponse* Client::create_request(const std::string& path, const RequestType& rt) {
//     // convert back to session
//     if (this->internal == nullptr) {
//         throw std::runtime_error("Client.win32.internal is null.");
//     }
//     auto wrapper = static_cast<HInternetWrapper*>(this->internal);
//     auto wdomain_name = to_wstring(this->data.domain_name);

//     if (wrapper->handle == nullptr) {
//         throw std::runtime_error("Client.win32.handle is null.");
//     }

//     // Set timeouts
//     WinHttpSetTimeouts(
//         wrapper->handle,
//         this->data.timeout,
//         this->data.timeout,
//         this->data.timeout,
//         this->data.timeout
//     );

//     // Get the port, HTTP/S.
//     int default_port;
//     if (this->use_https) {
//         default_port = INTERNET_DEFAULT_HTTPS_PORT;
//     } else {
//         default_port = INTERNET_DEFAULT_HTTP_PORT;
//     }

//     // Connect to the server yo!
//     HINTERNET h_connect = WinHttpConnect(
//         wrapper->handle,
//         wdomain_name.c_str(),
//         default_port,
//         0
//     );

//     if (!h_connect) {
//         // Get error
//         throw std::runtime_error(get_error());
//     }

//     // We are now connected to a server. Lets wrap it
//     auto connect_wrapper = HInternetWrapper(h_connect);

//     // Get the request type
//     // Create the request.
//     HINTERNET h_request = WinHttpOpenRequest(
//         connect_wrapper.handle,
//         get_request_type(rt).c_str(),
//         to_wstring(path).c_str(),
//         nullptr,
//         WINHTTP_NO_REFERER,
//         WINHTTP_DEFAULT_ACCEPT_TYPES,
//         WINHTTP_FLAG_SECURE
//     );

//     if (!h_request) {
//         throw std::runtime_error(get_error());
//     }
//     auto request_wrapper = HInternetWrapper(h_request);

//     // Get headers
//     wchar_t* headers_string = NULL;
//     auto header_parts = get_header_parts();
//     if (header_parts.size() > 0) {
//         // Do stuff
//         std::string total = utils::join("\r\n", header_parts);
//         auto wtotal = to_wstring(total);
//         headers_string = wcsdup(wtotal.c_str());
//     }

//     // The muthafucking body yo!
//     LPVOID tha_body = NULL;
//     if (this->data.body.size() > 0) {
//         tha_body = (LPVOID)this->data.body.data();
//     }

//     // Send request
//     BOOL ok;
//     if (rt == RequestType::GET) {
//         ok = WinHttpSendRequest(
//             request_wrapper.handle,
//             headers_string,
//             0,
//             WINHTTP_NO_REQUEST_DATA,
//             0,
//             0,
//             0
//         );
//     } else if (rt == RequestType::POST) {
//         // Send body
//         ok = WinHttpSendRequest(
//             request_wrapper.handle,
//             headers_string,
//             -1,
//             tha_body,
//             this->data.body.size(),
//             this->data.body.size(),
//             0
//         );
//     }
//     // todo(jc) Add other methods.

//     // Delete it if not null.
//     if (headers_string) {
//         free(headers_string);
//         headers_string = NULL;
//     }

//     if (!ok) {
//         throw std::runtime_error(get_error());
//     }
//     ok = WinHttpReceiveResponse(request_wrapper.handle, nullptr);
//     if (!ok) {
//         throw std::runtime_error(get_error());
//     }
//     // Read response
//     std::string response;
//     DWORD bytes_avail;
//     do {
//         if (!WinHttpQueryDataAvailable(request_wrapper.handle, &bytes_avail)) {
//             break;
//         }

//         if (bytes_avail == 0) {
//             break;
//         }

//         std::vector<char> buffer(bytes_avail);
//         DWORD bytes_read = 0;

//         if (!WinHttpReadData(
//             request_wrapper.handle,
//             buffer.data(),
//             bytes_avail,
//             &bytes_read
//         )) {
//             break;
//         }

//         response.append(buffer.data(), bytes_read);
//     } while (bytes_avail > 0);

//     // Get status code.
//     DWORD status_code = 0;
//     DWORD size = sizeof(status_code);

//     WinHttpQueryHeaders(
//         request_wrapper.handle,
//         WINHTTP_QUERY_STATUS_CODE | WINHTTP_QUERY_FLAG_NUMBER,
//         WINHTTP_HEADER_NAME_BY_INDEX,
//         &status_code,
//         &size,
//         WINHTTP_NO_HEADER_INDEX
//     );

//     // Now lets return the response yo!
//     auto cr = new ClientResponse();
//     cr->fill(this->data);
//     cr->data.body = response;
//     cr->status = status_code;
    
//     return cr;
// }

// Client::~Client() {
//     if (!this->internal) {
//         return;
//     }
//     delete static_cast<HInternetWrapper*>(this->internal);
// }