use prometheus_metric_storage::MetricStorage;

#[derive(MetricStorage)]
#[metric(labels("env", "env"))]
struct Metrics {
    /// A counter.
    counter: prometheus::IntCounter,
}

fn main() {}
