use std::sync::Arc;

use tokio::sync::Semaphore;

use crate::crypt::single::SingleFileDecryptor;
use crate::crypt::traits::SingleCrypt;
use crate::types::Report;

#[derive(Debug)]
pub struct BulkFileDecrypt {
    decryptors: Vec<SingleFileDecryptor>,
}

impl BulkFileDecrypt {
    pub fn new(decryptors: Vec<SingleFileDecryptor>) -> Self {
        Self { decryptors }
    }

    pub async fn decrypt(self) -> Report {
        let cpus_count = num_cpus::get();
        let semaphore = Arc::new(Semaphore::new(cpus_count));

        let mut handles = Vec::new();

        for decryptor in self.decryptors {
            let permit = semaphore.clone().acquire_owned().await.unwrap();

            let handle = tokio::spawn(async move {
                let result = SingleFileDecryptor::start(decryptor).await;

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
