//! Streaming XXH3, following the reference implementation's buffering so the
//! digest matches the one-shot functions for every split of the input.

use alloc::boxed::Box;

use whereat::At;

use super::{
    INIT_ACC, MID_SIZE_MAX, STRIPE_LEN, SecretStore, accumulate, derive_secret, finish_long_64,
    finish_long_128, validate_secret,
};
use crate::Error;

const BUFFER_SIZE: usize = 256;

/// Streaming XXH3. One state yields both the 64-bit ([`digest`](Self::digest))
/// and the 128-bit ([`digest128`](Self::digest128)) hash, equal to
/// [`xxh3_64`](crate::xxh3_64) / [`xxh3_128`](crate::xxh3_128) (or their seed
/// and secret variants) of everything passed to [`update`](Self::update).
///
/// ```
/// let mut h = zenhash::Xxh3::new();
/// h.update(b"hello ");
/// h.update(b"world");
/// assert_eq!(h.digest(), zenhash::xxh3_64(b"hello world"));
/// assert_eq!(h.digest128(), zenhash::xxh3_128(b"hello world"));
/// ```
#[derive(Clone, Debug)]
pub struct Xxh3 {
    acc: [u64; 8],
    buffer: [u8; BUFFER_SIZE],
    buffered: usize,
    stripes_so_far: usize,
    total_len: u64,
    secret: SecretStore,
}

impl Xxh3 {
    fn with_store(secret: SecretStore) -> Self {
        Self {
            acc: INIT_ACC,
            buffer: [0; BUFFER_SIZE],
            buffered: 0,
            stripes_so_far: 0,
            total_len: 0,
            secret,
        }
    }

    /// Creates a hasher with seed 0 and the default secret.
    pub fn new() -> Self {
        Self::with_store(SecretStore::Default)
    }

    /// Creates a hasher with `seed`.
    pub fn with_seed(seed: u64) -> Self {
        if seed == 0 {
            Self::new()
        } else {
            Self::with_store(SecretStore::Seeded(seed, Box::new(derive_secret(seed))))
        }
    }

    /// Creates a hasher with a custom `secret` of at least
    /// [`XXH3_SECRET_SIZE_MIN`](crate::XXH3_SECRET_SIZE_MIN) bytes. The secret is
    /// copied.
    pub fn with_secret(secret: &[u8]) -> Result<Self, At<Error>> {
        validate_secret(secret)?;
        Ok(Self::with_store(SecretStore::Custom(secret.into())))
    }

    /// Feeds `data` into the hash.
    pub fn update(&mut self, data: &[u8]) {
        self.total_len = self.total_len.wrapping_add(data.len() as u64);
        if data.len() <= BUFFER_SIZE - self.buffered {
            self.buffer[self.buffered..self.buffered + data.len()].copy_from_slice(data);
            self.buffered += data.len();
            return;
        }
        let secret = self.secret.long_secret();
        let mut data = data;
        if self.buffered > 0 {
            let (head, rest) = data.split_at(BUFFER_SIZE - self.buffered);
            self.buffer[self.buffered..].copy_from_slice(head);
            data = rest;
            let (stripes, _) = self.buffer.as_chunks::<STRIPE_LEN>();
            accumulate::consume(
                &mut self.acc,
                stripes,
                &mut self.stripes_so_far,
                secret,
                None,
            );
            self.buffered = 0;
        }
        // `data` is non-empty here. Keep at least one byte (and up to a full
        // buffer) back, because the final stripe gets special treatment.
        if data.len() > BUFFER_SIZE {
            let n = (data.len() - 1) / STRIPE_LEN;
            let (whole, rest) = data.split_at(n * STRIPE_LEN);
            let (stripes, _) = whole.as_chunks::<STRIPE_LEN>();
            accumulate::consume(
                &mut self.acc,
                stripes,
                &mut self.stripes_so_far,
                secret,
                None,
            );
            // The digest may need the last consumed stripe as the "previous
            // 64 bytes" of a short tail.
            self.buffer[BUFFER_SIZE - STRIPE_LEN..]
                .copy_from_slice(&whole[whole.len() - STRIPE_LEN..]);
            data = rest;
        }
        self.buffer[..data.len()].copy_from_slice(data);
        self.buffered = data.len();
    }

    /// Accumulators after the buffered tail and the final stripe.
    fn long_acc(&self) -> [u64; 8] {
        let secret = self.secret.long_secret();
        let mut acc = self.acc;
        let mut so_far = self.stripes_so_far;
        if self.buffered >= STRIPE_LEN {
            let n = (self.buffered - 1) / STRIPE_LEN;
            let (stripes, _) = self.buffer[..n * STRIPE_LEN].as_chunks::<STRIPE_LEN>();
            let last = self.buffer[..self.buffered].last_chunk::<STRIPE_LEN>();
            accumulate::consume(&mut acc, stripes, &mut so_far, secret, last);
        } else {
            // The final stripe straddles the end of the previous buffer.
            let catchup = STRIPE_LEN - self.buffered;
            let mut last = [0u8; STRIPE_LEN];
            last[..catchup].copy_from_slice(&self.buffer[BUFFER_SIZE - catchup..]);
            last[catchup..].copy_from_slice(&self.buffer[..self.buffered]);
            accumulate::consume(&mut acc, &[], &mut so_far, secret, Some(&last));
        }
        acc
    }

    /// Returns the 64-bit hash of everything fed so far. Does not reset the state.
    pub fn digest(&self) -> u64 {
        if self.total_len > MID_SIZE_MAX as u64 {
            finish_long_64(&self.long_acc(), self.secret.long_secret(), self.total_len)
        } else {
            self.secret.short_64(&self.buffer[..self.buffered])
        }
    }

    /// Returns the 128-bit hash of everything fed so far. Does not reset the state.
    pub fn digest128(&self) -> u128 {
        if self.total_len > MID_SIZE_MAX as u64 {
            finish_long_128(&self.long_acc(), self.secret.long_secret(), self.total_len)
        } else {
            self.secret.short_128(&self.buffer[..self.buffered])
        }
    }

    /// Resets to the freshly constructed state, keeping the seed or secret.
    pub fn reset(&mut self) {
        self.acc = INIT_ACC;
        self.buffered = 0;
        self.stripes_so_far = 0;
        self.total_len = 0;
    }
}

impl Default for Xxh3 {
    /// Seed 0, default secret.
    fn default() -> Self {
        Self::new()
    }
}

impl core::hash::Hasher for Xxh3 {
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        self.update(bytes);
    }

    /// The 64-bit digest.
    #[inline]
    fn finish(&self) -> u64 {
        self.digest()
    }
}

#[cfg(feature = "std")]
impl std::io::Write for Xxh3 {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.update(buf);
        Ok(buf.len())
    }

    #[inline]
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
