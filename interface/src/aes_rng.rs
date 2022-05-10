use aes::Aes128;
use cipher::{BlockCipherKey, FromBlockCipher, NewBlockCipher, StreamCipher};
use ctr::{Ctr, Ctr64LE};
use rand_core::{CryptoRng, Error as RandError, RngCore, SeedableRng};

/// An RNG whose stream is an AES-CTR keystream
pub struct Aes128Rng(Ctr64LE<Aes128>);

impl RngCore for Aes128Rng {
    fn next_u32(&mut self) -> u32 {
        rand_core::impls::next_u32_via_fill(self)
    }
    fn next_u64(&mut self) -> u64 {
        rand_core::impls::next_u64_via_fill(self)
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.0.apply_keystream(dest);
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), RandError> {
        self.fill_bytes(dest);
        Ok(())
    }
}

impl SeedableRng for Aes128Rng {
    type Seed = BlockCipherKey<Aes128>;

    /// The RNG is the keystream of AES-CTR(key=seed, iv=00...0), using 64-bit counters
    fn from_seed(seed: Self::Seed) -> Aes128Rng {
        let key = seed;
        let iv = BlockCipherKey::<Aes128>::default();

        let ciph = Aes128::new(&key);
        let stream = Ctr::from_block_cipher(ciph, &iv);
        Aes128Rng(stream)
    }
}

impl CryptoRng for Aes128Rng {}

#[cfg(test)]
mod tests {
    use super::*;

    // The size of the randomness buffer we use in tests
    const BUF_SIZE: usize = 100;

    #[test]
    fn fill_bytes() {
        // Make a seed
        let mut seed = <Aes128Rng as SeedableRng>::Seed::default();
        let seed_str = b"test fill_bytes";
        seed[..seed_str.len()].copy_from_slice(seed_str);

        // Instantiate the RNG
        let mut rng = Aes128Rng::from_seed(seed);

        // Generate bytes
        let mut buf = [0u8; BUF_SIZE];
        rng.fill_bytes(&mut buf);

        // Make sure the bytes weren't just zeros
        assert_ne!(buf, [0u8; BUF_SIZE]);
    }
}
