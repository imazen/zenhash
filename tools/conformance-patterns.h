/* Shared C input families; independently mirrored in Rust stability tests. */
static void extended_bytes(uint8_t *data,size_t len,unsigned pattern,uint64_t seed) {
    if(pattern<6) { bytes(data,len,pattern,seed); return; }
    const char text[]="user/0000000000000000/item";
    const size_t cycles[]={3,7,17};
    uint32_t words[]={0,1,UINT32_C(0x80000000),UINT32_MAX};
    uint64_t rank=seed%24;
    for(size_t i=0;i<4;i++) {
        size_t n=4-i,j=i+(size_t)(rank%n); uint32_t t=words[i]; words[i]=words[j]; words[j]=t; rank/=n;
    }
    for(size_t i=0;i<len;i++) {
        if(pattern==6) {
            size_t p=i%(sizeof(text)-1);
            data[i]=(p>=5 && p<21) ? (uint8_t)"0123456789abcdef"[(seed>>((20-p)*4))&15] : (uint8_t)text[p];
        } else if(pattern<=9) {
            size_t p=i%cycles[pattern-7]; data[i]=(uint8_t)((uint8_t)(p*37)+(uint8_t)(seed>>((p%8)*8)));
        } else if(pattern==10) data[i]=(uint8_t)(words[(i/4)%4]>>((i%4)*8));
        else data[i]=(i/8)%2 ? (uint8_t)~(seed>>((i%8)*8)) : (uint8_t)(seed>>((i%8)*8));
    }
}
