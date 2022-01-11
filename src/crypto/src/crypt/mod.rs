const AEAD_TAG_SIZE: usize = 16;
pub const AEAD_KEY_SIZE: usize = 32;
pub const AEAD_NONCE_SIZE: usize = 19;

mod decrypt;
mod encrypt;
mod key;

use std::{fmt::Display, path::PathBuf};

pub use decrypt::{FileDecryptBulk, FileDecryptUnit};
pub use encrypt::{FileEncryptBulk, FileEncryptUnit};
pub use key::generate_random_secure_key_nonce_pair;

#[derive(Debug)]
pub struct PathPair {
    pub source: PathBuf,
    pub destination: PathBuf,
}

impl Display for PathPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
