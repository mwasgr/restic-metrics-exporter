use anyhow::{Context, Result};
use chrono::{DateTime, NaiveDateTime, Utc};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_json;
use std::process::Command;

#[derive(Serialize, Deserialize)]
struct JsonSnapshot {
    time: String,
    hostname: String,
    paths: Vec<String>,
    short_id: String,
}

#[derive(Serialize, Deserialize)]
struct JsonSnapshotGroupStats {
    total_size: i64,
}

#[derive(Clone)]
pub struct Snapshot {
    pub time: i64,
    pub host: String,
    pub path: String,
    pub id: String,
}

#[derive(Clone)]
pub struct SnapshotGroup {
    pub host: String,
    pub path: String,
    pub snapshots: Vec<Snapshot>,
}

#[derive(Clone)]
pub struct SnapshotGroupWithDetails {
    pub group: SnapshotGroup,
    pub latest_time: i64,
    pub size: i64,
    pub count: usize,
}

pub trait GroupSnapshots {
    fn to_snapshot_groups(&self) -> Vec<SnapshotGroup>;
}

pub trait GetSnapshotGroupDetails {
    fn get_details(&self) -> Result<SnapshotGroupWithDetails>;
}

impl GroupSnapshots for Vec<Snapshot> {
    fn to_snapshot_groups(&self) -> Vec<SnapshotGroup> {
        self.iter()
            .sorted_by(|a, b| a.host.cmp(&b.host))
            .group_by(|s| &s.host)
            .into_iter()
            .flat_map(|(host, group)| {
                group
                    .sorted_by(|a, b| a.path.cmp(&b.path))
                    .group_by(|g| &g.path)
                    .into_iter()
                    .map(|(path, snapshots)| SnapshotGroup {
                        host: host.clone(),
                        path: path.clone(),
                        snapshots: snapshots.cloned().collect(),
                    })
                    .collect::<Vec<SnapshotGroup>>()
            })
            .collect()
    }
}

impl GetSnapshotGroupDetails for SnapshotGroup {
    fn get_details(&self) -> Result<SnapshotGroupWithDetails> {
        println!(
            "Getting snapshout group details for host \"{:?}\" and path \"{:?}\".",
            self.host, self.path
        );
        let result = run_restic(vec![
            "stats", "--json", "--host", &self.host, "--path", &self.path, "--mode", "raw-data",
        ])?;
        let stats: JsonSnapshotGroupStats = serde_json::from_str(&result)?;
        let min_time = self
            .snapshots
            .iter()
            .map(|s| s.time)
            .min()
            .context("Cannot get minimum")?;

        Ok(SnapshotGroupWithDetails {
            group: self.clone(),
            count: self.snapshots.len(),
            size: stats.total_size,
            latest_time: min_time,
        })
    }
}

pub fn get_all_snapshots() -> Result<Vec<Snapshot>> {
    let result = run_restic(vec!["snapshots", "--json"])?;
    let snapshots: Vec<JsonSnapshot> = serde_json::from_str(&result)?;
    let converted_snapshots = snapshots
        .iter()
        .map(|json_snapshot| Snapshot {
            host: json_snapshot.hostname.clone(),
            id: json_snapshot.short_id.clone(),
            path: json_snapshot.paths.first().unwrap().to_string(),
            time: iso_to_milliseconds(&json_snapshot.time).unwrap(),
        })
        .collect();
    Ok(converted_snapshots)
}

fn run_restic(args: Vec<&str>) -> Result<String> {
    println!("Executing restic command: {:?}", args);
    let output = Command::new("restic").args(args).output()?;
    let result = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(result)
}

fn iso_to_milliseconds(iso_time: &str) -> Result<i64> {
    let datetime: DateTime<Utc> = DateTime::parse_from_rfc3339(iso_time)?.into();
    let naive_datetime: NaiveDateTime = datetime.naive_utc();
    let timestamp_milliseconds: i64 = naive_datetime.timestamp_millis();
    Ok(timestamp_milliseconds)
}
