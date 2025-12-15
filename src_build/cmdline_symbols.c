// These symbols are normally defined in the Nim-generated main function,
// but when building as a library, they need to be defined explicitly.

#include <stddef.h>

#ifdef __ANDROID__
__attribute__((weak)) int cmdCount = 0;
__attribute__((weak)) char** cmdLine = NULL;
#else
int cmdCount = 0;
char** cmdLine = NULL;
#endif