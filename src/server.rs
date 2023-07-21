use metrics::{describe_gauge, register_gauge};
use metrics_exporter_prometheus::PrometheusBuilder;
use metrics_util::MetricKindMask;
use std::net::{IpAddr, Ipv4Addr};
use std::{net::SocketAddr, sync::mpsc::Receiver, time::Duration};

use crate::environment::get_environment_variable_or;
use crate::restic::SnapshotGroupWithDetails;

const SNAPSHOT_TIME_METRIC: &str = "snapshot_time";
const SNAPSHOT_SIZE_METRIC: &str = "snapshot_size";
const SNAPSHOT_COUNT_METRIC: &str = "snapshot_count";
const SNAPSHOT_GROUP_COUNT_METRIC: &str = "snapshot_groups";

const SNAPSHOT_COUNT_TOTAL_METRIC: &str = "snapshot_count_total";
const SNAPSHOT_SIZE_TOTAL_METRIC: &str = "snapshot_size_total";
const SNAPSHOT_TIME_MINIMUM_METRIC: &str = "snapshot_time_minimum";
const SNAPSHOT_TIME_MAXIMUM_METRIC: &str = "snapshot_time_maximum";
const METRIC_TIMEOUT_SECONDS: &str = "METRIC_TIMEOUT_SECONDS";
const LISTEN_ADDRESS: &str = "LISTEN_ADDRESS";

pub fn start(receiver: Receiver<Vec<SnapshotGroupWithDetails>>) {
    register_metrics();
    update_metrics(vec![], "empty default state");
    handle_web_server();
    tokio::spawn(handle_metric_updates(receiver));
}

pub fn update_metrics(snapshot_details: Vec<SnapshotGroupWithDetails>, source: &str) {
    let snapshout_group_count = register_gauge!(SNAPSHOT_GROUP_COUNT_METRIC);
    let total_snapshot_size = register_gauge!(SNAPSHOT_SIZE_TOTAL_METRIC);
    let total_snapshot_count = register_gauge!(SNAPSHOT_COUNT_TOTAL_METRIC);
    let minimum_snapshot_time = register_gauge!(SNAPSHOT_TIME_MINIMUM_METRIC);
    let maximum_snapshot_time = register_gauge!(SNAPSHOT_TIME_MAXIMUM_METRIC);

    let time_values: Vec<i64> = snapshot_details.iter().map(|d| d.latest_time).collect();
    let min_time = time_values.iter().min().unwrap_or(&0);
    let max_time = time_values.iter().max().unwrap_or(&0);

    snapshout_group_count.set(snapshot_details.len() as f64);
    total_snapshot_size.set(0 as f64);
    total_snapshot_count.set(0 as f64);
    minimum_snapshot_time.set(*min_time as f64);
    maximum_snapshot_time.set(*max_time as f64);

    for group in snapshot_details {
        total_snapshot_size.increment(group.size as f64);
        total_snapshot_count.increment(group.count as f64);

        let snapshot_time = register_gauge!(SNAPSHOT_TIME_METRIC, "host" => group.group.host.clone(), "path" => group.group.path.clone());
        let snapshot_size = register_gauge!(SNAPSHOT_SIZE_METRIC, "host" => group.group.host.clone(), "path" => group.group.path.clone());
        let snapshot_count = register_gauge!(SNAPSHOT_COUNT_METRIC, "host" => group.group.host.clone(), "path" => group.group.path.clone());

        snapshot_time.set(group.latest_time as f64);
        snapshot_size.set(group.size as f64);
        snapshot_count.set(group.count as f64);
    }

    println!("Prometheus metrics updated from {}", source);
}

fn register_metrics() {
    describe_gauge!(
        SNAPSHOT_TIME_METRIC,
        "Contains the last time a snapshot has been created for the given host and path."
    );

    describe_gauge!(
        SNAPSHOT_SIZE_METRIC,
        "Contains the size of all raw data for all snapshots for the given host and path."
    );

    describe_gauge!(
        SNAPSHOT_COUNT_METRIC,
        "Contains the number of snapshots for the given host and path."
    );

    describe_gauge!(
        SNAPSHOT_GROUP_COUNT_METRIC,
        "Contains the total number of snapshot groups of host and path."
    );

    describe_gauge!(
        SNAPSHOT_SIZE_TOTAL_METRIC,
        "Contains the total size of all raw data for all snapshots across all groups."
    );

    describe_gauge!(
        SNAPSHOT_COUNT_TOTAL_METRIC,
        "Contains the total number of snapshots across all groups."
    );

    describe_gauge!(
        SNAPSHOT_TIME_MINIMUM_METRIC,
        "Contains minimum (oldest) time across all snapshots of all groups."
    );

    describe_gauge!(
        SNAPSHOT_TIME_MAXIMUM_METRIC,
        "Contains maximum (newest) time across all snapshots of all groups."
    );
}

fn handle_web_server() {
    let listen_address = get_environment_variable_or(
        LISTEN_ADDRESS,
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 80),
    );

    let timeout_in_seconds = get_environment_variable_or(METRIC_TIMEOUT_SECONDS, 24 * 60 * 60);

    let metrics_timeout = Duration::from_secs(timeout_in_seconds);
    let mask = MetricKindMask::ALL;

    let builder = PrometheusBuilder::new()
        .with_http_listener(listen_address)
        .idle_timeout(mask, Some(metrics_timeout));

    builder.install().expect("expect server to be startable");

    println!("Server started. Listening on {}.", listen_address);
}

async fn handle_metric_updates(receiver: Receiver<Vec<SnapshotGroupWithDetails>>) {
    loop {
        let snapshot_details = receiver.recv().expect("metrics to be sent continously");
        update_metrics(snapshot_details, "periodic update");
    }
}
