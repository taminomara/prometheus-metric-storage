use prometheus_metric_storage::MetricStorage;

#[derive(MetricStorage)]
#[metric(unknown = "value")]
struct Metrics {
    /// A counter.
    counter: prometheus::IntCounter,
}

fn main() {}
