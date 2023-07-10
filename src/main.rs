use console::Term;
use std::sync::mpsc;
use tokio;

mod async_update;
mod restic;
mod server;

#[tokio::main]
async fn main() {
    let term = Term::stdout();

    let (sender, receiver) = mpsc::channel();

    server::start(receiver);
    async_update::start_metric_updates(sender).await;

    let _input = term.read_char().unwrap();
}
