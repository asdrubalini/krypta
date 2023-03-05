use database::Database;

pub async fn debug(_db: &mut Database) {
    vfs::KryptaFS::mount("/Users/giovanni/Desktop/ciao");
}
