//! [`PluginRegistry`] — the central container for all 10 plugin surfaces.
//!
//! The registry is constructed once at process start by the CLI entry point and
//! then shared immutably (wrapped in `Arc<PluginRegistry>`) across all pipeline
//! stages. Registration is synchronous and single-threaded during startup;
//! lookups are read-only and can occur from any thread.
//!
//! ## Construction
//!
//! The preferred construction path is [`PluginRegistryBuilder`]:
//!
//! ```no_run
//! # use slideforge_plugin_api::PluginRegistryBuilder;
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // All register_* methods take &mut self and return &mut Self.
//! // Call build() as a separate statement after registering all surfaces.
//! let mut builder = PluginRegistryBuilder::default();
//! // builder.register_data_source(Box::new(my_data_source));
//! // … register all 10 surfaces …
//! let registry = builder.build()?;
//! # Ok(())
//! # }
//! ```
//!
//! [`PluginRegistryBuilder::build`] returns [`Err(RegistryError::MissingSurface)`] if
//! any of the 10 required surfaces has zero registrations (BC-5.02.001 invariant 3).
//!
//! ## Design constraints
//!
//! - Stores `Box<dyn Trait + Send + Sync>` per surface — dynamic dispatch.
//! - Each surface uses `Vec<Box<dyn Trait>>` to allow multiple implementations.
//! - Lookups return `Option<&dyn Trait>` by trait `id()`.
//! - `PluginRegistry` is intentionally NOT `Clone` (`Box<dyn Trait>` is not
//!   `Clone`). Callers wrap in `Arc<PluginRegistry>` for shared ownership.
//! - No WASM loading or dynamic library loading in v1.0 — all plugins are
//!   statically compiled (plugin-architecture.md §v1.0 Plugin Loading).

use crate::traits::{
    BrandProvider, ChartRenderer, DataSource, DiagramRenderer, Exporter, InlineFormat,
    MathRenderer, SectionType, SlideType, Validator,
};

// ──────────────────────────────────────────────────────────────────────────────
// RegistryError
// ──────────────────────────────────────────────────────────────────────────────

/// Error returned by [`PluginRegistryBuilder::build`] when registry finalization
/// fails because a required plugin surface has no registered implementations.
///
/// ## Non-exhaustive
///
/// This enum is `#[non_exhaustive]`: future slideforge versions may add new error
/// variants (e.g., `DuplicateId`, `InvalidPlugin`) without a breaking API change.
/// Callers must include a wildcard arm when matching:
///
/// ```ignore
/// match err {
///     RegistryError::MissingSurface { surface } => { /* … */ }
///     _ => { /* future variants */ }
/// }
/// ```
///
/// ## Traceability
///
/// - BC-5.02.001 invariant 3: all 10 surfaces are required; enforcement is at
///   `PluginRegistryBuilder::build()`.
/// - AC-005 (STORY-083): `thiserror`-derived `Display` + `std::error::Error`.
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    /// A required plugin surface has zero registered implementations.
    ///
    /// `surface` is the canonical name of the missing surface
    /// (e.g., `"DataSource"`, `"Exporter"`). The first missing surface
    /// in declaration order is reported.
    #[error("required plugin surface '{surface}' has no registered implementations")]
    MissingSurface {
        /// The canonical name of the surface that has no registrations.
        surface: &'static str,
    },
}

// ──────────────────────────────────────────────────────────────────────────────
// Canonical surface names (declaration order)
// ──────────────────────────────────────────────────────────────────────────────

/// Canonical names of all 10 plugin surfaces, in declaration order.
///
/// Used by [`PluginRegistryBuilder::build`] to report the first missing surface
/// and by [`PluginRegistry::surface_names`] to enumerate registered surfaces.
///
/// The ordering matches the surface numbering in `slideforge-plugin-api/src/traits.rs`
/// and BC-5.02.001 invariant 1.
pub const SURFACE_NAMES: [&str; 10] = [
    "DataSource",
    "Exporter",
    "ChartRenderer",
    "DiagramRenderer",
    "Validator",
    "MathRenderer",
    "BrandProvider",
    "SlideType",
    "SectionType",
    "InlineFormat",
];

// ──────────────────────────────────────────────────────────────────────────────
// PluginRegistryBuilder
// ──────────────────────────────────────────────────────────────────────────────

/// Builder for [`PluginRegistry`] that enforces all-10-surfaces coverage at
/// finalization time (BC-5.02.001 invariant 3).
///
/// ## Usage
///
/// All `register_*` methods take `&mut self` and return `&mut Self`. Because
/// `build()` consumes `self`, chaining `register_*(...).build()` off a
/// temporary does not compile. Use separate statements instead:
///
/// ```no_run
/// # use slideforge_plugin_api::PluginRegistryBuilder;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut builder = PluginRegistryBuilder::default();
/// // builder.register_data_source(Box::new(JsonDataSource::new()));
/// // builder.register_exporter(Box::new(PptxExporter::new()));
/// // … register all 10 surfaces …
/// let registry = builder.build()?;
/// # Ok(())
/// # }
/// ```
///
/// ## Enforcement
///
/// [`build`](PluginRegistryBuilder::build) checks that every surface has at
/// least one registration. If any surface is empty, it returns
/// [`Err(RegistryError::MissingSurface)`] naming the first unregistered
/// surface in declaration order. A silent no-op or panic are both contract
/// violations (BC-5.02.001 invariant 3).
///
/// ## Multiple registrations per surface
///
/// All `register_*` methods are additive — registering the same surface twice
/// stores both plugins. On lookup, `PluginRegistry` returns the first
/// registration (insertion order, EC-001). `surface_count()` counts surfaces,
/// not total plugins.
#[derive(Default)]
pub struct PluginRegistryBuilder {
    /// Accumulated data source plugins.
    data_sources: Vec<Box<dyn DataSource + Send + Sync>>,
    /// Accumulated exporter plugins.
    exporters: Vec<Box<dyn Exporter + Send + Sync>>,
    /// Accumulated chart renderer plugins.
    chart_renderers: Vec<Box<dyn ChartRenderer + Send + Sync>>,
    /// Accumulated diagram renderer plugins.
    diagram_renderers: Vec<Box<dyn DiagramRenderer + Send + Sync>>,
    /// Accumulated validator plugins.
    validators: Vec<Box<dyn Validator + Send + Sync>>,
    /// Accumulated math renderer plugins.
    math_renderers: Vec<Box<dyn MathRenderer + Send + Sync>>,
    /// Accumulated brand provider plugins.
    brand_providers: Vec<Box<dyn BrandProvider + Send + Sync>>,
    /// Accumulated slide type plugins.
    slide_types: Vec<Box<dyn SlideType + Send + Sync>>,
    /// Accumulated section type plugins.
    section_types: Vec<Box<dyn SectionType + Send + Sync>>,
    /// Accumulated inline format plugins.
    inline_formats: Vec<Box<dyn InlineFormat + Send + Sync>>,
}

impl PluginRegistryBuilder {
    // ── Per-surface register_* builder methods ────────────────────────────────
    //
    // Each method appends to the corresponding surface vec and returns `&mut Self`
    // for method chaining. The methods are intentionally `&mut self -> &mut Self`
    // (not consuming `self -> Self`) to allow conditional registration patterns
    // without losing ownership:
    //
    //     let mut builder = PluginRegistryBuilder::default();
    //     builder.register_data_source(Box::new(JsonDataSource::new()));
    //     if cfg!(feature = "csv") {
    //         builder.register_data_source(Box::new(CsvDataSource::new()));
    //     }
    //     let registry = builder.build()?;

    /// Register a [`DataSource`] plugin.
    ///
    /// Appends to the `DataSource` surface list; may be called multiple times.
    /// Returns `&mut self` for chaining.
    pub fn register_data_source(&mut self, plugin: Box<dyn DataSource + Send + Sync>) -> &mut Self {
        self.data_sources.push(plugin);
        self
    }

    /// Register an [`Exporter`] plugin.
    ///
    /// Appends to the `Exporter` surface list; may be called multiple times.
    /// Returns `&mut self` for chaining.
    pub fn register_exporter(&mut self, plugin: Box<dyn Exporter + Send + Sync>) -> &mut Self {
        self.exporters.push(plugin);
        self
    }

    /// Register a [`ChartRenderer`] plugin.
    ///
    /// Appends to the `ChartRenderer` surface list; may be called multiple times.
    /// Returns `&mut self` for chaining.
    pub fn register_chart_renderer(
        &mut self,
        plugin: Box<dyn ChartRenderer + Send + Sync>,
    ) -> &mut Self {
        self.chart_renderers.push(plugin);
        self
    }

    /// Register a [`DiagramRenderer`] plugin.
    ///
    /// Appends to the `DiagramRenderer` surface list; may be called multiple times.
    /// Returns `&mut self` for chaining.
    pub fn register_diagram_renderer(
        &mut self,
        plugin: Box<dyn DiagramRenderer + Send + Sync>,
    ) -> &mut Self {
        self.diagram_renderers.push(plugin);
        self
    }

    /// Register a [`Validator`] plugin.
    ///
    /// Appends to the `Validator` surface list; may be called multiple times.
    /// Returns `&mut self` for chaining.
    pub fn register_validator(&mut self, plugin: Box<dyn Validator + Send + Sync>) -> &mut Self {
        self.validators.push(plugin);
        self
    }

    /// Register a [`MathRenderer`] plugin.
    ///
    /// Appends to the `MathRenderer` surface list; may be called multiple times.
    /// Returns `&mut self` for chaining.
    pub fn register_math_renderer(
        &mut self,
        plugin: Box<dyn MathRenderer + Send + Sync>,
    ) -> &mut Self {
        self.math_renderers.push(plugin);
        self
    }

    /// Register a [`BrandProvider`] plugin.
    ///
    /// Appends to the `BrandProvider` surface list; may be called multiple times.
    /// Returns `&mut self` for chaining.
    pub fn register_brand_provider(
        &mut self,
        plugin: Box<dyn BrandProvider + Send + Sync>,
    ) -> &mut Self {
        self.brand_providers.push(plugin);
        self
    }

    /// Register a [`SlideType`] plugin.
    ///
    /// Appends to the `SlideType` surface list; may be called multiple times.
    /// Returns `&mut self` for chaining.
    pub fn register_slide_type(&mut self, plugin: Box<dyn SlideType + Send + Sync>) -> &mut Self {
        self.slide_types.push(plugin);
        self
    }

    /// Register a [`SectionType`] plugin.
    ///
    /// Appends to the `SectionType` surface list; may be called multiple times.
    /// Returns `&mut self` for chaining.
    pub fn register_section_type(
        &mut self,
        plugin: Box<dyn SectionType + Send + Sync>,
    ) -> &mut Self {
        self.section_types.push(plugin);
        self
    }

    /// Register an [`InlineFormat`] plugin.
    ///
    /// Appends to the `InlineFormat` surface list; may be called multiple times.
    /// Returns `&mut self` for chaining.
    pub fn register_inline_format(
        &mut self,
        plugin: Box<dyn InlineFormat + Send + Sync>,
    ) -> &mut Self {
        self.inline_formats.push(plugin);
        self
    }

    // ── Surface-presence mapping (single source of truth) ────────────────────

    /// Return a fixed-size array pairing each canonical surface name with a
    /// flag indicating whether that surface has at least one registration.
    ///
    /// The order matches [`SURFACE_NAMES`] declaration order (BC-5.02.001
    /// invariant 1). This is the **single source of truth** for the
    /// name↔vec mapping: [`build`](Self::build) uses it for the empty-surface
    /// check. Adding an 11th surface requires touching only this method and the
    /// corresponding struct field.
    fn surface_presence(&self) -> [(&'static str, bool); 10] {
        [
            (SURFACE_NAMES[0], !self.data_sources.is_empty()),
            (SURFACE_NAMES[1], !self.exporters.is_empty()),
            (SURFACE_NAMES[2], !self.chart_renderers.is_empty()),
            (SURFACE_NAMES[3], !self.diagram_renderers.is_empty()),
            (SURFACE_NAMES[4], !self.validators.is_empty()),
            (SURFACE_NAMES[5], !self.math_renderers.is_empty()),
            (SURFACE_NAMES[6], !self.brand_providers.is_empty()),
            (SURFACE_NAMES[7], !self.slide_types.is_empty()),
            (SURFACE_NAMES[8], !self.section_types.is_empty()),
            (SURFACE_NAMES[9], !self.inline_formats.is_empty()),
        ]
    }

    // ── Finalization ──────────────────────────────────────────────────────────

    /// Finalize the builder into a [`PluginRegistry`].
    ///
    /// ## Enforcement (BC-5.02.001 invariant 3)
    ///
    /// Checks that all 10 required surfaces have at least one registration.
    /// If any surface is empty, returns
    /// `Err(RegistryError::MissingSurface { surface: "<name>" })` naming the
    /// first unregistered surface in declaration order.
    ///
    /// Returns `Ok(PluginRegistry)` only when all 10 surfaces are non-empty.
    ///
    /// ## Errors
    ///
    /// Returns [`RegistryError::MissingSurface`] if any required surface has
    /// zero registrations at call time.
    #[must_use = "registry build result must be checked; an Err means a required surface is missing"]
    pub fn build(self) -> Result<PluginRegistry, RegistryError> {
        // Check each required surface in SURFACE_NAMES declaration order via the
        // single-source surface_presence mapping (TD-VSDD-060 compliance).
        // Report the first empty surface with a typed error (BC-5.02.001 invariant 3).
        for (surface, present) in self.surface_presence() {
            if !present {
                return Err(RegistryError::MissingSurface { surface });
            }
        }
        Ok(PluginRegistry {
            data_sources: self.data_sources,
            exporters: self.exporters,
            chart_renderers: self.chart_renderers,
            diagram_renderers: self.diagram_renderers,
            validators: self.validators,
            math_renderers: self.math_renderers,
            brand_providers: self.brand_providers,
            slide_types: self.slide_types,
            section_types: self.section_types,
            inline_formats: self.inline_formats,
        })
    }
}

/// The central registry for all slideforge plugin implementations.
///
/// Constructed once at process start, then wrapped in `Arc<PluginRegistry>` and
/// shared across all pipeline stages. All registration methods take
/// `Box<dyn Trait + Send + Sync>` and append to the corresponding surface list.
///
/// ## Lookup semantics
///
/// - `lookup_*_by_id(id)` performs a linear search by `trait.id()` and returns
///   the first match.
/// - Multiple plugins with the same `id()` may be registered; the first
///   registered is returned on lookup (insertion order, EC-001).
/// - An empty registry returns `None` for all lookups (EC-003).
///
/// ## Thread safety
///
/// `PluginRegistry` is `Send + Sync` because all stored `Box<dyn Trait>` bounds
/// include `Send + Sync`. See the compile-time assertion in this module.
#[derive(Default)]
pub struct PluginRegistry {
    /// Registered data source plugins.
    data_sources: Vec<Box<dyn DataSource + Send + Sync>>,
    /// Registered exporter plugins.
    exporters: Vec<Box<dyn Exporter + Send + Sync>>,
    /// Registered chart renderer plugins.
    chart_renderers: Vec<Box<dyn ChartRenderer + Send + Sync>>,
    /// Registered diagram renderer plugins.
    diagram_renderers: Vec<Box<dyn DiagramRenderer + Send + Sync>>,
    /// Registered validator plugins.
    validators: Vec<Box<dyn Validator + Send + Sync>>,
    /// Registered math renderer plugins.
    math_renderers: Vec<Box<dyn MathRenderer + Send + Sync>>,
    /// Registered brand provider plugins.
    brand_providers: Vec<Box<dyn BrandProvider + Send + Sync>>,
    /// Registered slide type plugins.
    slide_types: Vec<Box<dyn SlideType + Send + Sync>>,
    /// Registered section type plugins.
    section_types: Vec<Box<dyn SectionType + Send + Sync>>,
    /// Registered inline format plugins.
    inline_formats: Vec<Box<dyn InlineFormat + Send + Sync>>,
}

// Compile-time assertion: PluginRegistry must be Send + Sync.
// If this function compiles, the assertion holds.
const _: fn() = || {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<PluginRegistry>();
};

impl PluginRegistry {
    /// Create a new empty plugin registry.
    ///
    /// No plugins are registered by default. The CLI assembles the registry
    /// by calling each `register_*` method before passing it to the pipeline.
    ///
    /// For the recommended construction path that enforces all-10-surfaces
    /// coverage, use [`PluginRegistryBuilder`] instead.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    // ──────────────────────────────────────────────────────────────────────────
    // Introspection (AC-003, AC-004 — STORY-083)
    // ──────────────────────────────────────────────────────────────────────────

    /// Return a fixed-size array pairing each canonical surface name with a
    /// flag indicating whether that surface has at least one registration.
    ///
    /// The order matches [`SURFACE_NAMES`] declaration order (BC-5.02.001
    /// invariant 1). This is the **single source of truth** for the
    /// name↔vec mapping on `PluginRegistry`: [`surface_count`](Self::surface_count)
    /// and [`surface_names`](Self::surface_names) both derive from it. Adding
    /// an 11th surface requires touching only this method and the corresponding
    /// struct field.
    fn surface_presence(&self) -> [(&'static str, bool); 10] {
        [
            (SURFACE_NAMES[0], !self.data_sources.is_empty()),
            (SURFACE_NAMES[1], !self.exporters.is_empty()),
            (SURFACE_NAMES[2], !self.chart_renderers.is_empty()),
            (SURFACE_NAMES[3], !self.diagram_renderers.is_empty()),
            (SURFACE_NAMES[4], !self.validators.is_empty()),
            (SURFACE_NAMES[5], !self.math_renderers.is_empty()),
            (SURFACE_NAMES[6], !self.brand_providers.is_empty()),
            (SURFACE_NAMES[7], !self.slide_types.is_empty()),
            (SURFACE_NAMES[8], !self.section_types.is_empty()),
            (SURFACE_NAMES[9], !self.inline_formats.is_empty()),
        ]
    }

    /// Return the number of plugin surfaces that have at least one registration.
    ///
    /// For a fully-registered registry (all 10 surfaces), returns `10`.
    /// For a partially-assembled registry built with the mutation API
    /// (`register_*` on `PluginRegistry` directly), returns the count of
    /// non-empty surface vecs.
    ///
    /// ## Traceability
    ///
    /// - AC-003 (STORY-083): "fully-registered registry → `surface_count() == 10`"
    /// - BC-5.02.001 postcondition 2
    #[must_use]
    pub fn surface_count(&self) -> usize {
        // Delegate to the single-source surface_presence mapping (TD-VSDD-060).
        self.surface_presence()
            .iter()
            .filter(|&&(_, present)| present)
            .count()
    }

    /// Return the canonical names of all surfaces that have at least one
    /// registration, in declaration order.
    ///
    /// For a fully-registered registry, returns all 10 canonical names:
    /// `["DataSource", "Exporter", "ChartRenderer", "DiagramRenderer",
    ///   "Validator", "MathRenderer", "BrandProvider", "SlideType",
    ///   "SectionType", "InlineFormat"]`.
    ///
    /// For a partially-assembled registry, returns only the names of non-empty
    /// surfaces (EC-002).
    ///
    /// ## Traceability
    ///
    /// - AC-004 (STORY-083): "fully-registered → `surface_names().len() == 10`"
    /// - BC-5.02.001 invariant 1
    #[must_use]
    pub fn surface_names(&self) -> Vec<&'static str> {
        // Delegate to the single-source surface_presence mapping (TD-VSDD-060).
        // Collect only names whose corresponding surface vec has at least one plugin
        // (EC-002: partial registries return only registered surface names).
        self.surface_presence()
            .iter()
            .filter_map(|&(name, present)| if present { Some(name) } else { None })
            .collect()
    }

    // ──────────────────────────────────────────────────────────────────────────
    // Surface 1: DataSource
    // ──────────────────────────────────────────────────────────────────────────

    /// Register a [`DataSource`] plugin.
    ///
    /// Multiple data source plugins with different `id()` values may be
    /// registered. Registering two plugins with the same `id()` is allowed;
    /// the first is returned on lookup (EC-001).
    pub fn register_data_source(&mut self, plugin: Box<dyn DataSource + Send + Sync>) {
        self.data_sources.push(plugin);
    }

    /// Look up a [`DataSource`] by its `id()`.
    ///
    /// Returns the first registered plugin whose `id()` matches `id`, or `None`
    /// if no match is found (EC-002, EC-003).
    #[must_use]
    pub fn lookup_data_source(&self, id: &str) -> Option<&dyn DataSource> {
        self.data_sources
            .iter()
            .find(|p| p.id() == id)
            .map(|p| p.as_ref() as &dyn DataSource)
    }

    // ──────────────────────────────────────────────────────────────────────────
    // Surface 2: Exporter
    // ──────────────────────────────────────────────────────────────────────────

    /// Register an [`Exporter`] plugin.
    pub fn register_exporter(&mut self, plugin: Box<dyn Exporter + Send + Sync>) {
        self.exporters.push(plugin);
    }

    /// Look up an [`Exporter`] by its `id()`.
    #[must_use]
    pub fn lookup_exporter(&self, id: &str) -> Option<&dyn Exporter> {
        self.exporters
            .iter()
            .find(|p| p.id() == id)
            .map(|p| p.as_ref() as &dyn Exporter)
    }

    // ──────────────────────────────────────────────────────────────────────────
    // Surface 3: ChartRenderer
    // ──────────────────────────────────────────────────────────────────────────

    /// Register a [`ChartRenderer`] plugin.
    pub fn register_chart_renderer(&mut self, plugin: Box<dyn ChartRenderer + Send + Sync>) {
        self.chart_renderers.push(plugin);
    }

    /// Look up a [`ChartRenderer`] by its `id()`.
    #[must_use]
    pub fn lookup_chart_renderer(&self, id: &str) -> Option<&dyn ChartRenderer> {
        self.chart_renderers
            .iter()
            .find(|p| p.id() == id)
            .map(|p| p.as_ref() as &dyn ChartRenderer)
    }

    // ──────────────────────────────────────────────────────────────────────────
    // Surface 4: DiagramRenderer
    // ──────────────────────────────────────────────────────────────────────────

    /// Register a [`DiagramRenderer`] plugin.
    pub fn register_diagram_renderer(&mut self, plugin: Box<dyn DiagramRenderer + Send + Sync>) {
        self.diagram_renderers.push(plugin);
    }

    /// Look up a [`DiagramRenderer`] by its `id()`.
    #[must_use]
    pub fn lookup_diagram_renderer(&self, id: &str) -> Option<&dyn DiagramRenderer> {
        self.diagram_renderers
            .iter()
            .find(|p| p.id() == id)
            .map(|p| p.as_ref() as &dyn DiagramRenderer)
    }

    // ──────────────────────────────────────────────────────────────────────────
    // Surface 5: Validator
    // ──────────────────────────────────────────────────────────────────────────

    /// Register a [`Validator`] plugin.
    pub fn register_validator(&mut self, plugin: Box<dyn Validator + Send + Sync>) {
        self.validators.push(plugin);
    }

    /// Look up a [`Validator`] by its `id()`.
    #[must_use]
    pub fn lookup_validator(&self, id: &str) -> Option<&dyn Validator> {
        self.validators
            .iter()
            .find(|p| p.id() == id)
            .map(|p| p.as_ref() as &dyn Validator)
    }

    // ──────────────────────────────────────────────────────────────────────────
    // Surface 6: MathRenderer
    // ──────────────────────────────────────────────────────────────────────────

    /// Register a [`MathRenderer`] plugin.
    pub fn register_math_renderer(&mut self, plugin: Box<dyn MathRenderer + Send + Sync>) {
        self.math_renderers.push(plugin);
    }

    /// Look up a [`MathRenderer`] by its `id()`.
    #[must_use]
    pub fn lookup_math_renderer(&self, id: &str) -> Option<&dyn MathRenderer> {
        self.math_renderers
            .iter()
            .find(|p| p.id() == id)
            .map(|p| p.as_ref() as &dyn MathRenderer)
    }

    // ──────────────────────────────────────────────────────────────────────────
    // Surface 7: BrandProvider
    // ──────────────────────────────────────────────────────────────────────────

    /// Register a [`BrandProvider`] plugin.
    pub fn register_brand_provider(&mut self, plugin: Box<dyn BrandProvider + Send + Sync>) {
        self.brand_providers.push(plugin);
    }

    /// Look up a [`BrandProvider`] by its `id()`.
    #[must_use]
    pub fn lookup_brand_provider(&self, id: &str) -> Option<&dyn BrandProvider> {
        self.brand_providers
            .iter()
            .find(|p| p.id() == id)
            .map(|p| p.as_ref() as &dyn BrandProvider)
    }

    // ──────────────────────────────────────────────────────────────────────────
    // Surface 8: SlideType
    // ──────────────────────────────────────────────────────────────────────────

    /// Register a [`SlideType`] plugin.
    pub fn register_slide_type(&mut self, plugin: Box<dyn SlideType + Send + Sync>) {
        self.slide_types.push(plugin);
    }

    /// Look up a [`SlideType`] by its keyword (`id()`).
    ///
    /// This is the primary lookup used by the layout engine when processing a
    /// `slide:` block. Returns the first registered slide type whose `id()`
    /// matches `keyword`.
    #[must_use]
    pub fn lookup_slide_type(&self, keyword: &str) -> Option<&dyn SlideType> {
        self.slide_types
            .iter()
            .find(|p| p.id() == keyword)
            .map(|p| p.as_ref() as &dyn SlideType)
    }

    // ──────────────────────────────────────────────────────────────────────────
    // Surface 9: SectionType
    // ──────────────────────────────────────────────────────────────────────────

    /// Register a [`SectionType`] plugin.
    pub fn register_section_type(&mut self, plugin: Box<dyn SectionType + Send + Sync>) {
        self.section_types.push(plugin);
    }

    /// Look up a [`SectionType`] by its `id()`.
    #[must_use]
    pub fn lookup_section_type(&self, id: &str) -> Option<&dyn SectionType> {
        self.section_types
            .iter()
            .find(|p| p.id() == id)
            .map(|p| p.as_ref() as &dyn SectionType)
    }

    // ──────────────────────────────────────────────────────────────────────────
    // Surface 10: InlineFormat
    // ──────────────────────────────────────────────────────────────────────────

    /// Register an [`InlineFormat`] plugin.
    pub fn register_inline_format(&mut self, plugin: Box<dyn InlineFormat + Send + Sync>) {
        self.inline_formats.push(plugin);
    }

    /// Look up an [`InlineFormat`] by its `id()`.
    #[must_use]
    pub fn lookup_inline_format(&self, id: &str) -> Option<&dyn InlineFormat> {
        self.inline_formats
            .iter()
            .find(|p| p.id() == id)
            .map(|p| p.as_ref() as &dyn InlineFormat)
    }
}

#[cfg(test)]
#[allow(clippy::unnecessary_literal_bound)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::uninlined_format_args)]
#[allow(clippy::used_underscore_items)]
#[allow(clippy::doc_markdown)]
mod tests {
    use super::*;
    use crate::traits::{
        BrandError, BrandSource, ChartError, DataSourceError, DataSourceOptions, DiagramError,
        DiagramOptions, ExportError, ExportOptions, InlineError, InlineOutputFormat, LayoutError,
        MathError, MathOutputFormat, ValidatorOptions,
    };
    use slideforge_layout::{LaidOutDeck, LaidOutSlide, PageSize};
    use slideforge_types::{
        Brand, BrandFonts, BrandPalette, ChartSpec, Deck, DeckMetadata, InlineNode, MathNode,
        OrderedMap, Slide, SourceSpan, Value,
    };

    // ── Stub implementations for each surface ─────────────────────────────────

    struct StubDataSource;
    impl DataSource for StubDataSource {
        fn id(&self) -> &str {
            "stub-data"
        }
        fn load(&self, _uri: &str, _opts: &DataSourceOptions) -> Result<Value, DataSourceError> {
            Ok(Value::Null)
        }
    }

    struct StubExporter;
    impl Exporter for StubExporter {
        fn id(&self) -> &str {
            "stub-export"
        }
        fn extension(&self) -> &str {
            "bin"
        }
        fn export(
            &self,
            _deck: &Deck,
            _laid_out: &LaidOutDeck,
            _brand: &Brand,
            _opts: &ExportOptions,
        ) -> Result<Vec<u8>, ExportError> {
            Ok(vec![])
        }
    }

    struct StubChartRenderer;
    impl ChartRenderer for StubChartRenderer {
        fn id(&self) -> &str {
            "stub-chart"
        }
        fn render(&self, _spec: &ChartSpec, _brand: &Brand) -> Result<Vec<u8>, ChartError> {
            Ok(vec![])
        }
    }

    struct StubDiagramRenderer;
    impl DiagramRenderer for StubDiagramRenderer {
        fn id(&self) -> &str {
            "stub-diagram"
        }
        fn render(&self, _source: &str, _opts: &DiagramOptions) -> Result<Vec<u8>, DiagramError> {
            Ok(vec![])
        }
    }

    struct StubValidator;
    impl Validator for StubValidator {
        fn id(&self) -> &str {
            "stub-validator"
        }
        fn validate(
            &self,
            _deck: &Deck,
            _opts: &ValidatorOptions,
        ) -> Vec<crate::traits::Diagnostic> {
            vec![]
        }
    }

    struct StubMathRenderer;
    impl MathRenderer for StubMathRenderer {
        fn id(&self) -> &str {
            "stub-math"
        }
        fn render(
            &self,
            _node: &MathNode,
            _format: MathOutputFormat,
        ) -> Result<Vec<u8>, MathError> {
            Ok(vec![])
        }
    }

    struct StubBrandProvider;
    impl BrandProvider for StubBrandProvider {
        fn id(&self) -> &str {
            "stub-brand"
        }
        fn load(&self, _source: &BrandSource) -> Result<Brand, BrandError> {
            Ok(Brand {
                name: std::sync::Arc::from("stub"),
                palette: BrandPalette {
                    primary: std::sync::Arc::from("#000000"),
                    secondary: std::sync::Arc::from("#ffffff"),
                    accent: std::sync::Arc::from("#ff0000"),
                    neutral: std::sync::Arc::from("#888888"),
                },
                fonts: BrandFonts {
                    heading: std::sync::Arc::from("Arial"),
                    body: std::sync::Arc::from("Arial"),
                    mono: std::sync::Arc::from("Courier"),
                },
                layouts: vec![],
                span: SourceSpan::default(),
            })
        }
    }

    struct StubSlideType;
    impl SlideType for StubSlideType {
        fn id(&self) -> &'static str {
            "stub-slide"
        }
        fn required_fields(&self) -> &[crate::traits::FieldDef] {
            &[]
        }
        fn optional_fields(&self) -> &[crate::traits::FieldDef] {
            &[]
        }
        fn layout_name(&self) -> &'static str {
            "blank"
        }
        fn lay_out(
            &self,
            slide: &Slide,
            _brand: &Brand,
            _canvas: crate::traits::Canvas,
        ) -> Result<LaidOutSlide, LayoutError> {
            Ok(LaidOutSlide {
                source_index: 0,
                slide_type_keyword: std::sync::Arc::clone(&slide.slide_type),
                frames: vec![],
                speaker_notes: None,
                register_tags: vec![],
                register_content: vec![],
            })
        }
    }

    struct StubSectionType;
    impl SectionType for StubSectionType {
        fn id(&self) -> &str {
            "stub-section"
        }
        fn generate(&self, _slides: &[Slide]) -> Vec<crate::traits::SectionBlock> {
            vec![]
        }
    }

    struct StubInlineFormat;
    impl InlineFormat for StubInlineFormat {
        fn id(&self) -> &str {
            "stub-inline"
        }
        fn render(
            &self,
            _node: &InlineNode,
            _format: InlineOutputFormat,
        ) -> Result<String, InlineError> {
            Ok(String::new())
        }
    }

    // ── Helper to build a minimal Deck for tests ───────────────────────────────

    fn stub_deck() -> Deck {
        Deck {
            slides: vec![],
            vars: OrderedMap::new(),
            metadata: DeckMetadata {
                title: None,
                slideforge_version: std::sync::Arc::from("0.1.0"),
                lang: None,
                author: None,
                section_order: None,
            },
            registers: OrderedMap::new(),
            section_blocks: vec![],
        }
    }

    fn stub_laid_out_deck() -> LaidOutDeck {
        LaidOutDeck {
            page_size: PageSize::default(),
            slides: vec![],
            sections: vec![],
            warnings: vec![],
        }
    }

    fn stub_brand() -> Brand {
        Brand {
            name: std::sync::Arc::from("test"),
            palette: BrandPalette {
                primary: std::sync::Arc::from("#003087"),
                secondary: std::sync::Arc::from("#0066CC"),
                accent: std::sync::Arc::from("#FF6B35"),
                neutral: std::sync::Arc::from("#F5F5F5"),
            },
            fonts: BrandFonts {
                heading: std::sync::Arc::from("Calibri Light"),
                body: std::sync::Arc::from("Calibri"),
                mono: std::sync::Arc::from("Courier New"),
            },
            layouts: vec![],
            span: SourceSpan::default(),
        }
    }

    // ── AC-003 (EC-003): empty registry returns None for all lookups ───────────

    #[test]
    fn test_bc_5_02_001_empty_registry_lookup_data_source_returns_none() {
        let registry = PluginRegistry::new();
        assert!(registry.lookup_data_source("anything").is_none());
    }

    #[test]
    fn test_bc_5_02_001_empty_registry_lookup_exporter_returns_none() {
        let registry = PluginRegistry::new();
        assert!(registry.lookup_exporter("pptx").is_none());
    }

    #[test]
    fn test_bc_5_02_001_empty_registry_lookup_chart_renderer_returns_none() {
        let registry = PluginRegistry::new();
        assert!(registry.lookup_chart_renderer("plotters").is_none());
    }

    #[test]
    fn test_bc_5_02_001_empty_registry_lookup_diagram_renderer_returns_none() {
        let registry = PluginRegistry::new();
        assert!(registry.lookup_diagram_renderer("mermaid").is_none());
    }

    #[test]
    fn test_bc_5_02_001_empty_registry_lookup_validator_returns_none() {
        let registry = PluginRegistry::new();
        assert!(registry.lookup_validator("overflow").is_none());
    }

    #[test]
    fn test_bc_5_02_001_empty_registry_lookup_math_renderer_returns_none() {
        let registry = PluginRegistry::new();
        assert!(registry.lookup_math_renderer("pulldown-latex").is_none());
    }

    #[test]
    fn test_bc_5_02_001_empty_registry_lookup_brand_provider_returns_none() {
        let registry = PluginRegistry::new();
        assert!(registry.lookup_brand_provider("default").is_none());
    }

    #[test]
    fn test_bc_5_02_001_empty_registry_lookup_slide_type_returns_none() {
        let registry = PluginRegistry::new();
        assert!(registry.lookup_slide_type("title").is_none());
    }

    #[test]
    fn test_bc_5_02_001_empty_registry_lookup_section_type_returns_none() {
        let registry = PluginRegistry::new();
        assert!(registry.lookup_section_type("toc").is_none());
    }

    #[test]
    fn test_bc_5_02_001_empty_registry_lookup_inline_format_returns_none() {
        let registry = PluginRegistry::new();
        assert!(registry.lookup_inline_format("default").is_none());
    }

    // ── Register-then-lookup round trips for all 10 surfaces ─────────────────

    #[test]
    fn test_bc_5_02_001_register_and_lookup_data_source() {
        let mut registry = PluginRegistry::new();
        registry.register_data_source(Box::new(StubDataSource));
        let plugin = registry.lookup_data_source("stub-data");
        assert!(plugin.is_some());
        assert_eq!(plugin.unwrap().id(), "stub-data");
    }

    #[test]
    fn test_bc_5_02_001_register_and_lookup_exporter() {
        let mut registry = PluginRegistry::new();
        registry.register_exporter(Box::new(StubExporter));
        let plugin = registry.lookup_exporter("stub-export");
        assert!(plugin.is_some());
        assert_eq!(plugin.unwrap().extension(), "bin");
    }

    #[test]
    fn test_bc_5_02_001_register_and_lookup_chart_renderer() {
        let mut registry = PluginRegistry::new();
        registry.register_chart_renderer(Box::new(StubChartRenderer));
        let plugin = registry.lookup_chart_renderer("stub-chart");
        assert!(plugin.is_some());
    }

    #[test]
    fn test_bc_5_02_001_register_and_lookup_diagram_renderer() {
        let mut registry = PluginRegistry::new();
        registry.register_diagram_renderer(Box::new(StubDiagramRenderer));
        let plugin = registry.lookup_diagram_renderer("stub-diagram");
        assert!(plugin.is_some());
    }

    #[test]
    fn test_bc_5_02_001_register_and_lookup_validator() {
        let mut registry = PluginRegistry::new();
        registry.register_validator(Box::new(StubValidator));
        let plugin = registry.lookup_validator("stub-validator");
        assert!(plugin.is_some());
    }

    #[test]
    fn test_bc_5_02_001_register_and_lookup_math_renderer() {
        let mut registry = PluginRegistry::new();
        registry.register_math_renderer(Box::new(StubMathRenderer));
        let plugin = registry.lookup_math_renderer("stub-math");
        assert!(plugin.is_some());
    }

    #[test]
    fn test_bc_5_02_001_register_and_lookup_brand_provider() {
        let mut registry = PluginRegistry::new();
        registry.register_brand_provider(Box::new(StubBrandProvider));
        let plugin = registry.lookup_brand_provider("stub-brand");
        assert!(plugin.is_some());
    }

    #[test]
    fn test_bc_5_02_001_register_and_lookup_slide_type() {
        let mut registry = PluginRegistry::new();
        registry.register_slide_type(Box::new(StubSlideType));
        let plugin = registry.lookup_slide_type("stub-slide");
        assert!(plugin.is_some());
        assert_eq!(plugin.unwrap().layout_name(), "blank");
    }

    #[test]
    fn test_bc_5_02_001_register_and_lookup_section_type() {
        let mut registry = PluginRegistry::new();
        registry.register_section_type(Box::new(StubSectionType));
        let plugin = registry.lookup_section_type("stub-section");
        assert!(plugin.is_some());
    }

    #[test]
    fn test_bc_5_02_001_register_and_lookup_inline_format() {
        let mut registry = PluginRegistry::new();
        registry.register_inline_format(Box::new(StubInlineFormat));
        let plugin = registry.lookup_inline_format("stub-inline");
        assert!(plugin.is_some());
    }

    // ── AC-015: all 10 surfaces registered and accessible ────────────────────

    #[test]
    fn test_bc_5_02_001_all_10_surfaces_register_and_lookup() {
        let mut registry = PluginRegistry::new();

        registry.register_data_source(Box::new(StubDataSource));
        registry.register_exporter(Box::new(StubExporter));
        registry.register_chart_renderer(Box::new(StubChartRenderer));
        registry.register_diagram_renderer(Box::new(StubDiagramRenderer));
        registry.register_validator(Box::new(StubValidator));
        registry.register_math_renderer(Box::new(StubMathRenderer));
        registry.register_brand_provider(Box::new(StubBrandProvider));
        registry.register_slide_type(Box::new(StubSlideType));
        registry.register_section_type(Box::new(StubSectionType));
        registry.register_inline_format(Box::new(StubInlineFormat));

        assert!(registry.lookup_data_source("stub-data").is_some());
        assert!(registry.lookup_exporter("stub-export").is_some());
        assert!(registry.lookup_chart_renderer("stub-chart").is_some());
        assert!(registry.lookup_diagram_renderer("stub-diagram").is_some());
        assert!(registry.lookup_validator("stub-validator").is_some());
        assert!(registry.lookup_math_renderer("stub-math").is_some());
        assert!(registry.lookup_brand_provider("stub-brand").is_some());
        assert!(registry.lookup_slide_type("stub-slide").is_some());
        assert!(registry.lookup_section_type("stub-section").is_some());
        assert!(registry.lookup_inline_format("stub-inline").is_some());
    }

    // ── EC-001: duplicate id — first registration returned ────────────────────

    struct StubDataSourceDuplicate;
    impl DataSource for StubDataSourceDuplicate {
        fn id(&self) -> &str {
            "stub-data" // same id as StubDataSource
        }
        fn load(&self, _uri: &str, _opts: &DataSourceOptions) -> Result<Value, DataSourceError> {
            Ok(Value::Int(42)) // returns different value to detect which plugin is used
        }
    }

    #[test]
    fn test_bc_5_02_001_ec001_duplicate_id_first_returned() {
        let mut registry = PluginRegistry::new();
        registry.register_data_source(Box::new(StubDataSource));
        registry.register_data_source(Box::new(StubDataSourceDuplicate));

        // Both are stored; first registered (StubDataSource) is returned on lookup.
        let plugin = registry.lookup_data_source("stub-data").unwrap();
        let result = plugin
            .load("x", &DataSourceOptions::default())
            .expect("load should succeed");
        // StubDataSource returns Null; StubDataSourceDuplicate returns Int(42).
        // The first registration should win.
        assert_eq!(result, Value::Null);
    }

    // ── Functional call-through tests ─────────────────────────────────────────

    #[test]
    fn test_bc_5_02_001_exporter_can_export_after_lookup() {
        let mut registry = PluginRegistry::new();
        registry.register_exporter(Box::new(StubExporter));
        let exporter = registry.lookup_exporter("stub-export").unwrap();
        let deck = stub_deck();
        let laid_out = stub_laid_out_deck();
        let brand = stub_brand();
        let result = exporter.export(&deck, &laid_out, &brand, &ExportOptions::default());
        assert!(result.is_ok());
    }

    #[test]
    fn test_bc_5_02_001_math_renderer_can_render_after_lookup() {
        let mut registry = PluginRegistry::new();
        registry.register_math_renderer(Box::new(StubMathRenderer));
        let renderer = registry.lookup_math_renderer("stub-math").unwrap();
        let node = MathNode {
            latex: std::sync::Arc::from("x^2"),
            display: false,
            span: SourceSpan::default(),
        };
        let result = renderer.render(&node, MathOutputFormat::MathMl);
        assert!(result.is_ok());
    }

    #[test]
    fn test_bc_5_02_001_registry_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<PluginRegistry>();
    }

    // ── STORY-083 / BC-5.02.001 invariant 3: PluginRegistryBuilder tests ─────

    // Compile-time check: RegistryError implements std::error::Error.
    // If this function compiles, the trait bound is satisfied.
    fn _assert_registry_error_is_std_error<E: std::error::Error>() {}
    const _: fn() = || {
        _assert_registry_error_is_std_error::<RegistryError>();
    };

    // ── AC-001: empty builder returns Err(MissingSurface { surface: "DataSource" }) ──

    /// BC-5.02.001 invariant 3: `PluginRegistryBuilder::default().build()` with no
    /// registrations MUST return `Err(RegistryError::MissingSurface { .. })` — not
    /// `Ok(..)`, not a panic, not an `anyhow` error.
    ///
    /// Also asserts the FIRST missing surface is `"DataSource"` — the first name in
    /// `SURFACE_NAMES` declaration order.
    // The `Err(other)` arms below are unreachable *within* this crate because
    // `RegistryError` is defined here and is exhaustively matched. They are kept
    // to document — and guard against — new variants added in the future. The
    // `#[allow(unreachable_patterns)]` suppresses the intra-crate lint while
    // preserving the intent.
    #[test]
    #[allow(unreachable_patterns)]
    fn test_bc_5_02_001_empty_builder_returns_err_missing_surface_datasource() {
        let result = PluginRegistryBuilder::default().build();
        match result {
            Err(RegistryError::MissingSurface { surface }) => {
                assert_eq!(
                    surface, "DataSource",
                    "first missing surface must be \"DataSource\" (declaration order per SURFACE_NAMES)"
                );
            },
            Ok(_) => {
                panic!("AC-001 VIOLATED: empty builder must return Err(MissingSurface), got Ok")
            },
            Err(other) => panic!(
                "AC-001 VIOLATED: expected MissingSurface variant, got: {:?}",
                other
            ),
        }
    }

    /// BC-5.02.001 invariant 3: the `matches!` form from the BC inline test vector.
    #[test]
    fn test_bc_5_02_001_empty_builder_matches_missing_surface_variant() {
        let result = PluginRegistryBuilder::default().build();
        assert!(
            matches!(result, Err(RegistryError::MissingSurface { .. })),
            "AC-001: PluginRegistryBuilder::default().build() must return Err(MissingSurface {{ .. }})"
        );
    }

    // ── AC-002: fully-registered builder returns Ok(registry) ─────────────────

    /// BC-5.02.001 invariant 3: a builder with all 10 surfaces registered must
    /// return `Ok(PluginRegistry)`.
    #[test]
    fn test_bc_5_02_001_fully_registered_builder_returns_ok() {
        let mut builder = PluginRegistryBuilder::default();
        builder.register_data_source(Box::new(StubDataSource));
        builder.register_exporter(Box::new(StubExporter));
        builder.register_chart_renderer(Box::new(StubChartRenderer));
        builder.register_diagram_renderer(Box::new(StubDiagramRenderer));
        builder.register_validator(Box::new(StubValidator));
        builder.register_math_renderer(Box::new(StubMathRenderer));
        builder.register_brand_provider(Box::new(StubBrandProvider));
        builder.register_slide_type(Box::new(StubSlideType));
        builder.register_section_type(Box::new(StubSectionType));
        builder.register_inline_format(Box::new(StubInlineFormat));

        let result = builder.build();
        assert!(
            result.is_ok(),
            "AC-002: fully-registered builder must return Ok(registry), got: {:?}",
            result.err()
        );
    }

    // ── AC-003: fully-registered registry has surface_count() == 10 ───────────

    /// BC-5.02.001 postcondition 2: `surface_count()` on the fully-registered
    /// registry returned by a 10-surface builder must equal 10.
    #[test]
    fn test_bc_5_02_001_surface_count_is_10_for_fully_registered_registry() {
        let mut builder = PluginRegistryBuilder::default();
        builder.register_data_source(Box::new(StubDataSource));
        builder.register_exporter(Box::new(StubExporter));
        builder.register_chart_renderer(Box::new(StubChartRenderer));
        builder.register_diagram_renderer(Box::new(StubDiagramRenderer));
        builder.register_validator(Box::new(StubValidator));
        builder.register_math_renderer(Box::new(StubMathRenderer));
        builder.register_brand_provider(Box::new(StubBrandProvider));
        builder.register_slide_type(Box::new(StubSlideType));
        builder.register_section_type(Box::new(StubSectionType));
        builder.register_inline_format(Box::new(StubInlineFormat));

        let registry = builder
            .build()
            .expect("all 10 surfaces registered; build must succeed");
        assert_eq!(
            registry.surface_count(),
            10,
            "AC-003: fully-registered registry must have surface_count() == 10"
        );
    }

    // ── AC-004: surface_names() returns all 10 canonical names in order ────────

    /// BC-5.02.001 invariant 1: `surface_names()` on the fully-registered registry
    /// must return exactly the 10 canonical names equal to `SURFACE_NAMES`, in
    /// declaration order.
    #[test]
    fn test_bc_5_02_001_surface_names_returns_all_10_canonical_names() {
        let mut builder = PluginRegistryBuilder::default();
        builder.register_data_source(Box::new(StubDataSource));
        builder.register_exporter(Box::new(StubExporter));
        builder.register_chart_renderer(Box::new(StubChartRenderer));
        builder.register_diagram_renderer(Box::new(StubDiagramRenderer));
        builder.register_validator(Box::new(StubValidator));
        builder.register_math_renderer(Box::new(StubMathRenderer));
        builder.register_brand_provider(Box::new(StubBrandProvider));
        builder.register_slide_type(Box::new(StubSlideType));
        builder.register_section_type(Box::new(StubSectionType));
        builder.register_inline_format(Box::new(StubInlineFormat));

        let registry = builder
            .build()
            .expect("all 10 surfaces registered; build must succeed");
        let names = registry.surface_names();

        assert_eq!(
            names.len(),
            10,
            "AC-004: fully-registered registry must have surface_names().len() == 10, got: {:?}",
            names
        );

        // Assert exact contents and order match SURFACE_NAMES.
        let expected: Vec<&'static str> = SURFACE_NAMES.to_vec();
        assert_eq!(
            names, expected,
            "AC-004: surface_names() must return names in SURFACE_NAMES declaration order"
        );
    }

    // ── AC-005: RegistryError Display produces expected human-readable message ──

    /// BC-5.02.001 invariant 3 / AC-005: `RegistryError::MissingSurface { surface }`
    /// Display output must name the missing surface in a human-readable message.
    ///
    /// Specified format (from STORY-083 AC-005):
    ///   "required plugin surface '{surface}' has no registered implementations"
    #[test]
    fn test_bc_5_02_001_registry_error_display_names_missing_surface() {
        let err = RegistryError::MissingSurface {
            surface: "DataSource",
        };
        let msg = err.to_string();
        assert!(
            msg.contains("DataSource"),
            "AC-005: Display must name the missing surface; got: {:?}",
            msg
        );
        assert_eq!(
            msg, "required plugin surface 'DataSource' has no registered implementations",
            "AC-005: exact Display format must match the thiserror template"
        );
    }

    /// AC-005: RegistryError implements Debug.
    #[test]
    fn test_bc_5_02_001_registry_error_implements_debug() {
        let err = RegistryError::MissingSurface {
            surface: "Exporter",
        };
        let debug_str = format!("{:?}", err);
        assert!(
            debug_str.contains("MissingSurface"),
            "AC-005: Debug output must contain variant name; got: {:?}",
            debug_str
        );
    }

    /// AC-005: RegistryError is #[non_exhaustive] — callers must have a wildcard arm.
    ///
    /// `#[non_exhaustive]` only applies across crate boundaries. Within this crate
    /// the compiler sees the enum exhaustively, so the wildcard arm is unreachable
    /// here. We suppress the lint with `#[allow(unreachable_patterns)]` while
    /// keeping the wildcard to document the required external-caller pattern and to
    /// guard against future variant additions that would silently drop into the arm.
    #[test]
    #[allow(unreachable_patterns)]
    fn test_bc_5_02_001_registry_error_non_exhaustive_requires_wildcard() {
        let err = RegistryError::MissingSurface {
            surface: "Validator",
        };
        let matched = match err {
            RegistryError::MissingSurface { surface } => {
                format!("missing: {surface}")
            },
            _ => String::from("other"),
        };
        assert!(
            matched.starts_with("missing:"),
            "AC-005: MissingSurface branch must match; got: {:?}",
            matched
        );
    }

    // ── Edge case: 9-of-10 surfaces registered → Err naming the missing one ───

    /// BC-5.02.001 invariant 3 edge case: registering exactly 9 surfaces (missing
    /// `InlineFormat`, the last in declaration order) must return
    /// `Err(MissingSurface { surface: "InlineFormat" })`.
    #[test]
    #[allow(unreachable_patterns)]
    fn test_bc_5_02_001_missing_last_surface_inline_format_reports_inline_format() {
        let mut builder = PluginRegistryBuilder::default();
        builder.register_data_source(Box::new(StubDataSource));
        builder.register_exporter(Box::new(StubExporter));
        builder.register_chart_renderer(Box::new(StubChartRenderer));
        builder.register_diagram_renderer(Box::new(StubDiagramRenderer));
        builder.register_validator(Box::new(StubValidator));
        builder.register_math_renderer(Box::new(StubMathRenderer));
        builder.register_brand_provider(Box::new(StubBrandProvider));
        builder.register_slide_type(Box::new(StubSlideType));
        builder.register_section_type(Box::new(StubSectionType));
        // InlineFormat deliberately NOT registered.

        let result = builder.build();
        match result {
            Err(RegistryError::MissingSurface { surface }) => {
                assert_eq!(
                    surface, "InlineFormat",
                    "missing InlineFormat (last in SURFACE_NAMES) must be reported; got: {:?}",
                    surface
                );
            },
            Ok(_) => panic!("9-of-10 builder must return Err(MissingSurface), got Ok"),
            Err(other) => panic!("expected MissingSurface variant, got: {:?}", other),
        }
    }

    /// BC-5.02.001 invariant 3 edge case: registering exactly 9 surfaces (missing
    /// `BrandProvider`, a middle surface at index 6) must return
    /// `Err(MissingSurface { surface: "BrandProvider" })`.
    #[test]
    #[allow(unreachable_patterns)]
    fn test_bc_5_02_001_missing_middle_surface_brand_provider_reports_brand_provider() {
        let mut builder = PluginRegistryBuilder::default();
        builder.register_data_source(Box::new(StubDataSource));
        builder.register_exporter(Box::new(StubExporter));
        builder.register_chart_renderer(Box::new(StubChartRenderer));
        builder.register_diagram_renderer(Box::new(StubDiagramRenderer));
        builder.register_validator(Box::new(StubValidator));
        builder.register_math_renderer(Box::new(StubMathRenderer));
        // BrandProvider deliberately NOT registered.
        builder.register_slide_type(Box::new(StubSlideType));
        builder.register_section_type(Box::new(StubSectionType));
        builder.register_inline_format(Box::new(StubInlineFormat));

        let result = builder.build();
        match result {
            Err(RegistryError::MissingSurface { surface }) => {
                assert_eq!(
                    surface, "BrandProvider",
                    "missing BrandProvider must be reported; got: {:?}",
                    surface
                );
            },
            Ok(_) => {
                panic!("9-of-10 builder (no BrandProvider) must return Err(MissingSurface), got Ok")
            },
            Err(other) => panic!("expected MissingSurface variant, got: {:?}", other),
        }
    }

    // ── EC-001 (builder): multiple register_* calls for same surface ──────────

    // ── AC-003 / AC-004 / EC-002: partial registry (3-of-10 surfaces) ──────────
    //
    // F-083-03 gap closure: the non-10 branch of surface_count() / surface_names()
    // had no behavioral coverage. An impl hardcoding `10` would pass all tests
    // above (TD-VSDD-059 paper-fix risk). These tests use the mutation API directly
    // on `PluginRegistry::new()` — the builder rejects partial registries, so only
    // the direct register_* path can construct one.
    //
    // The 3 surfaces chosen are the first 3 in SURFACE_NAMES canonical declaration
    // order: DataSource (index 0), Exporter (index 1), ChartRenderer (index 2).

    /// AC-003 (partial): `surface_count()` on a partially-assembled 3-of-10
    /// registry via the mutation API must return 3, NOT 10.
    ///
    /// This test falsifies any impl that hardcodes `10` or unconditionally
    /// returns the total number of struct fields.
    ///
    /// Traceability: AC-003 (STORY-083), BC-5.02.001 postcondition 2, F-083-03.
    #[test]
    fn test_bc_5_02_001_partial_registry_surface_count_returns_actual_nonzero_count() {
        let mut registry = PluginRegistry::new();
        // Register exactly 3 surfaces — first 3 in SURFACE_NAMES declaration order.
        registry.register_data_source(Box::new(StubDataSource));
        registry.register_exporter(Box::new(StubExporter));
        registry.register_chart_renderer(Box::new(StubChartRenderer));

        let count = registry.surface_count();
        assert_eq!(
            count, 3,
            "AC-003 (partial): surface_count() on a 3-of-10 partial registry must \
             return 3, not 10 — an impl hardcoding 10 would fail here (F-083-03)"
        );
    }

    /// AC-004 / EC-002 (partial): `surface_names()` on a partially-assembled
    /// 3-of-10 registry must return ONLY the names of the 3 registered surfaces,
    /// in canonical SURFACE_NAMES declaration order.
    ///
    /// Asserts both exact contents AND exact order.  The 7 unregistered surfaces
    /// must NOT appear in the result.
    ///
    /// Traceability: AC-004 (STORY-083), EC-002, BC-5.02.001 invariant 1, F-083-03.
    #[test]
    fn test_bc_5_02_001_partial_registry_surface_names_returns_only_registered_surfaces() {
        let mut registry = PluginRegistry::new();
        // Register exactly 3 surfaces — first 3 in SURFACE_NAMES declaration order.
        registry.register_data_source(Box::new(StubDataSource));
        registry.register_exporter(Box::new(StubExporter));
        registry.register_chart_renderer(Box::new(StubChartRenderer));

        let names = registry.surface_names();

        // Assert exact length.
        assert_eq!(
            names.len(),
            3,
            "AC-004 / EC-002 (partial): surface_names() must return exactly 3 names \
             for a 3-of-10 partial registry; got: {:?}",
            names
        );

        // Assert exact contents AND canonical declaration order (SURFACE_NAMES[0..=2]).
        let expected: Vec<&'static str> = vec!["DataSource", "Exporter", "ChartRenderer"];
        assert_eq!(
            names, expected,
            "AC-004 / EC-002 (partial): surface_names() must return exactly \
             [\"DataSource\", \"Exporter\", \"ChartRenderer\"] in declaration order; \
             got: {:?}",
            names
        );

        // Verify the expected names are the actual first 3 entries of SURFACE_NAMES
        // (guards against SURFACE_NAMES reordering breaking this test silently).
        assert_eq!(
            SURFACE_NAMES[0], "DataSource",
            "SURFACE_NAMES[0] must be \"DataSource\" per spec"
        );
        assert_eq!(
            SURFACE_NAMES[1], "Exporter",
            "SURFACE_NAMES[1] must be \"Exporter\" per spec"
        );
        assert_eq!(
            SURFACE_NAMES[2], "ChartRenderer",
            "SURFACE_NAMES[2] must be \"ChartRenderer\" per spec"
        );
    }

    /// Cross-consistency: `surface_names().len() == surface_count()` on a partial
    /// registry.  These two introspection methods must never diverge.
    ///
    /// Traceability: AC-003 + AC-004 consistency invariant, F-083-03.
    #[test]
    fn test_bc_5_02_001_partial_registry_surface_names_len_equals_surface_count() {
        let mut registry = PluginRegistry::new();
        registry.register_data_source(Box::new(StubDataSource));
        registry.register_exporter(Box::new(StubExporter));
        registry.register_chart_renderer(Box::new(StubChartRenderer));

        let count = registry.surface_count();
        let names_len = registry.surface_names().len();
        assert_eq!(
            names_len, count,
            "surface_names().len() must equal surface_count() on a partial registry; \
             got names_len={names_len}, surface_count={count}"
        );
    }

    // ── EC-001 (builder): multiple register_* calls for same surface ──────────

    /// BC-5.02.001 EC-001 / STORY-083 EC-001: registering the same surface twice is
    /// additive — both are stored; `surface_count()` counts surfaces, not total
    /// plugins; `build()` succeeds if all 10 surfaces have at least 1 each.
    #[test]
    fn test_bc_5_02_001_duplicate_surface_registration_counts_as_one_surface() {
        let mut builder = PluginRegistryBuilder::default();
        // Register DataSource TWICE — surface_count() must still count it as 1 surface.
        builder.register_data_source(Box::new(StubDataSource));
        builder.register_data_source(Box::new(StubDataSource));
        builder.register_exporter(Box::new(StubExporter));
        builder.register_chart_renderer(Box::new(StubChartRenderer));
        builder.register_diagram_renderer(Box::new(StubDiagramRenderer));
        builder.register_validator(Box::new(StubValidator));
        builder.register_math_renderer(Box::new(StubMathRenderer));
        builder.register_brand_provider(Box::new(StubBrandProvider));
        builder.register_slide_type(Box::new(StubSlideType));
        builder.register_section_type(Box::new(StubSectionType));
        builder.register_inline_format(Box::new(StubInlineFormat));

        let registry = builder
            .build()
            .expect("all 10 surfaces registered (DataSource twice); build must succeed");
        assert_eq!(
            registry.surface_count(),
            10,
            "EC-001: surface_count() counts surfaces, not total plugins; \
             registering DataSource twice must still yield 10, not 11"
        );
    }
}
