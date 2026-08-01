#include <stddef.h>

extern "C" void qtx_delete(void *ptr)
{
    operator delete(ptr);
}

extern "C" {
    size_t qtx_max_align = alignof(max_align_t);
}
