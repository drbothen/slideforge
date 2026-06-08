# Fresh-Eyes PR Review — PR #69 (Wave-5 Dependency Prep)

**Verdict: APPROVE**

Branch `chore/wave5-dep-prep` → `develop`. Behavior-preserving dependency
centralization + ADR-022 major-version migrations. 191 additions / 211 deletions
across `Cargo.toml`, `Cargo.lock`, 7 member manifests, and 3 Rust source sites.

This is a clean, well-scoped chore. I reviewed every changed file and verified the
two real migrations (`sha2` 0.10→0.11 hex boundary, `toml` 0.8→1.x parse API) for
behavior preservation. No blocking or warning findings.

---

## Checklist Results

| # | Item | Result |
|---|------|--------|
| 1 | Diff coherence | PASS — every change traces to ADR-022 dep centralization or its mechanical fallout (Cargo.lock dedup). No unrelated edits. |
| 2 | Description accuracy | PASS — PR body matches the diff exactly (toml 1.1.2, sha2 0.11.0, criterion 0.8.2, indexmap 2.14.0, notify 8.2.0, 16 inert catalog entries). |
| 3 | Test coverage | PASS (chore) — existing pptx/math/brand/data tests exercise all migrated call sites; 3497 pass. No new logic introduced. |
| 4 | Demo evidence | N/A — manifest-only chore, no user-facing AC. Acceptable for maintenance mode. |
| 5 | Commit quality | PASS — 2 conventional commits (`chore(deps): ...`); second is a rustfmt follow-up. No AI attribution. |
| 6 | Diff size | PASS — 402 lines, mostly mechanical Cargo.lock churn. Logical surface is ~40 lines of manifest + 2 source sites. |
| 7 | Missing changes | PASS — all member crates that consumed toml/sha2/dirs/indexmap/criterion/tempfile converted to `{ workspace = true }`. None re-pin. |
| 8 | Dependency status | PASS — ADR-021 and ADR-022 cited as accepted upstream. |

---

## Detailed Verifications

### sha2 0.11 hex boundary (CORRECT)
The two migrated sites — `crates/slideforge-pptx/examples/demo_pptx.rs:224` and
`crates/slideforge-pptx/src/tests/core_tests.rs:253` — replace
`format!("{:x}", finalize())` with `let digest: [u8; 32] = finalize().into();` then a
per-byte `write!(hex, "{b:02x}")` loop. This produces byte-for-byte identical lowercase,
zero-padded hex output as the old `LowerHex` impl. The pptx determinism tests only assert
`hash1 == hash2` (no hard-coded digest literal), so even the format change is invisible to
the assertion — and it is correct regardless.

`slideforge-math/src/font_engine.rs` already used the `[u8;32]`-style `write!("{b:02x}")`
loop before this PR (math was already on sha2 0.11), so it needed no hex change — only the
toml `from_str` reformat. Consistent.

### toml 1.x parse API (CORRECT)
`font_engine.rs:507` changed `manifest_text.parse()` →
`toml::from_str(&manifest_text)`. This is the correct toml 1.x fix: `str::parse::<toml::Value>`
in 1.x parses a single *value expression*, not a document, so the old `.parse()` would have
mis-parsed a full TOML document. The inline comment documents the rationale. Brand/data only
deserialize into typed structs via `from_str::<T>`, which is unchanged across 0.8→1.x.

### preserve_order feature propagation (behavior-neutral)
The workspace `toml` entry carries `features = ["preserve_order"]`. On `develop` only
`slideforge-data` had this feature; brand and math had bare `toml = "=0.8.23"`. Both now
inherit `preserve_order`. Verified behavior-neutral: brand builds its output TOML manually
(not `toml::to_string`) and otherwise only deserializes; math only reads `toml::Value` via
keyed access. `preserve_order` affects map iteration/serialization order, which neither path
exercises.

### Inert catalog additions are genuinely inert (VERIFIED)
Confirmed in `Cargo.lock`: none of tokio, reqwest, axum, crossterm, git2, opentelemetry(_sdk/-otlp),
tar, globset, hex, notify-debouncer-full, toml_edit, or `notify` 8.2.0 appear in the resolved
lockfile. Declaring them in `[workspace.dependencies]` without a member referencing them does not
pull them in. They become load-bearing only when a Wave-5 consumer adds `{ workspace = true }`.
(`flate2` already existed in the lock transitively, predating this PR.)

### Pin hygiene (PASS)
All NEW workspace catalog entries use exact 3-component `=` pins (e.g. `sha2 = "=0.11.0"`,
`tokio = "=1.52.3"`, `git2 = "=0.21.0"`). `criterion = "=0.8.2"` and `tempfile = "=3.27.0"`
are exact-pinned even though the dev-only allowlist permits looseness — stricter than required.
The pre-existing loose `insta = "1"` / `pretty_assertions = "1"` entries are unchanged and
explicitly exempted. `toml = "=1.1.2"` correctly matches the lock's `1.1.2+spec-1.1.0` (build
metadata is ignored in semver matching).

### Cargo.lock churn (all explained)
- `thiserror 1.0.69` removed: was pulled only by old `redox_users 0.4.6` (via `dirs-sys 0.4.1`);
  the dirs 5→6 bump moves redox_users to 0.5.2 (thiserror 2.x), eliminating the duplicate.
- `is-terminal` / `hermit-abi 0.5.2` removed, `alloca` + `page_size` + `winapi` added: criterion
  0.5→0.8 dependency-tree changes (dev-only).
- `windows-targets 0.48.x` family removed: dropped with `windows-sys 0.48.0` (old dirs-sys).
- toml internals restructured (`toml_edit`→`toml_parser`/`toml_writer`, winnow 0.7→1.0): toml 1.x tree.

All churn is a direct consequence of the declared bumps. No stray entries.

---

## Findings

| Severity | Category | Finding | Suggestion |
|----------|----------|---------|------------|
| NIT | dependency | `reqwest = "=0.13.4"` is an inert catalog entry, so its version is not validated by compilation in this PR. reqwest's widely-deployed stable line is 0.12.x; 0.13.4 should be confirmed to actually exist/resolve before the first Wave-5 consumer (preview server / package manager) adds `{ workspace = true }`, or that story will hit a resolution failure. | When the first reqwest consumer lands, run `cargo update -p reqwest` and confirm the lock resolves to `=0.13.4`. Same applies to the other inert entries (axum 0.8.9, opentelemetry 0.32.0, git2 0.21.0) — they are unverified until consumed. |
| NIT | dependency | criterion 0.8 pulls in `alloca` (a C-built crate via `cc`) into the dev dependency tree. Dev-only, no production/`forbid(unsafe_code)` impact, but it adds a C-toolchain requirement for `cargo bench`. | Informational only — no action required for this PR. |

No BLOCKING or WARNING findings.

---

## Conclusion

This is a textbook behavior-preserving dependency chore. The two real migrations are correct
and minimally invasive, the workspace centralization is complete and consistent, all pins are
exact 3-component `=`, member crates uniformly use `{ workspace = true }`, the inert catalog
entries are verifiably absent from the lockfile, and the entire Cargo.lock churn is explained
by the declared bumps. Test evidence (3497 pass, fmt/clippy-pedantic/rustdoc clean) is
consistent with a no-logic-change chore. The only findings are two NITs about unverified inert
catalog versions that will be validated when their Wave-5 consumers land.

**APPROVE.**
