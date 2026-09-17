// Windows native IMPL. Uses WinHTTP.
// https://simplifycpp.org/?id=a0912

use std::os::windows::ffi::OsStrExt;
use std::ffi::OsStr;
use std::ptr::null_mut;

use etffi::cstring::CStringSafe;

use crate::pxs_core::http::{Client, ClientCallbacks, ClientResponse, RequestType, get_header_parts};
use crate::shared::pxs_Opaque;

// ======== WinHTTP Binding ========

type LPVOID = pxs_Opaque;
type DWORD = i32;
type BOOL = i32;
type HINTERNET = pxs_Opaque;
type LPCWSTR = *mut u16;
type WideString = Vec<u16>;
type WORD = std::ffi::c_ushort;
type INTERNET_PORT = WORD;
type DWORD_PTR = std::ffi::c_ulonglong;
type LPDWORD = *mut DWORD;

#[allow(unused)]
const TRUE : i32 = 1;
const FALSE : i32 = 0;
const WINHTTP_ERROR_BASE: i32 = 12000;
const ERROR_WINHTTP_INCORRECT_HANDLE_TYPE: i32 = WINHTTP_ERROR_BASE + 18;
const ERROR_WINHTTP_INTERNAL_ERROR: i32 = WINHTTP_ERROR_BASE + 4;
const ERROR_WINHTTP_INVALID_URL: i32 = WINHTTP_ERROR_BASE + 5;
const ERROR_WINHTTP_OPERATION_CANCELLED: i32 = WINHTTP_ERROR_BASE + 17;
const ERROR_WINHTTP_UNRECOGNIZED_SCHEME: i32 = WINHTTP_ERROR_BASE + 6;
const ERROR_WINHTTP_SHUTDOWN: i32 = WINHTTP_ERROR_BASE + 12;
const ERROR_NOT_ENOUGH_MEMORY: i32 = 8;

const WINHTTP_ACCESS_TYPE_NO_PROXY: i32 = 1;
const WINHTTP_NO_PROXY_NAME: LPCWSTR = null_mut();
const WINHTTP_NO_PROXY_BYPASS: LPCWSTR = null_mut();
const WINHTTP_NO_REFERER: LPCWSTR = null_mut();
const WINHTTP_DEFAULT_ACCEPT_TYPES: LPCWSTR = null_mut();
const WINHTTP_FLAG_SECURE: DWORD = 0x00800000;  // use SSL if applicable (HTTPS)
const WINHTTP_NO_REQUEST_DATA: LPVOID = null_mut();
const WINHTTP_QUERY_STATUS_CODE: DWORD = 19;  // special: part of status line
const WINHTTP_QUERY_FLAG_NUMBER: DWORD = 0x20000000;
const WINHTTP_HEADER_NAME_BY_INDEX: LPCWSTR = null_mut();
const WINHTTP_NO_HEADER_INDEX: LPDWORD = null_mut();


const INTERNET_DEFAULT_HTTP_PORT: i32 = 80;          //    "     "  HTTP   "
const INTERNET_DEFAULT_HTTPS_PORT: i32 = 443;         //    "     "  HTTPS  "

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
    ) -> BOOL;

    fn WinHttpSetTimeouts(
        hInternet: HINTERNET,
        nResolveTimeout: i32,
        nConnectTimeout: i32,
        nSendTimeout: i32,
        nReceiveTimeout: i32
    ) -> BOOL;

    fn WinHttpConnect(
        hSession: HINTERNET,
        pswzServerName: LPCWSTR,
        nServerPort: INTERNET_PORT,
        dwReserved: DWORD
    ) -> HINTERNET;

    fn WinHttpOpenRequest(
        hConnect: HINTERNET ,
        pwszVerb: LPCWSTR,
        pwszObjectName: LPCWSTR,
        pwszVersion: LPCWSTR,
        pwszReferrer: LPCWSTR ,
        ppwszAcceptTypes: LPCWSTR,
        dwFlags: DWORD
    ) -> HINTERNET;

    fn WinHttpSendRequest(
        hRequest: HINTERNET,
        lpszHeaders: LPCWSTR,
        dwHeadersLength: DWORD,
        lpOptional: LPVOID,
        dwOptionalLength: DWORD,
        dwTotalLength: DWORD,
        dwContext: DWORD_PTR
    ) -> BOOL;

    fn WinHttpReceiveResponse(
        hRequest: HINTERNET,
        lpReserved: LPVOID
    ) -> BOOL;

    fn WinHttpQueryDataAvailable(
        hRequest: HINTERNET,
        lpdwNumberOfBytesAvailable: LPDWORD
    ) -> BOOL;

    fn WinHttpReadData(
        hRequest: HINTERNET,
        lpBuffer: LPVOID,
        dwNumberOfBytesRead: DWORD,
        lpdwNumberOfBytesRead: LPDWORD
    ) -> BOOL;

    fn WinHttpQueryHeaders(
        hRequest: HINTERNET,
        dwInfoLevel: DWORD,
        pwszName: LPCWSTR,
        lpBuffer: LPVOID,
        lpdwBufferLength: LPDWORD,
        lpdwIndex: LPDWORD
    ) -> BOOL;

}

#[link(name="kernel32")]
unsafe extern "system" {
    fn GetLastError() -> DWORD;
}

// ======== End WinHTTP Binding ========

/// Convert a &str into a wide string.
fn to_wstring(utf8_str: &str) -> WideString {
    OsStr::new(utf8_str).encode_wide()
    .chain(std::iter::once(0))
    .collect()
}

/// Get lat error for WinHTTP
fn get_error() -> String {
    let error = unsafe { GetLastError() };
    if error == ERROR_WINHTTP_INCORRECT_HANDLE_TYPE {
        "The type of handle supplied is incorrect for this operation.".to_string()
    } else if error == ERROR_WINHTTP_INTERNAL_ERROR {
        "An internal error has occurred.".to_string()
    } else if error == ERROR_WINHTTP_INVALID_URL {
        "The URL is invalid.".to_string()
    } else if error == ERROR_WINHTTP_OPERATION_CANCELLED {
        "The operation was canceled, usually because the handle on which the request was operating was closed before the operation completed.".to_string()
    } else if error == ERROR_WINHTTP_UNRECOGNIZED_SCHEME {
        "The URL scheme could not be recognized, or is not supported.".to_string()
    } else if error == ERROR_WINHTTP_SHUTDOWN {
        "The WinHTTP function support is being shut down or unloaded.".to_string()
    } else if error == ERROR_NOT_ENOUGH_MEMORY {
        "Not enough memory was available to complete the requested operation. (Windows error code)".to_string()
    } else {
        format!("Unkown error code: {error}")
    }
}

fn get_request_type(rt: RequestType) -> WideString {
    match rt {
        RequestType::GET => to_wstring("GET"),
        RequestType::POST => to_wstring("POST"),
        RequestType::PUT => to_wstring("PATCH"),
        RequestType::PATCH => to_wstring("PUT"),
        RequestType::DELETE => to_wstring("DELETE"),
    }
}

pub(super) struct WindowsHTTP {}

impl ClientCallbacks for WindowsHTTP {
    fn setup(client: &mut Client) -> Result<(), String> {
        // Already setup!
        if client.value != null_mut() {
            return Ok(());
        }

        let mut user_agent = to_wstring(&client.data.user_agent);
        let h_session = unsafe { WinHttpOpen(
            user_agent.as_mut_ptr(), 
            WINHTTP_ACCESS_TYPE_NO_PROXY, 
            WINHTTP_NO_PROXY_NAME, 
            WINHTTP_NO_PROXY_BYPASS, 
            0
        )};

        if h_session.is_null() {
            return Err("Client.win32.value is null.".to_string());
        }

        client.value = h_session;
        Ok(())
    }

    fn create_request(client: &mut Client, path: String, rt: RequestType) -> Result<ClientResponse, String> {
        if client.value.is_null() {
            return Err("Client.win32.value is null.".to_string());
        }

        let mut domain_name = to_wstring(&client.data.domain_name);
        let mut wpath = to_wstring(&path);

        // Set timeouts
        unsafe { WinHttpSetTimeouts(
            client.value, 
            client.data.timeout as i32, 
            client.data.timeout as i32, 
            client.data.timeout as i32, 
            client.data.timeout as i32
        )};

        // Get the port, HTTP/s
        let default_port = if client.https {
            INTERNET_DEFAULT_HTTPS_PORT
        } else {
            INTERNET_DEFAULT_HTTP_PORT
        };

        // Connect to server
        let h_connect = unsafe { WinHttpConnect(
            client.value,
            domain_name.as_mut_ptr(),
            default_port as u16,
            0
        ) };

        if h_connect.is_null() {
            return Err(get_error());
        }

        // Get request type
        let mut request_type = get_request_type(rt);
        // TODO: // Get version
        // let version = match client.data.version {
        //     super::HTTPVersion::HTTP_1_1 => todo!(),
        //     super::HTTPVersion::HTTP_2 => todo!(),
        //     super::HTTPVersion::HTTP_3 => todo!(),
        // };

        // Create request
        let h_request = unsafe { WinHttpOpenRequest(
            h_connect, 
            request_type.as_mut_ptr(), 
            wpath.as_mut_ptr(), 
            null_mut(), 
            WINHTTP_NO_REFERER, 
            WINHTTP_DEFAULT_ACCEPT_TYPES, 
            WINHTTP_FLAG_SECURE
        ) };

        if h_request.is_null() {
            Self::free(h_connect);
            return Err(get_error());
        }

        // Get headers
        let headers = get_header_parts(&client.data.headers);
        let mut header_string: WideString = vec![];
        // Make the headers use what windows expects.
        if headers.len() > 0 {
            // Do stuff....
            let total = headers.join("\r\n");
            header_string = to_wstring(&total);
        }
        let header_ptr = if header_string.len() > 0 {
            header_string.as_mut_ptr()
        } else {
            null_mut()
        };

        let mut cstring = CStringSafe::new();
        // The body
        let body: LPVOID = if client.data.body.len() > 0 {
            // This gets freed on rop.
            cstring.new_string(&client.data.body) as pxs_Opaque
        } else {
            null_mut()
        };

        // Send request
        let ok = unsafe { match rt {
            RequestType::GET => {
                WinHttpSendRequest(
                    h_request, 
                    header_ptr, 
                    0, 
                    WINHTTP_NO_REQUEST_DATA, 
                    0, 
                    0,
                    0
                )
            },
            RequestType::POST => {
                WinHttpSendRequest(
                    h_request, 
                    header_ptr, 
                    -1, 
                    body, 
                    client.data.body.len() as i32, 
                    client.data.body.len() as i32, 
                0)
            },
            RequestType::PUT => todo!(),
            RequestType::PATCH => todo!(),
            RequestType::DELETE => todo!(),
        }};

        if ok == FALSE {
            Self::free(h_connect);
            Self::free(h_request);
            return Err(get_error());
        }

        let ok = unsafe { WinHttpReceiveResponse(h_request, null_mut()) };
        if ok == FALSE {
            Self::free(h_connect);
            Self::free(h_request);
            return Err(get_error());
        }

        // Read response
        let mut response = vec![];
        let mut bytes_avail: DWORD = 0;
        unsafe { loop {
            let bytes_avail_ptr: LPDWORD = &mut bytes_avail as LPDWORD;
            if WinHttpQueryDataAvailable(h_request, bytes_avail_ptr) == FALSE {
                break;
            }
            if bytes_avail == 0 {
                break;
            }

            let mut buffer = vec![0;bytes_avail as usize];
            let mut bytes_read: DWORD = 0;
            let bytes_read_ptr: LPDWORD = &mut bytes_read as LPDWORD;

            if WinHttpReadData(h_request, buffer.as_mut_ptr() as LPVOID, bytes_avail, bytes_read_ptr) == FALSE {
                break;
            }
            response.append(&mut buffer);

            if bytes_avail <= 0 {
                break;
            }
        } }

        // Status code
        let mut status_code: DWORD = 0;
        let mut size = size_of::<DWORD>() as i32;

        let status_code_ptr: LPVOID = &mut status_code as LPDWORD as LPVOID;
        let size_ptr: LPDWORD = &mut size as LPDWORD;

        unsafe { WinHttpQueryHeaders(
            h_request, 
            WINHTTP_QUERY_STATUS_CODE | WINHTTP_QUERY_FLAG_NUMBER, 
            WINHTTP_HEADER_NAME_BY_INDEX, 
            status_code_ptr, 
            size_ptr, 
            WINHTTP_NO_HEADER_INDEX
        ) };

        Self::free(h_request);
        Self::free(h_connect);

        // Now to create the client response
        let mut client_response = ClientResponse::new();
        client_response.fill(&client.data);
        match String::from_utf8(response) {
            Ok(val) => client_response.data.body = val,
            Err(err) => return Err(err.to_string()),
        }
        client_response.status = status_code;
        Ok(client_response)
    }

    fn free(value: HINTERNET) {
        if value.is_null() {
            return;
        }
        unsafe { WinHttpCloseHandle(value) };
    }
}


