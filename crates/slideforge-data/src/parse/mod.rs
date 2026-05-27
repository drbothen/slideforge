//! Per-format parsers for slideforge data sources.
//!
//! Each sub-module provides a single entry-point function that takes raw bytes
//! (or a string slice) and returns a [`slideforge_types::Value`] or a
//! [`crate::DataError`].

pub mod csv;
pub mod json;
pub mod toml;
pub mod yaml;
