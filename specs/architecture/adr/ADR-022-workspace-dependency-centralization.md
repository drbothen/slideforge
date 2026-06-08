---
document_type: adr
adr_id: ADR-022
title: Workspace dependency centralization and major-version adoption with merged-crate migration plan
status: accepted
date: 2026-06-07
subsystems_affected:
  - SS-01
  - SS-02
  - SS-03
  - SS-04
  - SS-05
  - SS-06
  - SS-07
  - SS-08
  - SS-09
  - SS-10
  - SS-11
  - SS-12
  - SS-13
  - SS-14
  - SS-15
  - SS-16
  - SS-17
  - SS-18
supersedes: null
superseded_by: null
related_adrs:
  - ADR-001
  - ADR-003
  - ADR-008
  - ADR-021
traces_to:
  - ARCH-INDEX.md
  - .factory/specs/prd-supplements/nfr-catalog.md
---

# ADR-022: Workspace Dependency Centralization and Major-Version Adoption with Merged-Crate Migration Plan

## Context

The slideforge workspace currently has two dependency hygiene problems:

**Problem 1 — Decentralized pins.** Individual member crate `Cargo.toml` files re-declare
dependencies with varying precision: some use `=`-pinned versions, others use loose
`"^1.0"` compatible-version strings. `[workspace.dependencies]` exists but is incomplete.
When two member crates both declare `serde = "1.0"` directly (without `workspace = true`),
cargo resolves them consistently only by luck — a future crate could introduce a
conflicting constraint. The production-grade default requires a single source of truth:
every dependency that appears in more than one member crate MUST be declared in
`[workspace.dependencies]` with `=` pinning.

**Problem 2 — Stale major versions.** The Wave-5 remove-uncertainty pass (2026-06-07)
revealed that several workspace dependencies are one or two major versions behind the
current stable release. Stale major versions carry known API limitations and, in some
cases, RUSTSEC advisories against older minors. Two of these staleness issues directly
affect merged crates (slideforge-pptx and slideforge-data/brand/math): sha2 0.10 is in
the pptx GUID derivation path (STORY-082) while sha2 0.11 is already used by
slideforge-math — the workspace currently compiles two incompatible sha2 major versions.
toml 0.8 is used by data/brand/math while toml 1.x is the current stable with a
significantly restructured API.

This ADR records the canonical version pins, establishes the centralization rule, and
documents migration tasks for already-merged code that must transition to the new majors.

## Decision

### Decision 1: All multi-crate dependencies centralized in [workspace.dependencies]

Every production dependency used by more than one workspace member MUST be declared in
`[workspace.dependencies]` with a `=`-pinned version. Member `Cargo.toml` files MUST
use `{ workspace = true }` for these entries and MUST NOT re-pin with a looser version.
Dev-only and single-crate dependencies may remain local but SHOULD be hoisted for
visibility. `cargo deny` enforces: warn on non-workspace deps that duplicate a workspace
entry.

### Decision 2: Canonical pinned version table

All versions below are registry-verified stable releases as of 2026-06-07. MSRV ≤ 1.88
confirmed for every entry.

#### Already-present crates — version corrections

These crates are already in use; the workspace pins are updated to the correct values.

| Crate | Current pin | Correct pin | Notes |
|-------|-------------|-------------|-------|
| clap | =4.5 (loose) | =4.6.1 | No API changes; update pin |
| miette | =7.2 (loose) | =7.6.0 | No breaking changes in 7.x |
| tracing | =0.1 (loose) | =0.1.44 | Pin to exact minor |
| tracing-subscriber | =0.3 (loose) | =0.3.23 | features: env-filter |
| thiserror | =1.0 (loose) | =2.0.18 | thiserror 2.x — review callsites |
| serde | =1.0 (loose) | =1.0.228 | Pin to exact minor |
| serde_json | =1.0 (loose) | =1.0.150 | Pin to exact minor |
| axum | =0.8.1 | =0.8.9 | 0.8.2 yanked; 0.8.9 is safe minimum (see ADR-021) |
| tokio | =1.38 | =1.52.3 | (see ADR-021) |

#### New crates entering the workspace

| Crate | Pin | Features | Used In | Notes |
|-------|-----|----------|---------|-------|
| reqwest | =0.13.4 | default-features=false, rustls-tls+charset+http2 | slideforge-data | Replaces ureq for async HTTP (ADR-021) |
| crossterm | =0.29.0 | event-stream | slideforge-cli | Async keypress (ADR-021) |
| tokio-tungstenite | =0.29.0 | — | slideforge-preview (dev-only) | WS test client only |
| toml_edit | =0.25.12 | — | slideforge-config, slideforge-package | Structure-preserving TOML edits |
| notify-debouncer-full | =0.7.0 | — | slideforge-cli | Pairs with notify =8.2.0 |
| globset | =0.4.18 | — | slideforge-package, slideforge-config | Workspace-member glob discovery |
| dirs | =6.0.0 | — | slideforge-cli, slideforge-package | home_dir() + .slideforge/cache |
| hex | =0.4.3 | — | slideforge-pptx, slideforge-package | Hex encoding for GUIDs + checksums |
| tempfile | =3.27.0 | — | slideforge-package, tests | Temp dir/file management |

#### Major-version upgrades (merged-crate migration required)

| Crate | Old pin | New pin | Migration Tasks (see below) |
|-------|---------|---------|------------------------------|
| sha2 | =0.10.9 / =0.11.0 (split) | =0.11.0 | Unify; migrate slideforge-pptx GUID code |
| toml | =0.8.23 | =1.1.2 | Migrate slideforge-data, slideforge-brand, slideforge-math |
| notify | =6.1.1 | =8.2.0 | Migrate slideforge-cli watch-mode (EventHandler API change) |
| git2 | (not yet added) | =0.21.0 | default-features=false, vendored-libgit2 |
| tar | (not yet added) | =0.4.46 | Reproducible .tar.gz (see constraints) |
| flate2 | (not yet added) | =1.1.9 | YANKED check: 1.1.6 and 1.1.7 are yanked — 1.1.9 is safe |

### Decision 3: sha2 unification on 0.11.0

The workspace currently compiles two incompatible sha2 majors:
- `slideforge-math`: already on `=0.11.0`
- `slideforge-pptx`: on `=0.10.9` (STORY-082 GUID derivation code)

This is not acceptable — duplicate majors in the dep tree mean two copies of sha2 code
in the binary, inflated binary size, and potential confusion if both are resolved via
`Cargo.lock`. Unified pin: `sha2 = "=0.11.0"`.

**Migration impact for slideforge-pptx**: sha2 0.11 (digest 0.11) replaces
`generic-array::GenericArray<u8, U32>` with `hybrid_array::Array<u8, U32>` as the
digest output type. The STORY-082 GUID derivation code in `slideforge-pptx` uses
`sha2::Sha256::digest(name.as_bytes())` to produce the raw digest bytes. At the
`sha2`/`digest` boundary:

```rust
// sha2 0.10 pattern (OLD — must change):
use sha2::{Sha256, Digest};
let result: GenericArray<u8, U32> = Sha256::digest(name.as_bytes());
let bytes: &[u8] = result.as_slice();

// sha2 0.11 pattern (NEW — correct):
use sha2::{Sha256, Digest};
let result = Sha256::digest(name.as_bytes());  // returns hybrid_array::Array<u8, U32>
let bytes: [u8; 32] = result.into();           // decouple from array crate at the boundary
```

The `into()` call at the boundary converts the `hybrid_array::Array` to a plain
`[u8; 32]`, decoupling the GUID derivation logic from the concrete array crate type.
This is the correct pattern — slideforge code must not hold `hybrid_array::Array`
values past the digest call. Any function that was typed as returning
`GenericArray<u8, U32>` must be updated to return `[u8; 32]`.

### Decision 4: toml migration to 1.1.2

toml 1.x is two major versions ahead of 0.8.x with a restructured API. The significant
changes affecting slideforge:

- The top-level `toml::from_str` and `toml::to_string` functions are preserved and
  work identically for `#[derive(Deserialize/Serialize)]` structs — these call sites
  require no changes.
- `toml::Value` serialization/deserialization is preserved.
- `toml::Table` replaces direct `HashMap<String, Value>` usage (toml 1.x uses a
  dedicated ordered map type).

**sf.lock reproducibility constraint**: The `sf.lock` lockfile (slideforge-package)
must be byte-deterministic across platforms. This requires serializing from SORTED Rust
structs (`Vec<PackageEntry>` sorted by name), NOT from `toml::Value` intermediate
representations which do not guarantee key ordering. `HashMap` fields in lockfile structs
must be replaced with `BTreeMap` or `Vec<(String, T)>` sorted before serialization.

**`#[serde(flatten)]` with `Option<table>` + flattened scalars is INCONCLUSIVE in toml 1.x.**
The interaction between `serde(flatten)`, optional TOML tables, and scalar fields
at the same level is known to produce inconsistent behavior in toml 1.x. Before
committing to any flatten layout in lockfile or manifest structs, a serialize
round-trip unit test against `toml = "=1.1.2"` is REQUIRED. If the test fails or
produces non-deterministic output, fall back to explicit nesting (no flatten).
This is a binding constraint, not a suggestion.

### Decision 5: notify migration to 8.2.0

notify 7 (released between 6.x and 8.x) introduced the `EventHandler` trait.
notify 8.2.0 preserves the trait; closures still work via the blanket `impl EventHandler
for F where F: FnMut(notify::Result<Event>)`.

Migration from 6.1.1 to 8.2.0:

- `crossbeam` feature renamed to `crossbeam-channel` (now default-off in 8.x). If
  the slideforge-cli watch code uses `crossbeam-channel` from notify, the feature
  must be explicitly enabled or the channel usage replaced with `std::sync::mpsc`.
- Event types are re-exported from `notify-types` in 8.x. Import paths change:
  `notify::event::EventKind` → still accessible via `notify::EventKind` (re-export
  preserved). Check all `use notify::event::*` imports.
- Queue overflow: notify 8.x surfaces buffer overflow as `EventKind::Other` rather
  than a dedicated `Overflow` variant. The watch-mode handler must treat `EventKind::Other`
  as "trigger full rescan, not just incremental rebuild."
- `notify-debouncer-full = "=0.7.0"` pairs with notify 8.2.0. The debouncer API is
  stable across this pair.

### Decision 6: git2 constraints for slideforge-package

`git2 = { version = "=0.21.0", default-features = false, features = ["vendored-libgit2", "https"] }`

`default-features = false` disables the `ssh` feature, which would otherwise pull in
`libssh2` → `OpenSSL` — a chain that breaks musl static builds and Windows cross-compile.
`vendored-libgit2` statically links libgit2, eliminating the need for a system libgit2
installation (required for reproducible cross-platform builds).

**CRITICAL — no shallow/depth clone in libgit2 0.21.x**: libgit2 and its git2-rs binding
0.21 have no usable shallow/depth clone. The `fetch_options().depth()` API exists in
newer libgit2 versions but is NOT available in the libgit2 version vendored by git2 0.21.
Designing `slideforge-package` git fetch around `--depth 1` or any shallow clone will
fail silently or panic at runtime. The correct fetch design:

```rust
// Correct: full clone, then checkout specific ref
let repo = git2::build::RepoBuilder::new()
    .fetch_options(fetch_options)
    .clone(url, path)?;
repo.set_head_detached(oid)?;
let obj = repo.find_object(oid, None)?;
repo.checkout_tree(&obj, Some(checkout_opts))?;

// For annotated tags: peel to commit
let tag_obj = repo.revparse_single("refs/tags/v1.0.0")?;
let commit = tag_obj.peel_to_commit()?;
```

Annotated tags must be peeled via `Object::peel_to_commit()` — `repo.find_commit(oid)`
on a tag OID returns `ErrorCode::NotFound`.

### Decision 7: tar + flate2 reproducible .tar.gz recipe

For cross-platform reproducible tar archives (package cache, build artifacts):

```toml
tar    = "=0.4.46"
flate2 = "=1.1.9"
# flate2 1.1.6 and 1.1.7 are YANKED. 1.1.9 is the safe minimum.
```

Reproducible archive recipe (mandatory — `append_path` is FORBIDDEN):

```rust
// Always build Header manually to control metadata
fn add_entry(builder: &mut tar::Builder<impl Write>, path: &str, data: &[u8]) {
    let mut header = tar::Header::new_gnu();
    header.set_path(path).unwrap();
    header.set_size(data.len() as u64);
    header.set_mode(if path.ends_with('/') { 0o755 } else { 0o644 });
    header.set_mtime(0);               // deterministic: epoch 0
    header.set_uid(0);
    header.set_gid(0);
    header.set_username("").unwrap();  // empty user/group names
    header.set_groupname("").unwrap();
    header.set_cksum();
    builder.append(&header, data).unwrap();
}
// Entries MUST be appended in lexicographic order.

// GzBuilder with reproducible metadata
use flate2::write::GzEncoder;
use flate2::GzBuilder;
use flate2::Compression;

let gz = GzBuilder::new()
    .mtime(0)
    .filename("")
    .comment("")
    .write(output_writer, Compression::new(6));
let mut archive = tar::Builder::new(gz);
// ... append entries in sorted order ...
archive.finish()?;
```

**flate2 OS-byte hazard**: flate2 provides no API to set gzip header byte 9 (the OS
byte). Different OS builds produce byte 9 = 0x03 (Unix), 0x0B (NTFS), 0x07 (Mac).
This causes a cross-platform byte-diff even with identical content. Mitigation:
add a CI byte-diff test comparing archives built on Linux and macOS runners; if byte 9
diverges, post-process the gzip header: `output[9] = 0xFF` (unknown OS). Pin the same
flate2 backend on all hosts.

### Decision 8: dirs usage

`dirs = "=6.0.0"` — use `dirs::home_dir()` and manually join `.slideforge/cache`:

```rust
// Correct:
let cache_dir = dirs::home_dir()
    .ok_or(PackageError::NoCacheDir)?
    .join(".slideforge")
    .join("cache");

// WRONG — do not use:
dirs::cache_dir()  // Returns ~/Library/Caches on macOS, ~/.cache on Linux,
                   // %APPDATA%\Local on Windows — OS-divergent paths
```

`dirs::home_dir()` returns the same base path family across platforms; manual join of
`.slideforge/cache` produces a consistent, user-discoverable location.

### Decision 9: globset over glob

`globset = "=0.4.18"` is the workspace glob library for workspace-member directory
discovery (e.g., `crates/*` in workspace manifests, `.sfpackages/**/*.sf`).

`glob = "=0.3.3"` is low-maintenance (last meaningful release 2019), lacks brace
expansion (`{a,b,c}` patterns), and returns results in undefined order. It is NOT
added to the workspace. Existing code using `glob` should migrate to `globset` +
`walkdir` for directory traversal.

### Decision 10: hex encoding

`hex = "=0.4.3"` — no RUSTSEC advisory. Used for hex encoding in GUID strings and
package checksums. Note: `base16ct` (from the RustCrypto ecosystem) is a
RustCrypto-aligned alternative with MSRV 1.60 and active maintenance. Migrating to
`base16ct` is a polish-level improvement, not required for v1.0.

## Migration Tasks

These tasks touch already-merged code. Each is a regression risk that requires the full
workspace test suite (at minimum, the affected crate's test suite) as the safety net.
Each migration is executed in-scope when the first story consuming the new version lands.

| Migration | Scope | Safety Net | Story Anchor |
|-----------|-------|------------|--------------|
| slideforge-pptx: sha2 =0.10.9 → =0.11.0 | `src/sections.rs` GUID derivation; change `GenericArray` callsites to `[u8; 32]` via `.into()` | `cargo nextest run -p slideforge-pptx --no-fail-fast` + snapshot tests | STORY-082 or first pptx story consuming sha2 0.11 |
| slideforge-data, slideforge-brand, slideforge-math: toml =0.8.23 → =1.1.2 | `Cargo.toml` dep update; audit `toml::Value` / `toml::Table` usage; verify `from_str`/`to_string` callsites; add serialize round-trip test for any flattened struct before committing | `cargo nextest run -p slideforge-data -p slideforge-brand -p slideforge-math --no-fail-fast` | First story touching toml I/O in data/brand/math |
| slideforge-cli: notify =6.1.1 → =8.2.0 + notify-debouncer-full =0.7.0 | `crossbeam-channel` feature gate; `EventKind::Other` overflow handling; import path audit | `cargo nextest run -p slideforge-cli --no-fail-fast` | STORY-056 (watch-mode) |

## Rationale

**Why centralize in [workspace.dependencies]?** Decentralized pinning allows drift:
one crate pins `serde = "=1.0.200"` while another pins `serde = "1.0"`. Cargo resolves
these to the same version today, but a future `serde` minor release could surface an
incompatibility in one crate before the other is updated. `[workspace.dependencies]`
with `workspace = true` in member crates eliminates the ambiguity — there is exactly one
version in the workspace, everywhere, always.

**Why perform major-version upgrades now?** sha2 0.10 + 0.11 are already compiled
simultaneously (two crates in the dep tree with conflicting majors). This is not a
theoretical future problem — it is the current state. The correct resolution is
unification on 0.11. toml 0.8 → 1.x is a planned upgrade with known migration guidance
(from_str/to_string are preserved; the breaking change is in advanced features we need
to audit). Performing the upgrade now, before more stories land with toml 0.8 patterns,
is cheaper than deferring until Wave 6 with 10 more merged crates.

**Why not migrate toml immediately (without a story anchor)?** The toml migration
touches `slideforge-data` (CSV/JSON/TOML data source loading), `slideforge-brand`
(TOML brand config), and `slideforge-math` (TOML-driven math config). These have
non-trivial serialization logic. Performing the migration "in the background" without
test coverage is exactly the kind of invisible regression that the production-grade
default prohibits. The migration MUST happen with the test suite as the safety net, in
scope of the first story that would otherwise be implementing new toml 0.8 patterns.

**Why git2 vendored-libgit2?** slideforge targets macOS arm64+x86_64, Linux x86_64+arm64,
and Windows x86_64 from a single CI matrix. System libgit2 availability varies per OS
and Linux distro. Vendored libgit2 produces a self-contained binary on all targets.
The musl static binary (Linux) has no dynamic linker — vendored is the only viable
option. `default-features = false` to drop ssh/libssh2/OpenSSL is critical for the
musl build; HTTPS-only package fetch is an acceptable scope constraint for v1.0
(STORY-076 deep-link packages are an SSH-free design).

## Consequences

### Positive

- Single version per dep across all workspace members — `[workspace.dependencies]`
  becomes the canonical dep manifest (aligns with dependency-graph.md as the SoT).
- sha2 duplication resolved: binary size reduced; Cargo.lock simplified.
- toml 1.x brings ordered table API and better error messages for TOML parse failures.
- notify 8.2.0 + notify-debouncer-full 0.7.0 stabilizes the watch-mode event pipeline.
- git2 vendored-libgit2 enables reproducible cross-platform builds for slideforge-package.
- flate2 + tar reproducible recipe eliminates cross-platform byte-diff in package archives.

### Negative / Trade-offs

- Three migration tasks touch merged code (sha2, toml, notify). Each migration requires
  test coverage before committing. This is not deferred — it is a prerequisite that
  must be executed in-scope when the consuming story lands.
- toml 1.x `#[serde(flatten)]` gotcha requires a round-trip test before committing
  any flatten layout in lockfile structs. This adds a test-first discipline step to
  any story that designs TOML serialization.
- flate2 OS-byte post-processing is a non-trivial CI step. The byte-diff test and
  header patch must be implemented before any package cache story ships.
- git2 full-clone (no shallow) means package fetch downloads the full git history for
  each package. This is a performance trade-off: cold installs are slower. Mitigated
  by the local `.slideforge/cache` (packages are only cloned once per version). A
  shallow-clone option should be evaluated when libgit2 0.28+ (which adds usable
  shallow clone) is available in a future git2-rs release.
- thiserror 2.x (pin =2.0.18) may have callsite impact if any merged crate used
  features removed in the 2.x major bump. Audit merged crates at migration time.

## Source / Origin

- Human decision recorded 2026-06-07 during Wave-5 remove-uncertainty pass.
- Registry-verified versions as of 2026-06-07 by research-agent. MSRV ≤ 1.88 confirmed.
- Cross-references: ADR-021 (tokio/axum/reqwest/crossterm pins, which are also workspace
  deps governed by this ADR's centralization rule).
