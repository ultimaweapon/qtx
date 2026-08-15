#pragma once

#include <stddef.h>
#include <stdint.h>

struct Str {
    unsigned char *p;
    size_t l;
};

extern "C" Str qtx_str_from_utf16(const uint16_t *s, size_t l);
