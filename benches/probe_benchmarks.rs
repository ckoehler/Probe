use criterion::{Criterion, black_box, criterion_group, criterion_main};
use probe_zmq::probe::{
    app::App,
    config::ProbeConfig,
    state::{AppState, Probe},
};

// --- Helper Functions ---

fn create_probe_config(name: &str, filter: Option<&str>) -> ProbeConfig {
    ProbeConfig {
        name: name.to_string(),
        filter: filter.map(String::from),
        address: String::new(), // Address not relevant for these benchmarks
    }
}

fn create_test_probe(name: &str, filter_str: &str) -> Probe {
    Probe::try_from(create_probe_config(name, Some(filter_str))).unwrap()
}

fn create_test_app(num_probes: usize) -> App<'static> {
    let configs: Vec<ProbeConfig> = (0..num_probes)
        .map(|i| create_probe_config(&format!("probe_{}", i), Some(".*")))
        .collect();
    let app_state = AppState::from_probes(&configs);
    App::new("Benchmark App", app_state)
}

// --- Benchmarks ---

fn bench_probe_process_message(c: &mut Criterion) {
    let mut probe_match = create_test_probe("match_probe", "important_event");
    let mut probe_no_match = create_test_probe("no_match_probe", "specific_pattern");
    let matching_message = "This message contains an important_event";
    let non_matching_message = "This is a regular log message";

    c.bench_function("probe_process_message_match", |b| {
        b.iter(|| probe_match.process_message(black_box(matching_message)))
    });

    c.bench_function("probe_process_message_no_match", |b| {
        b.iter(|| probe_no_match.process_message(black_box(non_matching_message)))
    });

    let mut probe_complex_regex = create_test_probe(
        "complex_regex",
        r"(\d{4}-\d{2}-\d{2}) T (\d{2}:\d{2}:\d{2}) Z \[(\w+)\] (.*)",
    );
    let complex_message = "2023-10-27 T 10:00:00 Z [INFO] Complex message structure";
    c.bench_function("probe_process_message_complex_regex", |b| {
        b.iter(|| probe_complex_regex.process_message(black_box(complex_message)))
    });
}

fn bench_app_state_from_probes(c: &mut Criterion) {
    let sizes = [10, 100, 500];

    for size in sizes.iter() {
        c.bench_with_input(
            criterion::BenchmarkId::new("app_state_from_probes", size),
            size,
            |b, &s| {
                let configs: Vec<ProbeConfig> = (0..s)
                    .map(|i| create_probe_config(&format!("probe_{}", i), Some(".*")))
                    .collect();
                b.iter(|| AppState::from_probes(black_box(&configs)));
            },
        );
    }
}

fn bench_app_process_message(c: &mut Criterion) {
    let sizes = [10, 100, 500];
    let message = "Some message content";

    for size in sizes.iter() {
        c.bench_with_input(
            criterion::BenchmarkId::new("app_process_message_for_stream", size),
            size,
            |b, &s| {
                let mut app = create_test_app(s);
                let target_stream = format!("probe_{}", s / 2); // Target a probe in the middle
                b.iter(|| {
                    app.process_message_for_stream(black_box(&target_stream), black_box(message))
                });
            },
        );
    }
}
// --- Criterion Setup ---

criterion_group!(
    benches,
    bench_probe_process_message,
    bench_app_state_from_probes,
    bench_app_process_message
);
criterion_main!(benches);
