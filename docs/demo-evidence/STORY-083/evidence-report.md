# Evidence Report: STORY-083 — Plugin Registry Builder + Surface Enforcement

**Story:** STORY-083  
**Crate:** `slideforge-plugin-api`  
**BC:** BC-5.02.001 (invariant 3 — all 10 surfaces required)  
**Recorded:** 2026-06-04  
**Product type:** Library/API — recorded via runnable Rust example binary

---

## Artifacts Produced

| Artifact | Description |
|----------|-------------|
| `AC-001-005-registry-builder.tape` | VHS tape script (source) |
| `AC-001-005-registry-builder.gif` | GIF recording (PR embed) |
| `AC-001-005-registry-builder.webm` | WebM recording (archival) |

**Example binary source:**  
`crates/slideforge-plugin-api/examples/story_083_registry_builder.rs`

---

## AC Coverage Map

| AC | Title | Demonstrated In | Success Path | Error Path |
|----|-------|----------------|--------------|------------|
| AC-001 | `PluginRegistryBuilder::default().build()` → `Err(MissingSurface { surface: "DataSource" })` | `AC-001-005-registry-builder.*` | n/a | Yes — empty builder prints `Err(MissingSurface { surface: "DataSource" })` |
| AC-002 | Fully-registered builder (all 10 surfaces) → `Ok(registry)` | `AC-001-005-registry-builder.*` | Yes — 10-surface build prints `Ok(PluginRegistry)` + PASS | n/a |
| AC-003 | `surface_count() == 10` for fully-registered registry | `AC-001-005-registry-builder.*` | Yes — prints `surface_count() = 10` + PASS | Yes — partial 3-surface registry prints `surface_count() = 3` |
| AC-004 | `surface_names()` returns all 10 canonical names in order | `AC-001-005-registry-builder.*` | Yes — prints all 10 names + PASS | Yes — partial registry prints only 3 names |
| AC-005 | `RegistryError` implements `Debug`, `Display`, `std::error::Error`; is `#[non_exhaustive]` | `AC-001-005-registry-builder.*` | Yes — Display and Debug output printed side by side | n/a |

---

## Demonstrated Paths

### Success paths

1. **AC-002/003/004 — Full registry build:** Constructs a `PluginRegistryBuilder` with
   one stub implementation per surface (all 10), calls `build()`, and receives
   `Ok(PluginRegistry)`. Confirms `surface_count() == 10` and
   `surface_names() == ["DataSource", "Exporter", "ChartRenderer", "DiagramRenderer",
   "Validator", "MathRenderer", "BrandProvider", "SlideType", "SectionType", "InlineFormat"]`.

2. **AC-005 — Error formatting:** Constructs `RegistryError::MissingSurface { surface: "DataSource" }`
   and prints both `Display` (`required plugin surface 'DataSource' has no registered implementations`)
   and `Debug` (`MissingSurface { surface: "DataSource" }`) representations.

### Error paths

1. **AC-001 — Empty builder error:** `PluginRegistryBuilder::default().build()` returns
   `Err(RegistryError::MissingSurface { surface: "DataSource" })` — the first missing
   surface in SURFACE_NAMES declaration order. No panic, no silent no-op.

2. **AC-003/004 partial registry:** A 3-of-10 registry assembled via the mutation API
   (`PluginRegistry::register_data_source` / `register_exporter` / `register_chart_renderer`)
   shows `surface_count() == 3` and `surface_names() == ["DataSource", "Exporter", "ChartRenderer"]`,
   falsifying any implementation that hardcodes `10`.

---

## Build Gate Confirmation

All three gates passed before commit:

```
cargo build --example story_083_registry_builder -p slideforge-plugin-api   CLEAN
cargo clippy -p slideforge-plugin-api --all-targets --all-features \
  -- -D warnings -W clippy::missing_docs_in_private_items               CLEAN
cargo +nightly fmt --all -- --check                                          CLEAN
```
