#include "pxs_python.h"
#include "pxs_utils.h"
#include <stdlib.h>
#include <string.h>

char* pxspython_import(const char* file_path, int* data_size) {
    char* buffer = NULL;

    int size = pxspython_importfile(&buffer, file_path);
    if (size == PXSPYTHON_NOT_FOUND) {
        return NULL;
    }

    // Directory
    if (size == PXSPYTHON_IS_DIR) {
        // Allocate a empty string.
        char* res = malloc(1);
        res[0] = '\0';
        if(data_size) *data_size = (int)0;

        return res;
    }

    // Why do I check the buffer AFTER the other checks? Cause the buffer is null when its a dir DUH!
    if (buffer == NULL) {
        return NULL;
    }

    // Reallocate c string (for pocketpy)
    char* result = malloc(sizeof(char) * size);
    if (result == NULL) {
        return NULL;
    }
    if(data_size) *data_size = (int)size;

    // Copies memory from rust string.
    memcpy(result, buffer, size);

    // Now free buffer
    pxsutils_freestring(buffer);

    // Now it can be freed via `free`.
    return result;
}
