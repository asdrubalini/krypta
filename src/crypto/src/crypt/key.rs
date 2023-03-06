use chacha20poly1305::{AeadCore, KeyInit, XChaCha20Poly1305};
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;

use super::{KeyArray, NonceArray};

pub fn generate_random_secure_key_nonce_pair() -> (KeyArray, NonceArray) {
    // TODO: make sure that this Rng is crypto safe
    let mut rng = ChaCha20Rng::from_entropy();

    let key = XChaCha20Poly1305::generate_key(&mut rng);
    let nonce = XChaCha20Poly1305::generate_nonce(&mut rng);

    (key, nonce)
}

#[cfg(test)]
mod tests {
    use crate::crypt::{AEAD_KEY_SIZE, AEAD_NONCE_SIZE};

    use super::generate_random_secure_key_nonce_pair;

    #[test]
    fn test_random_key() {
        let (key, nonce) = generate_random_secure_key_nonce_pair();

        let key = key.as_slice();
        let nonce = nonce.as_slice();

        assert_ne!(key, [0u8; AEAD_KEY_SIZE]);
        assert_ne!(nonce, [0u8; AEAD_NONCE_SIZE]);
    }
}
