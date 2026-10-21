#define _GNU_SOURCE
#include <stdio.h>
#include <stdlib.h>
#include <stdarg.h>
#include <string.h>
#include <stddef.h>
#include <wchar.h>

/*
 * GLIBC 2.38 / ISO C23 and GCC 14 compatibility stubs for Debian Bookworm (GLIBC 2.36 / GCC 12).
 *
 * Precompiled static libraries (such as maplibre_native) built on Ubuntu 24.04 (GLIBC 2.39, GCC 14)
 * reference __isoc23_* symbols and GCC 14 libstdc++ _M_replace_cold.
 * When linking on Debian Bookworm (Raspberry Pi OS Bookworm target), these stubs resolve those symbols
 * using the standard GLIBC 2.36 implementations.
 */

#ifdef __cplusplus
extern "C" {
#endif

long __isoc23_strtol(const char *nptr, char **endptr, int base) {
    return strtol(nptr, endptr, base);
}

unsigned long __isoc23_strtoul(const char *nptr, char **endptr, int base) {
    return strtoul(nptr, endptr, base);
}

long long __isoc23_strtoll(const char *nptr, char **endptr, int base) {
    return strtoll(nptr, endptr, base);
}

unsigned long long __isoc23_strtoull(const char *nptr, char **endptr, int base) {
    return strtoull(nptr, endptr, base);
}

float __isoc23_strtof(const char *nptr, char **endptr) {
    return strtof(nptr, endptr);
}

double __isoc23_strtod(const char *nptr, char **endptr) {
    return strtod(nptr, endptr);
}

long double __isoc23_strtold(const char *nptr, char **endptr) {
    return strtold(nptr, endptr);
}

int __isoc23_sscanf(const char *s, const char *format, ...) {
    va_list args;
    va_start(args, format);
    int ret = vsscanf(s, format, args);
    va_end(args);
    return ret;
}

int __isoc23_vsscanf(const char *s, const char *format, va_list arg) {
    return vsscanf(s, format, arg);
}

int __isoc23_fscanf(FILE *stream, const char *format, ...) {
    va_list args;
    va_start(args, format);
    int ret = vfscanf(stream, format, args);
    va_end(args);
    return ret;
}

int __isoc23_vfscanf(FILE *stream, const char *format, va_list arg) {
    return vfscanf(stream, format, arg);
}

int __isoc23_scanf(const char *format, ...) {
    va_list args;
    va_start(args, format);
    int ret = vscanf(format, args);
    va_end(args);
    return ret;
}

int __isoc23_vscanf(const char *format, va_list arg) {
    return vscanf(format, arg);
}

/*
 * GCC 14 libstdc++ compatibility stubs for Debian Bookworm (GCC 12)
 *
 * Mangled: _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE15_M_replace_coldEPcmPKcmm
 * Demangled: std::__cxx11::basic_string<char, std::char_traits<char>, std::allocator<char> >::_M_replace_cold(char*, unsigned long, char const*, unsigned long, unsigned long)
 */
void _ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE15_M_replace_coldEPcmPKcmm(
    void* __this, char* __p, size_t __len1, const char* __s, size_t __len2, size_t __how_much)
{
    (void)__this;
    if (__len2 && __len2 <= __len1)
        memmove(__p, __s, __len2);
    if (__how_much && __len1 != __len2)
        memmove(__p + __len2, __p + __len1, __how_much);
    if (__len2 > __len1)
        memmove(__p, __s, __len2);
}

/*
 * Mangled: _ZNSt7__cxx1112basic_stringIwSt11char_traitsIwESaIwEE15_M_replace_coldEPwmPKwmm
 * Demangled: std::__cxx11::basic_string<wchar_t, std::char_traits<wchar_t>, std::allocator<wchar_t> >::_M_replace_cold(wchar_t*, unsigned long, wchar_t const*, unsigned long, unsigned long)
 */
void _ZNSt7__cxx1112basic_stringIwSt11char_traitsIwESaIwEE15_M_replace_coldEPwmPKwmm(
    void* __this, wchar_t* __p, size_t __len1, const wchar_t* __s, size_t __len2, size_t __how_much)
{
    (void)__this;
    if (__len2 && __len2 <= __len1)
        wmemmove(__p, __s, __len2);
    if (__how_much && __len1 != __len2)
        wmemmove(__p + __len2, __p + __len1, __how_much);
    if (__len2 > __len1)
        wmemmove(__p, __s, __len2);
}

#ifdef __cplusplus
}
#endif
