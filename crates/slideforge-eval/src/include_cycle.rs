//! `@include` cycle detection as a pre-pass over the include graph.
//!
//! # Behavioral Contract
//!
//! BC-1.06.002: Detect and reject circular `@include` chains (Fail-Closed).
//!
//! # Algorithm
//!
//! The cycle detector uses a depth-first search (DFS) with two sets:
//!
//! - `in_progress: HashSet<Arc<str>>` — files currently on the DFS call stack
//!   (back-edge detection).
//! - `completed: HashSet<Arc<str>>` — files already fully validated (diamond
//!   optimization: if we encounter a file that's `completed`, it has already
//!   been proven cycle-free — skip it).
//!
//! A **back-edge** (encounter a file that is `in_progress`) signals a cycle.
//! A **diamond include** (encounter a file that is `completed`) is NOT a cycle
//! — it is valid and must not produce an error.
//!
//! # Fail-Closed Invariant
//!
//! Cycle detection runs as a **pre-pass** on the include graph BEFORE any
//! expression evaluation begins. This ensures no partial evaluation of a
//! cyclic deck can occur. If any E-PAR-004 error is pushed, the evaluator
//! must abort expression evaluation and return `None`.
//!
//! # Include Graph Representation
//!
//! After STORY-008's `@include` inlining, the merged `DeckNode` no longer
//! contains live `@include` directives — they have been spliced out. To run
//! cycle detection at eval time, the include graph (file → included files)
//! must be preserved as metadata alongside the `DeckNode`.
//!
//! This module defines [`IncludeGraph`] as that metadata type.
//! `IncludeGraph` is a `HashMap<Arc<str>, Vec<Arc<str>>>` mapping each source
//! file's canonical path to the list of files it directly includes (in source
//! order).
//!
//! # Example
//!
//! ```text
//! // a.sf includes b.sf; b.sf includes c.sf
//! let mut graph = IncludeGraph::new();
//! graph.insert(Arc::from("a.sf"), vec![Arc::from("b.sf")]);
//! graph.insert(Arc::from("b.sf"), vec![Arc::from("c.sf")]);
//! graph.insert(Arc::from("c.sf"), vec![]);
//! ```

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use slideforge_syntax::DiagnosticSink;
use slideforge_syntax::error::ParseSeverity;
use slideforge_types::SourceSpan;

use crate::error::EvalError;

// ─── IncludeGraph ────────────────────────────────────────────────────────────

/// A map from each source file's canonical path to the list of files it
/// directly includes via `@include`.
///
/// Built by the parser/include-resolver (STORY-008) and passed to the
/// evaluator alongside the merged deck node. The evaluator uses it to run
/// the pre-pass cycle detection before expression evaluation begins.
///
/// The `Arc<str>` keys are canonical file paths (e.g., `"/project/a.sf"`).
/// Values are the direct includes of that file, in source order.
pub type IncludeGraph = HashMap<Arc<str>, Vec<Arc<str>>>;

// ─── check_include_cycles ────────────────────────────────────────────────────

/// Run a DFS cycle-detection pre-pass over the include graph.
///
/// Returns `true` if no cycles were detected (the graph is a DAG), or `false`
/// if at least one cycle was detected and E-PAR-004 was pushed to `sink`.
///
/// # Algorithm
///
/// For each file in the graph that has not been visited:
/// 1. Call `check_node` (recursive DFS).
/// 2. On back-edge: push `EvalError::IncludeCycle` and continue collecting
///    all cycles (not just the first).
///
/// # Performance
///
/// The `completed` set ensures each node is visited at most once
/// (diamond optimization). A graph with `n` files and `e` include edges
/// runs in O(n + e).
///
/// For AC-012: this implementation must handle include chains up to 200
/// files deep without stack overflow. The DFS uses recursion (see
/// `check_node`); the stack depth equals the include-chain depth. For
/// 200-file chains, default Rust thread stacks (8 MB) provide ample
/// headroom — each frame holds O(1) pointer-sized variables. An iterative
/// fallback would only be needed for chains of thousands of files.
///
/// # Parameters
///
/// - `root`: The canonical path of the root file (entry point of the deck).
/// - `graph`: The include graph mapping each file to its direct includes.
/// - `sink`: Accumulates all E-PAR-004 diagnostics found.
///
/// # Returns
///
/// `true` if no cycles detected; `false` if at least one cycle was found.
pub fn check_include_cycles(
    root: &Arc<str>,
    graph: &IncludeGraph,
    sink: &mut DiagnosticSink,
) -> bool {
    let mut in_progress: HashSet<Arc<str>> = HashSet::new();
    let mut completed: HashSet<Arc<str>> = HashSet::new();
    let mut path: Vec<Arc<str>> = Vec::new();
    let mut found_cycle = false;

    check_node(
        root,
        graph,
        &mut in_progress,
        &mut completed,
        &mut path,
        sink,
        &mut found_cycle,
    );

    !found_cycle
}

/// Recursive DFS over the include graph starting from `current`.
///
/// Uses `in_progress` for back-edge detection (cycle) and `completed`
/// for the diamond optimization (skip already-validated subtrees).
///
/// `path` is the ordered DFS stack: the sequence of files currently on the
/// call stack from the root to `current` (inclusive). When a back-edge is
/// found (child is already `in_progress`), we reconstruct the full cycle by
/// slicing `path` from the position of the back-edge target through `current`,
/// then appending the back-edge target again to show the closing edge.
///
/// Example: path = [a.sf, b.sf, c.sf], back-edge to a.sf →
/// cycle = [a.sf, b.sf, c.sf, a.sf] (full path).
///
/// This implementation is straightforward recursion. The stack depth equals
/// the include-chain depth. For AC-012 (200-file chains), default Rust stack
/// frames are small enough that 200-level recursion is safe on all supported
/// platforms (typical Rust thread stack is 8MB; each frame here is O(1)
/// pointer-sized variables). The iterative fallback would only be needed for
/// chains of thousands of files.
fn check_node(
    current: &Arc<str>,
    graph: &IncludeGraph,
    in_progress: &mut HashSet<Arc<str>>,
    completed: &mut HashSet<Arc<str>>,
    path: &mut Vec<Arc<str>>,
    sink: &mut DiagnosticSink,
    found_cycle: &mut bool,
) {
    // Diamond optimization: already fully validated — skip.
    if completed.contains(current.as_ref()) {
        return;
    }

    in_progress.insert(current.clone());
    path.push(current.clone());

    let includes = match graph.get(current.as_ref()) {
        Some(list) => list.as_slice(),
        None => &[],
    };

    for child in includes {
        if in_progress.contains(child.as_ref()) {
            // Back-edge: cycle detected.
            // Reconstruct the full cycle path from `path`. Find where `child`
            // first appears in the current DFS stack, then take everything from
            // that position to the end (= `current`), then append `child` again
            // to show the closing edge.
            //
            // For A→B→C→A: path = [A, B, C], child = A
            //   → position of A = 0
            //   → cycle = [A, B, C, A]   (full path showing all 3 unique nodes)
            //
            // For self-include deck.sf→deck.sf: path = [deck.sf], child = deck.sf
            //   → position of deck.sf = 0
            //   → cycle = [deck.sf, deck.sf]  (AC-010: exactly 2 occurrences)
            let start_pos = path
                .iter()
                .position(|f| f.as_ref() == child.as_ref())
                .unwrap_or(0);
            let mut cycle: Vec<Arc<str>> = path[start_pos..].to_vec();
            cycle.push(child.clone());

            sink.push_with_severity(
                EvalError::IncludeCycle {
                    cycle_path: cycle,
                    span: SourceSpan::default(),
                },
                ParseSeverity::Error,
            );
            *found_cycle = true;
        } else {
            check_node(
                child,
                graph,
                in_progress,
                completed,
                path,
                sink,
                found_cycle,
            );
        }
    }

    path.pop();
    in_progress.remove(current.as_ref());
    completed.insert(current.clone());
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::doc_markdown)]
#[allow(non_snake_case)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use slideforge_syntax::DiagnosticSink;

    use super::*;

    // ─── Test helpers ─────────────────────────────────────────────────────────

    /// Build an `IncludeGraph` from a list of (file, includes) pairs.
    fn make_graph(edges: &[(&str, &[&str])]) -> IncludeGraph {
        let mut graph = HashMap::new();
        for (file, includes) in edges {
            let key: Arc<str> = Arc::from(*file);
            let value: Vec<Arc<str>> = includes.iter().map(|s| Arc::from(*s)).collect();
            graph.insert(key, value);
        }
        graph
    }

    // ─── BC-1.06.002: no cycle — single file ─────────────────────────────────

    /// BC-1.06.002 invariant: single file with no includes → no error.
    ///
    /// Red Gate: fails with `todo!()` until check_include_cycles is implemented.
    #[test]
    fn test_BC_1_06_002_no_cycle_single_file() {
        let root = Arc::from("deck.sf");
        let graph = make_graph(&[("deck.sf", &[])]);
        let mut sink = DiagnosticSink::new();

        let ok = check_include_cycles(&root, &graph, &mut sink);

        assert!(
            ok,
            "single file with no includes must return true (no cycle)"
        );
        assert!(
            sink.is_empty(),
            "single file must produce no diagnostics; got: {:?}",
            sink.errors()
        );
    }

    // ─── BC-1.06.002 postcondition 1: direct cycle → E-PAR-004 ──────────────

    /// BC-1.06.002 postcondition 1 + postcondition 2 / AC-009:
    /// `a.sf → b.sf → a.sf` → E-PAR-004 with cycle path `["a.sf", "b.sf", "a.sf"]`.
    ///
    /// Red Gate: fails with `todo!()` until check_include_cycles is implemented.
    #[test]
    fn test_BC_1_06_002_direct_cycle() {
        let root = Arc::from("a.sf");
        // a.sf includes b.sf; b.sf includes a.sf (cycle)
        let graph = make_graph(&[("a.sf", &["b.sf"]), ("b.sf", &["a.sf"])]);
        let mut sink = DiagnosticSink::new();

        let ok = check_include_cycles(&root, &graph, &mut sink);

        assert!(!ok, "direct cycle a.sf→b.sf→a.sf must return false");
        assert!(!sink.is_empty(), "direct cycle must push E-PAR-004 to sink");
        // Verify error code is E-PAR-004.
        let code = sink.errors()[0]
            .code()
            .map(|c| c.to_string())
            .unwrap_or_default();
        assert_eq!(
            code, "E-PAR-004",
            "cycle detection error must be E-PAR-004; got: {code}"
        );
        // The error message must contain the cycle path.
        let msg = sink.errors()[0].to_string();
        assert!(
            msg.contains("a.sf"),
            "E-PAR-004 message must mention 'a.sf'; got: {msg}"
        );
        assert!(
            msg.contains("b.sf"),
            "E-PAR-004 message must mention 'b.sf'; got: {msg}"
        );
    }

    // ─── BC-1.06.002 postcondition 1: self-include → E-PAR-004 ──────────────

    /// BC-1.06.002 postcondition 1 / AC-010:
    /// `deck.sf → deck.sf` (self-include) → E-PAR-004.
    ///
    /// Red Gate: fails with `todo!()` until check_include_cycles is implemented.
    #[test]
    fn test_BC_1_06_002_self_include() {
        let root = Arc::from("deck.sf");
        // deck.sf includes itself
        let graph = make_graph(&[("deck.sf", &["deck.sf"])]);
        let mut sink = DiagnosticSink::new();

        let ok = check_include_cycles(&root, &graph, &mut sink);

        assert!(
            !ok,
            "self-include deck.sf→deck.sf must return false (cycle)"
        );
        assert!(!sink.is_empty(), "self-include must push E-PAR-004 to sink");
        let code = sink.errors()[0]
            .code()
            .map(|c| c.to_string())
            .unwrap_or_default();
        assert_eq!(
            code, "E-PAR-004",
            "self-include error must be E-PAR-004; got: {code}"
        );
        // Message must mention deck.sf twice (showing the cycle).
        let msg = sink.errors()[0].to_string();
        assert!(
            msg.contains("deck.sf"),
            "self-include message must mention 'deck.sf'; got: {msg}"
        );
        // Count occurrences: "deck.sf → deck.sf" must appear in the message.
        let count = msg.matches("deck.sf").count();
        assert!(
            count >= 2,
            "self-include message must show 'deck.sf' at least twice (A→A); got: {msg}"
        );
    }

    // ─── BC-1.06.002 invariant 3: diamond include → no error ─────────────────

    /// BC-1.06.002 invariant 3 / AC-011:
    /// Diamond include: A→B, A→C, B→D, C→D → no cycle. D is included twice but
    /// via distinct paths — this is valid. No E-PAR-004 must be produced.
    ///
    /// This tests the `completed` set optimization: when the DFS reaches D via
    /// the B-branch, it marks D as `completed`. When it reaches D again via the
    /// C-branch, it must recognize D as `completed` (not `in_progress`) and skip
    /// it — producing no false-positive cycle error.
    ///
    /// Red Gate: fails with `todo!()` until check_include_cycles is implemented.
    #[test]
    fn test_BC_1_06_002_diamond_include_no_cycle() {
        let root = Arc::from("a.sf");
        // A includes B and C; B includes D; C includes D (diamond pattern)
        let graph = make_graph(&[
            ("a.sf", &["b.sf", "c.sf"]),
            ("b.sf", &["d.sf"]),
            ("c.sf", &["d.sf"]),
            ("d.sf", &[]),
        ]);
        let mut sink = DiagnosticSink::new();

        let ok = check_include_cycles(&root, &graph, &mut sink);

        assert!(
            ok,
            "diamond include A→B→D, A→C→D must return true (no cycle); got false"
        );
        assert!(
            sink.is_empty(),
            "diamond include must produce no diagnostics; got: {:?}",
            sink.errors()
        );
    }

    // ─── BC-1.06.002 verification property / AC-012: deep chain no overflow ──

    /// BC-1.06.002 verification property / AC-012:
    /// 50-level include chain with no cycle → no error, no stack overflow.
    ///
    /// The implementation must handle include chains up to 200 files deep
    /// without stack overflow. 50 levels is a reasonable TDD proxy.
    ///
    /// Red Gate: fails with `todo!()` until check_include_cycles is implemented.
    #[test]
    fn test_BC_1_06_002_deep_chain_no_cycle() {
        const DEPTH: usize = 50;

        // Build a linear chain: 0.sf → 1.sf → 2.sf → ... → 49.sf (no cycle)
        let files: Vec<String> = (0..DEPTH).map(|i| format!("{i}.sf")).collect();
        let mut edges: Vec<(String, Vec<String>)> = Vec::new();
        for i in 0..DEPTH {
            let includes = if i + 1 < DEPTH {
                vec![files[i + 1].clone()]
            } else {
                vec![] // last file has no includes
            };
            edges.push((files[i].clone(), includes));
        }

        // Build the graph.
        let mut graph: IncludeGraph = HashMap::new();
        for (file, includes) in &edges {
            let key: Arc<str> = Arc::from(file.as_str());
            let value: Vec<Arc<str>> = includes.iter().map(|s| Arc::from(s.as_str())).collect();
            graph.insert(key, value);
        }

        let root = Arc::from(files[0].as_str());
        let mut sink = DiagnosticSink::new();

        let ok = check_include_cycles(&root, &graph, &mut sink);

        assert!(
            ok,
            "50-level linear chain (no cycle) must return true; got false"
        );
        assert!(
            sink.is_empty(),
            "50-level chain must produce no diagnostics; got: {:?}",
            sink.errors()
        );
    }

    // ─── E-PAR-004 constructible ──────────────────────────────────────────────

    /// Verify EvalError::IncludeCycle is constructible and has the correct
    /// error code E-PAR-004 (structural guard for the error.rs extension).
    ///
    /// This test does NOT invoke eval_if_chain or check_include_cycles.
    /// It exercises only the error variant directly.
    ///
    /// This test will pass even in Red Gate (the error type exists now).
    #[test]
    fn test_BC_1_06_002_include_cycle_error_constructible() {
        use miette::Diagnostic;
        use slideforge_types::SourceSpan;

        let err = EvalError::IncludeCycle {
            cycle_path: vec![Arc::from("a.sf"), Arc::from("b.sf"), Arc::from("a.sf")],
            span: SourceSpan::default(),
        };

        let msg = format!("{err}");
        assert!(
            msg.contains("a.sf"),
            "IncludeCycle message must mention 'a.sf'; got: {msg}"
        );
        assert!(
            msg.contains("b.sf"),
            "IncludeCycle message must mention 'b.sf'; got: {msg}"
        );
        assert!(
            msg.contains("→"),
            "IncludeCycle message must use arrow separator '→'; got: {msg}"
        );

        let code = err.code().map(|c| c.to_string()).unwrap_or_default();
        assert_eq!(
            code, "E-PAR-004",
            "IncludeCycle error code must be E-PAR-004; got: {code}"
        );
    }

    // ─── FINDING-002: 3-node cycle shows full path ───────────────────────────

    /// FINDING-002: A→B→C→A cycle must produce cycle path [A, B, C, A], not [A, C, A].
    ///
    /// The previous implementation emitted [child, current, child] which skipped
    /// intermediate nodes. This test verifies the full path is captured.
    #[test]
    fn test_three_node_cycle_full_path() {
        let root = Arc::from("a.sf");
        // a.sf → b.sf → c.sf → a.sf (3-node cycle)
        let graph = make_graph(&[
            ("a.sf", &["b.sf"]),
            ("b.sf", &["c.sf"]),
            ("c.sf", &["a.sf"]),
        ]);
        let mut sink = DiagnosticSink::new();

        let ok = check_include_cycles(&root, &graph, &mut sink);

        assert!(!ok, "3-node cycle a.sf→b.sf→c.sf→a.sf must return false");
        assert!(!sink.is_empty(), "3-node cycle must push E-PAR-004 to sink");

        let msg = sink.errors()[0].to_string();
        // The cycle path must include all 3 unique nodes, not just 2.
        assert!(
            msg.contains("a.sf") && msg.contains("b.sf") && msg.contains("c.sf"),
            "3-node cycle message must mention all 3 files (a.sf, b.sf, c.sf); got: {msg}"
        );
        // Full cycle: a.sf → b.sf → c.sf → a.sf
        assert!(
            msg.contains("a.sf → b.sf → c.sf → a.sf"),
            "3-node cycle path must be 'a.sf → b.sf → c.sf → a.sf'; got: {msg}"
        );
    }

    /// FINDING-005: Self-include deck.sf→deck.sf must show exactly 2 occurrences
    /// of 'deck.sf' in the message (not 3 as the old [child, current, child] would produce).
    #[test]
    fn test_self_include_exactly_two_nodes() {
        let root = Arc::from("deck.sf");
        let graph = make_graph(&[("deck.sf", &["deck.sf"])]);
        let mut sink = DiagnosticSink::new();

        let ok = check_include_cycles(&root, &graph, &mut sink);

        assert!(!ok, "self-include must return false");
        assert!(!sink.is_empty(), "self-include must push E-PAR-004");

        let msg = sink.errors()[0].to_string();
        let count = msg.matches("deck.sf").count();
        assert_eq!(
            count, 2,
            "self-include message must show 'deck.sf' exactly twice (deck.sf → deck.sf); got: {msg}"
        );
    }

    // ─── Cycle path format matches AC-009 ────────────────────────────────────

    /// AC-009: cycle error message format must be:
    /// `Include cycle detected: a.sf → b.sf → a.sf`
    ///
    /// Verify the exact message format from the BC postcondition 2.
    #[test]
    fn test_BC_1_06_002_cycle_error_message_format() {
        use slideforge_types::SourceSpan;

        let err = EvalError::IncludeCycle {
            cycle_path: vec![Arc::from("a.sf"), Arc::from("b.sf"), Arc::from("a.sf")],
            span: SourceSpan::default(),
        };
        let msg = format!("{err}");
        // Per BC-1.06.002 postcondition 2: format is "Include cycle detected: a.sf → b.sf → a.sf"
        assert!(
            msg.contains("Include cycle detected"),
            "message must start with 'Include cycle detected'; got: {msg}"
        );
        assert!(
            msg.contains("a.sf → b.sf → a.sf"),
            "message must contain cycle path 'a.sf → b.sf → a.sf'; got: {msg}"
        );
    }
}
