use byte_unit::Byte;
use database::{models, Database};

pub async fn execute(database: &Database) -> anyhow::Result<()> {
    let archive_size_bytes = models::File::archive_size(database)?;
    let archive_size = Byte::from_bytes(archive_size_bytes.into());

    let archive_count = models::File::count(database)?;

    println!(
        "The total size of the archive is {}",
        archive_size.get_appropriate_unit(false)
    );

    println!("The archive has {} files", archive_count);

    Ok(())
}
