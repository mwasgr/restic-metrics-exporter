use metrics::{describe_gauge, register_gauge};
use metrics_exporter_prometheus::PrometheusBuilder;
use metrics_util::MetricKindMask;
use std::{net::SocketAddr, sync::mpsc::Receiver, time::Duration};

use crate::restic::SnapshotGroupWithDetails;

const SNAPSHOT_TIME_METRIC: &str = "snapshot_time";
const SNAPSHOT_SIZE_METRIC: &str = "snapshot_size";
const SNAPSHOT_COUNT_METRIC: &str = "snapshot_count";
const SNAPSHOT_GROUP_COUNT_METRIC: &str = "snapshot_groups";

const SNAPSHOT_COUNT_TOTAL_METRIC: &str = "snapshot_count_total";
const SNAPSHOT_SIZE_TOTAL_METRIC: &str = "snapshot_size_total";
const SNAPSHOT_TIME_MINIMUM_METRIC: &str = "snapshot_time_minimum";
const SNAPSHOT_TIME_MAXIMUM_METRIC: &str = "snapshot_time_maximum";

pub fn start(receiver: Receiver<Vec<SnapshotGroupWithDetails>>) {
    let listen_address: SocketAddr = "0.0.0.0:9184".parse().expect("");
    let timeout_in_seconds = 24 * 60 * 60;
    let metrics_timeout = Duration::from_secs(timeout_in_seconds);
    let mask = MetricKindMask::ALL;

    let builder = PrometheusBuilder::new();
    builder
        .with_http_listener(listen_address)
        .idle_timeout(mask, Some(metrics_timeout))
        .install()
        .expect("expect server to be startable");

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

    println!("Server started. Listening on {:?}.", listen_address);

    tokio::spawn(handle_metric_updates(receiver));
}

async fn handle_metric_updates(receiver: Receiver<Vec<SnapshotGroupWithDetails>>) {
    loop {
        let snapshot_details = receiver.recv().expect("metrics to be sent continously");

        let snapshout_group_count = register_gauge!(SNAPSHOT_GROUP_COUNT_METRIC);
        let total_snapshot_size = register_gauge!(SNAPSHOT_SIZE_TOTAL_METRIC);
        let total_snapshot_count = register_gauge!(SNAPSHOT_COUNT_TOTAL_METRIC);
        let minimum_snapshot_time = register_gauge!(SNAPSHOT_TIME_MINIMUM_METRIC);
        let maximum_snapshot_time = register_gauge!(SNAPSHOT_TIME_MAXIMUM_METRIC);

        let time_values: Vec<i64> = snapshot_details.iter().map(|d| d.latest_time).collect();
        let min_time = time_values
            .iter()
            .min()
            .expect("A minimum time value must exist");
        let max_time = time_values
            .iter()
            .max()
            .expect("A maximum time value must exist");

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

        println!("Prometheus metrics refreshed successfully.")
    }
}
