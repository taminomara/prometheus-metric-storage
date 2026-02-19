use prometheus_metric_storage::MetricStorage;

#[derive(MetricStorage)]
#[metric(subsystem = "a", subsystem = "b")]
struct Metrics {
    /// A counter.
    counter: prometheus::IntCounter,
}

fn main() {}
