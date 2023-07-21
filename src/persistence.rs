use std::fs::File;
use std::io::Read;
use std::{error::Error, io::Write};

use crate::server;
use crate::{environment::get_environment_variable_or, restic::SnapshotGroupWithDetails};

pub fn save(data: Vec<SnapshotGroupWithDetails>) -> Result<(), Box<dyn Error>> {
    let json = serde_json::to_string(&data)?;

    let file_name = get_file_name();
    let mut file = File::create(file_name)?;

    file.write_all(json.as_bytes())?;
    Ok(())
}

pub fn load() -> Result<Vec<SnapshotGroupWithDetails>, Box<dyn Error>> {
    let file_name = get_file_name();
    let mut file = File::open(file_name)?;

    let mut json = String::new();
    file.read_to_string(&mut json)?;

    let data = serde_json::from_str(&json)?;
    Ok(data)
}

pub fn load_and_update_metrics() {
    let snapshot_details = load();
    match snapshot_details {
        Ok(metrics) => server::update_metrics(metrics, "loaded state"),
        Err(err) => println!("Unable to restore last state: {}", err),
    }
}

fn get_file_name() -> String {
    get_environment_variable_or(
        "METRICS_FILE",
        "/tmp/restic-metrics-exporter.json".to_string(),
    )
}
