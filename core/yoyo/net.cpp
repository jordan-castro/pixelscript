#ifdef YOYO_NET

#include "net.hpp"
#include <pixelscript_cpp.hpp>
#include "types.hpp"
#include "utils/debug.hpp"
#include "utils/bytes.hpp"
#include "utils/strutils.hpp"
#include "utils/pxs.hpp"
#include <vector>
#include <cstdlib>
#include "utils/exceptions.hpp"
#include <stdexcept>
#include <array>

#if defined(_WIN32)
// Windows native IMPL. Uses WinHTTP.
// https://simplifycpp.org/?id=a0912

#include <windows.h>
#include <winhttp.h>
#include <cwchar>
#pragma comment(lib, "winhttp.lib")
#undef DELETE

#elif defined(__APPLE__)
// TODO: APPLE
#elif defined(__ANDROID__)
// TODO: ANDROID
#elif defined(__linux__)
// TODO LINUX
#endif

namespace yoyo::net {
    // ================================= WINDOWS ================================= 
    #if defined(_WIN32)
    // Get last error for WinHttp
    std::string get_error() {
        int error = GetLastError();
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
    }

    // Get request type for windows
    std::wstring get_request_type(RequestType rt) {
        switch (rt) {
            case RequestType::GET:
                return L"GET";
            case RequestType::POST:
                return L"POST";
            case RequestType::PATCH:
                return L"PATCH";
            case RequestType::PUT:
                return L"PUT";
            case RequestType::DELETE:
                return L"DELETE";
        }
    }

    // RAII wrapper for HINTERNET handles.
    struct HInternetWrapper {
        HINTERNET handle;

        HInternetWrapper(HINTERNET o) : handle(o) {}
        ~HInternetWrapper() {
            if (handle) {
                WinHttpCloseHandle(handle);
            }
        }
    };

    void Client::setup() {
        // Already setup.
        if (this->internal) {
            return;
        }

        auto user_agent = this->data.user_agent;
        if (user_agent.empty()) {
            user_agent = "yoyo_rt";
        }

        std::wstring wuser_agent = yoyo::utils::str::to_wstring(user_agent);

        HINTERNET h_session = WinHttpOpen(
            wuser_agent.c_str(),
            WINHTTP_ACCESS_TYPE_NO_PROXY,
            WINHTTP_NO_PROXY_NAME,
            WINHTTP_NO_PROXY_BYPASS,
            0
        );

        if (!h_session) {
            throw std::runtime_error(get_error());
        }

        // Wrap it and save it.
        auto wrapper = new HInternetWrapper(h_session);

        this->internal = static_cast<void*>(wrapper);
    }
    ClientResponse* Client::create_request(const std::string& path, const RequestType& rt) {
        // convert back to session
        if (this->internal == nullptr) {
            throw std::runtime_error("Client.win32.internal is null.");
        }
        auto wrapper = static_cast<HInternetWrapper*>(this->internal);
        auto wdomain_name = yoyo::utils::str::to_wstring(this->data.domain_name);

        // Set timeouts
        WinHttpSetTimeouts(
            wrapper->handle,
            this->data.timeout,
            this->data.timeout,
            this->data.timeout,
            this->data.timeout
        );

        // Get the port, HTTP/S.
        int default_port;
        if (this->use_https) {
            default_port = INTERNET_DEFAULT_HTTPS_PORT;
        } else {
            default_port = INTERNET_DEFAULT_HTTP_PORT;
        }

        // Connect to the server yo!
        HINTERNET h_connect = WinHttpConnect(
            wrapper->handle,
            wdomain_name.c_str(),
            default_port,
            0
        );

        if (!h_connect) {
            // Get error
            throw std::runtime_error(get_error());
        }

        // We are now connected to a server. Lets wrap it
        auto connect_wrapper = HInternetWrapper(h_connect);

        // Get the request type
        // Create the request.
        HINTERNET h_request = WinHttpOpenRequest(
            connect_wrapper.handle,
            get_request_type(rt).c_str(),
            yoyo::utils::str::to_wstring(path).c_str(),
            nullptr,
            WINHTTP_NO_REFERER,
            WINHTTP_DEFAULT_ACCEPT_TYPES,
            WINHTTP_FLAG_SECURE
        );

        if (!h_request) {
            throw std::runtime_error(get_error());
        }
        auto request_wrapper = HInternetWrapper(h_request);

        // Get headers
        wchar_t* headers_string = NULL;
        auto header_parts = get_header_parts();
        if (header_parts.size() > 0) {
            // Do stuff
            std::string total = yoyo::utils::str::join("\r\n", header_parts);
            auto wtotal = yoyo::utils::str::to_wstring(total);
            headers_string = wcsdup(wtotal.c_str());
        }

        // The muthafucking body yo!
        LPVOID tha_body = NULL;
        if (this->data.body.size() > 0) {
            tha_body = (LPVOID)this->data.body.data();
        }

        // Send request
        BOOL ok;
        if (rt == RequestType::GET) {
            ok = WinHttpSendRequest(
                request_wrapper.handle,
                headers_string,
                0,
                WINHTTP_NO_REQUEST_DATA,
                0,
                0,
                0
            );
        } else if (rt == RequestType::POST) {
            // Send body
            ok = WinHttpSendRequest(
                request_wrapper.handle,
                headers_string,
                -1,
                tha_body,
                this->data.body.size(),
                this->data.body.size(),
                0
            );
        }
        // todo(jc) Add other methods.

        // Delete it if not null.
        if (headers_string) {
            free(headers_string);
            headers_string = NULL;
        }

        if (!ok) {
            throw std::runtime_error(get_error());
        }
        ok = WinHttpReceiveResponse(request_wrapper.handle, nullptr);
        if (!ok) {
            throw std::runtime_error(get_error());
        }
        // Read response
        std::string response;
        DWORD bytes_avail;
        do {
            if (!WinHttpQueryDataAvailable(request_wrapper.handle, &bytes_avail)) {
                break;
            }

            if (bytes_avail == 0) {
                break;
            }

            std::vector<char> buffer(bytes_avail);
            DWORD bytes_read = 0;

            if (!WinHttpReadData(
                request_wrapper.handle,
                buffer.data(),
                bytes_avail,
                &bytes_read
            )) {
                break;
            }

            response.append(buffer.data(), bytes_read);
        } while (bytes_avail > 0);

        // Get status code.
        DWORD status_code = 0;
        DWORD size = sizeof(status_code);

        WinHttpQueryHeaders(
            request_wrapper.handle,
            WINHTTP_QUERY_STATUS_CODE | WINHTTP_QUERY_FLAG_NUMBER,
            WINHTTP_HEADER_NAME_BY_INDEX,
            &status_code,
            &size,
            WINHTTP_NO_HEADER_INDEX
        );

        // Now lets return the response yo!
        auto cr = new ClientResponse();
        cr->data.headers = this->data.headers;
        cr->data.body = response;
        cr->data.domain_name = this->data.domain_name;
        cr->data.version = this->data.version;
        cr->data.user_agent = this->data.user_agent;
        cr->data.timeout = this->data.timeout;
        cr->data.request_type = rt;
        cr->status = status_code;
        
        return cr;
    }
    // ================================= WINDOWS END ================================= 
    #endif 

    void free_client(pxs_Opaque ptr) {
        if (!ptr) {
            return;
        }

        delete static_cast<Client*>(ptr);
    }

    void free_client_response(pxs_Opaque ptr) {
        if (!ptr) {
            return;
        }

        delete static_cast<ClientResponse*>(ptr);
    }

    Client::~Client() {
        if (!this->internal) {
            return;
        }

        #if defined(_WIN32)
            delete static_cast<HInternetWrapper*>(this->internal);
        #endif
    }

    pxs_VarT ClientResponse::into_pxs() {
        auto obj = pxs_newtype(static_cast<pxs_Opaque>(this), free_client_response, "ClientResponse", yoyo::types::NET_ClientResponse);
        pxs_object_addprop(obj, "version", ClientResponse::prop_version);
        pxs_object_addprop(obj, "status", ClientResponse::prop_status);
        pxs_object_addprop(obj, "bytes", ClientResponse::prop_bytes);
        pxs_object_addprop(obj, "text", ClientResponse::prop_text);
        return pxs_newhost(obj);
    }

    pxs_VarT ClientResponse::prop_version(pxs_VarT args) {
        auto self = yoyo::utils::pxs::get_type<ClientResponse>(args, 0, yoyo::types::NET_ClientResponse);
        if (!self) {
            return yoyo::utils::exceptions::expected_self(pxs_arg(args, 0));
        }

        return pxs_newint(static_cast<int>(self->data.version));
    }

    pxs_VarT ClientResponse::prop_status(pxs_VarT args) {
        auto self = yoyo::utils::pxs::get_type<ClientResponse>(args, 0, yoyo::types::NET_ClientResponse);
        if (!self) {
            return yoyo::utils::exceptions::expected_self(pxs_arg(args, 0));
        }

        return pxs_newint(self->status);
    }

    pxs_VarT ClientResponse::prop_bytes(pxs_VarT args) {
        auto self = yoyo::utils::pxs::get_type<ClientResponse>(args, 0, yoyo::types::NET_ClientResponse);
        if (!self) {
            return yoyo::utils::exceptions::expected_self(pxs_arg(args, 0));
        }

        // Convert response into bytes
        auto response = self->data.body;
        return yoyo::utils::bytes::make_byte_list(response.data(), response.size());
    }

    pxs_VarT ClientResponse::prop_text(pxs_VarT args) {
        auto self = yoyo::utils::pxs::get_type<ClientResponse>(args, 0, yoyo::types::NET_ClientResponse);
        if (!self) {
            return yoyo::utils::exceptions::expected_self(pxs_arg(args, 0));
        }

        return pxs_newstring(self->data.body.c_str());
    }

    pxs_VarT Client::new_client(pxs_VarT args) {
        // Create a new client
        auto client = new Client();
        auto object = pxs_newtype(static_cast<pxs_Opaque>(client), free_client, "Client", yoyo::types::NET_Client);
        pxs_object_addprop(object, "headers", Client::prop_headers);
        pxs_object_addfunc(object, "get_header", Client::get_header);
        pxs_object_addfunc(object, "set_header", Client::set_header);
        pxs_object_addprop(object, "body", Client::prop_body);
        pxs_object_addprop(object, "version", Client::prop_version);
        pxs_object_addprop(object, "domain", Client::prop_domain);
        pxs_object_addfunc(object, "make_request", Client::make_request);
        return pxs_newhost(object);
    }

    // Get the headers from a pxs_VarT
    std::map<std::string, std::string> get_headers(pxs_VarT rt, pxs_VarT arg) {
        std::map<std::string, std::string> result;
        // loop through values
        for (int i = 0; i < pxs_listlen(arg); i++) {
            // Get string values if strings
            auto item = pxs_listget(arg, i);
            if (!pxs_varis(item, pxs_List)) {
                continue;
            }

            // Get key, value
            auto key_arg = pxs_smart_getstring(rt, pxs_listget(item, 0));
            if (!key_arg) {
                continue;
            }
            std::string key(key_arg);
            pxs_freestr(key_arg);

            auto value_arg = pxs_smart_getstring(rt, pxs_listget(item, 1));
            if (!value_arg) {
                continue;
            }
            std::string value(value_arg);
            pxs_freestr(value_arg);

            result[key] = value;
        }

        return result;
    }

    pxs_VarT Client::prop_headers(pxs_VarT args) {
        auto self = yoyo::utils::pxs::get_type<Client>(args, 0, yoyo::types::NET_Client);
        if (!self) {
            return yoyo::utils::exceptions::expected_self(pxs_arg(args, 0));
        }
        auto argc = pxs_argc(args);
        if (argc == 1) {
            // Return headers
            auto headers = self->data.headers;
            auto result = pxs_newlist();

            for (const auto& [key, value] : headers) {
                auto it = pxs_newlist();
                pxs_listadd(it, pxs_newstring(key.c_str()));
                pxs_listadd(it, pxs_newstring(value.c_str()));
                pxs_listadd(result, it);
            }
            return result;
        } else if (argc == 2) {
            // Set headers
            auto headers = pxs_arg(args, 1);
            self->data.headers = get_headers(pxs_getrt(args), headers);
        }

        return pxs_newnull();
    }

    pxs_VarT Client::get_header(pxs_VarT args) {
        auto self = yoyo::utils::pxs::get_type<Client>(args, 0, yoyo::types::NET_Client);
        if (!self) {
            return yoyo::utils::exceptions::expected_self(pxs_arg(args, 0));
        }

        auto key_arg = pxs_arg(args, 1);
        std::string key(pxs_varsize(key_arg) / sizeof(char), '\0');
        pxs_smart_copystring(pxs_getrt(args), key_arg, key.data());

        if (self->data.headers.find(key) != self->data.headers.end()) {
            return pxs_newstring(self->data.headers.at(key).c_str());
        }
        return pxs_newnull();
    }

    pxs_VarT Client::set_header(pxs_VarT args) {
        auto self = yoyo::utils::pxs::get_type<Client>(args, 0, yoyo::types::NET_Client);
        if (!self) {
            return yoyo::utils::exceptions::expected_self(pxs_arg(args, 0));
        }

        auto rt = pxs_getrt(args);

        auto key_arg = pxs_arg(args, 1);
        std::string key(pxs_varsize(key_arg) / sizeof(char), '\0');
        pxs_smart_copystring(rt, key_arg, key.data());
        
        auto value_arg = pxs_arg(args, 2);
        std::string value(pxs_varsize(value_arg) / sizeof(char), '\0');
        pxs_smart_copystring(rt, value_arg, value.data());

        self->data.headers[key] = value;

        return pxs_newnull();
    }

    pxs_VarT Client::prop_body(pxs_VarT args) {
        auto self = yoyo::utils::pxs::get_type<Client>(args, 0, yoyo::types::NET_Client);
        if (!self) {
            return yoyo::utils::exceptions::expected_self(pxs_arg(args, 0));
        }

        // Check if get
        auto argc = pxs_argc(args);

        // This is a GET only.
        if (argc == 1) {
            return pxs_newstring(self->data.body.c_str());
        }

        if (argc == 2) {
            // This has value
            auto value_arg = pxs_arg(args, 1);
            std::string value(pxs_varsize(value_arg) / sizeof(char), '\0');
            pxs_smart_copystring(pxs_getrt(args), value_arg, value.data());
            self->data.body = value;
        } 
        return pxs_newnull();
    }

    pxs_VarT Client::prop_version(pxs_VarT args) {
        auto self = yoyo::utils::pxs::get_type<Client>(args, 0, yoyo::types::NET_Client);
        if (!self) {
            return yoyo::utils::exceptions::expected_self(pxs_arg(args, 0));
        }

        // Check argc
        auto argc = pxs_argc(args);
        if (argc == 1) {
            return pxs_newint(static_cast<int>(self->data.version));
        }
        
        if (argc == 2) {
            auto v = pxs_getint(pxs_arg(args, 1));
            if (v > -1) {
                self->data.version = static_cast<HttpVersion>(v);
            }
        }

        return pxs_newnull();
    }

    pxs_VarT Client::prop_domain(pxs_VarT args) {
        auto self = yoyo::utils::pxs::get_type<Client>(args, 0, yoyo::types::NET_Client);
        if (!self) {
            return yoyo::utils::exceptions::expected_self(pxs_arg(args, 0));
        }

        auto argc = pxs_argc(args);
        if (argc == 1) {
            return pxs_newstring(self->data.domain_name.c_str());
        }

        if (argc == 2) {
            auto domain_var = pxs_arg(args, 1);
            std::string domain(pxs_varsize(domain_var) / sizeof(char), '\0');
            pxs_smart_copystring(pxs_getrt(args), domain_var, domain.data());

            self->data.domain_name = domain;
        }

        return pxs_newnull();
    }

    pxs_VarT Client::make_request(pxs_VarT args) {
        auto self = yoyo::utils::pxs::get_type<Client>(args, 0, yoyo::types::NET_Client);
        if (!self) {
            return yoyo::utils::exceptions::expected_self(pxs_arg(args, 0));
        }
        // If not already!
        self->setup();

        // Get the URL
        auto url_arg = pxs_arg(args, 1);
        if (!pxs_varis(url_arg, pxs_String)) {
            return pxs_newexception("Expected URL to be string.");
        }
        std::string url(pxs_varsize(url_arg) / sizeof(char), '\0');
        pxs_copybytes(url_arg, static_cast<pxs_Opaque>(url.data()));

        // Get the request type
        auto request_t = static_cast<RequestType>(pxs_getint(pxs_arg(args, 2)));

        auto client_response = self->create_request(url, request_t);
        if (!client_response) {
            return pxs_newnull();
        }

        return client_response->into_pxs();
    }

    // Get domain name and path from a pxs_VarT url
    std::array<std::string, 2> get_domain_and_path(pxs_VarT rt, pxs_VarT url) {
        std::array<std::string, 2> result({"", ""});
        // Split by calling script code
        auto arena = pxs_newarena();
        auto rt = pxs_arenaput(arena, pxs_newint(1)); 
        auto hpargs = pxs_arenaput(arena, pxs_newlist());
        pxs_listadd(hpargs, pxs_newcopy(url));
        auto request_paths = pxs_arenaput(arena, pxs_call(rt, "yoyo_net_get_host_and_path", hpargs));
        // Result will be [string, string]
        auto domain_c = pxs_arena_putstr(arena, pxs_smart_getstring(rt, pxs_listget(request_paths, 0)));
        auto path_c = pxs_arena_putstr(arena, pxs_smart_getstring(rt, pxs_listget(request_paths, 1)));
        if (domain_c) {
            result[0] = domain_c;
        }
        if (path_c) {
            result[1] = path_c;
        }
        // Free memory.
        pxs_freearena(arena);

        return result;
    }

    pxs_VarT get(pxs_VarT args) {
        // Check URL
        auto argc = pxs_argc(args);
        if (argc == 0) {
            return pxs_newexception("Expected URL");
        }

        auto paths = get_domain_and_path(pxs_getrt(args), pxs_arg(args, 0));

        auto client = Client::new_client(nullptr);
        // pxs_list
        pxs_freevar(pxs_hostcall(client, Client::prop_domain, pxs_argsadd(pxs_argsadd(pxs_newargs(), pxs_newstring("test")), "test 2")));
        // pxs_freevar(pxs::call(Client::prop_domain, {client, paths[0]}));
        // pxs_freevar(pxs::call(Client::prop_headers, {client, pxs_arg(args, 1)}));
        // pxs_freevar(pxs::call(Client::prop_version, {client, pxs_arg(args, 2)}));

        auto result = pxs::call(Client::make_request, {client, paths[1], static_cast<int>(RequestType::GET)});
        pxs_freevar(client);
        
        return result;
    }

    pxs_VarT post(pxs_VarT args) {
        // Check URL
        auto argc = pxs_argc(args);
        if (argc == 0) {
            return pxs_newexception("Expected URL");
        }

        auto paths = get_domain_and_path(pxs_getrt(args), pxs_arg(args, 0));

        auto client = Client::new_client(nullptr);
        pxs_freevar(pxs::call(Client::prop_domain, {client, paths[0]}));
        pxs_freevar(pxs::call(Client::prop_body, {client, pxs_arg(args, 1)}));
        pxs_freevar(pxs::call(Client::prop_headers, {client, pxs_arg(args, 2)}));
        pxs_freevar(pxs::call(Client::prop_version, {client, pxs_arg(args, 3)}));

        auto result = pxs::call(Client::make_request, {client, paths[1], static_cast<int>(RequestType::POST)});
        pxs_freevar(client);
        
        return result;
    }

    void init(pxs_Module* yoyo_mod) {
        auto net_mod = pxs_newmod("net");

        pxs_addvar(net_mod, "HTTP_VERSION_1_1", pxs_newint(static_cast<int>(HttpVersion::HTTP_1_1)));
        pxs_addvar(net_mod, "HTTP_VERSION_2", pxs_newint(static_cast<int>(HttpVersion::HTTP_2)));
        pxs_addvar(net_mod, "HTTP_VERSION_3", pxs_newint(static_cast<int>(HttpVersion::HTTP_3)));

        auto client_mod = pxs_newmod("client");

        pxs_addobject(client_mod, "Client", Client::new_client);
        pxs_addfunc(client_mod, "get", get);
        pxs_addfunc(client_mod, "post", post);

        pxs_add_submod(net_mod, client_mod);
        pxs_add_submod(yoyo_mod, net_mod);
    }
};

#endif // YOYO_NET

