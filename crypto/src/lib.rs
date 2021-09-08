#![allow(dead_code, unused_variables)]

pub mod bulk;
pub mod error;
pub mod single;
pub mod types;

const BUFFER_SIZE: usize = 16384;

// #[cfg(test)]
// mod tests {
// use std::path::Path;

// use crate::{bulk::encryption::BulkFileEncrypt, single::encryption::SingleFileEncryptor};

// #[tokio::main]
// async fn test() {
// println!("ciao");

// let mappings = (0..32)
// .map(|i| {
// SingleFileEncryptor::new(
// &Path::new("/source"),
// &Path::new("/destination"),
// &[0; 32],
// )
// .unwrap()
// })
// .collect::<Vec<SingleFileEncryptor>>();

// let instance = BulkFileEncrypt::new(mappings);
// let report = instance.encrypt().await;

// println!("{:#?}", report);
// }
// }

// #[cfg(test)]
// mod tests {
// use std::{convert::TryInto, fs::remove_file, path::Path, time::Instant};

// use super::*;

// use file_diff::diff;

// #[tokio::test]
// async fn test_file_encryption_and_decryption() {
// // Create random file
// let key = "00000000000000000000000000000000";
// let key_slice = key[..].as_bytes();

// assert_eq!(key_slice.len(), 32);

// let encryptor = file::encryption::FileEncryptor::new(
// &Path::new("./plaintext"),
// &Path::new("./encrypted"),
// key_slice.try_into().unwrap(),
// )
// .unwrap();

// let start = Instant::now();
// encryptor.try_encrypt().await.unwrap();
// println!("Took {:?} to encrypt", start.elapsed());

// let decryptor = file::decryption::FileDecryptor::new(
// &Path::new("./encrypted"),
// &Path::new("./plaintext-recovered"),
// &key_slice.try_into().unwrap(),
// )
// .unwrap();

// let start = Instant::now();
// decryptor.try_decrypt().await.unwrap();
// println!("Took {:?} to decrypt", start.elapsed());

// assert!(diff("./plaintext", "./plaintext-recovered"));

// remove_file("./plaintext").unwrap();
// remove_file("./encrypted").unwrap();
// remove_file("./plaintext-recovered").unwrap();
// }
// }
