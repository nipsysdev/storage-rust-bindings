#include <stdbool.h>
#include <stdlib.h>

// Include the generated libcodex header from nimcache
#include "../vendor/nim-codex/nimcache/release/libcodex/libcodex.h"

// Ensure we have the necessary types and constants
#ifndef RET_OK
#define RET_OK 0
#define RET_ERR 1
#define RET_MISSING_CALLBACK 2
#define RET_PROGRESS 3
#endif

// Callback function type (should match the one in libcodex.h)
#ifndef CODEX_CALLBACK
typedef void (*CodexCallback)(int ret, const char* msg, size_t len, void* userData);
#define CODEX_CALLBACK
#endif