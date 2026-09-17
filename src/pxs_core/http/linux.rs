//! Linux native IMPL. Uses curl

// ======= BINDINGS ========

use std::{ffi::c_int, net::Shutdown::Write, ptr::null_mut};

use etffi::{borrow_string, cstring::CStringSafe, ptr_magic::PtrMagic};

use crate::pxs_core::http::{ClientCallbacks, ClientResponse, get_header_parts};

type CURL = std::ffi::c_void;
type CURL_PTR = *mut CURL;

#[repr(C)]
struct curl_slist {
    data: *mut std::ffi::c_char,
    next: *mut curl_slist
}

#[derive(Copy, Clone)]
#[repr(C)]
enum CURLcode {
  CURLE_OK = 0,
  CURLE_UNSUPPORTED_PROTOCOL,    /* 1 */
  CURLE_FAILED_INIT,             /* 2 */
  CURLE_URL_MALFORMAT,           /* 3 */
  CURLE_NOT_BUILT_IN,            /* 4 - [was obsoleted in August 2007 for
                                    7.17.0, reused in April 2011 for 7.21.5] */
  CURLE_COULDNT_RESOLVE_PROXY,   /* 5 */
  CURLE_COULDNT_RESOLVE_HOST,    /* 6 */
  CURLE_COULDNT_CONNECT,         /* 7 */
  CURLE_WEIRD_SERVER_REPLY,      /* 8 */
  CURLE_REMOTE_ACCESS_DENIED,    /* 9 a service was denied by the server
                                    due to lack of access - when login fails
                                    this is not returned. */
  CURLE_FTP_ACCEPT_FAILED,       /* 10 - [was obsoleted in April 2006 for
                                    7.15.4, reused in Dec 2011 for 7.24.0]*/
  CURLE_FTP_WEIRD_PASS_REPLY,    /* 11 */
  CURLE_FTP_ACCEPT_TIMEOUT,      /* 12 - timeout occurred accepting server
                                    [was obsoleted in August 2007 for 7.17.0,
                                    reused in Dec 2011 for 7.24.0]*/
  CURLE_FTP_WEIRD_PASV_REPLY,    /* 13 */
  CURLE_FTP_WEIRD_227_FORMAT,    /* 14 */
  CURLE_FTP_CANT_GET_HOST,       /* 15 */
  CURLE_HTTP2,                   /* 16 - A problem in the http2 framing layer.
                                    [was obsoleted in August 2007 for 7.17.0,
                                    reused in July 2014 for 7.38.0] */
  CURLE_FTP_COULDNT_SET_TYPE,    /* 17 */
  CURLE_PARTIAL_FILE,            /* 18 */
  CURLE_FTP_COULDNT_RETR_FILE,   /* 19 */
  CURLE_OBSOLETE20,              /* 20 - NOT USED */
  CURLE_QUOTE_ERROR,             /* 21 - quote command failure */
  CURLE_HTTP_RETURNED_ERROR,     /* 22 */
  CURLE_WRITE_ERROR,             /* 23 */
  CURLE_OBSOLETE24,              /* 24 - NOT USED */
  CURLE_UPLOAD_FAILED,           /* 25 - failed upload "command" */
  CURLE_READ_ERROR,              /* 26 - could not open/read from file */
  CURLE_OUT_OF_MEMORY,           /* 27 */
  CURLE_OPERATION_TIMEDOUT,      /* 28 - the timeout time was reached */
  CURLE_OBSOLETE29,              /* 29 - NOT USED */
  CURLE_FTP_PORT_FAILED,         /* 30 - FTP PORT operation failed */
  CURLE_FTP_COULDNT_USE_REST,    /* 31 - the REST command failed */
  CURLE_OBSOLETE32,              /* 32 - NOT USED */
  CURLE_RANGE_ERROR,             /* 33 - RANGE "command" did not work */
  CURLE_OBSOLETE34,              /* 34 */
  CURLE_SSL_CONNECT_ERROR,       /* 35 - wrong when connecting with SSL */
  CURLE_BAD_DOWNLOAD_RESUME,     /* 36 - could not resume download */
  CURLE_FILE_COULDNT_READ_FILE,  /* 37 */
  CURLE_LDAP_CANNOT_BIND,        /* 38 */
  CURLE_LDAP_SEARCH_FAILED,      /* 39 */
  CURLE_OBSOLETE40,              /* 40 - NOT USED */
  CURLE_OBSOLETE41,              /* 41 - NOT USED starting with 7.53.0 */
  CURLE_ABORTED_BY_CALLBACK,     /* 42 */
  CURLE_BAD_FUNCTION_ARGUMENT,   /* 43 */
  CURLE_OBSOLETE44,              /* 44 - NOT USED */
  CURLE_INTERFACE_FAILED,        /* 45 - CURLOPT_INTERFACE failed */
  CURLE_OBSOLETE46,              /* 46 - NOT USED */
  CURLE_TOO_MANY_REDIRECTS,      /* 47 - catch endless re-direct loops */
  CURLE_UNKNOWN_OPTION,          /* 48 - User specified an unknown option */
  CURLE_SETOPT_OPTION_SYNTAX,    /* 49 - Malformed setopt option */
  CURLE_OBSOLETE50,              /* 50 - NOT USED */
  CURLE_OBSOLETE51,              /* 51 - NOT USED */
  CURLE_GOT_NOTHING,             /* 52 - when this is a specific error */
  CURLE_SSL_ENGINE_NOTFOUND,     /* 53 - SSL crypto engine not found */
  CURLE_SSL_ENGINE_SETFAILED,    /* 54 - can not set SSL crypto engine as
                                    default */
  CURLE_SEND_ERROR,              /* 55 - failed sending network data */
  CURLE_RECV_ERROR,              /* 56 - failure in receiving network data */
  CURLE_OBSOLETE57,              /* 57 - NOT IN USE */
  CURLE_SSL_CERTPROBLEM,         /* 58 - problem with the local certificate */
  CURLE_SSL_CIPHER,              /* 59 - could not use specified cipher */
  CURLE_PEER_FAILED_VERIFICATION, /* 60 - peer's certificate or fingerprint
                                     was not verified fine */
  CURLE_BAD_CONTENT_ENCODING,    /* 61 - Unrecognized/bad encoding */
  CURLE_OBSOLETE62,              /* 62 - NOT IN USE since 7.82.0 */
  CURLE_FILESIZE_EXCEEDED,       /* 63 - Maximum file size exceeded */
  CURLE_USE_SSL_FAILED,          /* 64 - Requested FTP SSL level failed */
  CURLE_SEND_FAIL_REWIND,        /* 65 - Sending the data requires a rewind
                                    that failed */
  CURLE_SSL_ENGINE_INITFAILED,   /* 66 - failed to initialize ENGINE */
  CURLE_LOGIN_DENIED,            /* 67 - user, password or similar was not
                                    accepted and we failed to login */
  CURLE_TFTP_NOTFOUND,           /* 68 - file not found on server */
  CURLE_TFTP_PERM,               /* 69 - permission problem on server */
  CURLE_REMOTE_DISK_FULL,        /* 70 - out of disk space on server */
  CURLE_TFTP_ILLEGAL,            /* 71 - Illegal TFTP operation */
  CURLE_TFTP_UNKNOWNID,          /* 72 - Unknown transfer ID */
  CURLE_REMOTE_FILE_EXISTS,      /* 73 - File already exists */
  CURLE_TFTP_NOSUCHUSER,         /* 74 - No such user */
  CURLE_OBSOLETE75,              /* 75 - NOT IN USE since 7.82.0 */
  CURLE_OBSOLETE76,              /* 76 - NOT IN USE since 7.82.0 */
  CURLE_SSL_CACERT_BADFILE,      /* 77 - could not load CACERT file, missing
                                    or wrong format */
  CURLE_REMOTE_FILE_NOT_FOUND,   /* 78 - remote file not found */
  CURLE_SSH,                     /* 79 - error from the SSH layer, somewhat
                                    generic so the error message is of
                                    interest when this has happened */

  CURLE_SSL_SHUTDOWN_FAILED,     /* 80 - Failed to shut down the SSL
                                    connection */
  CURLE_AGAIN,                   /* 81 - socket is not ready for send/recv,
                                    wait till it is ready and try again (Added
                                    in 7.18.2) */
  CURLE_SSL_CRL_BADFILE,         /* 82 - could not load CRL file, missing or
                                    wrong format (Added in 7.19.0) */
  CURLE_SSL_ISSUER_ERROR,        /* 83 - Issuer check failed.  (Added in
                                    7.19.0) */
  CURLE_FTP_PRET_FAILED,         /* 84 - a PRET command failed */
  CURLE_RTSP_CSEQ_ERROR,         /* 85 - mismatch of RTSP CSeq numbers */
  CURLE_RTSP_SESSION_ERROR,      /* 86 - mismatch of RTSP Session Ids */
  CURLE_FTP_BAD_FILE_LIST,       /* 87 - unable to parse FTP file list */
  CURLE_CHUNK_FAILED,            /* 88 - chunk callback reported error */
  CURLE_NO_CONNECTION_AVAILABLE, /* 89 - No connection available, the
                                    session is queued */
  CURLE_SSL_PINNEDPUBKEYNOTMATCH, /* 90 - specified pinned public key did not
                                     match */
  CURLE_SSL_INVALIDCERTSTATUS,   /* 91 - invalid certificate status */
  CURLE_HTTP2_STREAM,            /* 92 - stream error in HTTP/2 framing layer
                                  */
  CURLE_RECURSIVE_API_CALL,      /* 93 - an api function was called from
                                    inside a callback */
  CURLE_AUTH_ERROR,              /* 94 - an authentication function returned an
                                    error */
  CURLE_HTTP3,                   /* 95 - An HTTP/3 layer problem */
  CURLE_QUIC_CONNECT_ERROR,      /* 96 - QUIC connection error */
  CURLE_PROXY,                   /* 97 - proxy handshake error */
  CURLE_SSL_CLIENTCERT,          /* 98 - client-side certificate required */
  CURLE_UNRECOVERABLE_POLL,      /* 99 - poll/select returned fatal error */
  CURLE_TOO_LARGE,               /* 100 - a value/data met its maximum */
  CURLE_ECH_REQUIRED,            /* 101 - ECH tried but failed */
  CURL_LAST /* never use! */
}

const CURL_GLOBAL_SSL: i32 = 1 << 0; /* no purpose since 7.57.0 */
const CURL_GLOBAL_WIN32: i32 = 1 << 1;
const CURL_GLOBAL_ALL: i32 = CURL_GLOBAL_SSL | CURL_GLOBAL_WIN32;
const CURL_GLOBAL_NOTHING: i32 = 0;
const CURL_GLOBAL_DEFAULT: i32 = CURL_GLOBAL_ALL;
const CURL_GLOBAL_ACK_EINTR: i32 = 1 << 2;

const CURLOPT_URL: i32 = 10002;
const CURLOPT_USERAGENT: i32 = 10018;
const CURLOPT_TIMEOUT_MS: i32 = 155;
const CURLOPT_CONNECTTIMEOUT_MS: i32 = 156;
const CURLOPT_HTTPGET: i32 = 80;
const CURLOPT_POST: i32 = 47;
const CURLOPT_CUSTOMREQUEST: i32 = 10036;
const CURLOPT_POSTFIELDSIZE: i32 = 60;
const CURLOPT_POSTFIELDS: i32 = 10015;
const CURLOPT_WRITEFUNCTION: i32 = 20011;
const CURLOPT_WRITEDATA: i32 = 10001;

const CURLINFO_RESPONSE_CODE: i32 = 2097154;

type WriteCallback = extern "C" fn (
    *mut std::ffi::c_void,
    usize,
    usize,
    *mut std::ffi::c_void
) -> usize;

#[link(name="curl")]
unsafe extern "C" {
    fn curl_easy_cleanup(curl: CURL_PTR);
    fn curl_slist_free_all(list: *mut curl_slist);
    fn curl_global_init(flags: std::ffi::c_long) -> CURLcode;
    fn curl_easy_init() -> CURL_PTR;
    fn curl_easy_reset(curl: CURL_PTR);
    fn curl_easy_setopt(curl: CURL_PTR, option: i32, ...) -> CURLcode;
    fn curl_slist_append(curl_slist: *mut curl_slist, data: *const std::ffi::c_char) -> *mut curl_slist;
    fn curl_easy_perform(curl: CURL_PTR) -> CURLcode;
    fn curl_easy_strerror(error: CURLcode) -> *const std::ffi::c_char;
    fn curl_easy_getinfo(curl: CURL_PTR, info: i32, ...) -> CURLcode;

}

// ======= End Bindings =======

/// Curl wrapper that will get dropped automatically.
struct CurlWrapper {
    handle: CURL_PTR,
    headers_list: *mut curl_slist
}

impl CurlWrapper {
    fn new(p: CURL_PTR) -> Self {
        Self {
            handle: p,
            headers_list: null_mut(),
        }
    }
}

impl PtrMagic for CurlWrapper {}

impl Drop for CurlWrapper {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe { curl_easy_cleanup(self.handle) };
        }
        if !self.headers_list.is_null() {
            unsafe { curl_slist_free_all(self.headers_list); }
        }
    }
}

struct WriteData {
    data: Vec<u8>
}

impl PtrMagic for WriteData {}

/// Callback to write response body chunks into std::string
extern "C" fn write_callback(contents: *mut std::ffi::c_void, size: usize, nmemb: usize, userp: *mut std::ffi::c_void) -> usize {
    let total_size = size * nmemb;
    let response_body = unsafe { WriteData::from_borrow_void(userp) };

    // Place in write data.
    let cstr = borrow_string!(contents as *mut std::ffi::c_char);
    let mut bytes = cstr.as_bytes().to_vec();
    response_body.data.append(&mut bytes);

    total_size
}

struct LinuxHTTP {}

impl ClientCallbacks for LinuxHTTP {
    fn setup(client: &mut super::Client) -> Result<(), String> {
        if client.value != null_mut() {
            return Ok(());
        }
    
        unsafe { curl_global_init(CURL_GLOBAL_DEFAULT) };
        let wrapper = CurlWrapper::new(unsafe { curl_easy_init() });
        client.value = wrapper.into_void();
        Ok(())
    }

    fn create_request(client: &mut super::Client, path: String, rt: super::RequestType) -> Result<super::ClientResponse, String> {
        if client.value.is_null() {
            return Err("Client.linux.value is null".to_string());
        }

        // Get curl
        let wrapper = unsafe{ CurlWrapper::from_borrow_void(client.value) };
        let curl = wrapper.handle;
        
        // Reset handle between requests.
        unsafe { curl_easy_reset(curl) };

        // Build url
        let scheme = if client.https {
            "https://"
        } else {
            "http://"
        }.to_owned();
        let mut full_url = scheme + &client.data.domain_name;
        if !path.is_empty() && path.chars().nth(0).unwrap_or_default() != '/' {
            full_url.push('/');
        }
        full_url.push_str(&path);

        let mut cstring = CStringSafe::new();
        unsafe {
            curl_easy_setopt(curl, CURLOPT_URL, cstring.new_string(&full_url));
            curl_easy_setopt(curl, CURLOPT_USERAGENT, cstring.new_string(&client.data.user_agent));
            curl_easy_setopt(curl, CURLOPT_TIMEOUT_MS, client.data.timeout as std::ffi::c_int);
            curl_easy_setopt(curl, CURLOPT_CONNECTTIMEOUT_MS, client.data.timeout as std::ffi::c_int);
        }

        // Headers
        let headers = get_header_parts(&client.data.headers);
        let mut headers_list = wrapper.headers_list;

        for h in headers {
            headers_list = unsafe { curl_slist_append(headers_list, cstring.new_string(&h)) };
        }

        // Method setup
        let _ = unsafe { match rt {
            super::RequestType::GET => curl_easy_setopt(curl, CURLOPT_HTTPGET, 1 as std::ffi::c_long),
            super::RequestType::POST => curl_easy_setopt(curl, CURLOPT_POST, 1 as std::ffi::c_long),
            super::RequestType::PUT => curl_easy_setopt(curl, CURLOPT_CUSTOMREQUEST, c"PUT".as_ptr()),
            super::RequestType::PATCH => curl_easy_setopt(curl, CURLOPT_CUSTOMREQUEST, c"PATCH".as_ptr()),
            super::RequestType::DELETE => curl_easy_setopt(curl, CURLOPT_CUSTOMREQUEST, c"DELETE".as_ptr()),
        } };

        // Body
        if client.data.body.is_empty() {
            unsafe { curl_easy_setopt(curl, CURLOPT_POSTFIELDSIZE, 0 as std::ffi::c_long) };
        } else {
            unsafe { curl_easy_setopt(curl, CURLOPT_POSTFIELDS, cstring.new_string(&client.data.body)) };
            unsafe { curl_easy_setopt(curl, CURLOPT_POSTFIELDSIZE, client.data.body.len() as std::ffi::c_long) };
        }

        // Write to body
        let response_body = WriteData{data: vec![]};
        let response_body_ptr = response_body.into_void();
        unsafe { curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, write_callback as WriteCallback) };
        unsafe { curl_easy_setopt(curl, CURLOPT_WRITEDATA, response_body_ptr) };

        // Exec
        let res = unsafe { curl_easy_perform(curl) };

        // Fail safe
        if res as i32 != CURLcode::CURLE_OK as i32 {
            return Err(format!("curl error: {}", borrow_string!(unsafe{curl_easy_strerror(res)})));
        }

        // Status code
        let mut status_code = 0;
        unsafe{ curl_easy_getinfo(curl, CURLINFO_RESPONSE_CODE, &mut status_code as *mut i32) };

        let mut cr = ClientResponse::new();
        // Remember to free the response body.
        // This will drop naturally now.
        let response_body = unsafe { WriteData::from_raw_void(response_body_ptr) };

        cr.fill(&client.data);

        // Convert back into string.
        match String::from_utf8(response_body.data) {
            Ok(val) => cr.data.body = val,
            Err(err) => {
                return Err(err.to_string());
            },
        }

        cr.status = status_code;

        Ok(cr)
    }

    fn free(v:crate::shared::pxs_Opaque) {
        if v.is_null() {
            return;
        }

        let _ = unsafe{ CurlWrapper::from_raw_void(v) };
    }
}
