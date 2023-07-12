use std::sync::mpsc;
use tokio;

mod async_update;
mod restic;
mod server;

#[tokio::main]
async fn main() {
    let (sender, receiver) = mpsc::channel();

    server::start(receiver);
    async_update::start_metric_updates(sender).await;

    println!("restic metrics updater is up and running!")
}
