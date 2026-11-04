/* Fixed cross-platform conformance fingerprints from unmodified C references.
 * Regenerate with just conformance-vectors. FNV-1a summarizes canonical output
 * bytes separately for each algorithm; it is only a compact test fixture. */
#define XXH_INLINE_ALL
#include "../tests/reference/xxhash.h"
#include "../tests/reference/rapidhash.h"
#include <inttypes.h>
#include <stdio.h>
#include <stdlib.h>

static const size_t edges[] = {
    271,272,273,287,288,289,319,320,321,335,336,337,511,512,513,
    1000,1023,1024,1025,1087,1088,1089,2047,2048,2049,4095,4096,4097,10000,16384,65536
};
static uint64_t next(uint64_t *state) {
    *state += UINT64_C(0x9e3779b97f4a7c15);
    uint64_t z = *state;
    z = (z ^ (z >> 30)) * UINT64_C(0xbf58476d1ce4e5b9);
    z = (z ^ (z >> 27)) * UINT64_C(0x94d049bb133111eb);
    return z ^ (z >> 31);
}
static void bytes(uint8_t *data, size_t len, unsigned pattern, uint64_t seed) {
    for (size_t i=0; i<len; i++) {
        switch(pattern) {
        case 0: data[i]=0; break;
        case 1: data[i]=255; break;
        case 2: data[i]=(uint8_t)i; break;
        case 3: data[i]=i%2 ? 0xaa : 0x55; break;
        case 4: data[i]=i%127 ? 0 : (uint8_t)(1u << (i%8)); break;
        default: data[i]=(uint8_t)next(&seed); break;
        }
    }
}
static void record(uint64_t *state, uint64_t value) {
    for (unsigned b=0; b<8; b++) {
        *state ^= (uint8_t)(value >> (8*b));
        *state *= UINT64_C(0x100000001b3);
    }
}
int main(void) {
    const uint64_t seeds[] = {0, 1, UINT64_MAX, UINT64_C(0x9e3779b185ebca87)};
    const size_t sizes[] = {136,137,171,192,200,1000};
    uint8_t *data=malloc(65536), *secret=malloc(1000);
    if (!data || !secret) return 1;
    puts("# C references: xxHash v0.8.3; rapidhash 1ae7842f59f25929a338e5a76e3fa350673e48e1");
    puts("# just conformance-vectors; LE u64 outputs, XXH128 low then high, FNV-1a per algorithm");
    puts("# pattern seed(hex) secret_len xxh32 xxh64 xxh3_64 xxh3_128 secret64 secret128 rapid rapid_secret");
    for (unsigned pattern=0; pattern<6; pattern++) for (size_t s=0; s<4; s++) for (size_t k=0; k<6; k++) {
        uint64_t seed=seeds[s], states[8], rapid_keys[8]={0};
        size_t secret_len=sizes[k];
        for (unsigned a=0; a<8; a++) states[a]=UINT64_C(0xcbf29ce484222325);
        bytes(secret,secret_len,pattern,seed ^ secret_len);
        for(unsigned a=0; a<8; a++) for(unsigned b=0; b<8; b++) rapid_keys[a]|=(uint64_t)secret[a*8+b]<<(8*b);
        for(size_t index=0; index<257+sizeof(edges)/sizeof(edges[0]); index++) {
            size_t len=index<257 ? index : edges[index-257];
            bytes(data,len,pattern,seed ^ len);
            record(&states[0],XXH32(data,len,(uint32_t)seed));
            record(&states[1],XXH64(data,len,seed));
            record(&states[2],XXH3_64bits_withSeed(data,len,seed));
            XXH128_hash_t h=XXH3_128bits_withSeed(data,len,seed);
            record(&states[3],h.low64); record(&states[3],h.high64);
            record(&states[4],XXH3_64bits_withSecret(data,len,secret,secret_len));
            h=XXH3_128bits_withSecret(data,len,secret,secret_len);
            record(&states[5],h.low64); record(&states[5],h.high64);
            record(&states[6],rapidhash_withSeed(data,len,seed));
            record(&states[7],rapidhash_internal(data,len,seed,rapid_keys));
        }
        printf("%u %016" PRIx64 " %zu",pattern,seed,secret_len);
        for(unsigned a=0; a<8; a++) printf(" %016" PRIx64,states[a]);
        puts("");
    }
    free(data); free(secret);
    return 0;
}
