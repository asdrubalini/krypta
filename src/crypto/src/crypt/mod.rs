const AEAD_TAG_SIZE: usize = 16;
pub const AEAD_KEY_SIZE: usize = 32;
pub const AEAD_NONCE_SIZE: usize = 19;

mod decrypt;
mod encrypt;
mod key;

pub use decrypt::{FileDecryptBulk, FileDecryptUnit};
pub use encrypt::{FileEncryptBulk, FileEncryptUnit};
pub use key::generate_random_secure_key_nonce_pair;
