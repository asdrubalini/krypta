use std::path::{Path, PathBuf};

use crypto::{crypt::FileEncryptBulk, errors::CryptoError, traits::ComputeBulk};
use database::{
    models::{self, Device},
    traits::UpdateMany,
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
    let encryptor = files_into_encryptor(need_encryption, locked_path, unlocked_path)?;
    let encryption_status = encryptor.start_all();
    log::trace!("Done with encryption job");

    // Error check
    let errors_count = encryption_status
        .iter()
        .filter(|(_, status)| !(**status))
        .count();
    log::info!(
        "Encrypted {} files, with {} errors",
        encryption_status.len(),
        errors_count
    );

    // Get file_device(s) and mark them as encrypted
    let file_devices = encryption_status_into_file_device(db, encryption_status, unlocked_path)?;
    let file_devices: Vec<models::FileDevice> = file_devices
        .into_iter()
        .map(|mut file_device| {
            file_device.is_encrypted = true;
            file_device
        })
        .collect();

    models::FileDevice::update_many(db, file_devices)?;

    Ok(())
}

/// Convert a bunch of file into bulk encryptor
fn files_into_encryptor(
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

/// Turn the encryption status into models::FileDevice(s)
fn encryption_status_into_file_device(
    db: &mut Database,
    encryption_status: impl IntoIterator<Item = (PathBuf, bool)>,
    unlocked_path: &Path,
) -> anyhow::Result<Vec<models::FileDevice>> {
    // Filter out only file paths that were encrypted successfully
    let successfully_encrypted_paths =
        encryption_status
            .into_iter()
            .filter_map(|(path, is_ok)| if is_ok { Some(path) } else { None });

    // Turn paths into relative
    let unlocked_path_len = unlocked_path.iter().count();
    let successfully_encrypted_paths_relative: Vec<PathBuf> = successfully_encrypted_paths
        .map(|path| path.iter().skip(unlocked_path_len).collect())
        .collect();

    Ok(models::FileDevice::find_by_paths(
        db,
        &successfully_encrypted_paths_relative,
    )?)
}
