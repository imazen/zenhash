/* Independent unsigned-arithmetic FNV-1a reference, including caller formulas.
 * GCC/Clang are required for unsigned __int128. Outputs are canonical LE words.
 * Regenerate with just fnv-vectors. */
#define main original_conformance_main
#include "hash-conformance.c"
#undef main

static int full_bases(void) {
    const uint32_t bases32[]={0,UINT32_MAX,UINT32_C(1)<<31,UINT32_C(0xa5a5a5a5)};
    const uint64_t bases64[]={0,UINT64_MAX,UINT64_C(1)<<63,UINT64_C(0xa5a5a5a5a5a5a5a5)};
    const unsigned __int128 bases128[]={0,~(unsigned __int128)0,(unsigned __int128)1<<127,
        ((unsigned __int128)UINT64_C(0xa5a5a5a5a5a5a5a5)<<64)|UINT64_C(0xa5a5a5a5a5a5a5a5)};
    uint8_t data[65536];
    puts("# FNV-1a full-width bases: zero, maximum, high-bit, repeated a5");
    puts("# pattern base_index fnv32 fnv64 fnv128; whole then prefix, canonical LE words");
    for(unsigned pattern=0;pattern<6;pattern++) for(unsigned basis=0;basis<4;basis++) {
        uint64_t states[3]={UINT64_C(0xcbf29ce484222325),UINT64_C(0xcbf29ce484222325),UINT64_C(0xcbf29ce484222325)};
        for(size_t index=0;index<257+sizeof(edges)/sizeof(edges[0]);index++) {
            size_t len=index<257?index:edges[index-257];
            bytes(data,len,pattern,UINT64_C(0x9e3779b185ebca87)^len);
            size_t split=((uint64_t)len*7^basis)%((uint64_t)len+1);
            for(unsigned part=0;part<2;part++) {
                uint32_t a=bases32[basis];uint64_t b=bases64[basis];unsigned __int128 c=bases128[basis];
                for(size_t i=0;i<(part?split:len);i++) {
                    a=(a^data[i])*UINT32_C(0x01000193);
                    b=(b^data[i])*UINT64_C(0x100000001b3);
                    c=(c^data[i])*(((unsigned __int128)1<<88)|0x13b);
                }
                record(&states[0],a);record(&states[1],b);record(&states[2],(uint64_t)c);record(&states[2],(uint64_t)(c>>64));
            }
        }
        printf("%u %u",pattern,basis);
        for(unsigned a=0;a<3;a++)printf(" %016" PRIx64,states[a]);
        puts("");
    }
    return 0;
}
int main(int argc,char **argv) {
    if(argc==2 && strcmp(argv[1],"bases")==0) return full_bases();
    const uint64_t seeds[] = {0, 1, UINT64_MAX, UINT64_C(0x9e3779b185ebca87)};
    uint8_t data[65536];
    puts("# FNV-1a 32/64/128; folded64 and fmix64; whole then prefix outputs");
    puts("# pattern basis(hex; 0 means standard) fnv32 fnv64 fnv128 fold32 fmix64");
    for (unsigned pattern=0; pattern<6; pattern++) for(unsigned s=0;s<4;s++) {
        uint64_t seed=seeds[s], states[5];
        for(unsigned a=0;a<5;a++) states[a]=UINT64_C(0xcbf29ce484222325);
        for(size_t index=0;index<257+sizeof(edges)/sizeof(edges[0]);index++) {
            size_t len=index<257? index:edges[index-257];
            bytes(data,len,pattern,seed^len);
            size_t split=((uint64_t)len*7^seed)%((uint64_t)len+1);
            for(unsigned part=0;part<2;part++) {
                uint32_t h32=seed?(uint32_t)seed:UINT32_C(0x811c9dc5);
                uint64_t h64=seed?seed:UINT64_C(0xcbf29ce484222325);
                unsigned __int128 h128=seed?seed:
                    ((unsigned __int128)UINT64_C(0x6c62272e07bb0142)<<64)|UINT64_C(0x62b821756295c58d);
                const unsigned __int128 prime128=((unsigned __int128)1<<88)|0x13b;
                for(size_t i=0;i<(part?split:len);i++) {
                    h32=(h32^data[i])*UINT32_C(0x01000193);
                    h64=(h64^data[i])*UINT64_C(0x100000001b3);
                    h128=(h128^data[i])*prime128;
                }
                uint64_t mix=h64;
                mix=(mix^(mix>>33))*UINT64_C(0xff51afd7ed558ccd);
                mix=(mix^(mix>>33))*UINT64_C(0xc4ceb9fe1a85ec53);
                mix^=mix>>33;
                record(&states[0],h32);record(&states[1],h64);
                record(&states[2],(uint64_t)h128);record(&states[2],(uint64_t)(h128>>64));
                record(&states[3],(uint32_t)(h64^(h64>>32)));record(&states[4],mix);
            }
        }
        printf("%u %016" PRIx64,pattern,seed);
        for(unsigned a=0;a<5;a++)printf(" %016" PRIx64,states[a]);
        puts("");
    }
}
