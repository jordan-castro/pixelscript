#include <string>
#include <vector>

namespace utils {
    // Join a vector<string> into a string with a delimiter
    inline std::string join(const std::string& dil, const std::vector<std::string>& v) {
        std::string res;
        for (size_t i = 0; i < v.size(); i++) {
            res += v[i];
            if (i < v.size() - 1) {
                res += dil;
            }
        }
        return res;
    }

    // Trim whitespace from left
    inline std::string trim_left(const std::string& str) {
        std::string res = str;
        while (!res.empty() && res[0] == ' ') {
            res.erase(0, 1);
        }
        return res;
    }
}