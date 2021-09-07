pub mod error;
pub mod file;

const BUFFER_SIZE: usize = 16384;

#[cfg(test)]
mod tests {
    use std::{convert::TryInto, path::Path};

    use super::*;

    #[tokio::test]
    async fn test_file_encryption_and_decryption() {
        let key = "00000000000000000000000000000000";
        let key_slice = key[..].as_bytes();

        let encryptor = file::encryption::FileEncryptor::new(
            &Path::new("./plaintext"),
            &Path::new("./encrypted"),
            key_slice.try_into().unwrap(),
        )
        .unwrap();

        encryptor.try_encrypt().await.unwrap();

        let decryptor = file::decryption::FileDecryptor::new(
            &Path::new("./encrypted"),
            &Path::new("./plaintext-recovered"),
            &key_slice.try_into().unwrap(),
        )
        .unwrap();

        decryptor.try_decrypt().await.unwrap();
    }
}

// use std::path::PathBuf;

// use bytes::BytesMut;
// use sodiumoxide::crypto::secretstream::{self, gen_key, Header, Key, Stream};
// use tokio::{
// fs::File,
// io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter},
// };

// pub struct EncryptedFile {
// path: PathBuf,
// header: Option<Header>,
// key: Option<Key>,
// }

// impl EncryptedFile {
// pub fn new(path: PathBuf) -> Self {
// Self {
// path,
// header: None,
// key: None,
// }
// }

// pub async fn encrypt(&mut self) {
// let key = gen_key();
// let (mut encryption_stream, header) = Stream::init_push(&key).unwrap();

// self.header = Some(header);
// self.key = Some(key);

// let file_input = File::open(&self.path).await.unwrap();
// let file_output = File::create("./encrypted.txt").await.unwrap();

// let mut reader = BufReader::new(file_input);
// let mut buffer_input = BytesMut::with_capacity(16384);

// let mut writer = BufWriter::new(file_output);

// while let Ok(size) = reader.read_buf(&mut buffer_input).await {
// if size == 0 {
// break;
// }

// let result = encryption_stream
// .push(&buffer_input, None, secretstream::Tag::Message)
// .unwrap();

// writer.write_all(&result).await.unwrap();

// buffer_input.clear();
// }

// writer.flush().await.unwrap();
// }

// pub async fn decrypt(&self) {
// let header = self.header.as_ref().unwrap();
// let key = self.key.as_ref().unwrap();

// let mut decryption_stream = Stream::init_pull(header, key).unwrap();

// let file_input = File::open("./encrypted.txt").await.unwrap();
// let file_output = File::create("./plaintext-recovered.txt").await.unwrap();

// let mut reader = BufReader::new(file_input);
// let mut buffer_input = BytesMut::with_capacity(16384);

// let mut writer = BufWriter::new(file_output);

// while let Ok(size) = reader.read_buf(&mut buffer_input).await {
// if size == 0 {
// break;
// }

// let (result, _tag) = decryption_stream.pull(&buffer_input, None).unwrap();
// writer.write_all(&result).await.unwrap();

// buffer_input.clear();
// }

// writer.flush().await.unwrap();
// }
// }
