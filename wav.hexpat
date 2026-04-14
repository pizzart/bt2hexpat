#pragma description Converted from 010 Editor Binary Template

import std.mem;

using ID = char;

struct WAVRIFFHEADER {
  id groupID;
  s32 size;
  id riffType;
}
struct FORMATCHUNK {
  id chunkID;
  s32 chunkSize;
  s16 wFormatTag;
  u16 wChannels;
  u32 dwSamplesPerSec;
  u32 dwAvgBytesPerSec;
  u16 wBlockAlign;
  u16 wBitsPerSample;
  if (wFormatTag == 17) {
  u16 wcbsize;
  u16 wSamplesPerBlock;
  }
  if (chunkSize > FTell() - pos) {
noop  }
  if (chunkSize & 1) {
noop  }
}
struct DATACHUNK {
  id chunkID;
  s32 chunkSize;
  if (! ! haveValidFormat) {
noopnoop  }
  if (format.wBitsPerSample != 8 && format.wBitsPerSample != 16 && format.wBitsPerSample != 32 || chunkSize % (s32)format.wBlockAlign != 0) {
  u8 waveformData;
  } else   if (format.wChannels == 1 && format.wBitsPerSample == 8) {
  u8 samples;
  } else   if (format.wChannels == 1 && format.wBitsPerSample == 16) {
  s16 samples;
  } else   if (format.wChannels == 1 && format.wBitsPerSample == 32) {
  s32 samples;
  } else {
noop  }



  if (chunkSize & 1 && FTell() < FileSize()) {
noop  }
}
struct FACTCHUNK {
  id chunkID;
  s32 chunkSize;
  u32 uncompressedSize;
}
struct CUEPOINT {
  s32 dwIdentifier;
  s32 dwPosition;
  id fccChunk;
  s32 dwChunkStart;
  s32 dwBlockStart;
  s32 dwSampleOffset;
}
struct CUECHUNK {
  id chunkID;
  s32 chunkSize;
  s32 dwCuePoints;
  cuepoint points[dwCuePoints];
  if (chunkSize > FTell() - pos) {
noop  }
}
struct LISTSUBCHUNK {
  id chunkID;
  s32 chunkSize;
  char listData[chunkSize];
  if (chunkSize & 1 && FTell() < FileSize()) {
noop  }
}
struct LISTCHUNK {
  id chunkID;
  s32 chunkSize;
  id chunkType;
  while (FTell() - pos < chunkSize) {
noop  if (FTell() - pos + size <= chunkSize) {
noop  } else {
  u8 unknown;
  }
  }
  if (chunkSize & 1 && FTell() < FileSize()) {
noop  }
}
struct UNKNOWNCHUNK {
  id chunkID;
  s32 chunkSize;
  u8 unknownData[chunkSize];
  if (chunkSize & 1 && FTell() < FileSize()) {
noop  }
}
  if (header.groupID != "RIFF" || header.riffType != "WAVE") {
noopnoop  }
  while (! ! FEof()) {
noopnoop  match (tag) {
      (0):
noopnoop      (1):
noopnoop      (2):
noopnoop      (3):
noopnoop      (_):
noopnoopnoopnoopnoopnoop  }
  }
