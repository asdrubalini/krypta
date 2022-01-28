use byte_unit::Byte;
use database::{models, traits::Count, Database};

pub async fn status(db: &Database) -> anyhow::Result<()> {
    let device = models::Device::find_or_create_current(db)?;

    let archive_size_bytes = models::File::archive_size(db)?;
    let archive_size = Byte::from_bytes(archive_size_bytes.into());

    let archive_count = models::File::count(db)?;
    let locked_count = models::File::count_locked(db, &device)?;
    let unlocked_count = models::File::count_unlocked(db, &device)?;

    println!("Files stored in database: {archive_count}");
    println!("Archive size: {}", archive_size.get_appropriate_unit(false));

    println!("Locked files on disk: {locked_count}");
    println!("Unlocked files on disk: {unlocked_count}");

    Ok(())
}
