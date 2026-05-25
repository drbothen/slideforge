//! Criterion benchmark for S14 — Mermaid render time per diagram type.
//! Run with:
//!   cargo bench --manifest-path .factory/planning/spikes/S14-code/Cargo.toml

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

const FLOWCHART_SIMPLE: &str = r#"flowchart LR
    A[Detection] --> B[Triage]
    B --> C[Containment]
    C --> D[Recovery]"#;

const FLOWCHART_MEDIUM: &str = r#"flowchart TD
    A[Detection] -->|23 min| B[Triage]
    B -->|15 min| C[Containment]
    C -->|4 hrs| D[Eradication]
    D -->|24 hrs| E[Recovery]
    E --> F[Lessons Learned]
    B --> G[Escalate SEV-1]
    G --> H[Engage CISO]
    H --> C"#;

const SEQUENCE: &str = r#"sequenceDiagram
    participant Client
    participant API
    participant Auth
    Client->>API: POST /login
    API->>Auth: ValidateCredentials
    Auth-->>API: JWT Token
    API-->>Client: 200 OK + Token"#;

const GANTT: &str = r#"gantt
    title Project Timeline
    dateFormat YYYY-MM-DD
    section Planning
    Requirements   :a1, 2026-01-01, 7d
    Design         :a2, after a1, 5d
    section Build
    Backend        :crit, b1, after a2, 10d
    Frontend       :b2, after a2, 8d"#;

fn bench_render(c: &mut Criterion) {
    let mut group = c.benchmark_group("mermaid_render");

    let diagrams = [
        ("flowchart_simple", FLOWCHART_SIMPLE),
        ("flowchart_medium", FLOWCHART_MEDIUM),
        ("sequence", SEQUENCE),
        ("gantt", GANTT),
    ];

    for (name, source) in diagrams {
        group.bench_with_input(BenchmarkId::new("mmdr", name), source, |b, source| {
            b.iter(|| {
                mermaid_rs_renderer::render(source).expect("render should succeed");
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_render);
criterion_main!(benches);
