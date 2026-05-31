# STORY-021 Lessons Learned

Archived from STATE.md on 2026-05-30. Covers 23-pass LOCAL adversary cascade for STORY-021 (DataSource HTTP cache + E-DAT policy errors).

## Process Lessons

- **AKM compounding novelty confirmed AGAIN (STORY-021, 23 passes).** 23-pass cascade converged at passes 21-22-23 (vs STORY-020: 29, STORY-028: 32). Each pass surfaced 1-5 new defect classes from fresh-context eyes. Real defects continued through Pass 20+; only P21/22/23 were genuinely zero-finding. The pattern is consistent: compounding novelty persists well past pass 10.

- **Display-vs-protocol coupling antipattern (STORY-021 Pass 14→15 structural fix — HIGHEST IMPACT).** Pass 14 added `--offline` hint to HttpError/NetworkError Display. Pass 15 discovered this broke the dispatcher's parsing of those Displays (extract_http_status + strip_prefix). Fix: decouple user-facing Display from inter-layer machine-friendly protocol. Source layer emits canonical machine format (e.g. `[E-DAT-NNN] HTTP {status} from '{url}'`); dispatcher constructs rich Display variant with spec-mandated `--offline` hint. This pattern is REQUIRED for any cross-layer error emission in slideforge-* crates. Eliminated whole regression class.

- **CI clippy version drift (STORY-021).** Local clippy was clean with `-D warnings`. CI uses Rust 1.95 with `-D clippy::pedantic -D clippy::unwrap_used`, surfacing 55 new errors (doc_markdown, unnecessary_literal_bound, uninlined_format_args). Most auto-fixable via `cargo clippy --fix`. Fix: `just check` must mirror CI clippy flags exactly. Include `rustup update stable && cargo clippy --workspace --all-targets --all-features -- -D clippy::pedantic -D clippy::unwrap_used` in the per-story-delivery pre-push checklist.

- **#[non_exhaustive] default for public enums.** Adding #[non_exhaustive] to DataError + DataSourceError required wildcard arms in match statements; implementer handled propagation cleanly. Process: all new public enums in slideforge-* crates should default to #[non_exhaustive] to prevent SemVer breakage on future variant additions.

- **Spec-entity retirement parity (STORY-021 Pass 13→14).** Pass 13 retired E-DAT-014 in code; Pass 14 found spec (error-taxonomy.md) not updated. "Constant retained for SemVer compat" does not excuse missing spec update. Rule: when code retires/deprecates a spec entity, the spec update MUST be in the same fix burst — or listed as an explicit known follow-up with a specific future story anchor before declaring closure.

- **PolicyRejected / SsrfBlocked / PathTraversalBlocked discrimination (STORY-021 Pass 3).** All three share E-DAT-006 but represent distinct policy classes. Label-driven discrimination at the dispatcher boundary (message_is_path_traversal, message_is_file_not_found patterns) is the established pattern. Any new shared-code variant with multiple behavioral sub-classes should follow this discipline from the start.

- **Implementer overclaim verification (TD-VSDD-059 at agent level).** STORY-021 Pass 6 implementer claimed to remove dead code clauses but missed one site; Pass 7 adversary caught it. Going forward: orchestrator should grep for the pattern claimed-removed and confirm zero occurrences before accepting closure.

## Key Structural Changes Shipped

| Pass | Change | Impact |
|------|--------|--------|
| P3 | PolicyRejected variant for E-DAT-006 body-cap (distinct from SsrfBlocked/PathTraversalBlocked) | Correct semantic discrimination |
| P4 | Label-driven IoError vs FileNotFound routing (message_is_file_not_found helper) | Eliminates IoError→FileNotFound over-promotion |
| P5 | #[non_exhaustive] on DataError + DataSourceError | SemVer safety for future variant additions |
| P9 | UTF-8 panic fix in strip_bracket_prefix (DoS vector) | Security correctness |
| P10 | parse_e_dat_code for E-DAT-007..014 granularity | Precise error routing |
| P13 | E-DAT-014 retired at dispatcher boundary | Spec-code alignment |
| P14 | --offline hint per spec (user-facing Display) | Spec compliance |
| P15 | STRUCTURAL FIX: http.rs Display decoupled from inter-layer protocol | Eliminates whole regression class |
