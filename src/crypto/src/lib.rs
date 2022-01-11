const BUFFER_SIZE: usize = 32768;
const AEAD_TAG_SIZE: usize = 16;

pub mod crypt;
pub mod hash;

pub mod errors;
pub mod types;

pub mod traits;
