#ifndef MYSTRING_H
#define MYSTRING_H

#include <stddef.h>

#define CPY_DIR_LOWER_TO_HIGHER 0
#define CPY_DIR_HIGHER_TO_LOWER 1

int strcmp(const char *s1, const char *s2);
void *mymemcpy(void *__restrict__ dest, const void *__restrict__ src, size_t n);
void *mymemmove(void *dest, const void *src, size_t n);
void *mymemset(void *s, int c, size_t n);
unsigned int strlen(const char *s);

#endif