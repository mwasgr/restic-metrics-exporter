use std::sync::mpsc::Sender;

use anyhow::Result;
use tokio::time;

use crate::restic::GroupSnapshots;
use crate::restic::{self, GetSnapshotGroupDetails, SnapshotGroupWithDetails};

pub async fn start_metric_updates(sender: Sender<Vec<SnapshotGroupWithDetails>>) {
    let seconds = 900;
    tokio::spawn(handle_metric_update_loop(seconds, sender));
}

async fn handle_metric_update_loop(seconds: u64, sender: Sender<Vec<SnapshotGroupWithDetails>>) {
    println!("Updating restic metrics every {:?} seconds", seconds);

    let duration = time::Duration::from_secs(seconds);

    loop {
        println!("Updating metrics...");
        let result = update_metrics(sender.clone());
        match result {
            Ok(_) => println!("Metrics updated successfully."),
            Err(_) => println!("An error occured while attempting to update the metrics."),
        }

        tokio::time::sleep(duration).await;
    }
}

fn update_metrics(sender: Sender<Vec<SnapshotGroupWithDetails>>) -> Result<()> {
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
        .collect::<Result<Vec<_>>>()?;

    sender.send(details)?;

    Ok(())
}
