#pragma once

#ifdef YOYO_NET
#include <pixelscript.h>
#include <map>
#include <string>
#include <cstdint>
#include <vector>

namespace yoyo::net {
    // 
    enum class RequestType : uint8_t {
        GET,
        POST,
        PUT,
        DELETE,
        PATCH,
    };

    // 
    enum class HttpVersion : uint8_t {
        HTTP_1_1,
        HTTP_2,
        HTTP_3
    };

    // @private
    /// Response data.
    struct ResponseData {
        // @private
        // Headers sent in a request.
        std::map<std::string, std::string> headers;
        // @private
        // the request body
        std::string body;
        // @private
        // The request type (GET,POST,etc)
        RequestType request_type;
        // @private
        // The HTTP version to use (if supported.)
        HttpVersion version;
        // @private
        // Timeout in milliseconds
        int timeout;
        // @private
        // The user agent. This can only be set once per client.
        std::string user_agent;
        // @private
        // The domain name.
        std::string domain_name;
    };

    class ClientResponse {
    public:
        // @private
        ResponseData data;

        // @private
        int status;

        // @private
        // Convert into its pxs type
        pxs_VarT into_pxs();

        // @self
        // @prop(get)
        // The HTTP version.
        // 
        // returns `int`
        static pxs_VarT prop_version(pxs_VarT args);

        // @self
        // @prop(get)
        // The response status.
        //
        // returns `int`
        static pxs_VarT prop_status(pxs_VarT args);

        // @self
        // @prop(get)
        // The response bytes.
        //
        // returns `[]uint`
        static pxs_VarT prop_bytes(pxs_VarT args);

        // @self
        // @prop(get)
        // The response text.
        // 
        // returns `string`
        static pxs_VarT prop_text(pxs_VarT args);
    };

    // A class that  `ClientResponse`.
    class Client {
        // @private
        // The internal type/value
        void* internal;
        
        // @private
        // Use HTTPS. Defaults to true.
        bool use_https = true;
        
        // @private
        // Get headers as std::vector<std::string> or parts (key:value).
        std::vector<std::string> get_header_parts();

    public:
        Client() {}
        ~Client();
        // @private
        // The data to create a response with.
        ResponseData data;
        // @private
        // Setup the native connection
        void setup();

        // @private
        // Return a non pxs_native `ClientResponse`.
        // It can be converted to a pxs_native object via `to_pxs`.
        ClientResponse* create_request(const std::string& path, const RequestType& rt);

        // @name(Client)
        // Create a new `Client`
        //
        // returns `Client
        static pxs_VarT new_client(pxs_VarT args);

        // @self
        // @prop(get,set)
        // The headers.
        // args:
        //  - headers: @set `[][]string` the headers to set.
        //
        // returns `[][]string`|`null`
        static pxs_VarT prop_headers(pxs_VarT args);
        
        // @self
        // Get a single header.
        // args:
        //  - key: `string` the header key.
        //
        // returns `string` value if found.
        static pxs_VarT get_header(pxs_VarT args);

        // @self
        // Set a single header.
        // args:
        //  - key: `string` header key.
        //  - value: `string` header value.
        static pxs_VarT set_header(pxs_VarT args);

        // @self
        // @prop(get,set)
        // 
        // args:
        //  - body: @set `string`|`[]uint` body as string or bytes.
        //
        // returns `string`|`null`
        static pxs_VarT prop_body(pxs_VarT args);

        // @self
        // @prop(get,set)
        // Version
        // args:
        //  - version: @set `int` the http version to use.
        //
        // returns `int`|`null`
        static pxs_VarT prop_version(pxs_VarT args);

        // @self
        // @prop(get,set)
        // Domain name
        // args:
        //  - dn: @set `string` the domain name.
        //
        // returns `string`|`null`
        static pxs_VarT prop_domain(pxs_VarT args);

        // @except
        // @self
        // Make a request
        // args:
        //  - url: `string` the url to make the request to.
        //  - rt: `RequestType` the request type to send.
        //
        // returns `string`
        static pxs_VarT make_request(pxs_VarT args);
    };

    // @except
    // Make a HTTP Get request.
    // args:
    //  - url: `string` the url to request to.
    //  - headers: @opt `[][]string` the headers to apply.
    //  - version: @opt `int` the HTTP version to use.
    //
    // returns `ClientResponse`
    pxs_VarT get(pxs_VarT args);

    // @except
    // Make a HTTP Post request.
    // args:
    //  - url: `string` the url to request to.
    //  - body: `string` the body to send.
    //  - headers: @opt `[][]string` the headers to apply.
    //  - version: @opt `int` the HTTP version to use.
    //
    // returns `ClientResponse`
    pxs_VarT post(pxs_VarT args);

    void init(pxs_Module* yoyo_mod);
};

#endif // YOYO_NET