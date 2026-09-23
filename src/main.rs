use crate::types::AppServer;

pub mod app;
mod games;
mod input;
mod server;
mod ssh;
mod types;

#[tokio::main]
async fn main() {
    let mut server = AppServer::default();
    server.run().await.expect("Failed running server");
}
