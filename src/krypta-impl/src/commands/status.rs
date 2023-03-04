use byte_unit::Byte;
use database::{models, traits::Count, Database};

pub async fn status(db: &Database) {
    let archive_size_bytes = models::File::archive_size(db).unwrap();
    let archive_size = Byte::from_bytes(archive_size_bytes.into());

    let archive_count = models::File::count(db).unwrap();

    println!("Files stored in database: {archive_count}");
    println!("Archive size: {}", archive_size.get_appropriate_unit(false));
}
