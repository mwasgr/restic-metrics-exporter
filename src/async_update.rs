use std::env;
use std::error::Error;
use std::sync::mpsc::Sender;

use tokio::time;

use crate::restic::GroupSnapshots;
use crate::restic::{self, SnapshotGroupWithDetails};

const UPDATE_INTERVALL_SECONDS: &str = "UPDATE_INTERVALL_SECONDS";

pub async fn start_metric_updates(sender: Sender<Vec<SnapshotGroupWithDetails>>) {
    let seconds_text = match env::var(UPDATE_INTERVALL_SECONDS) {
        Ok(v) => v,
        Err(_) => {
            println!("No update time defined in environment variable 'UPDATE_INTERVALL_SECONDS'. Using default update intervall of 4 hours.");
            "14400".to_string()
        }
    };
    let seconds = match seconds_text.parse::<u64>() {
        Ok(v) => v,
        Err(_) => {
            println!("Could not convert {:?} to seconds (only integer values are allowed). Using default update intervall of 4 hours.", seconds_text);
            14400
        }
    };

    tokio::spawn(handle_metric_update_loop(seconds, sender));
}

async fn handle_metric_update_loop(seconds: u64, sender: Sender<Vec<SnapshotGroupWithDetails>>) {
    println!("Updating restic metrics every {:?} seconds", seconds);

    let duration = time::Duration::from_secs(seconds);

    loop {
        println!("Updating metrics...");
        match update_metrics(sender.clone()) {
            Ok(_) => println!("Metrics updated successfully."),
            Err(err) => {
                let error_text = err.to_string();
                println!(
                    "An error occured while attempting to update the metrics: {:?}",
                    error_text
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
        "Found {:?} snapshots in {:?}`groups",
        snapshots.len(),
        groups.len()
    );

    let details: Vec<SnapshotGroupWithDetails> = groups
        .iter()
        .map(|g| g.get_details())
        .collect::<Result<Vec<_>, Box<dyn Error>>>()?;

    sender.send(details)?;

    Ok(())
}
