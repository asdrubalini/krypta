use byte_unit::Byte;

use crate::database::{models, Database};

pub async fn execute(database: &Database) {
    let archive_size_bytes = models::File::archive_size(database).await.unwrap();

    let archive_size = Byte::from_bytes(archive_size_bytes.into());

    println!(
        "The total size of the archive is {}",
        archive_size.get_appropriate_unit(false)
    );
}
