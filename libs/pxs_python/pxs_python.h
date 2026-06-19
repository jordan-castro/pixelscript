#ifndef PXS_PYTHON_H
#define PXS_PYTHON_H

const int PXSPYTHON_IS_DIR = -2;
const int PXSPYTHON_NOT_FOUND = -1;

// Defined in pixelscript:rust code
int pxspython_importfile(char** buffer, const char* file_path);
// Override for the pocketpy.callbacks.import function.
char* pxspython_import(const char* path, int* size);

#endif // PXS_PYTHON_H