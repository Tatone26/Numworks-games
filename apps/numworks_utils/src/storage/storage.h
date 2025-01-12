// code from https://framagit.org/Yaya.Cout/numworks-extapp-storage (thanks !)
// copyright (c) 2021-2022 Numworks

#ifndef STORAGE_H
#define STORAGE_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

uint32_t reverse32(uint32_t value);
int extapp_fileList(const char **filename, int maxrecord, const char *extension);
bool extapp_fileExists(const char *filename);
const char *extapp_fileRead(const char *filename, size_t *len);
bool extapp_fileWrite(const char *filename, const char *content, size_t len);
bool extapp_fileErase(const char *filename);
const uint32_t extapp_size();
uint32_t extapp_address();
const uint32_t extapp_used();
const uint32_t *extapp_nextFree();
bool extapp_isValid(const uint32_t *address);
// Return the calculator model : 0 is unknown, 1 is N0110/N0115, 2 is N0120
const uint8_t extapp_calculatorModel();
const uint32_t *extapp_userlandAddress();

#endif
