use std::error::Error;
use std::sync::mpsc::Sender;

use tokio::time;

use crate::environment::get_environment_variable_or;
use crate::persistence;
use crate::restic::GroupSnapshots;
use crate::restic::{self, SnapshotGroupWithDetails};

const UPDATE_INTERVALL_SECONDS: &str = "UPDATE_INTERVALL_SECONDS";

pub fn start_metric_updates(sender: Sender<Vec<SnapshotGroupWithDetails>>) {
    let seconds = get_environment_variable_or(UPDATE_INTERVALL_SECONDS, 14400);
    tokio::spawn(handle_metric_update_loop(seconds, sender));
}

async fn handle_metric_update_loop(seconds: u64, sender: Sender<Vec<SnapshotGroupWithDetails>>) {
    println!("Updating restic metrics every {} seconds", seconds);

    let duration = time::Duration::from_secs(seconds);

    loop {
        println!("Updating metrics...");
        match update_metrics(sender.clone()) {
            Ok(_) => println!("Metrics updated successfully."),
            Err(err) => {
                println!(
                    "An error occured while attempting to update the metrics: {}",
                    err
                );
            }
        }

        tokio::time::sleep(duration).await;
    }
}

fn update_metrics(sender: Sender<Vec<SnapshotGroupWithDetails>>) -> Result<(), Box<dyn Error>> {
    let snapshots = restic::get_all_snapshots()?;
    let groups = snapshots.to_snapshot_groups();

    println!(
        "Found {} snapshots in {}`groups",
        snapshots.len(),
        groups.len()
    );

    let details: Vec<SnapshotGroupWithDetails> = groups
        .iter()
        .map(|g| g.get_details())
        .collect::<Result<Vec<_>, Box<dyn Error>>>()?;

    match persistence::save(details.clone()) {
        Ok(_) => {}
        Err(err) => {
            println!("Error while persisting metrics state: {}", err);
        }
    }

    sender.send(details)?;

    Ok(())
}
