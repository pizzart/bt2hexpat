#pragma description Converted from 010 Editor Binary Template

import std.mem;

RequiresVersion(14);
using ID = char;

s32 haveValidFormat = false;
struct WAVRIFFHEADER {
    ID groupID;
    s32 size;
    ID riffType;
}
struct FORMATCHUNK {
    ID chunkID;
    s32 chunkSize;
    s32 pos = FTell();
    s16 wFormatTag;
    s16 wChannels;
    s32 dwSamplesPerSec;
    s32 dwAvgBytesPerSec;
    s16 wBlockAlign;
    s16 wBitsPerSample;
    if (wFormatTag == 17) {
        s16 wcbsize;
        s16 wSamplesPerBlock;
    }
    haveValidFormat = true;
    if (chunkSize > FTell() - pos) {
        u8 unknown[chunkSize - FTell() - pos];
    }
    if (chunkSize & 1) {
        u8 padding;
    }
}
struct DATACHUNK {
    ID chunkID;
    s32 chunkSize;
    if (!haveValidFormat) {
        Warning("File contains no valid WAVE format chunk.");
        return -1;
    }
    if (format.wBitsPerSample != 8 && format.wBitsPerSample != 16 && format.wBitsPerSample != 32 || chunkSize % (s32) format.wBlockAlign != 0) {
        char waveformData[chunkSize];
    } else     if (format.wChannels == 1 && format.wBitsPerSample == 8) {
        u8 samples[chunkSize];
    } else     if (format.wChannels == 1 && format.wBitsPerSample == 16) {
        s16 samples[chunkSize / 2];
    } else     if (format.wChannels == 1 && format.wBitsPerSample == 32) {
        s32 samples[chunkSize / 4];
    } else {
        struct SAMPLES {
        if (format.wBitsPerSample == 8) {
            u8 channels[format.wChannels];
        } else         if (format.wBitsPerSample == 16) {
            s16 channels[format.wChannels];
        } else         if (format.wBitsPerSample == 32) {
            s32 channels[format.wChannels];
        }


    }
 samples[chunkSize / (s32) format.wBlockAlign];
    }



    if (chunkSize & 1 && FTell() < FileSize()) {
        u8 padding;
    }
}
struct FACTCHUNK {
    ID chunkID;
    s32 chunkSize;
    s32 uncompressedSize;
}
struct CUEPOINT {
    s32 dwIdentifier;
    s32 dwPosition;
    ID fccChunk;
    s32 dwChunkStart;
    s32 dwBlockStart;
    s32 dwSampleOffset;
}
struct CUECHUNK {
    ID chunkID;
    s32 chunkSize;
    s32 pos = FTell();
    s32 dwCuePoints;
    CUEPOINT points[dwCuePoints];
    if (chunkSize > FTell() - pos) {
        u8 unknown[chunkSize - FTell() - pos];
    }
}
struct LISTSUBCHUNK {
    ID chunkID;
    s32 chunkSize;
    char listData[chunkSize];
    if (chunkSize & 1 && FTell() < FileSize()) {
        u8 padding;
    }
}
struct LISTCHUNK {
    ID chunkID;
    s32 chunkSize;
    s64 pos = FTell();
    ID chunkType;
    u32 size;
    while (FTell() - pos < chunkSize) {
        size = ReadUInt(FTell(), 4);
        if (FTell() - pos + size <= chunkSize) {
            LISTSUBCHUNK subchunk;
        } else {
        u8 unknown[chunkSize - FTell() - pos];
        }
    }
    if (chunkSize & 1 && FTell() < FileSize()) {
        u8 padding;
    }
}
struct UNKNOWNCHUNK {
    ID chunkID;
    s32 chunkSize;
    u8 unknownData[chunkSize];
    if (chunkSize & 1 && FTell() < FileSize()) {
        u8 padding;
    }
}
LittleEndian();
WAVRIFFHEADER header;
if (header.groupID != "RIFF" || header.riffType != "WAVE") {
    Warning("File is not a valid wave file. Template stopped.");
    return -1;
}
char tag[5];
u32 size;
while (!FEof()) {
    ReadBytes(tag, FTell(), 4);
    tag[4] = 0;
    match (tag) {
        (0):
            FORMATCHUNK format;
        (1):
            DATACHUNK data;
        (2):
            FACTCHUNK fact;
        (3):
            CUECHUNK cue;
        (4):
            LISTCHUNK list;
        (_):
            size = ReadUInt(FTell(), 4);
            Printf("Encountered unknown chunk '%s' of size %d at position %Ld.\n", tag, size, FTell());
            UNKNOWNCHUNK unknown;
    }
}
