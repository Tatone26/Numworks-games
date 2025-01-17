// https://framagit.org/Yaya.Cout/numworks-extapp-storage (thanks !)
// copyright (c) 2021-2022 Numworks
// Not the most optimised thing but pretty clean and working. Allows writing a lot of differents files to the storage.
// In storage.rs, I made a wrapper to call those functions easily in a concrete environment : storing game data and options.

#include <stdbool.h>
#include "storage.h"
#include <stdint.h>
#include "mystring.h"

// Taken from https://codereview.stackexchange.com/questions/151049/endianness-conversion-in-c/151070#151070
// I could convert the endianness manually, but it's less readable.
uint32_t reverse32(uint32_t value)
{
    return (((value & 0x000000FF) << 24) |
            ((value & 0x0000FF00) << 8) |
            ((value & 0x00FF0000) >> 8) |
            ((value & 0xFF000000) >> 24));
}

bool extapp_fileExists(const char *filename)
{
    uint32_t storageAddress = extapp_address();
    char *offset = (char *)storageAddress;
    const char *endAddress = (char *)extapp_size() + storageAddress;

    if (!extapp_isValid((const uint32_t *)offset))
    {
        // Storage is invalid
        return false;
    }

    offset += 4;
    int currentRecord = 0;

    while (offset < endAddress)
    {
        uint16_t size = *(uint16_t *)offset;
        if (size == 0)
        {
            break;
        }
        char *name = offset + 2;

        if (strcmp(name, filename) == 0)
        {
            // File was found
            return true;
        }

        offset += size;
        currentRecord++;
    }

    return false;
}

char *extapp_fileRead(const char *filename, size_t *len)
{
    uint32_t storageAddress = extapp_address();
    char *offset = (char *)storageAddress;
    const char *endAddress = (char *)extapp_size() + storageAddress;

    if (!extapp_isValid((const uint32_t *)offset))
    {
        // Storage is invalid
        return NULL;
    }

    offset += 4;

    while (offset < endAddress)
    {
        uint16_t size = *(uint16_t *)offset;
        if (size == 0)
        {
            break;
        }
        char *name = offset + 2;

        if (strcmp(name, filename) == 0)
        {
            // filename + \0
            uint16_t nameSize = strlen(name) + 1;
            // Size contains size + filename + real content. Here, we only want the
            // content
            *len = size - 2 - nameSize;
            //     offset + size + filename
            return offset + 2 + nameSize;
        }

        offset += size;
    }

    // File not found
    return NULL;
}

bool extapp_fileWrite(const char *filename, const char *content, size_t len)
{
    // Check if we have enough free space
    const uint32_t *recordStartPointer = extapp_nextFree();
    //                                                          Start Address  + size +     filename     + \0 + content
    const uint32_t *recordEndPointer = (uint32_t *)((char *)recordStartPointer + 2 + strlen(filename) + 1 + len);
    const uint32_t *storageEndPointer = (uint32_t *)(extapp_address() + extapp_size());

    // In case where we have overflown storage, we return an error
    if (storageEndPointer <= recordEndPointer)
    {
        return false;
    }

    char *writableRecordStartPointer = (char *)extapp_nextFree();

    // We have enough storage, so we can write the data
    // Write size :
    // size + filename + \0 + content
    const uint16_t totalSize = 2 + strlen(filename) + 1 + len;

    *(uint16_t *)writableRecordStartPointer = totalSize;

    // Write filename:
    mymemcpy(writableRecordStartPointer + 2, filename, strlen(filename) + 1); // COUPABLE DU CRASH !!!! ouuuiiiin (mon avis : résoudre ce problème = résoudre le problème de erase aussi)

    // Write content:
    mymemcpy(writableRecordStartPointer + 2 + strlen(filename) + 1, content, len);

    // The record is now written, so we can return
    return true;
}

bool extapp_fileErase(const char *filename)
{
    uint32_t storageAddress = extapp_address();
    char *offset = (char *)storageAddress;
    const char *endAddress = (char *)extapp_size() + storageAddress;

    if (!extapp_isValid((const uint32_t *)offset))
    {
        // Storage is invalid
        return false;
    }

    offset += 4;

    // Locate the record address
    char *recordAddress = NULL;
    uint16_t file_size;
    while (offset < endAddress)
    {
        uint16_t size = *(uint16_t *)offset;
        if (size == 0)
        {
            file_size = 0;
            break;
        }
        char *name = offset + 2;

        if (strcmp(name, filename) == 0)
        {
            recordAddress = offset;
            file_size = size;
            break;
        }

        offset += size;
    }

    // File not found
    if (recordAddress == NULL)
    {
        return false;
    }

    // Move the rest of the data
    // Why not len + 2 ?
    mymemmove(offset, offset + file_size, (char *)extapp_nextFree() - offset);

    // Overwrite the rest of the storage with zeroes
    // the + 1 in len + 1 is for the uint16_t used for file size
    mymemset(offset + *(uint16_t *)offset + 1, 0, file_size + 1);

    return true;
}

uint32_t extapp_address()
{
    return *(uint32_t *)((*extapp_userlandAddress()) + 0xC);
}

uint32_t extapp_size()
{
    return *(uint32_t *)((*extapp_userlandAddress()) + 0x10);
}

uint32_t *extapp_nextFree()
{
    uint32_t storageAddress = extapp_address();
    char *offset = (char *)storageAddress;
    const char *endAddress = (char *)extapp_size() + storageAddress;

    if (!extapp_isValid((const uint32_t *)offset))
    {
        // Storage is invalid
        return NULL;
    }

    offset += 4;

    while (offset < endAddress)
    {
        uint16_t size = *(uint16_t *)offset;
        if (size == 0)
        {
            // Here, we are at the place where new records should start
            return (uint32_t *)offset;
        }

        offset += size;
    }

    // If we exited the loop, it mean that we have gone out of the storage
    return NULL;
    // return (uint32_t *)storageAddress + extapp_size();
}

uint32_t extapp_used()
{
    return (uint32_t)extapp_nextFree() - extapp_address();
}

bool extapp_isValid(const uint32_t *address)
{
    return *address == reverse32(0xBADD0BEE);
}

uint8_t extapp_calculatorModel()
{
    // To guess the storage size without reading forbidden addresses, we try to
    // get the storage address from the userland header

    uint32_t *userlandMagicSlotAN0110 = *(uint32_t **)0x90010000;
    uint32_t *userlandMagicSlotBN0110 = *(uint32_t **)0x90410000;
    uint32_t *userlandMagicSlotAN0120 = *(uint32_t **)0x90020000;
    uint32_t *userlandMagicSlotBN0120 = *(uint32_t **)0x90420000;

    // On N0110, RAM start is at 0x20000000 and end is 0x20040000
    // On N0120, RAM start is at 0x20040000
    bool userlandMagicSlotAN0110IsValid = reverse32(0xfeedc0de) == (uint32_t)userlandMagicSlotAN0110;
    bool userlandMagicSlotBN0110IsValid = reverse32(0xfeedc0de) == (uint32_t)userlandMagicSlotBN0110;
    // TODO: Check the end address on N0120 (should be working, but good to check anyway)
    bool userlandMagicSlotAN0120IsValid = reverse32(0xfeedc0de) == (uint32_t)userlandMagicSlotAN0120;
    bool userlandMagicSlotBN0120IsValid = reverse32(0xfeedc0de) == (uint32_t)userlandMagicSlotBN0120;

    int N0110Counter = userlandMagicSlotAN0110IsValid + userlandMagicSlotBN0110IsValid;
    int N0120Counter = userlandMagicSlotAN0120IsValid + userlandMagicSlotBN0120IsValid;

    // At least one slot indicate N0110 and none N0120
    if ((N0110Counter > 0) && (N0120Counter == 0))
    {
        return 1;
    }

    // At least one slot indicate N0120 and none N0110
    if ((N0120Counter > 0) && (N0110Counter == 0))
    {
        return 2;
    }

    // In case where both matched, choose the one with most matches (for example,
    // if slot data made a false positive (should not happen unless someone flash
    // the wrong firmware on a calculator))
    if (N0110Counter > N0120Counter)
    {
        return 1;
    }
    if (N0120Counter > N0110Counter)
    {
        return 2;
    }

    // The remaining cases is equality (no match or as much matches). In both
    // cases, we cannot know
    return 0;
}

uint32_t *extapp_userlandAddress()
{
    // Get the model
    const uint8_t model = extapp_calculatorModel();

    if (model == 1)
    {
        return (uint32_t *)0x20000008;
    }
    if (model == 2)
    {
        return (uint32_t *)0x24000008;
    }

    // We don't know for other cases, so we suppose (arbitrary) that it's an
    // N0110/N0115 because N0120 is not the latest model and is much less used
    // than N0110/N0115
    return (uint32_t *)0x24000008;
}