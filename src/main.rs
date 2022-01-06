#[tokio::main]
async fn main() {
    vault_manager_impl::start().await;
}
