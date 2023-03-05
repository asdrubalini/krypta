mod decrypt;
mod encrypt;
mod key;

const AEAD_TAG_SIZE: usize = 16;
pub const AEAD_KEY_SIZE: usize = 32;
pub const AEAD_NONCE_SIZE: usize = 24;

type KeyArray = GenericArray<u8, U32>;
type NonceArray = GenericArray<u8, U24>;

use std::{fmt::Display, path::PathBuf};

use chacha20poly1305::{
    aead::generic_array::GenericArray,
    consts::{U24, U32},
};
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
