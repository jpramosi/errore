// Taken from https://github.com/HindrikStegenga/const-fnv1a-hash/blob/main/src/lib.rs

const FNV_OFFSET_BASIS_64: u64 = 0xcbf29ce484222325;
const FNV_PRIME_64: u64 = 0x00000100000001B3;

#[doc(hidden)]
pub const fn fnv1a_hash_64(bytes: &[u8]) -> u64 {
    let prime = FNV_PRIME_64;

    let mut hash = FNV_OFFSET_BASIS_64;
    let mut i = 0;
    let len = bytes.len();

    while i < len {
        hash ^= bytes[i] as u64;
        hash = hash.wrapping_mul(prime);
        i += 1;
    }
    hash
}
