use prometheus_metric_storage::MetricStorage;

#[derive(MetricStorage)]
struct Metrics {
    counter: prometheus::IntCounter,
}

fn main() {}
