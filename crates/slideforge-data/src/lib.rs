//! `DataSource` plugin implementations for slideforge.
//!
//! This crate provides the built-in `DataSource` plugin implementations
//! that load data from local files in JSON, CSV, YAML, and TOML formats,
//! as well as HTTP/HTTPS sources with SSRF protection.
//!
//! ## Supported formats
//!
//! | Format | Extensions | Parser |
//! |--------|-----------|--------|
//! | JSON   | `.json`   | [`serde_json`] |
//! | CSV    | `.csv`    | [`csv`] crate |
//! | YAML   | `.yaml`, `.yml` | [`serde_yaml_ng`] |
//! | TOML   | `.toml`   | [`toml`] crate |
//!
//! ## Architecture
//!
//! All implementations conform to the `DataSource` trait defined in
//! `slideforge-plugin-api`. The file-based loader dispatches to per-format
//! parsers in the [`parse`] module. The HTTP loader uses [`ureq`] with
//! SSRF allowlist enforcement from the [`allowlist`] module.
//!
//! ## Dog-fooding Guarantee (BC-5.02.002)
//!
//! `FileDataSource` and `HttpDataSource` implement `DataSource` using only
//! the public plugin-api traits. No internal bypass of the plugin system.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod allowlist;
pub mod context;
pub mod error;
pub mod file;
pub mod format;
pub mod http;
pub mod parse;

pub use context::DataSourceContext;
pub use error::DataError;
pub use file::FileDataSource;
pub use format::DataFormat;
pub use http::HttpDataSource;
