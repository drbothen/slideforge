---
document_type: prd-supplement
supplement_type: interface-definitions
level: L3
version: "1.3"
status: active
producer: product-owner
timestamp: 2026-05-29T00:00:00
phase: 1a
traces_to: .factory/specs/prd.md
primary_consumers: [implementer, test-writer]
---

# Interface Definitions — slideforge v1.0

> This supplement is extracted from PRD §3 for per-agent consumption (DF-021).
> Primary consumers: implementer, test-writer.

---

## 1. CLI Interface

### 1.1 Top-Level Command

```
slideforge [--version] [--help] <COMMAND>
```

**Global flags:**
- `--version` — print `slideforge <version>` and exit 0
- `--help` — print help text and exit 0

---

### 1.2 `slideforge build` — Compile Source to Output

```
slideforge build <SOURCE> [OPTIONS]

Arguments:
  <SOURCE>                Path to .sf entry point file (required)
                          Type: valid filesystem path with .sf extension
                          Example: deck.sf, reports/q3-brief.sf

Options:
  -o, --output <DIR>      Output directory [default: ./dist]
                          Type: filesystem path (created if absent)
  -f, --format <FMT>      Comma-separated list of output formats
                          Values: pptx | docx | pdf | html | preview
                          Default: pptx
                          Example: --format pptx,docx
  --variant <NAME>        Build a named variant declared in variants: block
                          Type: string matching a variant name in the source
                          Error if not found: E-CFG-001
  --warn-only             Promote all validation errors to warnings (except overflow
                          when --strict-overflow is also active; see §5.1 interaction rules)
  --offline               Skip HTTP data sources; use only file-based sources
                          Error if sf.lock absent with deps: E-PKG-001
  --template <FILE>       Override brand template
                          Type: .pptx file or brand.toml file
                          Error if not found: E-BRD-001
  --workspace             Build all [workspace] members in slideforge.toml
                          Mutually exclusive with <SOURCE> positional arg
  -v, --verbose           Show per-stage timing and structured diagnostics
  -q, --quiet             Suppress all output except errors
  --json                  Output diagnostics as JSON to stdout; silence prose output
```

### 1.3 `slideforge watch` — Live Reload

```
slideforge watch <SOURCE> [OPTIONS]

Arguments:
  <SOURCE>                Path to .sf entry point file (required)

Options:
  --port <PORT>           Web preview server port [default: 3000]
                          Type: u16 (1024–65535)
  --host <HOST>           Web preview server bind address [default: 127.0.0.1]
  --warn-only             Always use warn-only mode (forced; watch mode implies this)
  --template <FILE>       Brand template override (same as build)
```

**Behavior:** Starts axum HTTP server and WebSocket server. Opens web preview at
`http://<host>:<port>`. Watches all .sf files and data sources (file-based). HTTP
data sources polled every 30 seconds by default (configurable in slideforge.toml).
On change: re-parse, re-evaluate, re-layout, push delta to WebSocket clients.

### 1.4 `slideforge package install` — Install Package

```
slideforge package install <REPO> [--version <TAG>] [--name <ALIAS>]

Arguments:
  <REPO>                  Git repository URL
                          Type: valid git URL (https:// or ssh://)
                          Example: github.com/1898/slides

Options:
  --version <TAG>         Git tag or commit SHA [default: latest tag]
  --name <ALIAS>          Override package name (defaults to repo name)
```

**Effect:** Resolves repo, downloads manifest, writes to slideforge.toml [dependencies],
updates sf.lock with SHA-256 checksum. Fails if URL unreachable (E-PKG-002).

### 1.5 `slideforge init` — Scaffold a New Project

```
slideforge init [NAME] [OPTIONS]

Arguments:
  [NAME]                  Optional project directory name.
                          If omitted, scaffolds in the current directory.
                          If provided, creates <NAME>/ directory first.
                          Type: valid directory name (no path separators)

Options:
  --force                 Overwrite existing scaffold files.
                          CAUTION: destructive — existing deck.sf and
                          slideforge.toml will be overwritten.
                          Without --force, fails with E-CFG-008 if
                          slideforge.toml exists.
  --template <NAME>       Named scaffold template to use.
                          Type: string matching a bundled template name
                          Values: default (only bundled template in v1.0)
                          Default: default
```

**Effect:** Creates a buildable starter project:
- `deck.sf` — main presentation source with 3 example slides
- `brand.toml` — brand configuration placeholder (edit with your colors)
- `slideforge.toml` — project configuration
- `assets/` — directory for images and data files
- `dist/` — output directory (included in generated .gitignore)

Prints a file manifest and "get started" next-step instructions.
The generated `deck.sf` must build successfully without modification.

**Error if target directory not writable:** E-EXP-007.
**Error if slideforge.toml already exists (without --force):** E-CFG-008.

### 1.6 `slideforge extract-brand` — Extract Brand from Template

```
slideforge extract-brand <TEMPLATE> [--output <FILE>]

Arguments:
  <TEMPLATE>              Path to .pptx or .docx template file (required)

Options:
  --output <FILE>         Output brand.toml path [default: brand.toml]
```

**Effect:** Reads OOXML theme from the template, extracts the 12 color slots,
typography settings, and logo references. Writes brand.toml. Warns for any
inferred or missing values.

### 1.7 `slideforge config explain` — Configuration Provenance

```
slideforge config explain [KEY] [--workspace-root <DIR>]

Arguments:
  [KEY]                   Optional: specific config key to explain
                          Example: brand.primary, workspace.output_dir

Options:
  --workspace-root <DIR>  Root of the workspace [default: current dir]
```

**Output:** For each key, shows: value, source file, cascade level (workspace /
family / deck), whether it was overridden.

---

## 2. Exit Code Reference

| Code | Constant Name | Condition | Output Written? |
|------|--------------|-----------|----------------|
| 0 | `EXIT_SUCCESS` | All output produced successfully | Yes |
| 1 | `EXIT_PARSE_ERROR` | Parse errors encountered; no AST produced | No |
| 2 | `EXIT_VALIDATION_ERROR` | Validation errors in strict mode | No |
| 3 | `EXIT_EXPORT_ERROR` | PPTX/DOCX/PDF serialization or I/O failure | Partial (attempt rollback) |
| 4 | `EXIT_CONFIG_ERROR` | Brand not found, workspace misconfigured | No |
| 5 | `EXIT_PACKAGE_ERROR` | Package not found, sf.lock mismatch | No |
| 64 | `EXIT_USAGE_ERROR` | Invalid CLI arguments (clap EX_USAGE convention) | No |
| 130 | `EXIT_INTERRUPTED` | SIGINT received during watch mode | No |

---

## 3. JSON Output Schema (--json flag)

When `--json` is passed, all output is JSON to stdout. No prose diagnostics are
printed. The JSON object has the following structure:

```json
{
  "$schema": "https://slideforge.dev/schema/build-result/v1.json",
  "status": "success" | "warning" | "error",
  "version": "1.0.0",
  "source": "/absolute/path/to/deck.sf",
  "variant": "exec" | null,
  "diagnostics": [
    {
      "severity": "parse_error" | "validation_error" | "lint",
      "code": "E-PAR-001",
      "category": "PAR" | "EVL" | "DAT" | "LAY" | "EXP" | "BRD" | "PKG" | "CFG" | "A11",
      "message": "Human-readable message",
      "file": "/absolute/path/to/file.sf",
      "line": 42,
      "col": 7,
      "length": 12,
      "hint": "Did you mean 'slide content'?"
    }
  ],
  "outputs": [
    {
      "format": "pptx" | "docx" | "pdf" | "html",
      "path": "/absolute/path/to/output.pptx",
      "size_bytes": 102400
    }
  ],
  "timing": {
    "total_ms": 342,
    "parse_ms": 45,
    "evaluate_ms": 12,
    "brand_ms": 14,
    "validate_ms": 8,
    "layout_ms": 89,
    "export_ms": 196
  }
}
```

**Field constraints:**
- `status` is "success" if `diagnostics` is empty; "warning" if only lints/warnings;
  "error" if any `parse_error` or `validation_error`.
- `outputs` is empty array on error.
- `timing` values are wall-clock milliseconds. All values present even if a stage
  was skipped (zero for skipped stages).

---

## 4. Config File Schema

### 4.1 `slideforge.toml` — Project/Workspace Root

```toml
[workspace]
members = ["reports/q3.sf", "briefs/*.sf"]  # optional; enables --workspace
output_dir = "dist"                           # default output directory

[brand]
template = "brand.pptx"                       # .pptx or brand.toml
# OR:
# template = "brand.toml"

[data]
watch_interval_secs = 30                      # HTTP data poll interval in watch mode
allowed_domains = ["api.internal.example.com"] # SSRF allowlist for HTTP sources

[dependencies]
# Package dependencies (written by slideforge package install)
# "my-slides" = { git = "https://github.com/1898/slides", version = "1.2.0" }

[build]
default_format = "pptx"                       # default --format value
strict_overflow = false                       # promote canvas overflow to error
warn_only = false                             # global --warn-only default
```

### 4.2 `brand.toml` — Brand Synthesis Input

```toml
[colors]
# 12 OOXML scheme color slots (hex RGB, no #)
dk1 = "1F2937"    # Dark 1 (primary text)
lt1 = "FFFFFF"    # Light 1 (background)
dk2 = "374151"    # Dark 2
lt2 = "F9FAFB"    # Light 2
acc1 = "3B82F6"   # Accent 1 (primary brand)
acc2 = "10B981"   # Accent 2
acc3 = "F59E0B"   # Accent 3
acc4 = "EF4444"   # Accent 4 (danger)
acc5 = "8B5CF6"   # Accent 5
acc6 = "06B6D4"   # Accent 6
hlink = "2563EB"  # Hyperlink
folHlink = "7C3AED" # Followed hyperlink

[colors.semantic]
# Optional semantic aliases (used in DSL as brand.danger, brand.success, etc.)
danger  = "EF4444"
warning = "F59E0B"
success = "10B981"
info    = "3B82F6"

[typography]
heading_font = "Inter"
body_font    = "Inter"
code_font    = "JetBrains Mono"
base_size_pt = 18

[logo]
path  = "assets/logo.svg"
alt   = "Company logo"
width_emu  = 1143000   # 1.25 inches
height_emu = 457200    # 0.5 inches

[footer]
left_text  = "Company Name"
right_text = "Confidential"
font_size_pt = 9

[slide_size]
width_emu  = 9144000   # 10 inches (16:9)
height_emu = 5143500   # 5.625 inches
```

### 4.3 Key-to-CLI-Flag Mapping

| Config Key | CLI Flag Equivalent | Precedence |
|-----------|-------------------|-----------|
| `build.default_format` | `--format` | CLI flag wins |
| `build.warn_only` | `--warn-only` | CLI flag wins |
| `brand.template` | `--template` | CLI flag wins |
| `build.output_dir` | `--output` | CLI flag wins |
| `data.watch_interval_secs` | (watch-only, no CLI flag) | config only |
| `data.allowed_domains` | (no CLI flag — security policy) | config only |

---

## 5. Flag Interaction Rules

### 5.1 Mutually Exclusive and Composing Flags

| Flag A | Flag B | Behavior |
|--------|--------|----------|
| `<SOURCE>` (positional) | `--workspace` | Error E-CFG-002: cannot specify source file and --workspace together |
| `--quiet` | `--verbose` | Error E-CFG-004: contradictory verbosity |
| `--quiet` | `--json` | `--json` wins; no prose output in either case |

**`--warn-only` + `--strict-overflow` interaction (composing, not exclusive):**

These two flags compose correctly — they address different error scopes:
- `--warn-only` promotes ALL validation errors to warnings EXCEPT canvas overflow when
  `--strict-overflow` is also active. Non-overflow validation errors (accessibility,
  type errors, etc.) become warnings and allow output to be written.
- `--strict-overflow` applies only to `E-LAY-001` canvas overflow: it promotes overflow
  from degraded (warning) to broken (fatal, exit 2).
- Together: non-overflow errors are warnings; overflow is still fatal. This is the
  correct behavior for "warn on minor issues but hard-fail on layout overflow."

E-CFG-003 is retired. The combination `--warn-only --strict-overflow` is valid and
produces the scoped behavior described above. No error is emitted for this combination.

### 5.2 Flag Override Rules

| Flag | Overrides | Notes |
|------|-----------|-------|
| `--template` | `brand.template` in slideforge.toml | CLI is highest precedence |
| `--output` | `build.output_dir` in slideforge.toml | CLI wins |
| `--warn-only` | `build.warn_only = false` | CLI wins |
| `slideforge watch` (implicit) | `build.warn_only` | watch mode always uses warn-only |

### 5.3 --workspace Behavior

When `--workspace` is passed:
- All `[workspace].members` glob patterns are expanded
- Each member is built independently with its own output path
- Failures in one member do NOT abort other members
- Exit code is the highest exit code across all builds
- `--variant`, `--template`, and `--format` apply to ALL members

---

## 6. Plugin Trait Signatures

> The 10 plugin trait surfaces from CAP-021. These are the stable APIs that
> bundled plugins implement and third-party plugins will use.

```rust
/// DataSource — loads structured data at compile time
pub trait DataSource: Send + Sync {
    fn id(&self) -> &str;
    fn schemes(&self) -> &[&str];  // e.g., ["http", "https"]
    fn fetch(&self, uri: &str, opts: &FetchOptions) -> Result<DataValue, DataError>;
}

/// Exporter — produces output bytes from Deck + LaidOutDeck + Brand
pub trait Exporter: Send + Sync {
    fn id(&self) -> &str;
    fn extension(&self) -> &str;  // e.g., "pptx"
    fn export(&self, deck: &Deck, laid_out: &LaidOutDeck, brand: &Brand, opts: &ExportOptions) -> Result<Vec<u8>, ExportError>;
}

/// ChartRenderer — produces SVG from chart spec
pub trait ChartRenderer: Send + Sync {
    fn id(&self) -> &str;
    fn chart_types(&self) -> &[ChartType];
    fn render(&self, spec: &ChartSpec) -> Result<SvgData, RenderError>;
}

/// DiagramRenderer — produces SVG from diagram source
pub trait DiagramRenderer: Send + Sync {
    fn id(&self) -> &str;
    fn supported_langs(&self) -> &[DiagramLang];
    fn render(&self, source: &str, lang: DiagramLang) -> Result<SvgData, DiagramError>;
}

/// Validator — checks Deck for issues
pub trait Validator: Send + Sync {
    fn id(&self) -> &str;
    fn validate(&self, deck: &Deck, brand: &Brand) -> Vec<Diagnostic>;
}

/// MathRenderer — converts LaTeX to output format
pub trait MathRenderer: Send + Sync {
    fn id(&self) -> &str;
    fn render(&self, latex: &str, display: MathDisplay, target: MathTarget)
        -> Result<MathOutput, MathError>;
}

/// BrandProvider — loads or synthesizes Brand
pub trait BrandProvider: Send + Sync {
    fn id(&self) -> &str;
    fn load(&self, source: &BrandSource) -> Result<Brand, BrandError>;
}

/// SlideType — defines visual pattern and layout for a slide kind
pub trait SlideType: Send + Sync {
    fn id(&self) -> &str;
    fn required_fields(&self) -> &[FieldDef];
    fn optional_fields(&self) -> &[FieldDef];
    fn layout_name(&self) -> &str;  // OOXML layout name
    fn lay_out(&self, slide: &Slide, brand: &Brand, canvas: Canvas) -> Result<LaidOutSlide, LayoutError>;
}

/// SectionType — defines document section pattern
pub trait SectionType: Send + Sync {
    fn id(&self) -> &str;
    fn generate(&self, deck: &Deck) -> Result<DocumentSection, SectionError>;
}

/// InlineFormat — defines inline formatting rule
pub trait InlineFormat: Send + Sync {
    fn id(&self) -> &str;
    fn syntax_token(&self) -> &str;  // e.g., "**" for bold
    fn to_ir(&self, content: &str) -> InlineNode;
}
```

All trait objects are `Send + Sync` to support concurrent export of multiple formats.

---

## 7. DataSource Plugin Calling Convention (Adjudicated 2026-05-28)

> This section codifies the canonical `uri` parameter convention for all `DataSource`
> implementations. Adjudicated in adversary pass 1 on STORY-020, item G.

### 7.1 `DataSource::fetch` — `uri` Parameter Semantics

The `DataSource` trait's `fetch` method signature is:

```rust
fn fetch(&self, uri: &str, opts: &FetchOptions) -> Result<DataValue, DataError>;
```

The `uri` parameter follows this protocol, which is **canonical for ALL DataSource
implementations** (not STORY-020-only):

| `uri` value | Behavior |
|-------------|----------|
| Non-empty string | `uri` overrides the plugin's internal path/state. The plugin MUST use `uri` as the data source location, ignoring `self.path` or equivalent internal field. |
| Empty string (`""`) | Plugin-internal state is authoritative. The plugin uses `self.path` (or equivalent) as configured at construction time. |

**Rationale:** File-based DataSource implementations (`XlsxDataSource`, `SqliteDataSource`,
`JsonDataSource`, etc.) are constructed with a `path` field from the `@data` directive at
parse time. The `fetch` call passes `uri` as a potential runtime override. Passing an empty
`uri` is the normal case for compile-time file sources; a non-empty `uri` supports
future use-cases like `slideforge watch` refreshing with a remapped path.

**Invariant:** A DataSource plugin MUST NOT silently ignore a non-empty `uri`. If the plugin
cannot handle URI-based override (e.g., because the format is path-specific), it MUST
return `DataError::UnsupportedFormat` with a message explaining the constraint.

### 7.2 Implementing the Convention in File-Based Sources

For file-based plugins, the pattern is:

```rust
fn fetch(&self, uri: &str, opts: &FetchOptions) -> Result<DataValue, DataError> {
    let effective_path = if uri.is_empty() { &*self.path } else { uri };
    // ... use effective_path for all file I/O
}
```

This pattern is required for `XlsxDataSource`, `SqliteDataSource`, `JsonDataSource`,
`CsvDataSource`, `YamlDataSource`, and `TomlDataSource`.

### 7.3 URI Convention for HTTP-Based Sources

For HTTP DataSource implementations, `uri` is always expected to be non-empty (it IS the
URL). An empty `uri` for an HTTP source produces `DataError::ParseError` with message:
`"HTTP data source requires a non-empty URI"`.

### 7.4 Downstream Callers

The `DataSourceContext` caller (in the evaluator) passes:
- `""` (empty) for `@data` directives that specify a static file path at parse time
- The resolved path string for dynamic or remapped sources

This convention is enforced at the `DataSourceContext` level; individual plugins do not
need to handle partial paths or path resolution — they receive a full path or empty string.

---

## 8. LayoutError Field-Naming Convention (Adjudicated 2026-05-28)

> Codified in adversary pass 1 on STORY-028, item L.

### 8.1 Canonical Field Name: `source_slide_index`

The canonical field name for "which slide in the input Deck caused this error" in
`LayoutError` variants is **`source_slide_index: usize`**.

This is declared in the module doc-comment of `slideforge-layout/src/error.rs`:

```
All variants include a `source_slide_index: usize` where applicable so that
error messages can point to the offending slide by position in the input Deck.
```

The module header is the source of truth. All `LayoutError` variants that identify
a slide MUST use `source_slide_index` — not `slide_index`, `idx`, or other spellings.

### 8.2 Current State and Fix Obligation

As of STORY-028 implementation (HEAD cc037817), the following variants use the
non-canonical `slide_index` name:

- `LayoutError::MissingAlt { slide_index }`
- `LayoutError::MissingRiskCardField { slide_index, card_index, field }`
- `LayoutError::MalformedSeverityCards { slide_index, reason }`
- `LayoutError::UnresolvedSeverityCards { slide_index }`

These MUST be renamed to `source_slide_index` in a fix-burst dispatched by the
orchestrator after STORY-028 closes. The fix burst is owned by the data-engineer
(schema change) and requires a sibling-site sweep per TD-VSDD-060 (all callsites
in `slideforge-layout` and adjacent crates that pattern-match these variants).

### 8.3 Exception: `SlideCountMismatch`

`LayoutError::SlideCountMismatch { expected, actual, source_slide_index }` correctly
uses `source_slide_index` and requires no change.

### 8.4 Future Variant Authoring Rule

When adding a new `LayoutError` variant:
- If the variant identifies a specific slide: use `source_slide_index: usize`
- If the variant is slide-independent (e.g., empty deck): include `source_slide_index: usize`
  set to `0` for structural consistency (as `EmptyDeck` does)
- Never use `slide_index`, `idx`, `slide_idx`, or other spellings

---

## 9. Shape IR Conventions (Adjudicated 2026-05-29)

> Codified in adversary pass 2 on STORY-028, items M, N, O, P, S.

### 9.1 `LayoutError::multiple` — Smart Constructor (Item S / Item N)

```rust
impl LayoutError {
    /// Construct a Multiple variant from an accumulated error list.
    ///
    /// # Panics (debug builds)
    /// Panics via `debug_assert!` if `errors` is empty — an empty Multiple
    /// is a logic error in the accumulation loop and must not silently pass.
    ///
    /// # Flattening
    /// If any element of `errors` is itself `LayoutError::Multiple { inner }`,
    /// that inner vec is flattened into the result. No nested Multiple values
    /// are produced by this constructor.
    pub fn multiple(errors: Vec<Self>) -> Self {
        debug_assert!(!errors.is_empty(), "LayoutError::multiple called with empty vec");
        let flattened: Vec<Self> = errors
            .into_iter()
            .flat_map(|e| match e {
                LayoutError::Multiple { inner } => inner,
                other => vec![other],
            })
            .collect();
        LayoutError::Multiple { inner: flattened }
    }
}
```

**Usage rule:** ALL call sites that accumulate layout errors into a `Vec<LayoutError>` and
then return an error MUST use `LayoutError::multiple(accumulated)` rather than constructing
`LayoutError::Multiple { inner: accumulated }` directly. This ensures the flattening and
empty-guard invariants are consistently applied.

**Uniformity rule:** Even when only one error is accumulated, the return MUST be
`LayoutError::multiple(vec![one_error])`, which produces `Multiple { inner: vec![one_error] }`.
Callers pattern-match exclusively on `Multiple`. The asymmetric "single error = unwrapped
variant" API is explicitly forbidden per BC-3.04.001 postcondition 6.

### 9.2 `ShapeType::from_keyword` — Parser Bridge (Item P)

```rust
impl ShapeType {
    /// Convert a DSL keyword string to the resolved ShapeType enum variant.
    ///
    /// Called by: the DSL parser (future story) and ALL test construction sites.
    /// The IR MUST NOT store Arc<str> in ShapeSpec.shape_type.
    ///
    /// Returns Err(ParseError::UnknownShapeType { keyword, span }) on unknown input,
    /// which the caller maps to E-PAR-012.
    pub fn from_keyword(kw: &str) -> Result<ShapeType, ParseError>;
}
```

Accepted keywords (case-sensitive): `rect`, `ellipse`, `arrow`, `line`, `star`, `roundRect`.
All other inputs return `Err`. Keyword matching is case-sensitive and exact — no fuzzy matching.

This function lives in `slideforge-types` (alongside `ShapeType`) so that both the parser
crate and test helpers can depend on it without introducing additional crate dependencies.

### 9.3 `LaidOutDeck.warnings` — Warnings Field (Item O)

```rust
pub struct LaidOutDeck {
    // ... existing fields ...
    /// Accumulated layout warnings from layout::run.
    /// Populated by both run_inline_validation (XrefTargetNotFound) and
    /// layout_shapes (OffCanvas). NEVER empty-initialized and then dropped;
    /// the field is always present and callers may inspect it.
    pub warnings: Vec<LayoutWarning>,
}
```

`layout::run` is responsible for collecting ALL `LayoutWarning` values from both
`run_inline_validation` and `layout_shapes` and placing them in this field.
Exporters are consumers of `LaidOutDeck.warnings`; they MAY surface or suppress
individual warning types but MUST NOT further lose warnings.

The `LayoutWarning` enum variants relevant here:
- `LayoutWarning::XrefTargetNotFound { target: Arc<str>, source_slide_index: usize }`
- `LayoutWarning::OffCanvas { source_slide_index: usize, shape_type: ShapeType, x_emu: i64, y_emu: i64 }`

Both variants use `source_slide_index` (consistent with the §8 field-naming convention).

### 9.4 ArithmeticOverflow Error Return (Item M)

`ShapeUnit::from_inches` and `ShapeUnit::from_em` MUST use `i64::checked_mul` for the
milliunit-to-EMU multiplication. On overflow, they MUST return
`Err(LayoutError::ArithmeticOverflow { source_slide_index, span })`.

```rust
impl ShapeUnit {
    /// Convert to EMU at layout time given the current brand font size.
    ///
    /// Returns Err on i64 overflow — the caller accumulates this into
    /// LayoutError::multiple(accumulated_errors).
    pub fn to_emu(&self, brand_font_size_emu: i64, source_slide_index: usize, span: SourceSpan)
        -> Result<i64, LayoutError>;
}
```

`saturating_mul` without an error return is explicitly forbidden. The `ArithmeticOverflow`
variant MUST be reachable from production code; it is NOT a "future strict-mode" concern.
It maps to E-LAY-006 in the CLI diagnostic renderer.
