use prometheus_metric_storage::MetricStorage;

#[derive(MetricStorage)]
struct Metrics {
    /// A counter.
    #[metric(labels("status", "status"))]
    requests: prometheus::IntCounterVec,
}

fn main() {}
