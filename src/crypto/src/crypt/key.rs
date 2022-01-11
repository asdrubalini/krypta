use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha20Rng;

use crate::{AEAD_KEY_SIZE, AEAD_NONCE_SIZE};

pub fn generate_random_secure_key_nonce_pair() -> ([u8; AEAD_KEY_SIZE], [u8; AEAD_NONCE_SIZE]) {
    let mut rng = ChaCha20Rng::from_entropy();

    let mut key = [0u8; AEAD_KEY_SIZE];
    let mut nonce = [0u8; AEAD_NONCE_SIZE];

    rng.fill_bytes(&mut key);
    rng.fill_bytes(&mut nonce);

    (key, nonce)
}

#[cfg(test)]
mod tests {
    use crate::{AEAD_KEY_SIZE, AEAD_NONCE_SIZE};

    use super::generate_random_secure_key_nonce_pair;

    #[test]
    fn test_random_key() {
        let (key, nonce) = generate_random_secure_key_nonce_pair();

        assert_ne!(key, [0u8; AEAD_KEY_SIZE]);
        assert_ne!(nonce, [0u8; AEAD_NONCE_SIZE]);
    }
}
