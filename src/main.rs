use crate::types::AppServer;

mod app;
mod data;
mod input;
mod server;
mod ssh;
mod types;

#[tokio::main]
async fn main() {
    let mut server = AppServer::default();
    server.run().await.expect("Failed running server");
}
