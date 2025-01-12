#include "mystring.h"

// I couldn't make the libc work with Rust and all... so I just remade those 4 functions that storage.c uses. It is definitely not a good solution, and I may look at it again later.

// from github.com/numworks/epsilon/liba/src/memmove.c
int strcmp(const char *s1, const char *s2)
{
    while (*s1 != NULL && *s1 == *s2)
    {
        s1++;
        s2++;
    }
    return (*(unsigned char *)s1) - (*(unsigned char *)s2);
}

// from github.com/numworks/epsilon/liba/src/memmove.c
void *mymemcpy(void *__restrict__ dest, const void *__restrict__ src, size_t n)
{
    char *destination = (char *)dest;
    char *source = (char *)src;

    while (n--)
    {
        *destination = *source;
        destination++;
        source++;
    }

    return dest;
}

// from github.com/numworks/epsilon/liba/src/memmove.c
void *mymemmove(void *dest, const void *src, size_t n)
{
    char *destination = (char *)dest;
    char *source = (char *)src;

    if (source < destination && destination < source + n)
    {
        /* Copy backwards to avoid overwrites */
        source += n;
        destination += n;
        while (n--)
        {
            *--destination = *--source;
        }
    }
    else
    {
        while (n--)
        {
            *destination++ = *source++;
        }
    }

    return dest;
}

// from github.com/numworks/epsilon/liba/src/memmove.c
void *mymemset(void *s, int c, size_t n)
{
    char *destination = (char *)s;
    while (n--)
    {
        *destination++ = (unsigned char)c;
    }
    return s;
}

// from github.com/numworks/epsilon/liba/src/memmove.c
size_t strlen(const char *s)
{
    const char *str = s;
    while (*str)
        str++;
    return str - s;
}