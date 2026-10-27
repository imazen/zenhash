/* Bounded structured key families; independently evaluated by pinned C hashes.
 * Only canonical output values and canonical prefix lengths enter fingerprints. */
#define main original_conformance_main
#include "hash-conformance.c"
#undef main

#include "conformance-patterns.h"
/* C and Rust record whole message then prefix separately for each variant. */
static void test_case(const uint8_t *data,size_t len,uint64_t seed,const uint8_t *secret,size_t size,uint64_t *states) {
    size_t split=(size_t)((((uint64_t)len*7)^seed^(uint64_t)size)%((uint64_t)len+1));
    /* Record numeric outputs directly in variant order; no host byte encodings. */
    uint64_t keys[8]={0};
    for(unsigned a=0;a<8;a++) for(unsigned b=0;b<8;b++) keys[a]|=(uint64_t)secret[a*8+b]<<(8*b);
    for(unsigned part=0;part<2;part++) {
        size_t n=part ? split : len;
        record(&states[0],XXH32(data,n,(uint32_t)seed));
        record(&states[1],XXH64(data,n,seed));
        record(&states[2],XXH3_64bits_withSeed(data,n,seed));
        XXH128_hash_t h=XXH3_128bits_withSeed(data,n,seed);
        record(&states[3],h.low64); record(&states[3],h.high64);
        record(&states[4],XXH3_64bits_withSecret(data,n,secret,size));
        h=XXH3_128bits_withSecret(data,n,secret,size);
        record(&states[5],h.low64); record(&states[5],h.high64);
        record(&states[6],rapidhash_withSeed(data,n,seed));
        record(&states[7],rapidhash_internal(data,n,seed,keys));
    }
}
static int size_cmp(const void *a,const void *b) {size_t x=*(const size_t*)a,y=*(const size_t*)b;return (x>y)-(x<y);}
int main(void) {
    const uint64_t seeds[]={0,1,UINT64_MAX,UINT64_C(0x9e3779b185ebca87)};
    const size_t sizes[]={136,137,171,192,200,1000};
    const size_t flips[]={1,4,8,16,31,64,112,240,241,1024};
    const size_t correlated[]={0,8,12,15,16,17,32,64,128,241,1024};
    const size_t offsets[]={0,1,4,8,15};
    uint8_t *data=malloc(65536),*secret=malloc(1000);
    if(!data || !secret) return 1;
    puts("# Extended C-reference corpus v1; preserves original conformance.txt separately");
    puts("# family secret_len fingerprints: xxh32 xxh64 seeded3_64 seeded3_128 secret3_64 secret3_128 rapid rapid_secret");
    for(unsigned family=0;family<15;family++) for(size_t si=0;si<(family<12?6:1);si++) {
        size_t size=family<12?sizes[si]:192;
        size_t lengths[320],count=0;
        for(size_t i=0;i<=256;i++) lengths[count++]=i;
        for(size_t i=0;i<sizeof(edges)/sizeof(edges[0]);i++) lengths[count++]=edges[i];
        size_t block=((size-64)/8)*64;
        for(size_t b=1;b<=2;b++) for(int d=-1;d<=1;d++) lengths[count++]=block*b+(size_t)d;
        qsort(lengths,count,sizeof(size_t),size_cmp);
        size_t unique=0;for(size_t i=0;i<count;i++) if(!unique||lengths[i]!=lengths[unique-1]) lengths[unique++]=lengths[i];
        uint64_t states[8]; for(unsigned a=0;a<8;a++) states[a]=UINT64_C(0xcbf29ce484222325);
        for(unsigned sp=0;sp<6;sp++) for(size_t s=0;s<4;s++) {
            uint64_t seed=seeds[s]; extended_bytes(secret,size,sp,seed^size);
            if(family<12) {
                for(size_t i=0;i<unique;i++) {size_t len=lengths[i];extended_bytes(data,len,family,seed^len);test_case(data,len,seed,secret,size,states);}
            } else if(family==12) {
                for(size_t l=0;l<sizeof(flips)/sizeof(flips[0]);l++) {
                    size_t len=flips[l];extended_bytes(data,len,5,seed^len);
                    for(size_t bit=0;bit<len*8;bit++) {data[bit/8]^=(uint8_t)(1u<<(bit%8));test_case(data,len,seed,secret,size,states);data[bit/8]^=(uint8_t)(1u<<(bit%8));}
                }
            } else if(family==13) {
                for(unsigned bit=0;bit<64;bit++) {
                    uint64_t related[]={UINT64_C(1)<<bit,~(UINT64_C(1)<<bit),(UINT64_C(1)<<bit)|(UINT64_C(1)<<((bit+1)%64))};
                    for(unsigned r=0;r<3;r++) for(size_t l=0;l<sizeof(correlated)/sizeof(correlated[0]);l++) for(size_t o=0;o<5;o++) {
                        size_t len=correlated[l],off=offsets[o]; memset(data,0,len);
                        for(size_t i=0;i<8;i++) if(off+i<len) data[off+i]=(uint8_t)(related[r]>>(i*8));
                        test_case(data,len,related[r],secret,size,states);
                    }
                }
            } else {
                for(uint64_t rank=0;rank<24;rank++) for(size_t i=0;i<unique;i++) {
                    size_t len=lengths[i];extended_bytes(data,len,10,rank);test_case(data,len,seed,secret,size,states);
                }
            }
        }
        printf("%u %zu",family,size);for(unsigned a=0;a<8;a++) printf(" %016" PRIx64,states[a]);puts("");
    }
    free(data);free(secret);return 0;
}
