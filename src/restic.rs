use chrono::{DateTime, ParseError, Utc};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_json;
use std::error::Error;
use std::fmt;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct ResticError {
    command: String,
    error_message: String,
}

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

#[derive(Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub time: i64,
    pub host: String,
    pub path: String,
    pub id: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SnapshotGroup {
    pub host: String,
    pub path: String,
    pub snapshots: Vec<Snapshot>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SnapshotGroupWithDetails {
    pub group: SnapshotGroup,
    pub latest_time: i64,
    pub earliest_time: i64,
    pub size: i64,
    pub count: usize,
}

impl fmt::Display for ResticError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "Restic command '{}' failed with error message '{}'",
            self.command, self.error_message
        )
    }
}

impl Error for ResticError {}

pub trait GroupSnapshots {
    fn to_snapshot_groups(&self) -> Vec<SnapshotGroup>;
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

impl SnapshotGroup {
    pub fn get_details(&self) -> Result<SnapshotGroupWithDetails, Box<dyn Error>> {
        println!(
            "Getting snapshout group details for host \"{}\" and path \"{}\".",
            self.host, self.path
        );
        let result = run_restic(vec![
            "stats", "--json", "--host", &self.host, "--path", &self.path, "--mode", "raw-data",
        ])?;
        let stats: JsonSnapshotGroupStats = serde_json::from_str(&result)?;
        let min_time = Self::get_snapshot_time(&self.snapshots, true)?;
        let max_time = Self::get_snapshot_time(&self.snapshots, false)?;

        Ok(SnapshotGroupWithDetails {
            group: self.clone(),
            count: self.snapshots.len(),
            size: stats.total_size,
            earliest_time: min_time,
            latest_time: max_time,
        })
    }

    fn get_snapshot_time(snapshots: &Vec<Snapshot>, min: bool) -> Result<i64, Box<dyn Error>> {
        let times = snapshots.iter().map(|s| s.time);
        let result = match min {
            true => times.min(),
            false => times.max(),
        };
        match result {
            Some(v) => Ok(v),
            None => Err("Cannot get snapshot time value".into()),
        }
    }
}

pub fn get_all_snapshots() -> Result<Vec<Snapshot>, Box<dyn Error>> {
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

fn run_restic(args: Vec<&str>) -> Result<String, Box<dyn Error>> {
    println!("Executing restic command: {}", args.join(" "));
    let output = Command::new("restic").args(&args).output()?;
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    if !stderr.is_empty() {
        return Err(Box::new(ResticError {
            command: args.join(" "),
            error_message: stderr,
        }));
    }

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(stdout)
}

fn iso_to_milliseconds(iso_time: &str) -> Result<i64, ParseError> {
    let datetime: DateTime<Utc> = DateTime::parse_from_rfc3339(iso_time)?.into();
    let timestamp_milliseconds: i64 = datetime.timestamp_millis();
    Ok(timestamp_milliseconds)
}
