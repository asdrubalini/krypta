use database::{models, Database};

pub async fn execute(db: &mut Database) -> anyhow::Result<()> {
    let _current_device = models::Device::find_or_create_current(db)?;

    // TODO: compute hashes and compare them to what we have saved into database

    todo!()
}
