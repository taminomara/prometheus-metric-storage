use prometheus::Registry;
use prometheus_metric_storage::{MetricStorage, StorageRegistry};

// --- Named struct with basic fields ---

#[derive(MetricStorage)]
struct BasicMetrics {
    /// Number of requests received.
    requests: prometheus::IntCounter,
    /// Current number of active connections.
    connections: prometheus::IntGauge,
}

#[test]
fn basic_named_struct() {
    let registry = Registry::new();
    let m = BasicMetrics::new(&registry).unwrap();
    m.requests.inc();
    assert_eq!(m.requests.get(), 1);
    m.connections.set(42);
    assert_eq!(m.connections.get(), 42);
}

#[test]
fn basic_new_unregistered() {
    let m = BasicMetrics::new_unregistered().unwrap();
    m.requests.inc();
    assert_eq!(m.requests.get(), 1);
}

#[test]
fn basic_const_labels_empty() {
    assert_eq!(BasicMetrics::const_labels(), &[] as &[&str]);
}

// --- Subsystem ---

#[derive(MetricStorage)]
#[metric(subsystem = "http")]
struct SubsystemMetrics {
    /// Total requests.
    requests_total: prometheus::IntCounter,
}

#[test]
fn subsystem_prefixes_metric_name() {
    let registry = Registry::new();
    let m = SubsystemMetrics::new(&registry).unwrap();
    m.requests_total.inc();

    let families = registry.gather();
    let names: Vec<_> = families.iter().map(|f| f.get_name()).collect();
    assert!(
        names.contains(&"http_requests_total"),
        "expected 'http_requests_total' in {:?}",
        names
    );
}

// --- Struct-level labels ---

#[derive(MetricStorage)]
#[metric(labels("endpoint", "region"))]
struct LabeledMetrics {
    /// Requests received.
    requests: prometheus::IntCounter,
}

#[test]
fn struct_level_labels() {
    assert_eq!(LabeledMetrics::const_labels(), &["endpoint", "region"]);

    let registry = Registry::new();
    let m = LabeledMetrics::new(&registry, "0.0.0.0:8080", "us-east").unwrap();
    m.requests.inc();
    assert_eq!(m.requests.get(), 1);
}

#[test]
fn struct_level_labels_different_values_same_registry() {
    let registry = Registry::new();
    let _m1 = LabeledMetrics::new(&registry, "host-a", "us-east").unwrap();
    let _m2 = LabeledMetrics::new(&registry, "host-b", "eu-west").unwrap();
}

// --- Field-level labels (Vec metrics) ---

#[derive(MetricStorage)]
struct VecMetrics {
    /// Requests by status.
    #[metric(labels("status", "method"))]
    requests: prometheus::IntCounterVec,
}

#[test]
fn field_level_labels() {
    let registry = Registry::new();
    let m = VecMetrics::new(&registry).unwrap();
    m.requests.with_label_values(&["200", "GET"]).inc();
    m.requests.with_label_values(&["404", "POST"]).inc();
    assert_eq!(m.requests.with_label_values(&["200", "GET"]).get(), 1);
    assert_eq!(m.requests.with_label_values(&["404", "POST"]).get(), 1);
}

// --- Custom buckets ---

#[derive(MetricStorage)]
struct BucketMetrics {
    /// Request duration.
    #[metric(buckets(0.01, 0.05, 0.1, 0.5, 1.0, 5.0))]
    duration: prometheus::Histogram,
}

#[test]
fn custom_buckets() {
    let registry = Registry::new();
    let m = BucketMetrics::new(&registry).unwrap();
    m.duration.observe(0.03);
    assert_eq!(m.duration.get_sample_count(), 1);
}

// --- Integer buckets ---

#[derive(MetricStorage)]
struct IntBucketMetrics {
    /// Request duration with integer bucket bounds.
    #[metric(buckets(1, 2, 4, 8, 16))]
    duration: prometheus::Histogram,
}

#[test]
fn integer_buckets() {
    let registry = Registry::new();
    let m = IntBucketMetrics::new(&registry).unwrap();
    m.duration.observe(3.0);
    assert_eq!(m.duration.get_sample_count(), 1);
}

// --- Help override ---

#[derive(MetricStorage)]
struct HelpOverrideMetrics {
    /// This doc comment is ignored.
    #[metric(help = "Explicit help message.")]
    counter: prometheus::IntCounter,
}

#[test]
fn help_override() {
    let registry = Registry::new();
    let m = HelpOverrideMetrics::new(&registry).unwrap();
    m.counter.inc();

    let families = registry.gather();
    let family = families.iter().find(|f| f.get_name() == "counter").unwrap();
    assert_eq!(family.get_help(), "Explicit help message.");
}

// --- Doc comment becomes help text ---

#[derive(MetricStorage)]
struct DocHelpMetrics {
    /// Number of items processed.
    items: prometheus::IntCounter,
}

#[test]
fn doc_comment_becomes_help() {
    let registry = Registry::new();
    let _m = DocHelpMetrics::new(&registry).unwrap();

    let families = registry.gather();
    let family = families.iter().find(|f| f.get_name() == "items").unwrap();
    assert_eq!(family.get_help(), "Number of items processed.");
}

// --- Help attribute without doc comment ---

#[derive(MetricStorage)]
struct HelpNoDoccMetrics {
    #[metric(help = "Bytes sent over the wire.")]
    bytes_sent: prometheus::IntCounter,
}

#[test]
fn help_without_doc_comment() {
    let registry = Registry::new();
    let _m = HelpNoDoccMetrics::new(&registry).unwrap();

    let families = registry.gather();
    let family = families
        .iter()
        .find(|f| f.get_name() == "bytes_sent")
        .unwrap();
    assert_eq!(family.get_help(), "Bytes sent over the wire.");
}

// --- Name override ---

#[derive(MetricStorage)]
struct NameOverrideMetrics {
    /// Total events processed.
    #[metric(name = "events_total")]
    my_internal_field: prometheus::IntCounter,
}

#[test]
fn name_override() {
    let registry = Registry::new();
    let m = NameOverrideMetrics::new(&registry).unwrap();
    m.my_internal_field.inc();

    let families = registry.gather();
    let names: Vec<_> = families.iter().map(|f| f.get_name()).collect();
    assert!(
        names.contains(&"events_total"),
        "expected 'events_total' in {:?}",
        names
    );
    assert!(
        !names.contains(&"my_internal_field"),
        "should not contain field name 'my_internal_field'"
    );
}

// --- Tuple struct ---

#[derive(MetricStorage)]
struct TupleMetrics(
    #[metric(name = "tuple_requests", help = "Request counter.")] prometheus::IntCounter,
);

#[test]
fn tuple_struct() {
    let registry = Registry::new();
    let m = TupleMetrics::new(&registry).unwrap();
    m.0.inc();
    assert_eq!(m.0.get(), 1);
}

// --- Unit struct ---

#[derive(MetricStorage)]
struct UnitMetrics;

#[test]
fn unit_struct() {
    let registry = Registry::new();
    let _m = UnitMetrics::new(&registry).unwrap();
}

// --- All metric types ---

#[derive(MetricStorage)]
struct AllTypesMetrics {
    /// Counter metric.
    counter: prometheus::Counter,
    /// IntCounter metric.
    int_counter: prometheus::IntCounter,
    /// Gauge metric.
    gauge: prometheus::Gauge,
    /// IntGauge metric.
    int_gauge: prometheus::IntGauge,
    /// Histogram metric.
    histogram: prometheus::Histogram,
    /// CounterVec metric.
    #[metric(labels("a"))]
    counter_vec: prometheus::CounterVec,
    /// IntCounterVec metric.
    #[metric(labels("a"))]
    int_counter_vec: prometheus::IntCounterVec,
    /// GaugeVec metric.
    #[metric(labels("a"))]
    gauge_vec: prometheus::GaugeVec,
    /// IntGaugeVec metric.
    #[metric(labels("a"))]
    int_gauge_vec: prometheus::IntGaugeVec,
    /// HistogramVec metric.
    #[metric(labels("a"))]
    histogram_vec: prometheus::HistogramVec,
}

#[test]
fn all_metric_types() {
    let registry = Registry::new();
    let m = AllTypesMetrics::new(&registry).unwrap();

    m.counter.inc();
    m.int_counter.inc();
    m.gauge.set(1.5);
    m.int_gauge.set(10);
    m.histogram.observe(0.5);
    m.counter_vec.with_label_values(&["x"]).inc();
    m.int_counter_vec.with_label_values(&["x"]).inc();
    m.gauge_vec.with_label_values(&["x"]).set(2.0);
    m.int_gauge_vec.with_label_values(&["x"]).set(20);
    m.histogram_vec.with_label_values(&["x"]).observe(1.0);
}

// --- StorageRegistry ---

#[derive(MetricStorage)]
struct RegistryMetrics {
    /// Items processed.
    items_processed: prometheus::IntCounter,
}

#[test]
fn storage_registry_creates_and_reuses() {
    let registry = StorageRegistry::default();
    let m1 = RegistryMetrics::instance(&registry).unwrap();
    m1.items_processed.inc();

    let m2 = RegistryMetrics::instance(&registry).unwrap();
    assert_eq!(m2.items_processed.get(), 1);
    assert!(std::ptr::eq(m1, m2));
}

// --- StorageRegistry with labels ---

#[derive(MetricStorage)]
#[metric(labels("env"))]
struct RegistryLabeledMetrics {
    /// Requests.
    requests: prometheus::IntCounter,
}

#[test]
fn storage_registry_with_labels() {
    let registry = StorageRegistry::default();

    let prod = RegistryLabeledMetrics::instance(&registry, "prod").unwrap();
    prod.requests.inc();

    let staging = RegistryLabeledMetrics::instance(&registry, "staging").unwrap();
    staging.requests.inc();

    // Same label returns same instance.
    let prod2 = RegistryLabeledMetrics::instance(&registry, "prod").unwrap();
    assert_eq!(prod2.requests.get(), 1);
    assert!(std::ptr::eq(prod, prod2));

    // Different label has independent counter.
    assert_eq!(staging.requests.get(), 1);
    assert!(!std::ptr::eq(prod, staging));
}

// --- Combined attributes ---

#[derive(MetricStorage)]
#[metric(subsystem = "api", labels("service"))]
struct CombinedMetrics {
    /// Total API calls.
    #[metric(labels("method", "status"))]
    calls: prometheus::IntCounterVec,

    /// API call duration.
    #[metric(buckets(0.01, 0.1, 1.0, 10.0))]
    call_duration: prometheus::Histogram,

    /// Active sessions.
    #[metric(name = "active_sessions_total")]
    sessions: prometheus::IntGauge,
}

#[test]
fn combined_attributes() {
    let registry = Registry::new();
    let m = CombinedMetrics::new(&registry, "auth-service").unwrap();

    m.calls.with_label_values(&["POST", "200"]).inc();
    m.call_duration.observe(0.05);
    m.sessions.set(5);

    let families = registry.gather();
    let names: Vec<_> = families.iter().map(|f| f.get_name()).collect();
    assert!(names.contains(&"api_calls"), "names: {:?}", names);
    assert!(
        names.contains(&"api_call_duration"),
        "names: {:?}",
        names
    );
    assert!(
        names.contains(&"api_active_sessions_total"),
        "names: {:?}",
        names
    );
}

// --- HistogramVec with buckets ---

#[derive(MetricStorage)]
struct HistVecBucketMetrics {
    /// Duration by endpoint.
    #[metric(labels("endpoint"), buckets(0.005, 0.01, 0.025, 0.05, 0.1))]
    duration: prometheus::HistogramVec,
}

#[test]
fn histogram_vec_with_buckets() {
    let registry = Registry::new();
    let m = HistVecBucketMetrics::new(&registry).unwrap();
    m.duration.with_label_values(&["/api"]).observe(0.007);
}

// --- Double registration fails ---

#[derive(MetricStorage)]
struct DoubleRegMetrics {
    /// A counter.
    counter: prometheus::IntCounter,
}

#[test]
fn double_registration_fails() {
    let registry = Registry::new();
    let _m1 = DoubleRegMetrics::new(&registry).unwrap();
    let result = DoubleRegMetrics::new(&registry);
    assert!(result.is_err());
}

// --- Register after new_unregistered ---

#[derive(MetricStorage)]
struct ManualRegMetrics {
    /// A gauge.
    gauge: prometheus::IntGauge,
}

#[test]
fn manual_register_after_new_unregistered() {
    let registry = Registry::new();
    let m = ManualRegMetrics::new_unregistered().unwrap();
    m.gauge.set(99);
    assert_eq!(m.gauge.get(), 99);

    m.register(&registry).unwrap();
    let families = registry.gather();
    let names: Vec<_> = families.iter().map(|f| f.get_name()).collect();
    assert!(names.contains(&"gauge"), "names: {:?}", names);
}
