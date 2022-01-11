const AEAD_TAG_SIZE: usize = 16;
pub const AEAD_KEY_SIZE: usize = 32;
pub const AEAD_NONCE_SIZE: usize = 19;

mod decryption;
mod encryption;
mod key;

pub use decryption::{FileDecryptBulk, FileDecryptUnit};
pub use encryption::{FileEncryptBulk, FileEncryptUnit};
pub use key::generate_random_secure_key_nonce_pair;
