use std::sync::mpsc;
use tokio;
use tokio::time;

mod async_update;
mod environment;
mod persistence;
mod restic;
mod server;

#[tokio::main]
async fn main() {
    let (sender, receiver) = mpsc::channel();

    server::start(receiver);
    persistence::load_and_update_metrics();

    async_update::start_metric_updates(sender);

    println!("restic metrics updater is up and running!");

    //the http server of the prometheus exporter needs this thread alive or it'll terminate
    keep_alive().await
}

async fn keep_alive() {
    let one_day = time::Duration::from_secs(86400);
    loop {
        tokio::time::sleep(one_day).await;
    }
}
