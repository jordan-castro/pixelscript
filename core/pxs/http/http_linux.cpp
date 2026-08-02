// Linux native IMPL. Uses curl

#include "http.hpp"
#include "utils.hpp"
#include <curl/curl.h>
#include <stdexcept>
#include <string>
#include <vector>
#undef DELETE

// RAII wrapper
struct CurlWrapper {
    CURL* handle;
    struct curl_slist *headers_list = NULL;
    
    CurlWrapper(CURL* p) : handle(p) {}
    ~CurlWrapper() {
        if (handle) {
            curl_easy_cleanup(handle);
        }
        if (headers_list) {
            curl_slist_free_all(headers_list);
        }
    }
};

// Callback to write response body chunks into std::string
static size_t WriteCallback(void* contents, size_t size, size_t nmemb, void* userp) {
    size_t total_size = size * nmemb;
    auto* response = static_cast<std::string*>(userp);
    response->append(static_cast<char*>(contents), total_size);
    return total_size;
}

void Client::setup() {
    if (this->internal != nullptr) {
        return;
    }

    curl_global_init(CURL_GLOBAL_DEFAULT);

    auto wrapper = new CurlWrapper(curl_easy_init());
    this->internal = static_cast<void*>(wrapper);
}

ClientResponse* Client::create_request(const std::string& path, const RequestType& rt) {
    if (this->internal == nullptr) {
        throw std::runtime_error("Client.linux.internal is null");
    }

    // Get curl instance.
    auto wrapper = static_cast<CurlWrapper*>(this->internal);
    auto curl = wrapper->handle;
    if (!curl) {
        throw std::runtime_error("Client.linux.curl is null");
    }

    // Reset handle between requests
    curl_easy_reset(curl);

    // Build url
    std::string scheme = this->use_https ? "https://" : "http://";
    std::string full_url = scheme + this->data.domain_name;
    if (!path.empty() && path[0] != '/') {
        full_url += "/";
    }
    full_url += path;

    // Setup user agent.
    std::string user_agent = "yoyo_rt";
    if (!this->data.user_agent.empty()) {
        user_agent = this->data.user_agent;
    }

    curl_easy_setopt(curl, CURLOPT_URL, full_url.c_str());
    curl_easy_setopt(curl, CURLOPT_USERAGENT, user_agent.c_str());
    curl_easy_setopt(curl, CURLOPT_TIMEOUT_MS, this->data.timeout);
    curl_easy_setopt(curl, CURLOPT_CONNECTTIMEOUT_MS, this->data.timeout);

    // Headers
    auto headers = get_header_parts();
    auto headers_list = wrapper->headers_list;
    
    for (const auto& h : headers) {
        headers_list = curl_slist_append(headers_list, h.c_str());
    }
    if (headers_list) {
        curl_easy_setopt(curl, CURLOPT_HEADER, headers_list);
    }

    // Method setup
    switch (rt) {
        case RequestType::GET:
            curl_easy_setopt(curl, CURLOPT_HTTPGET, 1L);
            break;
        case RequestType::POST:
            curl_easy_setopt(curl, CURLOPT_POST, 1L);
            break;
        case RequestType::PATCH:
            curl_easy_setopt(curl, CURLOPT_CUSTOMREQUEST, "PATCH");
            break;
        case RequestType::PUT:
            curl_easy_setopt(curl, CURLOPT_CUSTOMREQUEST, "PUT");
            break;
        case RequestType::DELETE:
            curl_easy_setopt(curl, CURLOPT_CUSTOMREQUEST, "DELETE");
            break;
    }

    // Body
    if (!this->data.body.empty()) {
        curl_easy_setopt(curl, CURLOPT_POSTFIELDS, this->data.body.data());
        curl_easy_setopt(curl, CURLOPT_POSTFIELDSIZE, static_cast<long>(this->data.body.size()));
    } else {
        // If post, set the size to 0 since there is no body.
        if (rt == RequestType::POST) {
            curl_easy_setopt(curl, CURLOPT_POSTFIELDSIZE, 0L);
        }
    }

    // Write to body
    std::string response_body;
    curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, WriteCallback);
    curl_easy_setopt(curl, CURLOPT_WRITEDATA, &response_body);

    // Exec
    auto res = curl_easy_perform(curl);
    if (res != CURLE_OK) {
        throw std::runtime_error(std::string("curl error: ") + curl_easy_strerror(res));
    }

    // Status code
    long status_code = 0;
    curl_easy_getinfo(curl, CURLINFO_RESPONSE_CODE, &status_code);

    auto cr = new ClientResponse();
    cr->fill(this->data);
    cr->data.body = response_body;
    cr->status = status_code;
    
    return cr;
}

Client::~Client() {
    if (!this->internal) {
        return;
    }
    delete static_cast<CurlWrapper*>(this->internal);
}