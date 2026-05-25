//! S14 Spike: Mermaid Diagram Engine Evaluation
//!
//! Exercises mermaid-rs-renderer (Option A candidate) for all diagram types
//! relevant to slideforge's `slide diagram:` type. Measures render time per
//! diagram type and validates SVG output for PPTX embedding safety.
//!
//! Run with:
//!   cargo run --manifest-path .factory/planning/spikes/S14-code/Cargo.toml
//!
//! What this validates:
//! 1. mermaid-rs-renderer renders the diagram types slideforge users need
//! 2. SVG output is pure SVG (no foreignObject, no HTML labels)
//! 3. SVG output has proper viewBox + dimensions for OOXML embedding
//! 4. Render time is within the < 1s CI budget per diagram

use std::time::Instant;

fn main() {
    println!("S14 Mermaid Spike — mermaid-rs-renderer v0.2.2 evaluation");
    println!("{}", "=".repeat(60));

    let test_cases = test_diagrams();
    let mut results: Vec<RenderResult> = Vec::new();

    for (name, diagram_type, source) in &test_cases {
        let result = evaluate_diagram(name, diagram_type, source);
        print_result(&result);
        results.push(result);
    }

    println!("\n{}", "=".repeat(60));
    println!("SUMMARY");
    println!("{}", "=".repeat(60));
    let passed = results.iter().filter(|r| r.success).count();
    let total = results.len();
    println!("Passed: {passed}/{total}");

    let failures: Vec<&RenderResult> = results.iter().filter(|r| !r.success).collect();
    if !failures.is_empty() {
        println!("\nFailed diagrams:");
        for f in &failures {
            println!("  - {} ({}): {}", f.name, f.diagram_type, f.error.as_deref().unwrap_or("unknown"));
        }
    }

    let pptx_unsafe: Vec<&RenderResult> = results.iter()
        .filter(|r| r.success && !r.pptx_safe)
        .collect();
    if !pptx_unsafe.is_empty() {
        println!("\nPPTX-unsafe SVG (contains foreignObject or missing viewBox):");
        for r in &pptx_unsafe {
            println!("  - {} ({}): {:?}", r.name, r.diagram_type, r.pptx_safety_issues);
        }
    }

    println!("\nRender time budget (< 1000ms each):");
    for r in &results {
        if r.success {
            let budget_ok = if r.render_ms < 1000.0 { "OK" } else { "OVER BUDGET" };
            println!("  {} ({}) — {:.1}ms — {}", r.name, r.diagram_type, r.render_ms, budget_ok);
        }
    }
}

fn evaluate_diagram(name: &str, diagram_type: &str, source: &str) -> RenderResult {
    let start = Instant::now();

    let svg_result = mermaid_rs_renderer::render(source);

    let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;

    match svg_result {
        Ok(svg) => {
            let pptx_issues = check_pptx_safety(&svg);
            let pptx_safe = pptx_issues.is_empty();
            RenderResult {
                name: name.to_string(),
                diagram_type: diagram_type.to_string(),
                success: true,
                render_ms: elapsed_ms,
                svg_bytes: svg.len(),
                pptx_safe,
                pptx_safety_issues: pptx_issues,
                error: None,
            }
        }
        Err(e) => RenderResult {
            name: name.to_string(),
            diagram_type: diagram_type.to_string(),
            success: false,
            render_ms: elapsed_ms,
            svg_bytes: 0,
            pptx_safe: false,
            pptx_safety_issues: vec![],
            error: Some(e.to_string()),
        },
    }
}

/// Check SVG output for PPTX embedding safety.
/// Returns a list of issues found (empty = safe).
fn check_pptx_safety(svg: &str) -> Vec<String> {
    let mut issues = Vec::new();

    // foreignObject is not supported in PowerPoint SVG
    if svg.contains("<foreignObject") || svg.contains("<foreignobject") {
        issues.push("contains <foreignObject> (unsupported in PowerPoint)".to_string());
    }

    // Script elements forbidden
    if svg.contains("<script") {
        issues.push("contains <script> (forbidden in OOXML SVG)".to_string());
    }

    // Animation elements unsupported
    if svg.contains("<animate") || svg.contains("<animateTransform") {
        issues.push("contains SMIL animation (unsupported in PowerPoint)".to_string());
    }

    // viewBox is required for proper scaling in PPTX
    if !svg.contains("viewBox=") && !svg.contains("viewbox=") {
        issues.push("missing viewBox attribute (required for PPTX scaling)".to_string());
    }

    // width and height should be present
    if !svg.contains(" width=") {
        issues.push("missing width attribute".to_string());
    }
    if !svg.contains(" height=") {
        issues.push("missing height attribute".to_string());
    }

    issues
}

fn print_result(r: &RenderResult) {
    let status = if r.success { "PASS" } else { "FAIL" };
    let pptx = if r.pptx_safe { "PPTX-safe" } else { "PPTX-UNSAFE" };
    println!(
        "[{status}] {name} ({dtype}) — {ms:.1}ms — {bytes}B — {pptx}",
        name = r.name,
        dtype = r.diagram_type,
        ms = r.render_ms,
        bytes = r.svg_bytes,
        pptx = pptx,
    );
    if let Some(ref e) = r.error {
        println!("       Error: {e}");
    }
    for issue in &r.pptx_safety_issues {
        println!("       PPTX issue: {issue}");
    }
}

struct RenderResult {
    name: String,
    diagram_type: String,
    success: bool,
    render_ms: f64,
    svg_bytes: usize,
    pptx_safe: bool,
    pptx_safety_issues: Vec<String>,
    error: Option<String>,
}

/// Test diagrams representative of slideforge's slide diagram: use cases.
/// Covers the diagram types most likely to appear in security/incident/
/// architecture decks — the primary slideforge use case.
fn test_diagrams() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        // flowchart — incident response pipeline, top slideforge use case
        (
            "incident-response-flow",
            "flowchart",
            r#"flowchart TD
    A[Detection] -->|23 min| B[Triage]
    B -->|15 min| C[Containment]
    C -->|4 hrs| D[Eradication]
    D -->|24 hrs| E[Recovery]
    E --> F[Lessons Learned]"#,
        ),
        // flowchart — system architecture with subgraphs
        (
            "system-architecture",
            "flowchart",
            r#"flowchart LR
    subgraph Frontend
        UI[Web UI]
        CLI[CLI Tool]
    end
    subgraph Backend
        API[API Server]
        DB[(Database)]
    end
    UI --> API
    CLI --> API
    API --> DB"#,
        ),
        // sequenceDiagram — auth flow, common in security decks
        (
            "auth-sequence",
            "sequenceDiagram",
            r#"sequenceDiagram
    participant Client
    participant API
    participant Auth
    Client->>API: POST /login
    API->>Auth: ValidateCredentials
    Auth-->>API: JWT Token
    API-->>Client: 200 OK + Token"#,
        ),
        // stateDiagram — process state machine
        (
            "ticket-states",
            "stateDiagram-v2",
            r#"stateDiagram-v2
    [*] --> Open
    Open --> InProgress: assign
    InProgress --> Review: submit
    Review --> Done: approve
    Review --> InProgress: reject
    Done --> [*]"#,
        ),
        // gantt — project timeline
        (
            "project-gantt",
            "gantt",
            r#"gantt
    title Project Timeline
    dateFormat YYYY-MM-DD
    section Planning
    Requirements   :a1, 2026-01-01, 7d
    Design         :a2, after a1, 5d
    section Build
    Backend        :crit, b1, after a2, 10d
    Frontend       :b2, after a2, 8d"#,
        ),
        // classDiagram — data model, common in technical decks
        (
            "plugin-class",
            "classDiagram",
            r#"classDiagram
    class DiagramRenderer {
        +id() String
        +render(source, lang) SvgData
        +supported_langs() Vec~DiagramLang~
    }
    class MermaidRenderer {
        -engine: Engine
        +id() String
        +render(source, lang) SvgData
    }
    DiagramRenderer <|-- MermaidRenderer"#,
        ),
        // erDiagram — data model relationships
        (
            "data-model-er",
            "erDiagram",
            r#"erDiagram
    DECK ||--o{ SLIDE : contains
    SLIDE ||--o{ BLOCK : has
    BLOCK ||--o{ INLINE : contains
    DECK {
        string title
        string lang
    }
    SLIDE {
        string slide_type
        int index
    }"#,
        ),
        // pie chart — simple metrics breakdown
        (
            "incident-breakdown",
            "pie",
            r#"pie title Incident Types Q1 2026
    "Phishing" : 45
    "Ransomware" : 20
    "Insider Threat" : 15
    "Supply Chain" : 12
    "Other" : 8"#,
        ),
    ]
}
