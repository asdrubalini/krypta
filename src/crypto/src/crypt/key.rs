use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha20Rng;

pub fn generate_random_secure_key_nonce_pair() -> ([u8; 32], [u8; 24]) {
    let mut rng = ChaCha20Rng::from_entropy();

    let mut key = [0u8; 32];
    let mut nonce = [0u8; 24];

    rng.fill_bytes(&mut key);
    rng.fill_bytes(&mut nonce);

    (key, nonce)
}

#[cfg(test)]
mod tests {
    use super::generate_random_secure_key_nonce_pair;

    #[test]
    fn test_random_key() {
        let (key, nonce) = generate_random_secure_key_nonce_pair();

        assert_ne!(key, [0u8; 32]);
        assert_ne!(nonce, [0u8; 24]);
    }
}
