use std::sync::Arc;

use tokio::sync::Semaphore;

use crate::file::single::SingleFileEncryptor;
use crate::file::traits::SingleCrypt;
use crate::types::Report;

#[derive(Debug)]
pub struct BulkFileEncrypt {
    encryptors: Vec<SingleFileEncryptor>,
}

impl BulkFileEncrypt {
    pub fn new(encryptors: Vec<SingleFileEncryptor>) -> Self {
        Self { encryptors }
    }

    pub async fn encrypt(self) -> Report {
        let cpus_count = num_cpus::get();
        let semaphore = Arc::new(Semaphore::new(cpus_count));

        let mut handles = Vec::new();

        for encryptor in self.encryptors {
            let permit = semaphore.clone().acquire_owned().await.unwrap();

            let handle = tokio::spawn(async move {
                let result = SingleFileEncryptor::start(encryptor).await;

                drop(permit);
                result
            });

            handles.push(handle);
        }

        let mut errors_count: usize = 0;
        let mut processed_file_count: usize = 0;

        for handle in handles {
            if handle.await.unwrap().is_err() {
                errors_count += 1;
            }

            processed_file_count += 1;
        }

        Report {
            processed_file_count,
            errors_count,
        }
    }
}
