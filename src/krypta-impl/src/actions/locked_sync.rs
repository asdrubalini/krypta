use std::path::Path;

use crypto::{crypt::FileEncryptBulk, errors::CryptoError, traits::ComputeBulk};
use database::{
    models::{self, Device},
    Database,
};

pub async fn sync_locked_path_from_database(
    db: &mut Database,
    current_device: &Device,
    locked_path: impl AsRef<Path>,
    unlocked_path: impl AsRef<Path>,
) -> anyhow::Result<()> {
    let unlocked_path = unlocked_path.as_ref();
    let locked_path = locked_path.as_ref();

    // Find paths that need encryption
    let need_encryption = models::File::find_need_encryption(db, current_device)?;

    // Start encryption job
    log::trace!("Encryption job started");
    let encryptor = into_encryptor(need_encryption, locked_path, unlocked_path)?;
    let encryption_status = encryptor.start_all();
    log::trace!("Done with encryption job");

    let errors_count = encryption_status
        .iter()
        .filter(|(_, status)| **status)
        .count();
    log::info!(
        "Encrypted {} files, with {} errors",
        encryption_status.len(),
        errors_count
    );

    // Filter out successfully encrypted files
    // Update models::FileDevice is_encrypted = 1

    Ok(())
}

fn into_encryptor(
    need_encryption: impl IntoIterator<Item = models::File>,
    locked_path: &Path,
    unlocked_path: &Path,
) -> anyhow::Result<Box<FileEncryptBulk>> {
    let encryptors = need_encryption
        .into_iter()
        .map(|file| models::File::try_into_encryptor(file, locked_path, unlocked_path))
        .collect::<Result<Vec<_>, CryptoError>>()?;

    Ok(FileEncryptBulk::new(encryptors))
}
