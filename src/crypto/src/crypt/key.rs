use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha20Rng;

pub fn generate_random_secure_key() -> [u8; 32] {
    let mut rng = ChaCha20Rng::from_entropy();
    let mut key = [0u8; 32];

    rng.fill_bytes(&mut key);

    key
}

#[cfg(test)]
mod tests {
    use super::generate_random_secure_key;

    #[test]
    fn test_random_key() {
        let key = generate_random_secure_key();
        assert_ne!(key, [0u8; 32]);
    }
}
