// Golden producer using Austin Appleby's public-domain SMHasher reference.
// Source revision: 07bb4de10a63e8cc2e1724865454eba635742383.
// Run on little-endian machines: the original C++ body reads native words.
#include "../tests/reference/MurmurHash3.h"
#include <array>
#include <cstdint>
#include <cstdio>
#include <vector>
int main() {
    const uint32_t endian = 1;
    if (*reinterpret_cast<const uint8_t*>(&endian) != 1) return 1;
    const std::array<uint32_t, 4> seeds = {0, 42, 3242157231u, 0xffffffffu};
    std::vector<int> lengths;
    for (int n = 0; n <= 67; ++n) lengths.push_back(n);
    for (int n : {255, 256, 257, 511, 512, 513, 1023, 1024, 4095, 4096, 4097}) lengths.push_back(n);
    std::puts("# seed_hex length pattern37_hash_hex");
    for (uint32_t seed : seeds) {
        for (int len : lengths) {
            std::vector<uint8_t> input(static_cast<size_t>(len) + 4);
            for (int i = 0; i < len; ++i) input[i] = static_cast<uint8_t>(i * 37 + 11);
            uint32_t out = 0;
            MurmurHash3_x86_32(input.data(), len, seed, &out);
            std::printf("%08x %d %08x\n", seed, len, out);
        }
    }
}
