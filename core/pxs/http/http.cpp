#include "http.hpp"
#include <array>
#include <sstream>
#include <pixelscript_cpp.hpp>
#include <iostream>

const int CLIENT_RESPONSE_TYPE = 6;
const int CLIENT_TYPE = 7;

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

std::vector<std::string> Client::get_header_parts() {
    std::vector<std::string> res;
    for (const auto& [key, value] : this->data.headers) {
        auto full_str = key + ":\t" + value;
        res.push_back(full_str);
    }
    return res;
}

void ClientResponse::fill(const ResponseData& other) {
    this->data.headers = other.headers;
    this->data.request_type = other.request_type;
    this->data.version = other.version;
    this->data.timeout = other.timeout;
    this->data.user_agent = other.user_agent;
    this->data.domain_name = other.domain_name;
}

pxs_VarT ClientResponse::into_pxs() {
    auto obj = pxs_newtype(static_cast<pxs_Opaque>(this), free_client_response, "ClientResponse", CLIENT_RESPONSE_TYPE);
    pxs_object_addprop(obj, "version", ClientResponse::prop_version);
    pxs_object_addprop(obj, "status", ClientResponse::prop_status);
    pxs_object_addprop(obj, "bytes", ClientResponse::prop_bytes);
    pxs_object_addprop(obj, "text", ClientResponse::prop_text);
    return pxs_newhost(obj);
}

pxs_VarT ClientResponse::prop_version(pxs_VarT args) {
    auto self = static_cast<ClientResponse*>(pxs_gettype(pxs_getrt(args), pxs_arg(args, 0), CLIENT_RESPONSE_TYPE));
    if (!self) {
        return pxs_newexception("Expected self");
    }

    return pxs_newint(static_cast<int>(self->data.version));
}

pxs_VarT ClientResponse::prop_status(pxs_VarT args) {
    auto self = static_cast<ClientResponse*>(pxs_gettype(pxs_getrt(args), pxs_arg(args, 0), CLIENT_RESPONSE_TYPE));
    if (!self) {
        return pxs_newexception("Expected self");
    }

    return pxs_newint(self->status);
}

pxs_VarT ClientResponse::prop_bytes(pxs_VarT args) {
    auto self = static_cast<ClientResponse*>(pxs_gettype(pxs_getrt(args), pxs_arg(args, 0), CLIENT_RESPONSE_TYPE));
    if (!self) {
        return pxs_newexception("Expected self");
    }

    // Convert response into bytes
    auto response = self->data.body;
    return pxs_newbytes(static_cast<pxs_Opaque>(response.data()), sizeof(char), response.size());
}

pxs_VarT ClientResponse::prop_text(pxs_VarT args) {
    auto self = static_cast<ClientResponse*>(pxs_gettype(pxs_getrt(args), pxs_arg(args, 0), CLIENT_RESPONSE_TYPE));
    if (!self) {
        return pxs_newexception("Expected self");
    }

    return pxs_newstring(self->data.body.c_str());
}

pxs_VarT Client::new_client(pxs_VarT args) {
    // Create a new client
    auto client = new Client();
    auto object = pxs_newtype(static_cast<pxs_Opaque>(client), free_client, "Client", CLIENT_TYPE);
    pxs_object_addprop(object, "headers", &Client::prop_headers);
    pxs_object_addfunc(object, "get_header", &Client::get_header);
    pxs_object_addfunc(object, "set_header", &Client::set_header);
    pxs_object_addprop(object, "body", &Client::prop_body);
    pxs_object_addprop(object, "version", &Client::prop_version);
    pxs_object_addprop(object, "domain", &Client::prop_domain);
    pxs_object_addprop(object, "timeout", &Client::prop_timeout);
    pxs_object_addfunc(object, "make_request", &Client::make_request);
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
    auto self = static_cast<Client*>(pxs_gettype(pxs_getrt(args), pxs_arg(args, 0), CLIENT_TYPE));
    if (!self) {
        return pxs_newexception("Expected self");
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
    auto self = static_cast<Client*>(pxs_gettype(pxs_getrt(args), pxs_arg(args, 0), CLIENT_TYPE));
    if (!self) {
        return pxs_newexception("Expected self");
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
    auto self = static_cast<Client*>(pxs_gettype(pxs_getrt(args), pxs_arg(args, 0), CLIENT_TYPE));
    if (!self) {
        return pxs_newexception("Expected self");
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
    auto self = static_cast<Client*>(pxs_gettype(pxs_getrt(args), pxs_arg(args, 0), CLIENT_TYPE));
    if (!self) {
        return pxs_newexception("Expected self");
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
    auto self = static_cast<Client*>(pxs_gettype(pxs_getrt(args), pxs_arg(args, 0), CLIENT_TYPE));
    if (!self) {
        return pxs_newexception("Expected self");
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
    auto self = static_cast<Client*>(pxs_gettype(pxs_getrt(args), pxs_arg(args, 0), CLIENT_TYPE));
    if (!self) {
        return pxs_newexception("Expected self");
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

pxs_VarT Client::prop_timeout(pxs_VarT args) {
    auto self = static_cast<Client*>(pxs_gettype(pxs_getrt(args), pxs_arg(args, 0), CLIENT_TYPE));
    if (!self) {
        return pxs_newexception("Expected self");
    }

    auto argc = pxs_argc(args);
    if (argc == 1) {
        return pxs_newint(self->data.timeout);
    }

    if (argc == 2) {
        auto timeout_var = pxs_arg(args, 1);
        if (!pxs_isint(timeout_var) && !pxs_isfloat(timeout_var))  {
            return pxs_newexception("Expected int or float.");
        }
        int val = pxs_getint(timeout_var);
        self->data.timeout = val;
    }

    return pxs_newnull();
}

pxs_VarT Client::make_request(pxs_VarT args) {
    auto self = static_cast<Client*>(pxs_gettype(pxs_getrt(args), pxs_arg(args, 0), CLIENT_TYPE));
    if (!self) {
        return pxs_newexception("Expected self");
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

    ClientResponse* client_response = nullptr;    
    try {
        client_response = self->create_request(url, request_t);
    } catch(const std::exception& e) {
        return pxs_newexception(e.what());
    }
    if (!client_response) {
        return pxs_newnull();
    }

    return client_response->into_pxs();
}

// Get domain name and path from a pxs_VarT url
std::array<std::string, 2> get_domain_and_path(pxs_VarT url) {
    std::array<std::string, 2> result({"", ""});

    auto cstr = pxs_getstring(url);
    if (!cstr) {
        return result;
    }

    std::stringstream Url(cstr);
    std::vector<std::string> paths;
    std::string token;
    pxs_freestr(cstr);

    while (std::getline(Url, token, '/')) {
        paths.push_back(token);
    }

    if (paths.size() >= 3) {
        result[0] = paths[2];
        
        if (paths.size() > 3) {
            std::string path;
            for (int i = 3; i < paths.size(); i++) {
                path += paths[i];
                if (i < paths.size() - 1) {
                    path += "/";
                }
            }
            result[1] = path;
        }
    }

    return result;
}

// @except
// Make a HTTP Get request.
// args:
//  - url: `string` the url to request to.
//  - headers: @opt `[][]string` the headers to apply.
//  - version: @opt `int` the HTTP version to use.
//
// returns `ClientResponse`
pxs_VarT get(pxs_VarT args) {
    // Check URL
    auto argc = pxs_argc(args);
    if (argc == 0) {
        return pxs_newexception("Expected URL");
    }

    auto paths = get_domain_and_path(pxs_arg(args, 0));

    auto client = Client::new_client(nullptr);
    pxs_freevar(pxs::call(Client::prop_domain, {pxs_new_shallowcopy(client), paths[0]}));
    pxs_freevar(pxs::call(Client::prop_headers, {pxs_new_shallowcopy(client), pxs_arg(args, 1)}));
    pxs_freevar(pxs::call(Client::prop_version, {pxs_new_shallowcopy(client), pxs_arg(args, 2)}));

    auto result = pxs::call(Client::make_request, {pxs_new_shallowcopy(client), paths[1], static_cast<int>(RequestType::GET)});
    pxs_freevar(client);
    
    return result;
}

// @except
// Make a HTTP Post request.
// args:
//  - url: `string` the url to request to.
//  - body: `string` the body to send.
//  - headers: @opt `[][]string` the headers to apply.
//  - version: @opt `int` the HTTP version to use.
//
// returns `ClientResponse`
pxs_VarT post(pxs_VarT args) {
    // Check URL
    auto argc = pxs_argc(args);
    if (argc == 0) {
        return pxs_newexception("Expected URL");
    }

    auto paths = get_domain_and_path(pxs_arg(args, 0));

    auto client = Client::new_client(nullptr);
    pxs_freevar(pxs::call(Client::prop_domain, {pxs_new_shallowcopy(client), paths[0]}));
    pxs_freevar(pxs::call(Client::prop_body, {pxs_new_shallowcopy(client), pxs_arg(args, 1)}));
    pxs_freevar(pxs::call(Client::prop_headers, {pxs_new_shallowcopy(client), pxs_arg(args, 2)}));
    pxs_freevar(pxs::call(Client::prop_version, {pxs_new_shallowcopy(client), pxs_arg(args, 3)}));

    auto result = pxs::call(Client::make_request, {pxs_new_shallowcopy(client), paths[1], static_cast<int>(RequestType::POST)});
    pxs_freevar(client);
    
    return result;
}

void pxs_corelib_http_init() {
    auto net_mod = pxs_newmod("pxs_http");

    // Variables
    pxs_addvar(net_mod, "HTTP_VERSION_1_1", pxs_newint(static_cast<int>(HttpVersion::HTTP_1_1)));
    pxs_addvar(net_mod, "HTTP_VERSION_2", pxs_newint(static_cast<int>(HttpVersion::HTTP_2)));
    pxs_addvar(net_mod, "HTTP_VERSION_3", pxs_newint(static_cast<int>(HttpVersion::HTTP_3)));

    // Sub Modules
    auto client_mod = pxs_newmod("client");
        // Functions
        pxs_addfunc(client_mod, "get", get);
        pxs_addfunc(client_mod, "post", post);

        // Objects
        pxs_addobject(client_mod, "Client", Client::new_client);
    pxs_add_submod(net_mod, client_mod);
    
    pxs_addmod(net_mod);
}
