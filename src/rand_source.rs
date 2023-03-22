use std::io::{Read, Write};

use rand::{Rng, RngCore, SeedableRng};
use rand_chacha::ChaCha20Rng;

pub struct RandomSource {
    rng: Box<dyn RngCore>,
}

impl RandomSource {
    pub fn chacha20_from_seed(seed: u64) -> Self {
        let rng = ChaCha20Rng::seed_from_u64(seed);
        let rng = Box::new(rng);
        Self { rng }
    }
}

impl Read for RandomSource {
    fn read(&mut self, mut buf: &mut [u8]) -> std::io::Result<usize> {
        let random_bytes = self.rng.gen::<[u8; 32]>();
        buf.write(&random_bytes)
    }
}
